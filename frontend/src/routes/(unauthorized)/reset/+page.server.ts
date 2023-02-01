import { PUBLIC_API_URL } from '$env/static/public';
import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load = (async ({ url }) => {
	if (url.searchParams.get('token') === null) {
		throw error(400, 'Bad Request');
	}
	const token = url.searchParams.get('token');
	const response = await fetch(PUBLIC_API_URL + '/test-reset-token', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ token })
	});
	if (response.status != 200) {
		throw error(400, 'Bad Request');
	}
}) satisfies PageServerLoad;

export const actions = {
	resetPassword: async ({ request, url }) => {
		const data = await request.formData();
		const password = data.get('password') as string;
		const passwordConfirmed = data.get('passwordConfirmed') as string;
		const token = url.searchParams.get('token');

		if (password !== passwordConfirmed) {
			return fail(400, { error: 'Passwords do not match' });
		}

		const response = await fetch(PUBLIC_API_URL + '/reset', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ reset_token: token, new_password: password })
		});
		if (response.status != 200) {
			return fail(400, { error: 'An error occured while resetting your password' });
		} else {
			return {
				status: 200,
				success: 'Password reset successful'
			};
		}
	}
} satisfies Actions;
