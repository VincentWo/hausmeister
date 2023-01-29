<script lang="ts">
	import { page } from '$app/stores';
	import { ProgressRadial } from '@skeletonlabs/skeleton';
	import axios from 'axios';

	export const token = $page.url.searchParams.get('token');

	/**
	 * In general the state transistion goes:
	 * Checking Token
	 *  | Invalid Response -> InvalidToken,
	 *  | Valid Response   -> AcceptingInput
	 *
	 * AcceptingInput
	 *  -> Sending Reset Request
	 *     | Invalid Response -> Invalid Token
	 *     | Valid Response   -> PasswordWasReset
	 */
	enum State {
		CheckingToken,
		AcceptingInput,
		PasswordWasReset,
		InvalidToken
	}

	export let currentState = State.CheckingToken;
	$: inputDisabled = currentState !== State.AcceptingInput;

	export const tokenCheck = axios
		.post('http://localhost:3779/test_reset_token', {
			reset_token: token
		})
		.then((result) => result.data)
		.catch(() => false)
		.then((isValid) => {
			if (isValid) {
				currentState = State.AcceptingInput;
			} else {
				currentState = State.InvalidToken;
			}
		});

	export let password = '';
	export let passwordConfirmed = '';
	export let passwordInputConfirmed: HTMLInputElement | null;

	export const resetPassword = () => {
		axios
			.post('http://localhost:3779/reset', {
				reset_token: token,
				new_password: password
			})
			.then(() => {
				currentState = State.PasswordWasReset;
			})
			.catch(() => {
				currentState = State.InvalidToken;
			});
	};

	$: {
		let validity = password === passwordConfirmed ? '' : 'Passwords should be equal';
		passwordInputConfirmed?.setCustomValidity(validity);
	}
</script>

<form
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
	on:submit|preventDefault={resetPassword}
>
	<h1 class="text-center h1 mb-3">Reset Password</h1>
	{#if inputDisabled}
		<div
			class="alert"
			class:variant-ghost-primary={currentState === State.PasswordWasReset}
			class:variant-ghost-secondary={currentState === State.CheckingToken}
			class:variant-ghost-error={currentState === State.InvalidToken}
		>
			<div class="flex items-center justify-around w-full">
				<p class="alert-message">
					{#if currentState === State.CheckingToken}
						Checking Reset Token...
					{:else if currentState === State.InvalidToken}
						Reset Token is invalid, request a new one
						<a href="request-reset"> here </a>
					{:else if currentState === State.PasswordWasReset}
						Password was reset, you can now <a href="/login">login</a>.
					{/if}
				</p>
				{#if currentState == State.CheckingToken}
					<div class="w-12">
						<ProgressRadial />
					</div>
				{/if}
			</div>
		</div>
	{/if}
	<label class="input-label" for="password">
		<span> New password: </span>
		<input type="password" id="password" required bind:value={password} disabled={inputDisabled} />
	</label>
	<label class="input-label">
		<span> Confirm new password: </span>
		<input
			bind:this={passwordInputConfirmed}
			type="password"
			required
			bind:value={passwordConfirmed}
			disabled={inputDisabled}
		/>
	</label>
	<input
		class="btn variant-filled-primary btn-lg mx-auto mt-8"
		type="submit"
		value="Reset password"
		disabled={inputDisabled}
	/>
</form>
