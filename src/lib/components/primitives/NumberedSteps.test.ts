import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import { createRawSnippet } from 'svelte';
import NumberedStep from './NumberedStep.svelte';

const body = createRawSnippet(() => ({ render: () => '<strong>Step body</strong>' }));

describe('NumberedStep', () => {
  it('renders the step number', () => {
    const { container } = render(NumberedStep, { props: { n: 3, children: body } });
    expect(container.querySelector('.step-num')?.textContent).toBe('3');
  });

  it('uses an <li> with class step', () => {
    const { container } = render(NumberedStep, { props: { n: 1, children: body } });
    expect(container.querySelector('li.step')).toBeTruthy();
  });

  it('renders the children snippet inside the body div', () => {
    const { container } = render(NumberedStep, { props: { n: 2, children: body } });
    expect(container.querySelector('li.step > div')?.textContent).toContain('Step body');
  });
});
