import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import Input from './Input.svelte';

describe('Input', () => {
  it('renders an <input> with the given placeholder', () => {
    const { container } = render(Input, { props: { placeholder: 'name' } });
    const el = container.querySelector('input') as HTMLInputElement;
    expect(el.placeholder).toBe('name');
    expect(el.type).toBe('text');
  });

  it('honors the type prop', () => {
    const { container } = render(Input, { props: { type: 'email' } });
    expect((container.querySelector('input') as HTMLInputElement).type).toBe('email');
  });

  it('applies variant=mono class', () => {
    const { container } = render(Input, { props: { variant: 'mono' } });
    expect(container.querySelector('input')?.className).toContain('variant-mono');
  });

  it('applies invalid class when invalid=true', () => {
    const { container } = render(Input, { props: { invalid: true } });
    expect(container.querySelector('input')?.className).toContain('invalid');
  });

  it('disabled attribute reflects the prop', () => {
    const { container } = render(Input, { props: { disabled: true } });
    expect((container.querySelector('input') as HTMLInputElement).disabled).toBe(true);
  });

  it('fires oninput on user typing', async () => {
    const oninput = vi.fn();
    const { container } = render(Input, { props: { oninput } });
    const el = container.querySelector('input') as HTMLInputElement;
    await fireEvent.input(el, { target: { value: 'hi' } });
    expect(oninput).toHaveBeenCalledOnce();
  });

  it('fires onkeydown', async () => {
    const onkeydown = vi.fn();
    const { container } = render(Input, { props: { onkeydown } });
    const el = container.querySelector('input') as HTMLInputElement;
    await fireEvent.keyDown(el, { key: 'Enter' });
    expect(onkeydown).toHaveBeenCalledOnce();
  });
});
