import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SearchBar from '$lib/components/shared/SearchBar.svelte';

describe('SearchBar', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render with default placeholder', () => {
			render(SearchBar, { props: { value: '' } });

			const input = screen.getByPlaceholderText('Search...');
			expect(input).toBeInTheDocument();
		});

		it('should render with custom placeholder', () => {
			render(SearchBar, { props: { value: '', placeholder: 'Find items...' } });

			const input = screen.getByPlaceholderText('Find items...');
			expect(input).toBeInTheDocument();
		});

		it('should render input with provided value', () => {
			render(SearchBar, { props: { value: 'test query' } });

			const input = screen.getByDisplayValue('test query');
			expect(input).toBeInTheDocument();
		});

		it('should render search icon', () => {
			const { container } = render(SearchBar, { props: { value: '' } });

			// Search icon should be present (SVG with magnifying glass path)
			const svg = container.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});

		it('should not show clear button when value is empty', () => {
			render(SearchBar, { props: { value: '' } });

			// Clear button should not exist
			const clearButton = screen.queryByRole('button');
			expect(clearButton).not.toBeInTheDocument();
		});

		it('should show clear button when value is not empty', () => {
			render(SearchBar, { props: { value: 'test' } });

			const clearButton = screen.getByRole('button');
			expect(clearButton).toBeInTheDocument();
		});
	});

	describe('input handling', () => {
		it('should call onchange when input value changes', async () => {
			const onchange = vi.fn();
			render(SearchBar, { props: { value: '', onchange } });

			const input = screen.getByPlaceholderText('Search...');
			await fireEvent.input(input, { target: { value: 'new value' } });

			expect(onchange).toHaveBeenCalledWith('new value');
		});

		it('should not throw when onchange is not provided', async () => {
			render(SearchBar, { props: { value: '' } });

			const input = screen.getByPlaceholderText('Search...');

			// Should not throw
			await expect(
				fireEvent.input(input, { target: { value: 'test' } })
			).resolves.not.toThrow();
		});

		it('should handle multiple input changes', async () => {
			const onchange = vi.fn();
			render(SearchBar, { props: { value: '', onchange } });

			const input = screen.getByPlaceholderText('Search...');

			await fireEvent.input(input, { target: { value: 'a' } });
			await fireEvent.input(input, { target: { value: 'ab' } });
			await fireEvent.input(input, { target: { value: 'abc' } });

			expect(onchange).toHaveBeenCalledTimes(3);
			expect(onchange).toHaveBeenLastCalledWith('abc');
		});
	});

	describe('clear button', () => {
		it('should clear value when clear button is clicked', async () => {
			const onchange = vi.fn();
			render(SearchBar, { props: { value: 'test query', onchange } });

			const clearButton = screen.getByRole('button');
			await fireEvent.click(clearButton);

			expect(onchange).toHaveBeenCalledWith('');
		});

		it('should render clear button with X icon', () => {
			const { container } = render(SearchBar, { props: { value: 'test' } });

			const clearButton = screen.getByRole('button');
			expect(clearButton).toBeInTheDocument();
			// Button should contain the X icon
			const svg = clearButton.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});

		it('should not throw when clearing without onchange handler', async () => {
			render(SearchBar, { props: { value: 'test' } });

			const clearButton = screen.getByRole('button');

			// Should not throw
			await expect(fireEvent.click(clearButton)).resolves.not.toThrow();
		});
	});

	describe('styling', () => {
		it('should have correct input type', () => {
			render(SearchBar, { props: { value: '' } });

			const input = screen.getByPlaceholderText('Search...');
			expect(input).toHaveAttribute('type', 'text');
		});

		it('should apply correct CSS classes to container', () => {
			const { container } = render(SearchBar, { props: { value: '' } });

			const wrapper = container.querySelector('.relative');
			expect(wrapper).toBeInTheDocument();
		});
	});
});
