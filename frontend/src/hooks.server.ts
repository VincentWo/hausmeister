import { PUBLIC_API_URL } from '$env/static/public';
import { redirect, type Handle } from '@sveltejs/kit';

enum publicRoutes {
	'/login',
	'/register',
	'/request-reset',
	'/reset'
}

export const handle: Handle = async ({ event, resolve }) => {
	const session = event.cookies.get('session');

	const res = await fetch(PUBLIC_API_URL + '/user', {
		method: 'GET',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${session}`
		}
	});

	if (res.status === 200) {
		const user = await res.json();
		event.locals.user = user;
	}

	if (!event.locals.user && !Object.values(publicRoutes)?.includes(event.url.pathname)) {
		console.log(event.locals.user, event.url.pathname);
		throw redirect(302, '/login');
	}

	return await resolve(event);
};
