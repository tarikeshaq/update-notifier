use super::*;
use chrono::Datelike;
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
        ErrorKind::RegistryError("Not Found".to_string()).to_string()
    );
}

#[test]
fn test_same_version() {
    let m = mock("GET", "/api/v1/crates/sameVersion/versions")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "sameVersion",
                    "num": "0.1.3"
                }
            ]}"#,
        )
        .create();
    check_version("sameVersion", "0.1.3", Duration::from_nanos(0)).unwrap();
    m.expect(1).assert();
}

#[test]
fn test_not_update_available() {
    let m = mock("GET", "/api/v1/crates/noUpdate/versions")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "noUpdate",
                    "num": "0.1.3"
                }
            ]}"#,
        )
        .create();
    check_version("noUpdate", "0.1.2", Duration::from_secs(0)).unwrap();
    m.expect(1).assert();
}

#[test]
fn test_output() {
    assert_eq!(generate_notice("asdev", "0.1.2", "0.1.3"), "\n────────────────────────────────────────────────────────────\n\n    A new version of \u{1b}[1;32masdev\u{1b}[0m is available! \u{1b}[1;31m0.1.2\u{1b}[0m → \u{1b}[1;32m0.1.3\u{1b}[0m\n    Use `\u{1b}[1;34mcargo install asdev\u{1b}[0m` to install version \u{1b}[1;32m0.1.3\u{1b}[0m\n    Check \u{1b}[33mhttps://crates.io/crates/asdev\u{1b}[0m for more details\n    \n────────────────────────────────────────────────────────────\n");
}

#[test]
fn test_interval_not_exceeded() {
    let m = mock("GET", "/api/v1/crates/notExceeded/versions")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "notExceeded",
                    "num": "0.1.3"
                }
            ]}"#,
        )
        .create();
    check_version("notExceeded", "0.1.2", Duration::from_secs(0)).unwrap();
    check_version("notExceeded", "0.1.2", Duration::from_secs(1000 * 1000)).unwrap();
    m.expect(1).assert()
}

#[test]
fn test_interval_exceeded() {
    let m = mock("GET", "/api/v1/crates/intervalExceeded/versions")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"
            {"versions" : [
                {
                    "id": 229435,
                    "crate": "intervalExceeded",
                    "num": "0.1.3"
                }
            ]}"#,
        )
        .create();

    update_time(Utc::now().with_year(1999).unwrap(), "intervalExceeded").unwrap();
    check_version("intervalExceeded", "0.1.2", Duration::from_secs(1000)).unwrap();
    m.expect(1).assert()
}
