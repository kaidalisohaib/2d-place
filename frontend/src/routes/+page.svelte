<script lang="ts">
	import { onMount } from 'svelte';
	import {
		BebopData,
		DeltaGrid,
		DeltaGridOpcode,
		Grid,
		GridOpcode,
		Pixel,
		PixelOpcode,
		type IDeltaGrid,
		type IGrid,
		type IPixel
	} from '$lib/schemas/schemas';

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null;

	let initialGrid: IGrid | null;

	let content: string = '';
	onMount(() => {
		let socket = new WebSocket('ws://127.0.0.1:8080');
		socket.binaryType = 'arraybuffer';
		socket.onopen = (e) => {
			content += '\nCONNECTED';
		};
		socket.onclose = (_e) => {
			content += '\nCLOSED';
		};
		socket.onmessage = (e) => {
			let binary_data: Uint8Array = new Uint8Array(e.data);
			let bebop_data = BebopData.decode(binary_data);
			console.log(bebop_data.opcode);
			let structData: IGrid | IPixel | IDeltaGrid | null = null;
			switch (bebop_data.opcode) {
				case GridOpcode:
					structData = Grid.decode(bebop_data.encodedData);
					initialGrid = structData;
					break;

				case PixelOpcode:
					structData = Pixel.decode(bebop_data.encodedData);
					break;

				case DeltaGridOpcode:
					structData = DeltaGrid.decode(bebop_data.encodedData);
					if (initialGrid) {
						setInitialImage(initialGrid, structData);
					}

					break;

				default:
					break;
			}
			console.log(structData);
			content += '\nRECEIVED MESSAGE';
		};
		if (canvas.getContext) {
			ctx = canvas.getContext('2d');
		}
	});

	function setInitialImage(grid: IGrid, deltaGird: IDeltaGrid) {
		if (!ctx) {
			return;
		}
		let initailImageData: ImageData = ctx.createImageData(500, 500);
		let redIdx: number = 0;
		grid.rows.forEach((row, rowIdx) => {
			row.pixels.forEach((color, pixelIdx) => {
				initailImageData.data[redIdx] = color.red;
				initailImageData.data[redIdx + 1] = color.green;
				initailImageData.data[redIdx + 2] = color.blue;
				initailImageData.data[redIdx + 3] = 255;
				redIdx += 4;
			});
		});
		deltaGird.delta.forEach((pixel) => {
			redIdx = pixel.y * 500 * 4 + pixel.x * 4;
			initailImageData.data[redIdx] = pixel.color.red;
			initailImageData.data[redIdx + 1] = pixel.color.green;
			initailImageData.data[redIdx + 2] = pixel.color.blue;
			initailImageData.data[redIdx + 3] = 255;
		});
		console.log(initailImageData);
		ctx.putImageData(initailImageData, 0, 0);
		initialGrid = null;
	}

	function setPixel(newPixel: IPixel) {
		if (!ctx) {
			return;
		}
		ctx.fillStyle = `rgb(${newPixel.color.red},${newPixel.color.green},${newPixel.color.blue})`;
		ctx.fillRect(newPixel.x, newPixel.y, 1, 1);
	}
</script>

<h1>Welcome to SvelteKit</h1>
<pre>{content}</pre>
<canvas bind:this={canvas} width="500" height="500" />

<style>
	canvas {
		border: 5px solid black;
	}
</style>
