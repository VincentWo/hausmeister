<script lang="ts">
	import { RotateCwIcon, SaveIcon } from 'svelte-feather-icons';
	import SyncedInput from '$lib/components/SyncedInput.svelte';
	import type { ActionData, PageData } from './$types';
	import { enhance, applyAction } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/stores';

	export let form: ActionData;

	$: name = $page.data.user.name;
	$: email = $page.data.user.email;

	$: newName = name;
	$: newEmail = email;
</script>

<main class="max-w-md">
	<h1 class="mb-5">Profile</h1>
	<h2 class="mb-2">Personal Information</h2>
	{#if form?.message}
		<div
			class={form.status == 200
				? 'alert my-3 variant-ghost-success'
				: 'alert my-3 variant-ghost-error'}
		>
			<p class="alert-message">
				{form.message}
			</p>
		</div>
	{/if}
	<form
		action="?/editName"
		method="POST"
		use:enhance={() => {
			// prevent default callback from resetting the form
			return async ({ result }) => {
				if (result.type === 'success') {
					await invalidateAll();
				}
				await applyAction(result);
			};
		}}
	>
		<label class="input-label">
			<span> Name </span>
			<div class="input-group input-group-divider grid-cols-[1fr_auto_auto]">
				<input
					class="transition-colors"
					type="text"
					name="name"
					bind:value={newName}
					class:input-warning={newName !== name}
				/>
				<button
					class="transition-colors"
					type="submit"
					class:variant-filled-primary={newName !== name}
					disabled={newName === name}
				>
					<SaveIcon />
				</button>
				<button
					class="transition-colors"
					type="reset"
					class:variant-filled-primary={newName !== name}
					disabled={newName === name}
					on:click|preventDefault={() => {
						newName = name;
					}}
				>
					<RotateCwIcon />
				</button>
			</div>
		</label>
	</form>
	<label class="input-label">
		<span> E-mail </span>
		<SyncedInput value={newEmail} />
	</label>
</main>
