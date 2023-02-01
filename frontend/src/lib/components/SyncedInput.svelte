<script lang="ts">
	import { RotateCwIcon, SaveIcon } from "svelte-feather-icons";

  export let reset = () => {};
  export let update = () => {};
  export let value = ""
  let originalValue = ""

  let hasBeenSet = false;
  $: if (value != "" && !hasBeenSet) {
    originalValue = value;
    hasBeenSet = true;
  }
</script>
<div class="input-group input-group-divider grid-cols-[1fr_auto_auto]">
  	<input
			class="transition-colors"
			type="text"
			bind:value={value}
			class:input-warning={value !== originalValue}
		/>
		<button
			class="transition-colors"
			class:variant-filled-primary={value !== originalValue}
			disabled={value === originalValue}
			on:click={update}
		>
			<SaveIcon />
		</button>
		<button
			class="transition-colors"
			class:variant-filled-primary={originalValue !== value}
			disabled={originalValue === value}
			on:click={reset}
		>
			<RotateCwIcon />
		</button>
</div>
