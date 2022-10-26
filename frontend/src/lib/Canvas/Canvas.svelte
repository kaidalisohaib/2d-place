<script lang="ts">
	import { onMount, tick } from 'svelte';
	import {
		BebopData,
		DeltaGrid,
		DeltaGridOpcode,
		Grid,
		GridOpcode,
		Pixel,
		PixelOpcode,
		type IBebopData,
		type IColor,
		type IDeltaGrid,
		type IGrid,
		type IPixel
	} from '$lib/schemas/schemas';
	import ColorPicker from '$lib/Canvas/ColorPicker.svelte';
	interface Vec2D {
		x: number;
		y: number;
	}
	const zoomScaleMax: number = 30;
	let canvasContainer: HTMLDivElement;
	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null;

	let initialGrid: IGrid | null;

	let drag: boolean = false;
	let cancelClickCanvas = false;

	let currentMousePosition: Vec2D = { x: 0, y: 0 };

	let deltaCanvas: Vec2D = { x: 0, y: 0 };
	let zoomScale: number = 1;

	let currentPixelColor: IColor = { red: 0, green: 0, blue: 0 };
	let socket: WebSocket;

	onMount(() => {
		if (canvas.getContext) {
			ctx = canvas.getContext('2d');
			if (ctx) {
				ctx.imageSmoothingEnabled = false;
			}
		}
		canvasContainer = document.getElementById('canvasContainer') as HTMLDivElement;

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
		const mouseRelativePosition = getCanvasRelativeMousePosition(event);
		const x = Math.floor(mouseRelativePosition.x);
		const y = Math.floor(mouseRelativePosition.y);

		if (!ctx || 0 > x || x >= canvas.width || 0 > y || y >= canvas.height || cancelClickCanvas) {
			cancelClickCanvas = false;
			return;
		}
		const newPixel: IPixel = { color: currentPixelColor, x, y };
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
		const targetId: string = (event.target as HTMLElement).id;

		const x: number = event.clientX;
		const y: number = event.clientY;

		if (targetId !== 'canvas' && targetId !== 'canvasContainer') {
			drag = false;

			return;
		}

		if (event.buttons === 1) {
			drag = true;
			deltaCanvas.x += x - currentMousePosition.x;
			deltaCanvas.y += y - currentMousePosition.y;
		}

		currentMousePosition = { x, y };
	}

	function mouseMoveCanvas(event: MouseEvent) {}

	function getCanvasRelativeMousePosition(event: MouseEvent): { x: number; y: number } {
		const rect = canvas.getBoundingClientRect();
		const ratio = 500 / rect.width;
		const mouseX = (event.clientX - rect.x) * ratio;
		const mouseY = (event.clientY - rect.y) * ratio;
		return { x: mouseX, y: mouseY };
	}

	function solveCanvasPosition(
		event: MouseEvent,
		relativeMousePosition: { x: number; y: number }
	): { x: number; y: number } {
		const rect = canvas.getBoundingClientRect();
		const ratio = 500 / rect.width;
		const canvasX = event.clientX - relativeMousePosition.x / ratio;
		const canvasY = event.clientY - relativeMousePosition.y / ratio;
		return { x: canvasX, y: canvasY };
	}

	function mouseUpContainer() {
		if (!drag) {
			return;
		}
		drag = false;
		cancelClickCanvas = true;
	}

	async function mouseZoom(event: WheelEvent) {
		const lastRelativeMousePosition: { x: number; y: number } =
			getCanvasRelativeMousePosition(event);
		if (event.deltaY > 0) {
			zoomScale *= 0.9;
		} else if (event.deltaY < 0) {
			zoomScale *= 1.1;
		}
		zoomScale = Math.min(Math.max(zoomScale, -zoomScaleMax), zoomScaleMax);
		await tick();
		const newCanvasPosition = solveCanvasPosition(event, lastRelativeMousePosition);
		deltaCanvas = { x: newCanvasPosition.x, y: newCanvasPosition.y };
	}
</script>

<div
	id="canvasContainer"
	on:mouseup={mouseUpContainer}
	on:mousemove={mouseMoved}
	on:wheel={mouseZoom}
	style:cursor={drag ? 'move' : 'default'}
>
	<ColorPicker bind:currentColor={currentPixelColor} />
	<canvas
		id="canvas"
		bind:this={canvas}
		on:click={clickCanvas}
		on:mousemove={mouseMoveCanvas}
		width="500"
		height="500"
		style:translate={`${deltaCanvas.x}px ${deltaCanvas.y}px`}
		style:scale={zoomScale}
	/>
</div>

<style>
	#canvasContainer {
		position: relative;
		overflow: hidden;
		user-select: none;
		background-color: #434c5e;
		height: 100%;
	}

	canvas {
		image-rendering: pixelated;
		transform-origin: top left;
	}
</style>
