import { PUBLIC_API_URL } from '$env/static/public';
import type { Actions } from './$types';

export const actions = {
	editName: async ({ request, cookies, locals }) => {
		const data = await request.formData();
		const name = data.get('name') as string;
		const token = cookies.get('session');

		const response = await fetch(PUBLIC_API_URL + '/user', {
			method: 'PATCH',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${token}`
			},
			body: JSON.stringify({ name })
		});

		if (response.status === 200) {
			if (locals.user) {
				locals.user.name = name;
			}
			return {
				status: 200,
				message: 'Profile updated.',
				name
			};
		} else {
			return {
				status: 404,
				message: 'An Error occured.',
				name
			};
		}
	}
} satisfies Actions;
