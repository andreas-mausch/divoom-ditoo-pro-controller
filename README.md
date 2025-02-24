# bluetooth-serial-port deprecated, Node 23 issue

The npm dependency `nan` currently does not support Node 23 (which is out 4 months already):
[Support Node 23 #979](https://github.com/nodejs/nan/pull/979)

`bluetooth-serial-port` requires `nan`.
Note that `bluetooth-serial-port` itself is deprecated now, and I couldn't find a working fork.

# Configure your device' MAC address

```bash
echo 'MAC_ADDRESS=11:22:33:44:55:66' > .env
```

# Run

```bash
npm start
```

Note: This script currently only works for 16x16 pixel images which have their full palette defined in the first frame.

# Show debug information about image

```bash
npm run debug-image ../images/bunny.divoom16
```

# Convert divoom16 image to .gif

```bash
npm run convert-divoom16-to-gif ../images/bunny.divoom16 ./output.gif
```
