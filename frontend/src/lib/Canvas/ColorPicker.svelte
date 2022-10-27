<script lang="ts">
	import type { IColor } from '$lib/schemas/schemas';
	import { onMount } from 'svelte';
	const colors: IColor[] = [
		{ red: 0, green: 0, blue: 0 },
		{ red: 255, green: 255, blue: 255 },
		{ red: 255, green: 0, blue: 0 },
		{ red: 0, green: 255, blue: 0 },
		{ red: 0, green: 0, blue: 255 },
		{ red: 255, green: 255, blue: 0 },
		{ red: 0, green: 255, blue: 255 },
		{ red: 255, green: 0, blue: 255 },
		{ red: 122, green: 0, blue: 0 },
		{ red: 122, green: 122, blue: 0 },
		{ red: 0, green: 122, blue: 0 },
		{ red: 0, green: 122, blue: 122 },
		{ red: 0, green: 0, blue: 122 },
		{ red: 122, green: 122, blue: 122 }
	];

	export let currentColor: IColor = { red: 0, green: 0, blue: 0 };
	let showingColors: boolean = true;
	onMount(async () => {
		let colorsDiv: HTMLElement | null = document.getElementById('colorsToPick');
		colors.forEach((color: IColor) => {
			let newChild: HTMLSpanElement = document.createElement('span');
			newChild.className = 'color';
			newChild.style.backgroundColor = `rgb(${color.red},${color.green},${color.blue})`;
			newChild.onclick = () => {
				currentColor = color;
			};

			colorsDiv?.append(newChild);
		});
	});

	function updateColorsSelection() {
		showingColors = !showingColors;
		let colorsDiv: HTMLElement | null = document.getElementById('colorsToPick');
		if (!colorsDiv) {
			return;
		}
		if (showingColors) {
			colorsDiv.classList.remove('hidden');
		} else {
			colorsDiv.classList.add('hidden');
		}
	}
</script>

<div id="colorPickerWrapper">
	<div id="colorsToPick" />
	<div
		id="colorPicker"
		on:click={updateColorsSelection}
		style:background-color={`rgb(${currentColor.red},${currentColor.green},${currentColor.blue})`}
	/>
</div>

<style>
	#colorPickerWrapper {
		margin: 10px;
		background-color: #383f4ee4;
		display: flex;
		flex-direction: column;
		gap: 7px;
		align-items: flex-end;
		width: 150px;
		padding: 7px;
		border-radius: 7px;
		pointer-events: all !important;
	}
	#colorsToPick {
		display: flex;
		flex-direction: row;
		flex-wrap: wrap;
		align-content: flex-start;
		justify-content: space-around;
		gap: 5px;
	}
	#colorPicker {
		width: 100%;
		height: 20px;
		border-radius: 7px;
		border: black solid 2px;
		box-sizing: border-box;
		outline-width: 1px;
		outline-color: white;
		outline-style: solid;
		cursor: pointer;
	}
	:global(.color) {
		border-radius: 50%;
		width: 25px;
		height: 25px;
		cursor: pointer;
		border: black solid 2px;
		box-sizing: border-box;
		outline-width: 1px;
		outline-color: white;
		outline-style: solid;
	}
	#colorsToPick:global(.hidden) {
		display: none;
		z-index: -1;
	}
</style>
