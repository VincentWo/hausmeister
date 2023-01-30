<script lang="ts">
	import { PUBLIC_API_URL } from '$env/static/public';
	import axios from 'axios';

	export let email = '';
	export let msg = '';
	export let status = '';

	export let requestReset = () => {
		axios
			.post(PUBLIC_API_URL + '/request-reset', {
				email
			})
			.then(() => {
				msg = 'E-mail has been sent, also check your spam.';
				status = 'success';
			})
			.catch((e) => {
				if (e.response) {
					if (e.response.status === 404) {
						msg = 'User does not exist';
						status = 'error';
					}
				}
			});
	};
</script>

<form
	on:submit={requestReset}
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
>
	<h1 class="text-center">Request password reset</h1>
	{#if msg !== ''}
		<aside
			class="alert"
			class:variant-ghost-error={status === 'error'}
			class:variant-ghost-success={status === 'success'}
		>
			<p class="alert-message">
				{msg}
			</p>
		</aside>
	{/if}
	<label for="mail-input" class="input-label">
		<span>E-mail:</span>
		<input type="email" id="mail-input" required bind:value={email} />
	</label>
	<input
		value="Request reset e-mail"
		type="submit"
		id="submit-input"
		class="btn variant-filled-primary variant-lg mx-auto mt-8"
	/>
</form>
