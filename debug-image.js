import fullcolor from "fullcolor";
import { readFileSync } from "fs";
import { sprintf } from "sprintf-js";
import { loadImage } from "./divoom16.js";

const debugImage = buffer => {
  console.log("Debug image information:");

  const image = loadImage(buffer);
  console.log("Dimensions", image.width, image.height);
  console.log("Palette color count", image.palette.length);
  console.log("bitsPerPixel", image.bitsPerPixel);
  console.log("frames", image.frames.length);

  image.palette.forEach((color, index) => {
    const hexString = sprintf("#%02x%02x%02x", color.red, color.green, color.blue);
    console.log(sprintf("Color [%03d]: %s %s", index, hexString, fullcolor("████", hexString)));
  })

  image.frames.forEach((frame, index) => {
    console.log(sprintf("Frame %d: %dms", index, frame.timeInMilliseconds));
    for (let y = 0; y < image.height; y++) {
      for (let x = 0; x < image.width; x++) {
        const index = y * image.width + x;
        const red = frame.buffer.readUInt8(index * 3 + 0);
        const green = frame.buffer.readUInt8(index * 3 + 1);
        const blue = frame.buffer.readUInt8(index * 3 + 2);
        const hexString = sprintf("#%02x%02x%02x", red, green, blue);
        process.stdout.write(fullcolor("██", hexString));
      }
      console.log();
    }
    console.log();
  });
}

debugImage(readFileSync(process.argv[2]));
