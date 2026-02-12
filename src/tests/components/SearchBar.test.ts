import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SearchBar from '$lib/components/shared/SearchBar.svelte';

describe('SearchBar Component', () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should render input with placeholder', () => {
		render(SearchBar, { props: { value: '', placeholder: 'Search MCPs...' } });

		const input = screen.getByPlaceholderText('Search MCPs...');
		expect(input).toBeInTheDocument();
	});

	it('should render with default placeholder', () => {
		render(SearchBar, { props: { value: '' } });

		const input = screen.getByPlaceholderText('Search...');
		expect(input).toBeInTheDocument();
	});

	it('should call onchange when user types', async () => {
		const onchange = vi.fn();
		render(SearchBar, { props: { value: '', onchange } });

		const input = screen.getByPlaceholderText('Search...');
		await fireEvent.input(input, { target: { value: 'test' } });

		expect(onchange).toHaveBeenCalledWith('test');
	});

	it('should show clear button when value is non-empty', () => {
		render(SearchBar, { props: { value: 'search text' } });

		const clearButton = screen.getByRole('button');
		expect(clearButton).toBeInTheDocument();
	});

	it('should not show clear button when value is empty', () => {
		render(SearchBar, { props: { value: '' } });

		const clearButton = screen.queryByRole('button');
		expect(clearButton).not.toBeInTheDocument();
	});

	it('should call onchange with empty string when clear button clicked', async () => {
		const onchange = vi.fn();
		render(SearchBar, { props: { value: 'text', onchange } });

		const clearButton = screen.getByRole('button');
		await fireEvent.click(clearButton);

		expect(onchange).toHaveBeenCalledWith('');
	});
});
