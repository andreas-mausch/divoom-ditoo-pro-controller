# Goal

I am trying to write an app that can do basic stuff on a Divoom Ditoo Pro,
like changing the image.

The original app from the vendor is proprietary.
The protocol however is basic Bluetooth, which can be reverse-engineered.

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

TODO
