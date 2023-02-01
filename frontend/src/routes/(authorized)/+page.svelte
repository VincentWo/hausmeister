<script lang="ts">
	import { RotateCwIcon, SaveIcon } from 'svelte-feather-icons';
	import SyncedInput from '$lib/components/SyncedInput.svelte';
	import { page } from '$app/stores';
	import type { ActionData } from './$types';
	import { enhance } from '$app/forms';

	export let form: ActionData;

	let name = $page.data.user.name;
	let email = $page.data.user.email;

	let newName = name;
	let newEmail = email;
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
	<form action="?/editName" method="POST" use:enhance>
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
					on:click={() => {
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
