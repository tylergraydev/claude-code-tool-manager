import { readable } from 'svelte/store';

// Mock page store
export const page = readable({
	url: new URL('http://localhost:5173/'),
	params: {},
	route: { id: '/' },
	status: 200,
	error: null,
	data: {},
	form: null
});

// Mock navigating store
export const navigating = readable(null);

// Mock updated store
export const updated = readable(false);
