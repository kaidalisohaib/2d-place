<script lang="ts">
	import type { IColor } from '$lib/schemas/schemas';
	import { onMount } from 'svelte';
	const colors: IColor[] = [
		{ red: 0, green: 0, blue: 0 },
		{ red: 255, green: 0, blue: 0 },
		{ red: 0, green: 255, blue: 0 },
		{ red: 0, green: 0, blue: 255 }
	];

	export let currentColor: IColor = { red: 0, green: 0, blue: 0 };
    let showingColors: boolean = true;
	onMount(async () => {
		let colorsDiv: HTMLElement | null = document.getElementById('colorsToPick');
        colors.forEach((color: IColor) => {
            let newChild: HTMLSpanElement = document.createElement("span");
            newChild.className = "color";
            newChild.style.backgroundColor = `rgb(${color.red},${color.green},${color.blue})`
            newChild.onclick = () => {
                currentColor = color;
            }
            
            colorsDiv?.append(newChild)
        })
	});

    function updateColorsSelection(){
        showingColors = !showingColors;
        console.log(showingColors)
        let colorsDiv: HTMLElement | null = document.getElementById('colorsToPick');
        if ( !colorsDiv ){
            return;
        }
        if( showingColors ){
            colorsDiv.classList.remove("hidden");
        }else {
            colorsDiv.classList.add("hidden");
        }
    }
</script>

<div id="colorPickerWrapper">
	<div id="colorsToPick" />
	<div
		id="colorPicker"
        class="color"
        on:click="{updateColorsSelection}"
		style:background-color={`rgb(${currentColor.red},${currentColor.green},${currentColor.blue})`}
	/>
</div>

<style>
	#colorPickerWrapper {
        z-index: 1;
		position: absolute;
		bottom: 0;
		right: 0;
		margin: 10px;
        background-color: green;
	}
	#colorsToPick {
		display: flex;
	}
	#colorPicker {
	}
    :global(.color) {
		border-radius: 50%;
		width: 25px;
		height: 25px;
		cursor: pointer;
    }
    #colorsToPick:global(.hidden) {
        display: none;
        z-index: -1;
    }
</style>
