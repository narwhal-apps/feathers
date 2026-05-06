import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Button from './Button.svelte';

describe('Button', () => {
  it('renders the label', () => {
    const { getByRole } = render(Button, { props: { label: 'Push' } });
    expect(getByRole('button')).toHaveTextContent('Push');
  });

  it('applies the variant class', () => {
    const { getByRole } = render(Button, {
      props: { label: 'X', variant: 'danger' },
    });
    expect(getByRole('button').className).toContain('btn-danger');
  });

  it('applies the size class', () => {
    const { getByRole } = render(Button, { props: { label: 'X', size: 'sm' } });
    expect(getByRole('button').className).toContain('btn-sm');
  });
});
