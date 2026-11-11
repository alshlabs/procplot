use std::env;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use sysinfo::Pid;
use sysinfo::ProcessesToUpdate;
use sysinfo::System;

use crate::cmd::exec::args::MemoryUnits;
use crate::cmd::exec::args::TimeUnits;
use crate::cmd::exec::reporter::Columns;
use crate::cmd::exec::reporter::Reporter;
use crate::cmd::exec::reporter::Row;

#[derive(Debug, Clone, clap::Args)]
// The flags are user-facing CLI options; grouping them into a sub-struct would
// only complicate the interface.
#[allow(clippy::struct_excessive_bools)]
pub struct ExecCommand {
  /// Command to run. Use `--` to separate procplot's options from the command
  #[clap(allow_hyphen_values = true, trailing_var_arg = true)]
  pub command: Vec<String>,

  /// Output path for the generated report
  #[clap(short = 'r', long = "report", env = "PM_REPORT", value_parser = parse_file_path, default_value = "procplot.csv")]
  pub report_path: std::path::PathBuf,

  /// How often to probe the process for details in milliseconds
  #[clap(short = 'i', long = "interval", env = "PM_INTERVAL", value_parser = parse_duration, default_value = "200")]
  pub poll_interval: Duration,

  /// What units to use for recording memory
  #[clap(
    short = 'm',
    long = "mem-units",
    env = "PM_MEM_UNITS",
    value_enum,
    default_value = "mb"
  )]
  pub mem_units: MemoryUnits,

  /// What units to use for recording time
  #[clap(
    short = 't',
    long = "time-units",
    env = "PM_TIME_UNITS",
    value_enum,
    default_value = "ms"
  )]
  pub time_units: TimeUnits,

  /// Override report file if exists
  #[clap(short = 'o', long = "override-report")]
  pub override_report: bool,

  /// Don't measure CPU usage
  #[clap(long = "no-cpu", env = "PM_NO_CPU")]
  pub no_measure_cpu: bool,

  /// Don't measure memory usage
  #[clap(long = "no-memory", env = "PM_NO_MEMORY")]
  pub no_measure_mem: bool,

  /// Don't measure disk usage
  #[clap(long = "no-disk", env = "PM_NO_DISK")]
  pub no_measure_disk: bool,
}

fn parse_file_path(arg: &str) -> anyhow::Result<std::path::PathBuf> {
  let mut target_path = std::path::PathBuf::from(arg);
  if target_path.is_relative() {
    target_path = env::current_dir()?.join(target_path);
  }
  Ok(target_path)
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
  let millis = arg.parse()?;
  Ok(Duration::from_millis(millis))
}

// Sysinfo reports a non-negative percentage, so clamping and truncating are safe
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn cpu_usage(cpu: f32) -> u64 {
  cpu.round().max(0.0) as u64
}

pub fn main(args: &ExecCommand) -> anyhow::Result<()> {
  if args.command.is_empty() {
    anyhow::bail!("No command provided. Usage: procplot exec [OPTIONS] -- <COMMAND>");
  }

  let reporter = Reporter::new(
    &args.report_path,
    args.override_report,
    Columns {
      cpu: !args.no_measure_cpu,
      memory: !args.no_measure_mem,
      disk: !args.no_measure_disk,
    },
    args.clone(),
  )?;

  let (sender, receiver) = channel::<(u32, Duration)>();

  // Monitor
  let h0: thread::JoinHandle<anyhow::Result<()>> = {
    let args = args.clone();
    let reporter = reporter.clone();

    thread::spawn(move || {
      let (pid, start_time) = receiver.recv()?;
      let pid = Pid::from(pid as usize);

      // Please note that we use "new_all" to ensure that all list of
      // components, network interfaces, disks and users are already filled!
      let mut sys = System::new_all().context("Unable to initialise system")?;

      loop {
        if sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true) == 0 {
          break;
        }
        let info = sys.process(pid).context("Unable to get process")?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let mut row = Row::default();

        if !args.no_measure_cpu {
          row.cpu = Some(cpu_usage(info.cpu_usage()));
        }

        if !args.no_measure_mem {
          row.memory = Some(info.memory());
        }

        if !args.no_measure_disk {
          let disk = info.disk_usage();
          row.disk_read = Some(disk.read_bytes);
          row.disk_write = Some(disk.written_bytes);
        }

        row.time = now
          .checked_sub(start_time)
          .context("Clock moved backwards")?;

        reporter.write(row)?;
        thread::sleep(args.poll_interval);
      }

      Ok(())
    })
  };

  // Process
  let h1: thread::JoinHandle<anyhow::Result<()>> = {
    let args = args.clone();
    let reporter = reporter.clone();

    thread::spawn(move || {
      let mut command_line = args.command.clone();
      let program = command_line.remove(0);

      let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?;

      reporter.write(Row {
        time: Duration::from_millis(0),
        memory: Some(0),
        cpu: Some(0),
        disk_read: Some(0),
        disk_write: Some(0),
      })?;

      let mut child = Command::new(program)
        .args(command_line)
        .current_dir(env::current_dir()?)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

      sender.send((child.id(), start_time))?;
      child.wait()?;

      let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?;

      reporter.write(Row {
        time: end_time
          .checked_sub(start_time)
          .context("Clock moved backwards")?,
        memory: Some(0),
        cpu: Some(0),
        disk_read: Some(0),
        disk_write: Some(0),
      })?;

      Ok(())
    })
  };

  h1.join()
    .map_err(|_| anyhow::anyhow!("Process thread panicked"))??;
  h0.join()
    .map_err(|_| anyhow::anyhow!("Monitor thread panicked"))??;

  Ok(())
}
