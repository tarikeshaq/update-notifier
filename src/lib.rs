use ansi_term::Color::{Blue, Green, Red, Yellow};
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::Value;
use thiserror;
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Unable to find crate")]
    CrateDoesNotExist,
    #[error("Versions Do not exist on crates.io")]
    VersionDoesNotExistCratesIO,
    #[error("Unable to parse json")]
    UnableToParseJson,
}

fn get_latest_from_json(resp: &Value) -> std::result::Result<String, Box<dyn std::error::Error>> {
    if let Some(obj) = resp.as_object() {
        if let Some(versions) = obj.get("versions") {
            if let Some(versions_arr) = versions.as_array() {
                if let Some(val) = versions_arr.first() {
                    if let Some(version) = val.as_object() {
                        if let Some(num) = version.get("num") {
                            if let Some(res) = num.as_str() {
                                return Ok(res.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(ErrorKind::CrateDoesNotExist)?
    }
    Err(ErrorKind::UnableToParseJson)?
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
    let json_resp = client.get(&url).send()?.json()?;
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
            ErrorKind::CrateDoesNotExist.to_string()
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
