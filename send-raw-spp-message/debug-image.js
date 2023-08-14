import fullcolor from "fullcolor";
import { sprintf } from "sprintf-js";

const debugImage = (imageBuffer) => {
  console.log("Debug image information:");

  const colorCount = 8; // imageBuffer.readUInt8(6);
  console.log("colorCount", colorCount);

  let palette = [];

  for (let i = 0; i < colorCount; i++) {
    const red = imageBuffer.readUInt8(7 + i * 3);
    const green = imageBuffer.readUInt8(8 + i * 3);
    const blue = imageBuffer.readUInt8(9 + i * 3);

    const hexString = sprintf("#%02x%02x%02x", red, green, blue);
    console.log(sprintf("Color [%03d]: %s %s", i, hexString, fullcolor("████", hexString)));

    palette.push(hexString);
  }

  const pixelsStart = 7 + colorCount * 3;

  for (let frame = 0; imageBuffer.length >= pixelsStart + (frame + 1) * 16 * 16 * 3 / 8; frame++) {
    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const index = y * 16 + x;
        const startingBit = pixelsStart * 8 + frame * (16 * 16 * 3 + 56) + index * 3;
        const startingByteIndex = Math.floor(startingBit / 8);
        const firstByte = imageBuffer.readUInt8(startingByteIndex);

        let color = 0;
        if (startingBit % 8 == 0) {
          color = firstByte & 0x7;
        }
        else if (startingBit % 8 == 1) {
          color = (firstByte & (0x7 << 1)) >> 1;
        }
        else if (startingBit % 8 == 2) {
          color = (firstByte & (0x7 << 2)) >> 2;
        }
        else if (startingBit % 8 == 3) {
          color = (firstByte & (0x7 << 3)) >> 3;
        }
        else if (startingBit % 8 == 4) {
          color = (firstByte & (0x7 << 4)) >> 4;
        }
        else if (startingBit % 8 == 5) {
          color = (firstByte & (0x7 << 5)) >> 5;
        }
        else if (startingBit % 8 == 6) {
          const secondByte = imageBuffer.readUInt8(startingByteIndex + 1);
          color = ((firstByte & (0x3 << 6)) >> 6) + ((secondByte & 0x1) << 2);
        }
        else if (startingBit % 8 == 7) {
          const secondByte = imageBuffer.readUInt8(startingByteIndex + 1);
          color = ((firstByte & (0x1 << 7)) >> 7) + ((secondByte & 0x3) << 1);
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
// debugImage(Buffer.from(image, "hex"));
