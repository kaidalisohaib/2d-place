import { IGrid, Grid, BebopData, GridOpcode, IPixel, Pixel, IBebopData, ProtocolVersion, PixelOpcode } from "./schemas";
import WebSocket from "ws";
// helper function: log message to screen
// function log(msg: string) {

//     document.getElementById('log').textContent += msg + '\n';
// }

// setup websocket with callbacks

var ws = new WebSocket('ws://localhost:8080/');
ws.binaryType = 'arraybuffer'
ws.onopen = function () {
    console.log('CONNECT');
    let pixel = { x: 13, y: 17, color: { red: 0, green: 0, blue: 0 } };
    let encoded_pixel = Pixel.encode(pixel);
    console.log(encoded_pixel)
    let bebop_data = { protocolVersion: ProtocolVersion, opcode: PixelOpcode, encodedData: encoded_pixel };
    let encoded_data = BebopData.encode(bebop_data);
    console.log(encoded_data)
    console.log(BebopData.decode(encoded_data))
    ws.send(encoded_data);
};
ws.onclose = function () {
    console.log('DISCONNECT');
};
ws.onmessage = function (event) {
    console.log("Received data:");
    // const start = new Date().getTime();
    let binary_data = new Uint8Array(event.data as ArrayBuffer);
    let bebop_data = BebopData.decode(binary_data);
    console.log(bebop_data.opcode);
    // let struct_data = null;
    // switch (bebop_data.opcode) {
    //     case GridOpcode:
    //         struct_data = Grid.decode(bebop_data.encodedData);
    //         console.log(struct_data.rows.length);
    //         break;

    //     default:
    //         break;
    // }
    // console.log("Decoded the data.");
    // let elapsed = new Date().getTime() - start;
    // console.log(elapsed);
};
