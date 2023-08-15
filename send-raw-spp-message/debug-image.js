import fullcolor from "fullcolor";
import { readFileSync } from "fs";
import { sprintf } from "sprintf-js";

const BITS_PER_BYTE = 8;
const FRAME_HEADER_LENGTH = 7;

const readBitsFromByte = (byte, startingBit, bitCount) => {
  const bitmask = Math.pow(2, bitCount) - 1;
  const maximumByteBitmask = Math.pow(2, BITS_PER_BYTE) - 1;
  const bitShift = startingBit % BITS_PER_BYTE;

  return ((byte & (bitmask << bitShift)) & maximumByteBitmask) >> bitShift;
};

const readBitsFromBuffer = (buffer, startingBit, bitCount) => {
  let bitsLeftToRead = bitCount;
  let result = 0;

  while (bitsLeftToRead > 0) {
    const bitsRead = bitCount - bitsLeftToRead;
    const bitsToReadFromCurrentByte = Math.min(BITS_PER_BYTE - startingBit % BITS_PER_BYTE, bitsLeftToRead);
    result += readBitsFromByte(buffer.readUInt8(Math.floor((startingBit + bitsRead) / BITS_PER_BYTE)), (startingBit + bitsRead) % BITS_PER_BYTE, bitsToReadFromCurrentByte) << bitsRead;
    bitsLeftToRead -= bitsToReadFromCurrentByte;
  }

  return result;
};

const frameHeader = (buffer, offset) => ({
  length: buffer.readUInt16LE(offset + 1),
  timeInMilliseconds: buffer.readUInt16LE(offset + 3),
  resetPalette: buffer.readUInt8(offset + 5),
  colorCount: buffer.readUInt8(offset + 6)
});

const debugImage = (imageBuffer) => {
  console.log("Debug image information:");

  const firstFrameHeader = frameHeader(imageBuffer, 0);
  const bitsPerPixel = Math.ceil(Math.log2(firstFrameHeader.colorCount));
  console.log("header", firstFrameHeader);
  console.log("bitsPerPixel", bitsPerPixel);

  let palette = [];

  for (let i = 0; i < firstFrameHeader.colorCount; i++) {
    const red = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + i * 3);
    const green = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + 1 + i * 3);
    const blue = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + 2 + i * 3);

    const hexString = sprintf("#%02x%02x%02x", red, green, blue);
    console.log(sprintf("Color [%03d]: %s %s", i, hexString, fullcolor("████", hexString)));

    palette.push(hexString);
  }

  const pixelsStart = FRAME_HEADER_LENGTH + firstFrameHeader.colorCount * 3;
  const imageWidth = 16;
  const imageHeight = 16;

  for (let frame = 0; imageBuffer.length >= pixelsStart + (frame + 1) * imageWidth * imageHeight * bitsPerPixel / BITS_PER_BYTE; frame++) {
    const frameStart = pixelsStart + frame * imageWidth * imageHeight * bitsPerPixel / BITS_PER_BYTE;
    const framePixelStart = frameStart + frame * FRAME_HEADER_LENGTH;

    for (let y = 0; y < imageHeight; y++) {
      for (let x = 0; x < imageWidth; x++) {
        const index = y * imageWidth + x;
        const startingBit = framePixelStart * BITS_PER_BYTE + index * bitsPerPixel;
        const color = readBitsFromBuffer(imageBuffer, startingBit, bitsPerPixel);

        if (color >= palette.length) {
          console.log("ERROR", color);
        }
        process.stdout.write(fullcolor("██", palette[color]));
      }
      console.log();
    }
    console.log();
  }
}

debugImage(readFileSync("../images/witch.divoom16"));
debugImage(readFileSync("../images/bunny.divoom16"));
