use ansi_term::Color::{Blue, Green, Red, Yellow};
use reqwest::blocking::Client;
use reqwest::header;
use serde_derive::Deserialize;

const REGISTRY_URL: &str = "https://crates.io";

#[cfg(test)]
use mockito;

fn get_base_url() -> String {
    #[cfg(not(test))]
    let url = format!("{}/api/v1/crates", REGISTRY_URL);
    #[cfg(test)]
    let url = format!("{}/api/v1/crates", mockito::server_url());
    url
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Unable to find crate")]
    CrateDoesNotExist,
    #[error("Versions do not exist on the registry")]
    VersionDoesNotExistCratesIO,
    #[error("Error while parsing json: {0}")]
    UnableToParseJson(String),
    #[error("Error received from registry: {0}")]
    RegistryError(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct VersionResponse {
    versions: Option<Vec<Version>>,
    errors: Option<Vec<JsonError>>,
}

#[derive(Deserialize, Debug, Clone)]
struct JsonError {
    detail: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Version {
    num: String,
}

fn get_latest_from_json(
    resp: &VersionResponse,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    if let Some(versions) = &resp.versions {
        match versions.first() {
            Some(version) => Ok(version.num.clone()),
            None => Err(ErrorKind::UnableToParseJson("Versions array is empty".to_string()).into()),
        }
    } else if let Some(errors) = &resp.errors {
        match errors.first() {
            Some(error) => Err(ErrorKind::RegistryError(error.detail.clone()).into()),
            None => Err(
                ErrorKind::UnableToParseJson("No errors in the errors array".to_string()).into(),
            ),
        }
    } else {
        Err(ErrorKind::UnableToParseJson(
            "Invalid json response, does not have versions or errors".to_string(),
        )
        .into())
    }
}

pub fn get_latest_version(
    crate_name: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("Update-notifier (teshaq@mozilla.com)"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    let base_url = get_base_url();
    let url = format!("{}/{}/versions", base_url, crate_name);
    let json_resp = match client.get(&url).send()?.json() {
        Ok(resp) => resp,
        Err(e) => return Err(ErrorKind::UnableToParseJson(e.to_string()).into()),
    };
    get_latest_from_json(&json_resp)
}

pub fn generate_notice(name: &str, current_version: &str, latest_version: &str) -> String {
    let line_1 = format!(
        "A new version of {} is available! {} → {}",
        Green.bold().paint(name),
        Red.bold().paint(current_version),
        Green.bold().paint(latest_version)
    );
    let line_2 = format!(
        "Use `{}` to install version {}",
        Blue.bold().paint(format!("cargo install {}", name)),
        Green.bold().paint(latest_version)
    );
    let line_3 = format!(
        "Check {} for more details",
        Yellow.paint(format!("{}/crates/{}", REGISTRY_URL, name))
    );
    format!(
        "\n───────────────────────────────────────────────────────\n
    {}
    {}
    {}
    \n───────────────────────────────────────────────────────\n",
        line_1, line_2, line_3
    )
}

fn print_notice(name: &str, current_version: &str, latest_version: &str) {
    print!("{}", generate_notice(name, current_version, latest_version));
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
mod tests;
