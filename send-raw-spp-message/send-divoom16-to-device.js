import { BluetoothSerialPort } from "bluetooth-serial-port";

const checksum = (buffer) => {
  let sum = 0;
  for (let i = 1; i <= buffer.length - 3; i++) {
    sum += buffer.readUInt8(i) & 255;
    sum = sum & 65535;
  }
  return sum;
}

const createNetworkPacket = (command, payload) => {
  const buffer = Buffer.alloc(payload.length + 7);
  buffer.writeUInt8(1, 0);
  buffer.writeUInt16LE(buffer.length - 4, 1);
  buffer.writeUInt8(command, 3);
  payload.copy(buffer, 4);
  buffer.writeUInt16LE(checksum(buffer), buffer.length - 3);
  buffer.writeUInt8(2, buffer.length - 1);
  return buffer;
}

const findSerialPortChannelAsync = (port, address) =>
  new Promise((resolve, reject) => {
    port.findSerialPortChannel(address, resolve, () => reject("Could not find serial port channel"));
  });

const connectAsync = (port, address, channel) =>
  new Promise((resolve, reject) => {
    port.connect(address, channel, resolve, reject);
  });

const writeAsync = (port, buffer) =>
  new Promise((resolve, reject) => {
    port.write(buffer, (err, bytesWritten) => {
      if (err) {
        reject(err);
      } else {
        resolve(bytesWritten);
      }
    });
  });

const send = async (macAddress) => {
  const port = new BluetoothSerialPort();

  const channel = await findSerialPortChannelAsync(port, macAddress);
  console.log("found serial port channel:", channel);

  await connectAsync(port, macAddress, channel);
  console.log("connected");

  await writeAsync(port, createNetworkPacket(139, Buffer.from("00b4010000", "hex")));
  console.log("Sent command 1");

  await writeAsync(port, createNetworkPacket(139, Buffer.from("01b40100000000aa7f007f0000080000009b9b9b36055e926dc4e0c3c38f00ffcc8989a13d3d400200929400000048d22600012049daa600002449db2600902449db240080364e9a20004844729280040054b29300000044929c24000920499a120000206d920000002469136003c07f69d36f1b40024892001b0000401000c0009000009000aa67007f000100009000920424000048d22600482049da2624002449db2600902449db240080364e9a20004044729212200054b29300000044929c24004022499a800400206d920000002469136003c07f69d36f1b00904892001b0000401000c0000024000024aa67007f000100090024920400000048d22600092249daa60000", "hex")));
  console.log("Sent command 2");

  await writeAsync(port, createNetworkPacket(139, Buffer.from("01b401000001002449db2600902449db240080364e9a20000044729290000054b29300000044929c24000120499a002400206d920000002469136003c07f69d36f1b01004892003b0000401000c0090000090000aa67007f000100400200929400000048d22600402249da2624002449db2600902449db240080364e9a20000144729200240054b29300000044929c24000920499a800400206d920000002469136003c07f69d36f1b09004892001b0000401000c0400200400200", "hex")));
  console.log("Sent command 3");

  port.close();
};

export { send };
