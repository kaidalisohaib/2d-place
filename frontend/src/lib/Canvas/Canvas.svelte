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

	let deltaCanvasPosition: Vec2D = { x: 0, y: 0 };
	let zoomScale: number = 1;

	let deltaPixelFocusPosition: Vec2D = { x: 0, y: 0 };
	let currentSelectedPixelPosition: Vec2D = { x: -1, y: -1 };
	let showPixelFocus: boolean = false;

	let currentPixelColor: IColor = { red: 0, green: 0, blue: 0 };
	let socket: WebSocket;
	$: {
		if (canvas) {
			const rect = canvas.getBoundingClientRect();
			const minX = -rect.width + 25;
			const minY = -rect.height + 25;
			const maxX = canvasContainer.clientWidth - 25;
			const maxY = canvasContainer.clientHeight - 25;
			deltaCanvasPosition.x = Math.max(Math.min(deltaCanvasPosition.x, maxX), minX);
			deltaCanvasPosition.y = Math.max(Math.min(deltaCanvasPosition.y, maxY), minY);
		}
	}
	$: {
		deltaPixelFocusPosition = { x: deltaCanvasPosition.x, y: deltaCanvasPosition.y };
		deltaPixelFocusPosition.x += currentSelectedPixelPosition.x * zoomScale;
		deltaPixelFocusPosition.y += currentSelectedPixelPosition.y * zoomScale;
	}
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
		showPixelFocus = true;
		currentSelectedPixelPosition = { x, y };
	}

	function placePixel(position: Vec2D) {
		const newPixel: IPixel = { color: currentPixelColor, x: position.x, y: position.y };
		const encodedNewPixel: Uint8Array = new Uint8Array(Pixel.encode(newPixel));
		const message: IBebopData = {
			protocolVersion: PixelOpcode,
			opcode: PixelOpcode,
			encodedData: encodedNewPixel
		};
		const encodedMessage: Uint8Array = new Uint8Array(BebopData.encode(message));
		socket.send(encodedMessage);
		showPixelFocus = false;
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
			deltaCanvasPosition.x += x - currentMousePosition.x;
			deltaCanvasPosition.y += y - currentMousePosition.y;
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
		zoomScale = Math.min(Math.max(zoomScale, 1 / zoomScaleMax), zoomScaleMax);
		await tick();
		const newCanvasPosition = solveCanvasPosition(event, lastRelativeMousePosition);
		deltaCanvasPosition = { x: newCanvasPosition.x, y: newCanvasPosition.y };
	}
</script>

<div
	id="canvasContainer"
	on:mouseup={mouseUpContainer}
	on:mousemove={mouseMoved}
	on:wheel={mouseZoom}
	style:cursor={drag ? 'move' : 'default'}
>
	<canvas
		id="canvas"
		bind:this={canvas}
		on:click={clickCanvas}
		on:mousemove={mouseMoveCanvas}
		width="500"
		height="500"
		style:translate={`${deltaCanvasPosition.x}px ${deltaCanvasPosition.y}px`}
		style:scale={zoomScale}
	/>
	<div
		id="pixelFocus"
		on:click={() => (showPixelFocus = false)}
		style:translate={`${deltaPixelFocusPosition.x}px ${deltaPixelFocusPosition.y}px`}
		style:width={`${zoomScale}px`}
		style:height={`${zoomScale}px`}
		style:display={showPixelFocus ? 'inline' : 'none'}
	/>
	<div id="bottomUI">
		<button
			id="pixelPlace"
			on:click|self={() => placePixel(currentSelectedPixelPosition)}
			style:visibility={showPixelFocus ? 'visible' : 'hidden'}
		>
			PLACE THE PIXEL <button id="removePixelPlace" on:click={() => (showPixelFocus = false)}
				>✘</button
			>
		</button>
		<ColorPicker bind:currentColor={currentPixelColor} />
	</div>
</div>

<style>
	#canvasContainer {
		position: relative;
		overflow: hidden;
		user-select: none;
		background-color: #434c5e;
		height: 100%;
	}
	#pixelFocus {
		background-color: transparent;
		border: white solid 2px;
		outline: black solid 2px;
		border-radius: 1px;
		transform-origin: center;
		position: absolute;
		box-sizing: border-box;
		animation: scallingIn 0.6s cubic-bezier(0.45, 0, 0.55, 1) infinite alternate-reverse;
		z-index: 1;
	}
	#bottomUI {
		z-index: 2;
		position: absolute;
		bottom: 0px;
		margin: 10px;
		display: flex;
		width: 100%;
		flex-direction: row;
		flex-wrap: wrap;
		align-items: center;
		pointer-events: none;
		background: none;
	}
	#bottomUI > * {
		pointer-events: all !important;
	}
	#pixelPlace {
		background-color: rgb(9, 142, 9);
		color: white;
		font-size: larger;
		border: none;
		border-radius: 7px;
		padding: 7px;
		cursor: pointer;
		margin-left: auto;
		margin-right: auto;
	}
	#pixelPlace:hover {
		background-color: rgb(8, 126, 8);
	}
	#removePixelPlace {
		background-color: rgb(252, 73, 73);
		border-radius: 7px;
		padding: 7px;
		color: white;
		border: none;
		cursor: pointer;
	}
	#removePixelPlace:hover {
		background-color: rgb(205, 60, 60);
	}
	canvas {
		image-rendering: pixelated;
		transform-origin: top left;
		position: absolute;
	}
	@keyframes scallingIn {
		from {
			scale: 1;
		}
		to {
			scale: 1.5;
		}
	}
</style>
