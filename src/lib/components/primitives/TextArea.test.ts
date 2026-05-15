import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import TextArea from './TextArea.svelte';

describe('TextArea', () => {
  it('renders a <textarea> with placeholder + default rows=4', () => {
    const { container } = render(TextArea, { props: { placeholder: 'why?' } });
    const el = container.querySelector('textarea') as HTMLTextAreaElement;
    expect(el.placeholder).toBe('why?');
    expect(el.rows).toBe(4);
  });

  it('honors a custom rows prop', () => {
    const { container } = render(TextArea, { props: { rows: 8 } });
    expect((container.querySelector('textarea') as HTMLTextAreaElement).rows).toBe(8);
  });

  it('applies the resize style', () => {
    const { container } = render(TextArea, { props: { resize: 'none' } });
    expect((container.querySelector('textarea') as HTMLTextAreaElement).style.resize).toBe('none');
  });

  it('applies invalid class', () => {
    const { container } = render(TextArea, { props: { invalid: true } });
    expect(container.querySelector('textarea')?.className).toContain('invalid');
  });

  it('applies variant=mono class', () => {
    const { container } = render(TextArea, { props: { variant: 'mono' } });
    expect(container.querySelector('textarea')?.className).toContain('variant-mono');
  });

  it('fires oninput on user typing', async () => {
    const oninput = vi.fn();
    const { container } = render(TextArea, { props: { oninput } });
    await fireEvent.input(container.querySelector('textarea')!, { target: { value: 'x' } });
    expect(oninput).toHaveBeenCalledOnce();
  });
});
