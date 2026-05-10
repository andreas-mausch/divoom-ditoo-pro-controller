# Development

## Build

### macOS

```bash
make build
```

Compiles the Rust binary, compiles the Objective-C Bluetooth bridge (`src/platform/macos/bluetooth.m`) via the `cc` crate, links `IOBluetooth.framework` and `Foundation.framework`, and applies an ad-hoc code signature with the `com.apple.security.device.bluetooth` entitlement.

If you edit `bluetooth.m`, force a full recompile before testing:

```bash
cargo clean -p divoom-ditoo-pro-controller
make build
```

### Linux

```bash
cargo build --release
```

Requires `libbluetooth-dev` (BlueZ headers).

## Run tests

```bash
cargo test --all
# or via make:
make test
```

## Run a command during development

```bash
make run ARGS="send b1-21-81-10-b0-4e get-settings"
make run ARGS="send b1-21-81-10-b0-4e set-channel 0"
make run ARGS="send b1-21-81-10-b0-4e animation images/witch.divoom16"
```

`make run` rebuilds and re-signs before running.

## Toolchain

Tested with Rust stable. The CI matrix builds on Ubuntu (stable) and macOS (stable).

## Maintenance

List outdated dependencies:

```bash
cargo outdated
```

Upgrade:

```bash
cargo upgrade --incompatible allow
cargo update
```

Format and lint:

```bash
cargo +nightly fmt
cargo clippy --all-targets --all-features -- --deny warnings
```
