import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Pill from './Pill.svelte';

describe('Pill', () => {
  it('renders the label', () => {
    const { container } = render(Pill, { props: { label: 'main' } });
    expect(container.textContent).toContain('main');
  });

  it('applies the tone class', () => {
    const { container } = render(Pill, {
      props: { label: 'PR open', tone: 'success' },
    });
    expect(container.querySelector('.pill')?.className).toContain('pill-success');
  });
});
