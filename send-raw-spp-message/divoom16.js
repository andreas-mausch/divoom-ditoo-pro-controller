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

const loadImage = (imageBuffer) => {
  const firstFrameHeader = frameHeader(imageBuffer, 0);
  const bitsPerPixel = Math.ceil(Math.log2(firstFrameHeader.colorCount));

  const palette = [];
  for (let i = 0; i < firstFrameHeader.colorCount; i++) {
    const red = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + i * 3);
    const green = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + 1 + i * 3);
    const blue = imageBuffer.readUInt8(FRAME_HEADER_LENGTH + 2 + i * 3);
    palette.push({ red, green, blue });
  }

  const imageWidth = 16;
  const imageHeight = 16;

  const frames = [];
  let frameOffset = 0;

  for (let frame = 0; imageBuffer.length >= frameOffset + imageWidth * imageHeight * bitsPerPixel / BITS_PER_BYTE; frame++) {
    const header = frameHeader(imageBuffer, frameOffset);
    const pixelsOffset = frameOffset + FRAME_HEADER_LENGTH + header.colorCount * 3;

    const frameBuffer = Buffer.alloc(imageWidth * imageHeight * 3);

    for (let y = 0; y < imageHeight; y++) {
      for (let x = 0; x < imageWidth; x++) {
        const index = y * imageWidth + x;
        const startingBit = pixelsOffset * BITS_PER_BYTE + index * bitsPerPixel;
        const color = readBitsFromBuffer(imageBuffer, startingBit, bitsPerPixel);
        frameBuffer.writeUInt8(palette[color].red, index * 3 + 0);
        frameBuffer.writeUInt8(palette[color].green, index * 3 + 1);
        frameBuffer.writeUInt8(palette[color].blue, index * 3 + 2);
      }
    }

    frames.push({
      buffer: frameBuffer,
      timeInMilliseconds: header.timeInMilliseconds
    });

    frameOffset = pixelsOffset + imageWidth * imageHeight * bitsPerPixel / BITS_PER_BYTE;
  }

  return {
    width: imageWidth,
    height: imageHeight,
    palette,
    bitsPerPixel,
    frames
  };
}

export { loadImage };
