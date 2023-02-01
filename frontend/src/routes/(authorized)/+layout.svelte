<script lang="ts">
	import { enhance } from '$app/forms';
	import { AppRail, AppRailTile, AppShell } from '@skeletonlabs/skeleton';
	import md5 from 'md5';
	import { writable } from 'svelte/store';
	import type { LayoutData } from './$types';

	export let data: LayoutData;

	const hash = encodeURIComponent(md5(data.user.email));
	const profile_picture = `https://gravatar.com/avatar/${hash}?d=robohash&s=50`;

	export const selected = writable('profile');
</script>

<AppShell>
	<svelte:fragment slot="pageHeader">
		<!--
			<AppBar>
				<svelte:fragment slot="trail">
					<img
						src={profile_picture}
						alt="Profile"
						class="rounded-full"
						>
				</svelte:fragment>
			</AppBar>
		-->
	</svelte:fragment>
	<svelte:fragment slot="sidebarLeft">
		<AppRail {selected}>
			<svelte:fragment slot="lead">
				<AppRailTile label="Profile" title="Profile" value="profile">
					<img src={profile_picture} alt="Profile" class="rounded-full" />
				</AppRailTile>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<form action="/logout" method="POST" use:enhance>
					<AppRailTile label="Logout" title="Logout" value="logout" tag="button" />
				</form>
			</svelte:fragment>
		</AppRail>
	</svelte:fragment>
	<!-- <svelte:fragment slot="sidebarRight">Sidebar Right</svelte:fragment> -->
	<!-- <svelte:fragment slot="pageHeader">Page Header</svelte:fragment> -->
	<!-- Router Slot -->
	<div class="p-5">
		<slot />
	</div>
	<!-- ---- / ---- -->
	<!-- <svelte:fragment slot="pageFooter">Page Footer</svelte:fragment> -->
	<!-- <svelte:fragment slot="footer">Footer</svelte:fragment> -->
</AppShell>
