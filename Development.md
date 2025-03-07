# Two branches

I started with a JavaScript version, which can be found in the
[javascript branch](https://github.com/andreas-mausch/divoom-ditoo-pro-controller/tree/javascript).

It works well enough to change the image, and it can send SPP messages
to an already connected device by just using the MAC address.

Now I try to port the code to [Rust](https://github.com/andreas-mausch/divoom-ditoo-pro-controller/tree/rust).
Here I still need to re-connect every time I run the program.

These are my first steps in Bluetooth programming with Rust,
so please see this project as an experiment.

# Build and Test

## Requirements

- Rust
- cargo
- cargo-edit
- cargo-outdated
- clippy

I have compiled this repo with Rust 1.82.0 on Manjaro Linux.

Note that the dependency `bluetooth-serial-port` only works *on Linux/BlueZ*.

## Build

Just use the default `cargo` commands.

```bash
cargo build
```

## Run tests

```bash
cargo test --all
```

# Maintenance

## Update dependencies

`cargo update` only updates dependencies inside `Cargo.lock`.
To update your dependencies in the `Cargo.toml`, use
[cargo-edit](https://archlinux.org/packages/extra/x86_64/cargo-edit/) and
[cargo-outdated](https://archlinux.org/packages/extra/x86_64/cargo-outdated/).

List outdated dependencies:

```bash
cargo outdated
```

To update/upgrade dependencies, use this:

```bash
cargo upgrade --incompatible allow
cargo update
```

## Format code, fix warnings

```bash
cargo +nightly fmt
cargo check
cargo fix
cargo clippy --all-targets --all-features -- --deny warnings
```
