<script lang="ts">
	import { enhance } from '$app/forms';
	import { EyeIcon, EyeOffIcon } from 'svelte-feather-icons';
	import ErrorAlert from '$lib/components/ErrorAlert.svelte';
	import type { ActionData } from './$types';

	export let form: ActionData;

	let isVisible = false;
</script>

<form
	method="POST"
	action="?/login"
	use:enhance
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
>
	<h1 class="text-center mb-5">Login</h1>
	{#if form?.missing}
		<ErrorAlert>E-Mail and Password are required.</ErrorAlert>
	{/if}
	{#if form?.notFound}
		<ErrorAlert>Account not found.</ErrorAlert>
	{/if}
	{#if form?.incorrect}
		<ErrorAlert>Incorrect password.</ErrorAlert>
	{/if}
	<label class="input-label">
		<span>E-Mail:</span>
		<input
			name="email"
			type="email"
			value={form?.email ?? ''}
			required
			class={form?.notFound ? 'input-error' : ''}
		/>
	</label>
	<label class="input-label">
		<span>Password:</span>
		<div class="input-group input-group-divider grid-cols-[1fr_auto]">
			<input
				name="password"
				type={isVisible ? 'text' : 'password'}
				required
				class={form?.incorrect ? 'input-error' : ''}
			/>
			<button
				class=""
				type="button"
				on:click={() => {
					isVisible = !isVisible;
				}}
			>
				{#if isVisible}
					<EyeIcon />
				{:else}
					<EyeOffIcon />
				{/if}
			</button>
		</div>
	</label>
	<input value="Login" type="submit" class="btn variant-filled-primary variant-lg mx-auto mt-8" />
	<aside class="text-sm text-center mt-8">
		Forgot your password? <a href="request-reset">Reset it</a>
	</aside>
</form>
