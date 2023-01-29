<script lang="ts">
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';
	import { sessionId } from '../../stores';

	$: if ($sessionId !== null) {
		if (browser) {
			goto('/');
		}
	}
</script>

<div class="h-full flex justify-center align-center items-center">
	<slot />
</div>
