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
		type IBebopData,
		type IDeltaGrid,
		type IGrid,
		type IPixel
	} from '$lib/schemas/schemas';
	import ColorPicker from '$lib/Canvas/ColorPicker.svelte';

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null;

	let initialGrid: IGrid | null;

	let drag: boolean = false;
	let cancelClickCanvas = false;

	let currentX: number = 0;
	let currentY: number = 0;

	let deltaX: number = 0;
	let deltaY: number = 0;
	let scale: number = 1;

	let socket: WebSocket;

	onMount(() => {
		socket = new WebSocket('ws://127.0.0.1:8080');
		socket.binaryType = 'arraybuffer';
		socket.onopen = (e) => {};
		socket.onclose = (_e) => {};
		socket.onmessage = (e) => {
			let binary_data: Uint8Array = new Uint8Array(e.data);
			let bebop_data = BebopData.decode(binary_data);
			let structData: IGrid | IPixel | IDeltaGrid | null = null;
			switch (bebop_data.opcode) {
				case GridOpcode:
					structData = Grid.decode(bebop_data.encodedData);
					initialGrid = structData;
					break;

				case PixelOpcode:
					structData = Pixel.decode(bebop_data.encodedData);
					setPixel(structData);
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
		};
		if (canvas.getContext) {
			ctx = canvas.getContext('2d');
			if (ctx) {
				ctx.imageSmoothingEnabled = false;
			}
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

	function clickCanvas(event: MouseEvent) {
		const x: number = Math.round(event.offsetX * scale);
		const y: number = Math.round(event.offsetY * scale);
		console.log(x, y, cancelClickCanvas, canvas.width, canvas.height);
		if (!ctx || 0 > x || x >= canvas.width || 0 > y || y >= canvas.height || cancelClickCanvas) {
			cancelClickCanvas = false;
			return;
		}
		const newPixel: IPixel = { color: { red: 255, green: 0, blue: 0 }, x, y };
		const encodedNewPixel: Uint8Array = new Uint8Array(Pixel.encode(newPixel));
		const message: IBebopData = {
			protocolVersion: PixelOpcode,
			opcode: PixelOpcode,
			encodedData: encodedNewPixel
		};
		const encodedMessage: Uint8Array = new Uint8Array(BebopData.encode(message));
		socket.send(encodedMessage);
	}

	function mouseMoved(event: MouseEvent) {
		const x: number = event.clientX;
		const y: number = event.clientY;

		if (event.buttons === 1) {
			drag = true;
			deltaX += x - currentX;
			deltaY += y - currentY;
		}

		currentX = x;
		currentY = y;
	}

	function mouseUpContainer() {
		if (!drag) {
			return;
		}
		drag = false;
		cancelClickCanvas = true;
	}

	function mouseZoom(event: WheelEvent) {
		if (event.deltaY > 0) {
			scale *= 0.9;
		} else if (event.deltaY < 0) {
			scale *= 1.1;
		}
	}
</script>

<div
	id="canvasContainer"
	on:mouseup={mouseUpContainer}
	on:mousemove={mouseMoved}
	on:wheel={mouseZoom}
	style:cursor={drag ? 'move' : 'default'}
>
	<ColorPicker />
	<canvas
		bind:this={canvas}
		on:click={clickCanvas}
		width="500"
		height="500"
		style:translate={`${deltaX}px ${deltaY}px`}
		style:scale
	/>
</div>

<style>
	#canvasContainer {
		position: relative;
		overflow: hidden;
	}
	canvas {
		border: 5px solid black;
		image-rendering: pixelated;
	}
</style>
