import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import { createRawSnippet } from 'svelte';
import Tag from './Tag.svelte';

function textChildren(s: string) {
  return createRawSnippet(() => ({ render: () => s }));
}

describe('Tag', () => {
  it('renders children text', () => {
    const { container } = render(Tag, { props: { children: textChildren('+12') } });
    expect(container.querySelector('.tag')?.textContent).toBe('+12');
  });

  it('defaults to neutral tone, soft variant, sm size', () => {
    const { container } = render(Tag, { props: { children: textChildren('x') } });
    const el = container.querySelector('.tag');
    expect(el?.className).toContain('t-neutral');
    expect(el?.className).toContain('v-soft');
    expect(el?.className).toContain('s-sm');
  });

  it('applies tone, variant, size classes', () => {
    const { container } = render(Tag, {
      props: { children: textChildren('x'), tone: 'add', variant: 'solid', size: 'xs' },
    });
    const el = container.querySelector('.tag');
    expect(el?.className).toContain('t-add');
    expect(el?.className).toContain('v-solid');
    expect(el?.className).toContain('s-xs');
  });

  it('applies uppercase + mono modifier classes', () => {
    const { container } = render(Tag, {
      props: { children: textChildren('x'), uppercase: true, mono: true },
    });
    const el = container.querySelector('.tag');
    expect(el?.classList.contains('uppercase')).toBe(true);
    expect(el?.classList.contains('mono')).toBe(true);
  });

  it('forwards title attribute', () => {
    const { container } = render(Tag, {
      props: { children: textChildren('x'), title: 'tooltip' },
    });
    expect(container.querySelector('.tag')?.getAttribute('title')).toBe('tooltip');
  });
});
