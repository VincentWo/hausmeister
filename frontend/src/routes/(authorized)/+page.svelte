<script lang="ts">
	import { RotateCwIcon, SaveIcon } from 'svelte-feather-icons';
	import { user, sessionId } from '../../stores';
	import axios from 'axios';
	import type { Unsubscriber } from 'svelte/store';
	import SyncedInput from '../../components/SyncedInput.svelte';

	let name = '';
	let email = '';

	let unsubscribe: Unsubscriber;
	unsubscribe = user.subscribe((newUser) => {
		if (newUser !== null) {
			let { name: newName, email: newEmail } = newUser;
			if (name !== newName) {
				name = newName;
			}
			if (email !== newEmail) {
				email = newEmail;
			}
			if (unsubscribe) {
				unsubscribe();
			}
		}
	});

	function updateName() {
		axios
			.patch(
				'http://localhost:3779/user',
				{
					name
				},
				{
					headers: {
						Authorization: `Bearer ${$sessionId}`
					}
				}
			)
			.then((r) => ($user = r.data));
	}

	function resetName() {
		name = $user?.name ?? ""
	}

	$: console.log($user);
</script>

<main class="max-w-md">
	<h1 class="mb-5">Profile</h1>
	<h2 class="mb-2">Personal Information</h2>
	<label class="input-label">
		<span> Name </span>
		<div class="input-group input-group-divider grid-cols-[1fr_auto_auto]">
			<input
				class="transition-colors"
				type="text"
				bind:value={name}
				class:input-warning={$user && $user?.name !== name}
			/>
			<button
				class="transition-colors"
				class:variant-filled-primary={$user && $user?.name !== name}
				disabled={$user?.name === name}
				on:click={updateName}
			>
				<SaveIcon />
			</button>
			<button
				class="transition-colors"
				class:variant-filled-primary={$user && $user?.name !== name}
				disabled={$user?.name === name}
				on:click={resetName}
			>
				<RotateCwIcon />
			</button>
		</div>
	</label>
	<label class="input-label">
		<span> E-mail </span>
		<SyncedInput value={$user?.email ?? ""}/>
	</label>
</main>
