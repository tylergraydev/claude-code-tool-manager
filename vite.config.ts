import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true
	},
	envPrefix: ['VITE_', 'TAURI_'],
	build: {
		target: 'esnext',
		rollupOptions: {
			output: {
				manualChunks(id) {
					if (id.includes('lucide-svelte')) return 'icons';
					if (id.includes('@tauri-apps')) return 'tauri';
				}
			}
		}
	}
});
