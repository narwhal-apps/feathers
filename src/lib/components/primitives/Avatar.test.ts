import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Avatar from './Avatar.svelte';

describe('Avatar', () => {
  it('renders initials from a name', () => {
    const { container } = render(Avatar, { props: { name: 'Mikael Kristiansson' } });
    expect(container.querySelector('.avatar')?.textContent?.trim()).toBe('MK');
  });

  it('uses "?" when no name is given', () => {
    const { container } = render(Avatar, { props: { name: '' } });
    expect(container.querySelector('.avatar')?.textContent?.trim()).toBe('?');
  });

  it('caps initials at 2 letters', () => {
    const { container } = render(Avatar, { props: { name: 'A B C D E' } });
    expect(container.querySelector('.avatar')?.textContent?.trim()).toBe('AB');
  });

  it('sets width/height to the given size in px', () => {
    const { container } = render(Avatar, { props: { name: 'X', size: 24 } });
    const el = container.querySelector('.avatar') as HTMLElement;
    expect(el.style.width).toBe('24px');
    expect(el.style.height).toBe('24px');
  });

  it('exposes name as aria-label and title', () => {
    const { container } = render(Avatar, { props: { name: 'Alice' } });
    const el = container.querySelector('.avatar') as HTMLElement;
    expect(el.getAttribute('aria-label')).toBe('Alice');
    expect(el.getAttribute('title')).toBe('Alice');
  });

  it('falls back to "unknown author" aria-label when name empty', () => {
    const { container } = render(Avatar, { props: { name: '' } });
    expect(container.querySelector('.avatar')?.getAttribute('aria-label')).toBe('unknown author');
  });

  it('renders an <img> when url is provided', () => {
    const { container } = render(Avatar, {
      props: { name: 'X', url: 'https://example.com/a.png' },
    });
    expect(container.querySelector('.avatar.has-img')).toBeTruthy();
    expect(container.querySelector('img')?.getAttribute('src')).toBe('https://example.com/a.png');
  });

  it('renders initials (no img) when url is null/undefined', () => {
    const { container } = render(Avatar, { props: { name: 'Bob', url: null } });
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('.avatar')?.textContent?.trim()).toBe('B');
  });

  it('produces a stable hue per email (gradient background)', () => {
    const { container: c1 } = render(Avatar, { props: { name: 'X', email: 'a@a.com' } });
    const { container: c2 } = render(Avatar, { props: { name: 'X', email: 'a@a.com' } });
    const bg1 = (c1.querySelector('.avatar') as HTMLElement).style.background;
    const bg2 = (c2.querySelector('.avatar') as HTMLElement).style.background;
    expect(bg1).toBe(bg2);
    expect(bg1).toMatch(/linear-gradient/);
  });
});
