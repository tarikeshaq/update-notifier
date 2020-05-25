# Update-notifier [![crates.io](https://img.shields.io/crates/v/update-notifier)](https://crates.io/crates/update-notifier) [![status](https://github.com/tarikeshaq/update-notifier/workflows/Rust/badge.svg)](https://github.com/tarikeshaq/update-notifier/actions)

Update-notifier will notify your Rust CLI's users if there is an update available! You also have the freedom of setting the interval at which it will notify your users.
This was built based on NPM's [update-notifier](https://www.npmjs.com/package/update-notifier)

* [API Documentation](https://docs.rs/update-notifier/)
* Cargo package: [update-notifier](https://crates.io/crates/update-notifier)

## Usage

To use `update-notifier`, add this to your `Cargo.toml`:

```toml
[dependencies]
update-notifier = "0.1"
```

### How to use

update-notifier::check_version takes the name and current version of your crate, along with an interval.
It will print directly to stdout if the interval has been exceeded and an update is available

```rust,ignore
use update_notifier::check_version;
fn main() {
    // Will notify users in one day intervals if an update is available
    check_version(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), Duration::from_secs(60 * 60 * 24));
}
```


### Example
![Example Image](https://i.ibb.co/jLwXqFv/Screen-Shot-2020-05-25-at-11-27-01-AM.png)

## Contributing

All contributions are welcome, feel free to file an issue or even a pull-request 🤝

## License

This project is licensed under the [Mozilla Public License 2.0](https://github.com/tarikeshaq/update-notifier/blob/master/LICENSE)
