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
