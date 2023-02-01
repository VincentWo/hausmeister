<script lang="ts">
	import type { ActionData } from './$types';

	export let form: ActionData;
</script>

<form
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
	method="POST"
	action="?/resetPassword"
>
	<h1 class="text-center h1 mb-3">Reset Password</h1>
	{#if form?.error}
		<div class="alert variant-ghost-error my-3">
			<p class="alert-message">{form.error}</p>
		</div>
	{/if}
	{#if form?.success}
		<div class="alert variant-ghost-success my-3">
			<p class="alert-message">{form.success}</p>
		</div>
	{/if}

	<label class="input-label">
		<span> New password: </span>
		<input type="password" name="password" required />
	</label>
	<label class="input-label">
		<span> Confirm new password: </span>
		<input name="passwordConfirmed" type="password" required />
	</label>
	<input
		class="btn variant-filled-primary btn-lg mx-auto mt-8"
		type="submit"
		value="Reset password"
	/>
</form>
