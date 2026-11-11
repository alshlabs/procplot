use super::constants::TARGET_REPO;
use super::upgrade::UpgradeOptions;
use super::upgrade::UpgradeOutcome;
use super::upgrade::check_for_update;
use super::upgrade::try_upgrade;

#[derive(Debug, clap::Parser)]
pub struct UpgradeCommand {
  /// Check whether a newer version is available without installing it
  #[clap(long)]
  pub check: bool,
}

pub fn main(args: &UpgradeCommand) -> anyhow::Result<()> {
  let current_version = env!("CARGO_PKG_VERSION");

  let options = UpgradeOptions {
    target_repo: TARGET_REPO,
    current_version,
  };

  if args.check {
    return check(&options, current_version);
  }

  println!("Starting Self Upgrade");

  match try_upgrade(&options) {
    Ok(UpgradeOutcome::Skip) => {
      println!("Already on the latest version ({current_version})");
      Ok(())
    }
    Ok(UpgradeOutcome::Success) => {
      println!("Upgrade complete");
      Ok(())
    }
    Err(err) => Err(err.context("Upgrade failed")),
  }
}

fn check(
  options: &UpgradeOptions<'_>,
  current_version: &str,
) -> anyhow::Result<()> {
  match check_for_update(options)? {
    Some(version) => {
      println!("Update available: {current_version} -> {version}");
    }
    None => {
      println!("Already on the latest version ({current_version})");
    }
  }

  Ok(())
}
