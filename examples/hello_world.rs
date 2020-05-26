use update_notifier::check_version;

fn main() {
    // You can use .ok() if you want to ignore the error and let check_version fail silently without affecting your CLI
    check_version(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        std::time::Duration::from_secs(0),
    )
    .ok();
    println!("Hello world");
}
