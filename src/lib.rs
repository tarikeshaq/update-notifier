use crates_io_api::{SyncClient, Version};
use toml;
use std::fs;
use serde::Deserialize;
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
#[derive(Deserialize)]
struct CargoToml {
    package: Package
}

#[derive(Deserialize)]
struct Package {
    name: Option<String>,
    version: Option<String>,
}

fn get_latest_version(crate_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = SyncClient::new();
    let retrieved_crate = match client.get_crate(crate_name) {
        Ok(val) => val,
        Err(_) => Err(Error::CrateDoesNotExist)?
    };
    let versions: Vec<Version> = retrieved_crate.versions;
    match versions.first() {
        Some(version) => Ok(version.num.clone()),
        None => Err(Error::VersionDoesNotExistCratesIO)?
    }
}

/// Validates current version of crate
/// Takes a path to the Cargo.toml associated with the crate
/// Prints directly to stdout (Will probably change to be more asynchrounos)
pub fn check_version(cargo_toml_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read(cargo_toml_path)?;
    let cargo_toml_file: CargoToml = toml::from_slice(&file_content)?;
    let current_version =  match cargo_toml_file.package.version {
        Some(val) => val,
        None => Err(Error::VersionDoesNotExist)?
    };
    let package_name = match cargo_toml_file.package.name {
        Some(name) => name,
        None => Err(Error::NameDoesNotExist)?
    };

    let latest_version = get_latest_version(&package_name)?;
    if latest_version != current_version {
        println!("Version {} is available!", latest_version);
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
        check_version("./test/test.toml").unwrap();
    }
}
