# Goal

I am trying to write an app that can do basic stuff on a Divoom Ditoo Pro,
like changing the image.

The original app from the vendor is proprietary.
The protocol however is basic Bluetooth, which can be reverse-engineered.

# Blog post

Bluetooth Speaker with 16x16 Display (Divoom Ditoo Pro):
<https://andreas-mausch.de/blog/2023-08-14-divoom-ditoo-pro/>

# How to run

## Find your device

```shell-session
$ cargo run list-devices
[2025-02-24T17:00:00Z INFO  divoom_ditoo_pro_controller] Scanning bluetooth devices for 20s
// ...
[2025-02-24T17:01:01Z INFO  divoom_ditoo_pro_controller] Found bluetooth devices [BtDevice { name: "DitooPro-Audio", addr: 11:22:33:44:55:66 }]
// ...
```

Look for a line containing `DitooPro-Light` or `DitooPro-Audio` and remember the MAC address.

## Send commads

Then, run the second command:

```shell-session
$ cargo run send 11:22:33:44:55:66 alarm off
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller] Disabling alarm..
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller] Connecting to device with MAC address 11:22:33:44:55:66
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller] Connection successful, socket over RFCOMM/SPP acquired
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller] Sending packet 1/1..
[2025-02-24T17:20:00Z DEBUG divoom_ditoo_pro_controller]   010d004300000d25000000000064e60002
[2025-02-24T17:20:00Z INFO  divoom_ditoo_pro_controller]   Wrote 17/17 bytes (100%)
```

Replace the MAC address by the one from the command above.

More examples:

```bash
cargo run debug-image ./images/witch.divoom16
cargo run send 11:22:33:44:55:66 animation ./images/witch.divoom16
cargo run send 11:22:33:44:55:66 set-date-time 2025-03-25T21:22:59
cargo run convert to-gif ./images/witch.divoom16 ./out.gif
cargo run convert to-divoom16 ./images/witch.gif ./out.divoom16
```

# Bluetooth adapter

Please note the Bluetooth adapter is chosen automatically.
There is currently no way to configure it.

# Development

See [Development.md](Development.md).

# Protocol

- Protocol introduction:
  <https://docin.divoom-gz.com/web/#/5/146>
- App new send gif cmd (0x8b):
  <https://docin.divoom-gz.com/web/#/5/293>
- Example images from the Pixoo64 to decode:
  <https://github.com/Grayda/pixoo64_example_images>
- node-divoom-timebox-evo: PROTOCOL
  <https://github.com/RomRider/node-divoom-timebox-evo/blob/0.3.0/PROTOCOL.md>

# Pixel art

- <https://pixeljoint.com/pixels/new_icons.asp?search=&dimo=%3D&dim=16&colorso=%3E%3D&colors=2&tran=&anim=&iso=&av=&owner=&d=&dosearch=1&ob=search&action=search>
