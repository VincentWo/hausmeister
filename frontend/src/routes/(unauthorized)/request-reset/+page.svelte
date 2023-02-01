<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData } from './$types';

	export let form: ActionData;
</script>

<form
	method="POST"
	use:enhance
	class="container mx-auto flex flex-col justify-center align-center max-w-lg max-h-auto p-10 rounded-container-token shadow-lg"
>
	<h1 class="text-center">Request password reset</h1>
	{#if form?.message}
		<aside
			class="alert"
			class:variant-ghost-error={form.status === 404}
			class:variant-ghost-success={form.status === 200}
		>
			<p class="alert-message">
				{form.message}
			</p>
		</aside>
	{/if}
	<label class="input-label">
		<span>E-mail:</span>
		<input type="email" required name="email" value={form?.email ? form.email : ''} />
	</label>
	<input
		value="Request reset"
		type="submit"
		id="submit-input"
		class="btn variant-filled-primary variant-lg mx-auto mt-8"
	/>
</form>
