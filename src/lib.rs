use ansi_term::Color::{Blue, Green, Red, Yellow};
use chrono::{DateTime, Utc};
use configstore::{AppUI, Configstore};
use curl::easy::{Easy, List};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

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
enum ErrorKind {
    #[error("Error while parsing json: {0}")]
    UnableToParseJson(String),
    #[error("Error received from registry: {0}")]
    RegistryError(String),
}

#[derive(Deserialize, Debug, Clone)]
struct VersionResponse {
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

fn get_latest_version(crate_name: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    // We use curl-rust here to save us importing a bunch of dependencies pulled in with reqwest
    // We're okay with a blocking api since it's only one small request
    let mut easy = Easy::new();
    let base_url = get_base_url();
    let url = format!("{}/{}/versions", base_url, crate_name);
    easy.url(&url)?;
    let mut list = List::new();
    list.append("USER-AGENT Update-notifier (teshaq@mozilla.com)")?;
    easy.http_headers(list)?;
    let mut resp_buf = Vec::new();
    // Create a different lifetime for `transfer` since it
    // borrows resp_buf in it's closure
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            resp_buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    let resp = std::str::from_utf8(&resp_buf)?;

    let json_resp: VersionResponse = serde_json::from_str(resp)?;
    get_latest_from_json(&json_resp)
}

fn generate_notice(name: &str, current_version: &str, latest_version: &str) -> String {
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
    let mut border_line = String::from("\n───────────────────────────────────────────────────────");
    let extension = "─";
    for _ in 0..name.len() {
        border_line.push_str(extension);
    }
    border_line.push('\n');
    format!(
        "{}
    {}
    {}
    {}
    {}",
        border_line, line_1, line_2, line_3, border_line
    )
}

fn print_notice(name: &str, current_version: &str, latest_version: &str) {
    print!("{}", generate_notice(name, current_version, latest_version));
}

#[derive(Deserialize, Serialize)]
struct Config {
    last_checked: DateTime<Utc>,
}

fn get_app_name(name: &str) -> String {
    let mut app_name: String = String::from(name);
    app_name.push_str("-update-notifier");
    #[cfg(test)]
    app_name.push_str("-test");
    app_name
}

fn update_time(date_time: DateTime<Utc>, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        last_checked: date_time,
    };
    let config_store = Configstore::new(&get_app_name(name), AppUI::CommandLine)?;
    config_store.set("config", config)?;
    Ok(())
}

fn compare_with_latest(
    name: &str,
    current_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let latest_version = get_latest_version(name)?;
    if latest_version != current_version {
        print_notice(name, current_version, &latest_version);
    }
    let date_time = Utc::now();
    update_time(date_time, name)
}

/// Checks if there exists an update by checking against crates.io and notifies the user by printing to stdout
///
/// # Arguments
///
///   * `name` -  The name of the crate, you can use `env!("CARGO_PKG_NAME")`
///   * `current_version` - The version of the CLI, use `env!("CARGO_PKG_VERSION")`
///   * `interval` - Duration representing the interval.
///
/// # Examples
///
/// ```
/// use update_notifier::check_version;
///
/// check_version(
///   env!("CARGO_PKG_NAME"),
///   env!("CARGO_PKG_VERSION"),
///   std::time::Duration::from_secs(0),
///   ).ok();
//
///```
///
/// # Errors
///
/// Could error either if your plateform does not have a config directory or if an the crate name is not in the registry
pub fn check_version(
    name: &str,
    current_version: &str,
    interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let date_time_now = Utc::now();
    let config_store = Configstore::new(&get_app_name(name), AppUI::CommandLine)?;
    match config_store.get::<Config>("config") {
        Ok(config) => {
            let prev_time = config.last_checked;
            let duration_interval = chrono::Duration::from_std(interval)?;
            if date_time_now.signed_duration_since(prev_time) >= duration_interval {
                compare_with_latest(name, current_version)
            } else {
                Ok(())
            }
        }
        Err(_) => compare_with_latest(name, current_version),
    }
}

#[cfg(test)]
mod tests;
