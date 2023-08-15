import { BluetoothSerialPort } from "bluetooth-serial-port";
import { sprintf } from "sprintf-js";

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

const createNetworkPacketsFromImage = image => {
  const SPP_APP_NEW_GIF_CMD2020 = 139;
  const packets = [];
  packets.push(createNetworkPacket(SPP_APP_NEW_GIF_CMD2020, Buffer.from("00b4010000", "hex")));

  let bytesLeft = image.length;
  while (bytesLeft > 0) {
    const size = Math.min(bytesLeft, 256);
    const buffer = Buffer.alloc(7 + size);
    buffer.writeUInt8(1, 0);
    buffer.writeUint16LE(image.length, 1);
    buffer.writeUint16LE(0, 3);
    buffer.writeUint16LE(packets.length - 1, 5);
    image.copy(buffer, 7, image.length - bytesLeft, image.length - bytesLeft + size);
    packets.push(createNetworkPacket(SPP_APP_NEW_GIF_CMD2020, buffer));
    bytesLeft -= size;
  }

  return packets;
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

const send = async (macAddress, image) => {
  const port = new BluetoothSerialPort();

  const channel = await findSerialPortChannelAsync(port, macAddress);
  console.log("found serial port channel:", channel);

  await connectAsync(port, macAddress, channel);
  console.log("connected");

  const packets = createNetworkPacketsFromImage(image);

  for (const [index, packet] of packets.entries()) {
    await writeAsync(port, packet);
    console.log(sprintf("Sent packet %d/%d (%d bytes)", index + 1, packets.length, packet.length));
  }

  port.close();
};

export { send };
