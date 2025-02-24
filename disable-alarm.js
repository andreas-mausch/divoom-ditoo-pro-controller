import "dotenv/config";
import { BluetoothSerialPort } from "bluetooth-serial-port";

const port = new BluetoothSerialPort();

port.findSerialPortChannel(process.env.MAC_ADDRESS, channel => {
  console.log("found serial port channel:", channel);
  port.connect(process.env.MAC_ADDRESS, channel, () => {
    console.log("connected");

    port.write(Buffer.from("010d00430000142e000200000028bc0002", "hex"), (err, bytesWritten) => {
      console.log("Sent command to disable alarm", err, bytesWritten);
    })

    port.on('data', function (buffer) {
      console.log("Received data: ", buffer.toString("hex"));
    });
  }, () => {
    console.log("cannot connect");
  });
}, () => {
  console.log("found nothing");
});
