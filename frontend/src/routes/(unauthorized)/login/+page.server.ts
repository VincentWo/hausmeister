import { PUBLIC_API_URL } from '$env/static/public';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions = {
	login: async ({ cookies, request }) => {
		const data = await request.formData();
		const email = data.get('email') as string;
		const password = data.get('password') as string;
		if (!email) {
			return fail(400, { email, missing: true });
		}
		if (!password) {
			return fail(400, { password, missing: true });
		}
		const response = await fetch(PUBLIC_API_URL + '/login', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ email, password })
		});

		console.log(response);
		if (response.status === 200) {
			const res = await response.json();
			console.log(res);
			cookies.set('session', res.session_id, {
				path: '/',
				maxAge: 60 * 60 * 24 * 7,
				sameSite: 'lax',
				httpOnly: true
			});
			throw redirect(302, '/');
		} else if (response.status === 404) {
			return fail(404, { email, notFound: true });
		} else if (response.status === 401) {
			return fail(401, { email, incorrect: true });
		}
	}
} satisfies Actions;
