import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Banner from './Banner.svelte';

describe('Banner', () => {
  it('defaults to tone=info with role=status', () => {
    const { container } = render(Banner, { props: { title: 'Hi' } });
    const el = container.querySelector('.banner') as HTMLElement;
    expect(el.className).toContain('tone-info');
    expect(el.getAttribute('role')).toBe('status');
  });

  it('uses role=alert for tone=error', () => {
    const { container } = render(Banner, { props: { tone: 'error', title: 'Boom' } });
    const el = container.querySelector('.banner') as HTMLElement;
    expect(el.className).toContain('tone-error');
    expect(el.getAttribute('role')).toBe('alert');
  });

  it('renders the title', () => {
    const { container } = render(Banner, { props: { title: 'A title' } });
    expect(container.querySelector('.title')?.textContent).toBe('A title');
  });

  it('renders the default icon by default', () => {
    const { container } = render(Banner, { props: { title: 'X' } });
    expect(container.querySelector('.icon')).toBeTruthy();
  });

  it('hides the icon when icon=null', () => {
    const { container } = render(Banner, { props: { title: 'X', icon: null } });
    expect(container.querySelector('.icon')).toBeNull();
  });

  it('omits title element when no title given', () => {
    const { container } = render(Banner, { props: {} });
    expect(container.querySelector('.title')).toBeNull();
  });
});
