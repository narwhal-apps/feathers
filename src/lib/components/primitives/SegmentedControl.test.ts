import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import SegmentedControl from './SegmentedControl.svelte';

describe('SegmentedControl', () => {
  const opts = [
    { value: 'unified', label: 'Unified' },
    { value: 'split', label: 'Split' },
  ];

  it('renders one role=radio per option, marks the selected one aria-checked', () => {
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'split' },
    });
    const radios = container.querySelectorAll('[role="radio"]');
    expect(radios).toHaveLength(2);
    expect(radios[0].getAttribute('aria-checked')).toBe('false');
    expect(radios[1].getAttribute('aria-checked')).toBe('true');
  });

  it('exposes role=radiogroup with the given aria-label', () => {
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'unified', ariaLabel: 'Diff layout' },
    });
    const grp = container.querySelector('[role="radiogroup"]') as HTMLElement;
    expect(grp.getAttribute('aria-label')).toBe('Diff layout');
  });

  it('fires onChange and updates value when an unselected option is clicked', async () => {
    const onChange = vi.fn();
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'unified', onChange },
    });
    const buttons = container.querySelectorAll('button');
    await fireEvent.click(buttons[1]);
    expect(onChange).toHaveBeenCalledWith('split');
  });

  it('does NOT fire onChange when re-clicking the already-selected option', async () => {
    const onChange = vi.fn();
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'unified', onChange },
    });
    await fireEvent.click(container.querySelectorAll('button')[0]);
    expect(onChange).not.toHaveBeenCalled();
  });

  it('applies the size class', () => {
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'unified', size: 'sm' },
    });
    expect(container.querySelector('.seg')?.className).toContain('seg-sm');
  });

  it('disabled flag disables every button + adds .disabled to the group', () => {
    const { container } = render(SegmentedControl, {
      props: { options: opts, value: 'unified', disabled: true },
    });
    expect(container.querySelector('.seg.disabled')).toBeTruthy();
    container.querySelectorAll('button').forEach((b) => {
      expect((b as HTMLButtonElement).disabled).toBe(true);
    });
  });
});
