import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Layout from './+layout.svelte';

describe('+layout.svelte', () => {
  it('renders the titlebar and sidebar around the slot', () => {
    const { container } = render(Layout);
    expect(container.querySelector('.titlebar')).toBeInTheDocument();
    expect(container.querySelector('.sidebar')).toBeInTheDocument();
  });
});
