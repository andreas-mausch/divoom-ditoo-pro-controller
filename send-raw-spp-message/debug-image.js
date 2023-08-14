import fullcolor from "fullcolor";
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
  colorCount: buffer.readUInt8(offset + 6)
});

const debugImage = (imageBuffer) => {
  console.log("Debug image information:");

  const firstFrameHeader = frameHeader(imageBuffer, 0);
  const bitsPerPixel = Math.ceil(Math.log2(firstFrameHeader.colorCount));
  console.log("colorCount", firstFrameHeader.colorCount);
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

const witch = "aa7f007f0000080000009b9b9b36055e926dc4e0c3c38f00ffcc8989a13d3d400200929400000048d22600012049daa600002449db2600902449db240080364e9a20004844729280040054b29300000044929c24000920499a120000206d920000002469136003c07f69d36f1b40024892001b0000401000c0009000009000aa67007f000100009000920424000048d22600482049da2624002449db2600902449db240080364e9a20004044729212200054b29300000044929c24004022499a800400206d920000002469136003c07f69d36f1b00904892001b0000401000c0000024000024aa67007f000100090024920400000048d22600092249daa600002449db2600902449db240080364e9a20000044729290000054b29300000044929c24000120499a002400206d920000002469136003c07f69d36f1b01004892003b0000401000c0090000090000aa67007f000100400200929400000048d22600402249da2624002449db2600902449db240080364e9a20000144729200240054b29300000044929c24000920499a800400206d920000002469136003c07f69d36f1b09004892001b0000401000c0400200400200";
debugImage(Buffer.from(witch, "hex"));

const image = "aaab00f401000cffffffcfe8feffa1d3e57db4ffe2f0ffc24c000000ffb0db8bbe74926319ce994762994a00101111111111110021221123121111112122112322111111212222232211001111222432230100111141452222010011112224222212001111220000201211111102060006121111140206050612114145210000701111118411227007191111110800009091111111110000a0a911888bbb0008909abbbbb888888bbbb888aa8700f40101000010111111111111002022112312111100212211232211111121222223221111111122243223110011114145222201001111222422220200111122000020120011140206000612114145020605061211118421000070191111110822709791111111110000a0a9111111110000909a11888bbb0008b0bbbbbbb888888bbbb888";
debugImage(Buffer.from(image, "hex"));
