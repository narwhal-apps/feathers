import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import ContextMenuItem from './ContextMenuItem.svelte';

describe('ContextMenuItem', () => {
  it('renders the label with role=menuitem', () => {
    const { getByRole } = render(ContextMenuItem, { props: { label: 'Reset' } });
    const el = getByRole('menuitem');
    expect(el).toHaveTextContent('Reset');
  });

  it('fires onclick on click', async () => {
    const onclick = vi.fn();
    const { getByRole } = render(ContextMenuItem, {
      props: { label: 'Reset', onclick },
    });
    await fireEvent.click(getByRole('menuitem'));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('sets the disabled attribute when disabled=true', () => {
    const { getByRole } = render(ContextMenuItem, {
      props: { label: 'X', onclick: () => {}, disabled: true },
    });
    expect((getByRole('menuitem') as HTMLButtonElement).disabled).toBe(true);
  });

  it('applies the danger modifier class', () => {
    const { getByRole } = render(ContextMenuItem, {
      props: { label: 'Delete', danger: true },
    });
    expect(getByRole('menuitem').className).toContain('danger');
  });

  it('forwards title attribute', () => {
    const { getByRole } = render(ContextMenuItem, {
      props: { label: 'X', title: 'tooltip' },
    });
    expect(getByRole('menuitem').getAttribute('title')).toBe('tooltip');
  });
});
