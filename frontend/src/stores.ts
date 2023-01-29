// While server determines whether the user is logged in by examining RequestEvent.locals.user, the
import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import axios from 'axios';

let storedSessionId = null;
const storedUser = null;
if (browser) {
	storedSessionId = localStorage.getItem('session_id');
}

interface User {
	name: string;
	email: string;
}

export const sessionId = writable<string | null>(storedSessionId);
export const user = writable<User | null>(storedUser);

if (browser) {
	sessionId.subscribe((value) => {
		if (value) {
			localStorage.setItem('session_id', value);
		} else {
			localStorage.removeItem('session_id');
		}
	});
}

if (storedSessionId) {
	axios
		.get('http://localhost:3779/user', {
			headers: {
				Authorization: `Bearer ${storedSessionId}`
			}
		})
		.then((r) => user.set(r.data))
		.catch(() => {
			sessionId.set(null);
		});
}
