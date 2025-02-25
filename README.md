# Goal

I am trying to write an app that can do basic stuff on a Divoom Ditoo Pro,
like changing the image.

The original app from the vendor is proprietary.
The protocol however is basic Bluetooth, which can be reverse-engineered.

# Requirements

- Rust
- cargo
- cargo-edit
- cargo-outdated
- clippy

I have compiled this repo with Rust 1.82.0.

# Blog post

Bluetooth Speaker with 16x16 Display (Divoom Ditoo Pro):
<https://andreas-mausch.de/blog/2023-08-14-divoom-ditoo-pro/>

# Two branches

I started with a JavaScript version, which can be found in the
[javascript branch](https://github.com/andreas-mausch/divoom-ditoo-pro-controller/tree/javascript).

It works well enough to change the image, and it can send SPP messages
to an already connected device by just using the MAC address.

Now I try to port the code to [Rust](https://github.com/andreas-mausch/divoom-ditoo-pro-controller/tree/rust).
Here I still need to re-connect every time I run the program.

These are my first steps in Bluetooth programming with Rust,
so please see this project as an experiment.

# How to run

```shell-session
$ cargo run list-devices
[2025-02-24T17:00:00Z INFO  divoom_ditoo_pro_controller] Scanning bluetooth devices for 20s
// ...
[2025-02-24T17:01:01Z INFO  divoom_ditoo_pro_controller] Found bluetooth devices [BtDevice { name: "DitooPro-Audio", addr: 11:22:33:44:55:66 }]
// ...
```

Look for a line containing `DitooPro-Light` or `DitooPro-Audio` and remember the MAC address.

Then, run the second command:

```shell-session
$ cargo run send 11:22:33:44:55:66 alert off
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller] Connecting to device with MAC address 11:22:33:44:55:66
[2025-02-24T17:20:01Z INFO  divoom_ditoo_pro_controller] Connection successful, socket over RFCOMM/SPP acquired
[2025-02-24T17:20:02Z INFO  divoom_ditoo_pro_controller] Sending message..
[2025-02-24T17:20:03Z INFO  divoom_ditoo_pro_controller] Wrote 17/17 bytes (100%)
```

Replace the MAC address by the one from the command above.

Note: This will always send a message to disable the alarm, custom messages are not implemented yet.
Development is still in progress here.

# Bluetooth adapter

Please note the Bluetooth adapter is chosen automatically.
There is currently no way to configure it.

# Update dependencies

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

# Format code, fix warnings

```bash
cargo fmt
cargo check
cargo fix
cargo clippy --all-targets --all-features -- --deny warnings
```

# Protocol

- Protocol introduction:
  <https://docin.divoom-gz.com/web/#/5/146>
- App new send gif cmd (0x8b):
  <https://docin.divoom-gz.com/web/#/5/293>
- Example images from the Pixoo64 to decode:
  <https://github.com/Grayda/pixoo64_example_images>
- node-divoom-timebox-evo: PROTOCOL
  <https://github.com/RomRider/node-divoom-timebox-evo/blob/0.3.0/PROTOCOL.md>
