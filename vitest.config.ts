import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
	plugins: [svelte({ hot: !process.env.VITEST })],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		globals: true,
		environment: 'jsdom',
		setupFiles: ['./src/tests/setup.ts'],
		alias: {
			$lib: resolve('./src/lib'),
			$app: resolve('./src/tests/mocks/app')
		}
	},
	resolve: {
		alias: {
			$lib: resolve('./src/lib'),
			$app: resolve('./src/tests/mocks/app')
		}
	}
});
