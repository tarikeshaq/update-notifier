use crates_io_api::{SyncClient, Version};
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to find crate")]
    CrateDoesNotExist,
    #[error("Versions Do not exist on crates.io")]
    VersionDoesNotExistCratesIO,
    #[error("Version does not exist on Cargo.toml")]
    VersionDoesNotExist,
    #[error("Cargo.Toml does not contain name")]
    NameDoesNotExist,
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

/// Uses cargo environment variables to check for version and name
/// does has the same behavior as check_version after that
pub fn check_version_with_env() -> Result<(), Box<dyn std::error::Error>> {
    let version = std::env::var("CARGO_PKG_VERSION")?;
    let name = std::env::var("CARGO_PKG_NAME")?;
    println!("Name: {}, Version: {}", name, version);
    check_version(&name, &version)
}

/// Validates current version of crate
/// Takes the current name and version
/// Prints directly to stdout (Will probably change to be more asynchrounos)
pub fn check_version(name: &str, current_version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let latest_version = get_latest_version(name)?;
    if latest_version != current_version {
        println!("===================================");
        println!();
        println!("A new version of {} is available!", name);
        println!(
            "Use `$ cargo install {}` to install version {}",
            name, latest_version
        );
        println!("Disregard this message of you are intentionally using an older version, or are working on an unpublished version");
        println!();
        println!("===================================");
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

    #[test]
    fn test_env_variables() {
        std::env::set_var("CARGO_PKG_NAME", "asdev");
        std::env::set_var("CARGO_PKG_VERSION", "0.1.2");
        check_version_with_env().unwrap();
    }
}
