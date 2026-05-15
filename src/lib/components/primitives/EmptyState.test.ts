import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import EmptyState from './EmptyState.svelte';

describe('EmptyState', () => {
  it('renders the title', () => {
    const { getByText } = render(EmptyState, { props: { title: 'Nothing here' } });
    expect(getByText('Nothing here')).toBeInTheDocument();
  });

  it('renders description when provided', () => {
    const { getByText } = render(EmptyState, {
      props: { title: 'X', description: 'Try cloning a repo' },
    });
    expect(getByText('Try cloning a repo')).toBeInTheDocument();
  });

  it('omits description when not provided', () => {
    const { container } = render(EmptyState, { props: { title: 'X' } });
    expect(container.querySelector('.desc')).toBeNull();
  });

  it('defaults to size=md', () => {
    const { container } = render(EmptyState, { props: { title: 'X' } });
    expect(container.querySelector('.empty')?.className).toContain('size-md');
  });

  it('applies the size class', () => {
    const { container } = render(EmptyState, { props: { title: 'X', size: 'sm' } });
    expect(container.querySelector('.empty')?.className).toContain('size-sm');
  });

  it('renders an illustration mask when illustration name is given', () => {
    const { container } = render(EmptyState, {
      props: { title: 'X', illustration: 'rocket' },
    });
    const ill = container.querySelector('.illustration') as HTMLElement;
    expect(ill).toBeTruthy();
    expect(ill.style.getPropertyValue('--ill-src')).toContain('/illustrations/rocket.svg');
    expect(ill.getAttribute('aria-hidden')).toBe('true');
  });

  it('omits the illustration node when no name is given', () => {
    const { container } = render(EmptyState, { props: { title: 'X' } });
    expect(container.querySelector('.illustration')).toBeNull();
  });
});
