import {
  IGrid,
  Grid,
  BebopData,
  GridOpcode,
  IPixel,
  Pixel,
  IBebopData,
  ProtocolVersion,
  PixelOpcode,
  DeltaGridOpcode,
  DeltaGrid,
} from "./schemas";
import WebSocket from "ws";
// helper function: log message to screen
// function log(msg: string) {

//     document.getElementById('log').textContent += msg + '\n';
// }

// setup websocket with callbacks
function delay(ms: number) {
  return new Promise((resolve: any) => setTimeout(resolve, ms));
}
var ws = new WebSocket("ws://localhost:8080/");
ws.binaryType = "arraybuffer";
ws.onopen = async function () {
  console.log("CONNECT");
  let pixel = { x: 0, y: 0, color: { red: 255, green: 0, blue: 0 } };
  let encoded_pixel = new Uint8Array(Pixel.encode(pixel));
  let bebop_data = {
    protocolVersion: ProtocolVersion,
    opcode: PixelOpcode,
    encodedData: encoded_pixel,
  };
  let encoded_data = BebopData.encode(bebop_data);
  console.log(encoded_data);
  await delay(1000);
  ws.send(encoded_data);
};
ws.onclose = function () {
  console.log("DISCONNECT");
};
ws.onmessage = function (event) {
  console.log("Received data:");
  const start = new Date().getTime();
  let binary_data = new Uint8Array(event.data as ArrayBuffer);
  let bebop_data = BebopData.decode(binary_data);
  console.log(bebop_data.opcode);
  let struct_data = null;
  switch (bebop_data.opcode) {
    case GridOpcode:
      struct_data = Grid.decode(bebop_data.encodedData);
      break;

    case PixelOpcode:
      struct_data = Pixel.decode(bebop_data.encodedData);
      break;

    case DeltaGridOpcode:
      struct_data = DeltaGrid.decode(bebop_data.encodedData);
      break;

    default:
      break;
  }
  console.log("Decoded the data.");
  let elapsed = new Date().getTime() - start;
  console.log(elapsed);
};
