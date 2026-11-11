#![deny(unused_crate_dependencies)]

mod cmd;

use clap::Parser;
use clap::Subcommand;

/// A command line utility that spawns a process and produces a csv report on
/// its resource usage
#[derive(Debug, Parser)]
#[command(name = "procplot", version, about, long_about = None)]
struct Command {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
  /// Spawn a process and record its resource usage
  #[clap(name = "exec", visible_aliases = ["e"])]
  Exec(cmd::exec::ExecCommand),

  /// Update the binary to the latest release
  #[clap(name = "upgrade", visible_aliases = ["up", "u"])]
  Upgrade(cmd::upgrade::UpgradeCommand),

  /// Print the current version
  Version,
}

fn main() -> anyhow::Result<()> {
  match Command::parse().command {
    Commands::Exec(args) => cmd::exec::main(&args),
    Commands::Upgrade(args) => cmd::upgrade::main(&args),
    Commands::Version => {
      println!("{}", env!("CARGO_PKG_VERSION"));
      Ok(())
    }
  }
}
