<script language="ts">
	import { goto } from '$app/navigation';
	import { AppRail, AppRailTile, AppShell } from '@skeletonlabs/skeleton';
	import { sessionId } from '../../stores';
	import { browser } from '$app/environment';
	import md5 from 'md5';
	import { writable } from 'svelte/store';
	import axios from 'axios';

	const hash = md5('vincent@woltmann.art');
	export const profile_picture = `https://gravatar.com/avatar/${hash}?s=50`;

	$: if ($sessionId === null) {
		if (browser) {
			goto('login');
		}
	}

	export const selected = writable('profile');

	$: if ($selected === 'logout') {
		axios.post('http://localhost:3779/logout', undefined, {
			headers: {
				Authorization: `Bearer ${$sessionId}`
			}
		});
		$sessionId = null;
	}
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
				<AppRailTile label="Logout" title="Logout" value="logout" />
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
