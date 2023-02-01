import { PUBLIC_API_URL } from '$env/static/public';
import type { Actions } from './$types';

export const actions = {
	default: async ({ request }) => {
		const data = await request.formData();
		const email = data.get('email');

		const response = await fetch(PUBLIC_API_URL + '/request-reset', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ email })
		});

		if (response.status === 200) {
			return {
				status: 200,
				message: 'Email sent.'
			};
		} else {
			return {
				status: 404,
				message: 'An Error occured.',
				email
			};
		}
	}
} satisfies Actions;
