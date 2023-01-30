<script lang="ts">
	import axios from 'axios';
	import { goto } from '$app/navigation';
	import { sessionId, user } from '../../../stores';
	import { EyeIcon, EyeOffIcon } from 'svelte-feather-icons';
	import { PUBLIC_API_URL } from '$env/static/public';

	export let email = '';
	export let password = '';
	export let error = '';
	export let isVisible = false;
	export let passwordInput: HTMLInputElement;
	export let changeVisibility = () => {
		isVisible = !isVisible;
		passwordInput.type = isVisible ? 'text' : 'password';
	};
	export async function login() {
		axios
			.post(PUBLIC_API_URL + '/login', {
				email,
				password
			})
			.then((r) => {
				sessionId.set(r.data.session_id);
				user.set(r.data.user);
				return goto('/');
			})
			.catch((e) => {
				if (e.response) {
					if (e.response.status === 404) {
						error = 'User does not exist';
					} else if (e.response.status === 401) {
						error = 'Wrong Password';
					}
				}
			});
	}
</script>

<form
	on:submit={login}
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
>
	<h1 class="text-center mb-5">Login</h1>
	{#if error != ''}
		<div class="alert variant-ghost-error my-3">
			<p class="alert-message">
				{error}
			</p>
		</div>
	{/if}
	<label for="mail-input" class="input-label">
		<span>E-Mail:</span>
		<input type="email" id="mail-input" required bind:value={email} />
	</label>
	<label for="password-input" class="input-label">
		<span>Password:</span>
		<div class="input-group input-group-divider grid-cols-[1fr_auto]">
			<input
				type="password"
				id="password-input"
				required
				bind:this={passwordInput}
				bind:value={password}
			/>
			<button class="" type="button" on:click={changeVisibility}>
				{#if isVisible}
					<EyeIcon />
				{:else}
					<EyeOffIcon />
				{/if}
			</button>
		</div>
	</label>
	<input
		value="Login"
		type="submit"
		id="submit-input"
		class="btn variant-filled-primary variant-lg mx-auto mt-8"
	/>
	<aside class="text-sm text-center mt-8">
		Forgot your password? <a href="request-reset">Reset it</a>
	</aside>
</form>
