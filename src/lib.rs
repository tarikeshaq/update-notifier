use ansi_term::Color::{Blue, Green, Red, Yellow};
use crates_io_api::{SyncClient, Version};
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to find crate")]
    CrateDoesNotExist,
    #[error("Versions Do not exist on crates.io")]
    VersionDoesNotExistCratesIO,
}

fn get_latest_version(crate_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = SyncClient::new();
    let retrieved_crate = match client.get_crate(crate_name) {
        Ok(val) => val,
        Err(_) => Err(Error::CrateDoesNotExist)?,
    };
    let versions: Vec<Version> = retrieved_crate.versions;
    match versions.first() {
        Some(version) => Ok(version.num.clone()),
        None => Err(Error::VersionDoesNotExistCratesIO)?,
    }
}

fn print_notice(name: &str, current_version: &str, latest_version: &str) {
    println!();
    println!("────────────────────────────────────────────────────────────────────────────");
    println!();
    let line_1 = format!(
        "A new version of {} is available! {} → {}",
        Green.bold().paint(name),
        Red.bold().paint(current_version),
        Green.bold().paint(latest_version)
    );
    let line_2 = format!(
        "Use `{}` to install version {}",
        Blue.bold().paint(format!("$ cargo install {}", name)),
        Green.bold().paint(latest_version)
    );
    let line_3 = format!(
        "Check {} for more details",
        Yellow.paint(format!("https://crates.io/crates/{}", name))
    );
    println!("{}", line_1);
    println!("{}", line_2);
    println!("{}", line_3);
    println!("");
    println!("────────────────────────────────────────────────────────────────────────────");
    println!();
}

/// Validates current version of crate
/// Takes the current name and version
/// Prints directly to stdout (Will probably change to be more asynchrounos)
pub fn check_version(name: &str, current_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let latest_version = get_latest_version(name)?;
    if latest_version != current_version {
        print_notice(name, current_version, &latest_version);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_latest_version() {
        let latest_version = get_latest_version("asdev").unwrap();
        assert_eq!(latest_version, "0.1.3")
    }

    #[test]
    fn test_not_current_version() {
        check_version("asdev", "0.1.2").unwrap();
    }
}
