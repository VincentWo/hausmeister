import { error } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load = (async ({ locals }) => {
	if (!locals.user) {
		throw error(401, 'Unauthorized');
	}

	return {
		user: locals.user
	};
}) satisfies LayoutServerLoad;
