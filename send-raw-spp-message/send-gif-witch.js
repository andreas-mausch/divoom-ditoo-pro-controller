import { BluetoothSerialPort } from "bluetooth-serial-port";

const MAC_ADDRESS = "11:22:33:44:55:66";
const port = new BluetoothSerialPort();

port.findSerialPortChannel(MAC_ADDRESS, channel => {
  console.log("found serial port channel:", channel);
  port.connect(MAC_ADDRESS, channel, () => {
    console.log("connected");

    port.write(Buffer.from("0108008b00b4010000480102", "hex"), (err, bytesWritten) => {
      console.log("Sent command 1", err, bytesWritten);
    })

    port.write(Buffer.from("010a018b01b40100000000aa7f007f0000080000009b9b9b36055e926dc4e0c3c38f00ffcc8989a13d3d400200929400000048d22600012049daa600002449db2600902449db240080364e9a20004844729280040054b29300000044929c24000920499a120000206d920000002469136003c07f69d36f1b40024892001b0000401000c0009000009000aa67007f000100009000920424000048d22600482049da2624002449db2600902449db240080364e9a20004044729212200054b29300000044929c24004022499a800400206d920000002469136003c07f69d36f1b00904892001b0000401000c0000024000024aa67007f000100090024920400000048d22600092249daa60000154602", "hex"), (err, bytesWritten) => {
      console.log("Sent command 2", err, bytesWritten);
    })

    port.write(Buffer.from("01be008b01b401000001002449db2600902449db240080364e9a20000044729290000054b29300000044929c24000120499a002400206d920000002469136003c07f69d36f1b01004892003b0000401000c0090000090000aa67007f000100400200929400000048d22600402249da2624002449db2600902449db240080364e9a20000144729200240054b29300000044929c24000920499a800400206d920000002469136003c07f69d36f1b09004892001b0000401000c0400200400200cb2d02", "hex"), (err, bytesWritten) => {
      console.log("Sent command 3", err, bytesWritten);
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
