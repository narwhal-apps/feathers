import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Spinner from './Spinner.svelte';

describe('Spinner', () => {
  it('has role="status" and a default aria-label of "Loading"', () => {
    const { getByRole } = render(Spinner);
    const el = getByRole('status');
    expect(el.getAttribute('aria-label')).toBe('Loading');
  });

  it('respects a custom label', () => {
    const { getByRole } = render(Spinner, { props: { label: 'Fetching' } });
    expect(getByRole('status').getAttribute('aria-label')).toBe('Fetching');
  });

  it('size=xs gives a 10px box with 1.5px stroke', () => {
    const { container } = render(Spinner, { props: { size: 'xs' } });
    const el = container.querySelector('.spinner') as HTMLElement;
    expect(el.style.width).toBe('10px');
    expect(el.style.height).toBe('10px');
    expect(el.style.borderWidth).toBe('1.5px');
  });

  it('size=sm gives a 12px box with 1.5px stroke', () => {
    const { container } = render(Spinner, { props: { size: 'sm' } });
    const el = container.querySelector('.spinner') as HTMLElement;
    expect(el.style.width).toBe('12px');
    expect(el.style.borderWidth).toBe('1.5px');
  });

  it('size=md gives a 16px box with 2px stroke', () => {
    const { container } = render(Spinner, { props: { size: 'md' } });
    const el = container.querySelector('.spinner') as HTMLElement;
    expect(el.style.width).toBe('16px');
    expect(el.style.borderWidth).toBe('2px');
  });
});
