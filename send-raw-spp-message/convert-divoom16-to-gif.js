import { GifFrame, GifUtil } from "gifwrap";
import { loadImage } from "./divoom16.js";
import { readFileSync } from "fs";

const image = loadImage(readFileSync(process.argv[2]));

const frames = image.frames.map(frame => {
  const gifFrame = new GifFrame(image.width, image.height, { delayCentisecs: 50 });

  for (let y = 0; y < image.height; y++) {
    for (let x = 0; x < image.width; x++) {
      const index = y * image.width + x;
      const red = frame.readUInt8(index * 3 + 0);
      const green = frame.readUInt8(index * 3 + 1);
      const blue = frame.readUInt8(index * 3 + 2);

      gifFrame.bitmap.data.writeUInt8(red, index * 4 + 0);
      gifFrame.bitmap.data.writeUInt8(green, index * 4 + 1);
      gifFrame.bitmap.data.writeUInt8(blue, index * 4 + 2);
      gifFrame.bitmap.data.writeUInt8(255, index * 4 + 3);
    }
  }
  return gifFrame;
});

await GifUtil.write(process.argv[3], frames);
