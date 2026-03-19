import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

	export default defineConfig({
		plugins: [svelte({ hot: !process.env.VITEST })],
		test: {
			include: ['src/**/*.{test,spec}.{js,ts}'],
			globals: true,
			environment: 'happy-dom',
		env: {
			STL_SKIP_AUTO_CLEANUP: '1'
		},
			setupFiles: ['./src/tests/setup.ts'],
			testTimeout: 10000,
			hookTimeout: 30000,
			fileParallelism: true,
			isolate: true,
		alias: {
			$lib: resolve('./src/lib'),
			$app: resolve('./src/tests/mocks/app')
		},
		coverage: {
			provider: 'v8',
			reporter: ['text', 'html', 'lcov'],
			reportsDirectory: './coverage',
			include: ['src/lib/**/*.{ts,svelte}'],
			exclude: [
				'src/lib/types/**',
				'src/**/*.test.ts',
				'src/**/*.spec.ts',
				'src/tests/**'
			],
			reportOnFailure: true,
			thresholds: {
				statements: 0,
				branches: 0,
				functions: 0,
				lines: 0
			}
		}
	},
	resolve: {
		conditions: ['browser'],
		alias: {
			$lib: resolve('./src/lib'),
			$app: resolve('./src/tests/mocks/app'),
			'lucide-svelte': resolve('./src/tests/mocks/lucide-svelte.ts')
		}
	}
});
