<script lang="ts">
	import { page } from '$app/stores';
</script>

<div class="h-full flex justify-center align-center items-center">
	<div class="card">
		<header class="card-header">
			<h1 class="card-title">Error {$page.status}</h1>
		</header>
		<div class="p-4">{$page.error?.message}</div>
	</div>
</div>
