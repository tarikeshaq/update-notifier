use ansi_term::Color::{Blue, Green, Red, Yellow};
use reqwest::blocking::Client;
use reqwest::header;
use serde_derive::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Unable to find crate")]
    CrateDoesNotExist,
    #[error("Versions Do not exist on crates.io")]
    VersionDoesNotExistCratesIO,
    #[error("Error while parsing json: {0}")]
    UnableToParseJson(String),
    #[error("Error recieved from crates.io: {0}")]
    CratesIOError(String),
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
            None => Err(ErrorKind::UnableToParseJson(
                "Versions array is empty".to_string(),
            ))?,
        }
    } else if let Some(errors) = &resp.errors {
        match errors.first() {
            Some(error) => Err(ErrorKind::CratesIOError(error.detail.clone()))?,
            None => Err(ErrorKind::UnableToParseJson(
                "No errors in the erros array".to_string(),
            ))?,
        }
    } else {
        Err(ErrorKind::UnableToParseJson(
            "Invalid json response, does not have versions or errors".to_string(),
        ))?
    }
}

fn get_latest_version(crate_name: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("Update-notifer (teshaq@mozilla.com)"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    let base_url = get_base_url();
    let url = format!("{}/{}/versions", base_url, crate_name);
    let json_resp = match client.get(&url).send()?.json() {
        Ok(resp) => resp,
        Err(e) => Err(ErrorKind::UnableToParseJson(e.to_string()))?,
    };
    get_latest_from_json(&json_resp)
}

fn print_notice(name: &str, current_version: &str, latest_version: &str) {
    println!();
    println!("───────────────────────────────────────────────────────");
    println!();
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
        Yellow.paint(format!("https://crates.io/crates/{}", name))
    );
    println!("{}", line_1);
    println!("{}", line_2);
    println!("{}", line_3);
    println!("");
    println!("───────────────────────────────────────────────────────");
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
use mockito;

#[cfg(not(test))]
const CRATES_IO_URL: &str = "https://crates.io";

fn get_base_url() -> String {
    #[cfg(not(test))]
    let url = format!("{}/api/v1/crates", CRATES_IO_URL);
    #[cfg(test)]
    let url = format!("{}/api/v1/crates", mockito::server_url());
    url
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    #[test]
    fn test_latest_version() {
        let m = mock("GET", "/api/v1/crates/asdev/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
        {"versions" : [
            {
                "id": 229435,
                "crate": "asdev",
                "num": "0.1.3"
            }
        ]}"#,
            )
            .create();
        let latest_version = get_latest_version("asdev").unwrap();
        m.expect(1).assert();
        assert_eq!(latest_version, "0.1.3")
    }

    #[test]
    fn test_no_crates_io_entry() {
        let m = mock(
            "GET",
            "/api/v1/crates/kefjhkajvcnklsajdfhwksajnceknc/versions",
        )
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"
        {"errors":[{"detail":"Not Found"}]}"#,
        )
        .create();
        let latest_version =
            get_latest_version("kefjhkajvcnklsajdfhwksajnceknc").expect_err("Should be an error");
        m.expect(1).assert();
        assert_eq!(
            latest_version.to_string(),
            ErrorKind::CratesIOError("Not Found".to_string()).to_string()
        );
    }

    #[test]
    fn test_same_version() {
        let m = mock("GET", "/api/v1/crates/asdev/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
        {"versions" : [
            {
                "id": 229435,
                "crate": "asdev",
                "num": "0.1.3"
            }
        ]}"#,
            )
            .create();
        check_version("asdev", "0.1.3").unwrap();
        m.expect(1).assert();
    }

    #[test]
    fn test_not_update_available() {
        let m = mock("GET", "/api/v1/crates/asdev/versions")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"
        {"versions" : [
            {
                "id": 229435,
                "crate": "asdev",
                "num": "0.1.3"
            }
        ]}"#,
            )
            .create();
        check_version("asdev", "0.1.2").unwrap();
        m.expect(1).assert();
    }
}
