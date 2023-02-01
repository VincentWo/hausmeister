import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	default: async ({ cookies, locals }) => {
		cookies.delete('session');
		locals.user = null;
		throw redirect(302, '/login');
	}
} satisfies Actions;
