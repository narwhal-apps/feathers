import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Kbd from './Kbd.svelte';

describe('Kbd', () => {
  it('renders one <kbd> per key', () => {
    const { container } = render(Kbd, { props: { keys: ['⌘', 'P'] } });
    const kbds = container.querySelectorAll('kbd');
    expect(kbds).toHaveLength(2);
    expect(kbds[0].textContent).toBe('⌘');
    expect(kbds[1].textContent).toBe('P');
  });

  it('defaults to the "default" tone class', () => {
    const { container } = render(Kbd, { props: { keys: ['A'] } });
    expect(container.querySelector('.grp')?.className).toContain('tone-default');
  });

  it('applies the on-accent tone class', () => {
    const { container } = render(Kbd, { props: { keys: ['A'], tone: 'on-accent' } });
    expect(container.querySelector('.grp')?.className).toContain('tone-on-accent');
  });

  it('renders no <kbd> when keys is empty', () => {
    const { container } = render(Kbd, { props: { keys: [] } });
    expect(container.querySelectorAll('kbd')).toHaveLength(0);
  });
});
