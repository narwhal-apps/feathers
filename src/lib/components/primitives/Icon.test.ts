import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Icon from './Icon.svelte';

describe('Icon', () => {
  it('renders an SVG for a known icon name', () => {
    const { container } = render(Icon, { props: { name: 'Check', size: 20 } });
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });

  it('falls back to HelpCircle for an unknown icon name', () => {
    const { container } = render(Icon, { props: { name: 'NopeNotReal' } });
    expect(container.querySelector('svg')).toBeInTheDocument();
  });
});
