import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

// Mock window.__TAURI__
Object.defineProperty(window, '__TAURI__', {
	value: {
		invoke: vi.fn()
	}
});

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
	observe: vi.fn(),
	unobserve: vi.fn(),
	disconnect: vi.fn()
}));

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
	writable: true,
	value: vi.fn().mockImplementation((query) => ({
		matches: false,
		media: query,
		onchange: null,
		addListener: vi.fn(),
		removeListener: vi.fn(),
		addEventListener: vi.fn(),
		removeEventListener: vi.fn(),
		dispatchEvent: vi.fn()
	}))
});
