"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const schemas_1 = require("./schemas");
const ws_1 = __importDefault(require("ws"));
// helper function: log message to screen
// function log(msg: string) {
//     document.getElementById('log').textContent += msg + '\n';
// }
// setup websocket with callbacks
function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}
var ws = new ws_1.default("ws://localhost:8080/");
ws.binaryType = "arraybuffer";
ws.onopen = async function () {
    console.log("CONNECT");
    let pixel = { x: 0, y: 0, color: { red: 255, green: 0, blue: 0 } };
    let encoded_pixel = new Uint8Array(schemas_1.Pixel.encode(pixel));
    let bebop_data = {
        protocolVersion: schemas_1.ProtocolVersion,
        opcode: schemas_1.PixelOpcode,
        encodedData: encoded_pixel,
    };
    let encoded_data = schemas_1.BebopData.encode(bebop_data);
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
    let binary_data = new Uint8Array(event.data);
    let bebop_data = schemas_1.BebopData.decode(binary_data);
    console.log(bebop_data.opcode);
    let struct_data = null;
    switch (bebop_data.opcode) {
        case schemas_1.GridOpcode:
            struct_data = schemas_1.Grid.decode(bebop_data.encodedData);
            break;
        case schemas_1.PixelOpcode:
            struct_data = schemas_1.Pixel.decode(bebop_data.encodedData);
            break;
        case schemas_1.DeltaGridOpcode:
            struct_data = schemas_1.DeltaGrid.decode(bebop_data.encodedData);
            break;
        default:
            break;
    }
    console.log("Decoded the data.");
    let elapsed = new Date().getTime() - start;
    console.log(elapsed);
};
