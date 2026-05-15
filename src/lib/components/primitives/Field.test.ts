import { render } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import Field from './Field.svelte';

describe('Field', () => {
  it('renders the label text', () => {
    const { container } = render(Field, { props: { label: 'Branch name' } });
    expect(container.querySelector('.label')?.textContent).toContain('Branch name');
  });

  it('appends "(optional)" when optional=true', () => {
    const { container } = render(Field, { props: { label: 'URL', optional: true } });
    expect(container.querySelector('.optional')?.textContent).toContain('optional');
  });

  it('omits "(optional)" by default', () => {
    const { container } = render(Field, { props: { label: 'URL' } });
    expect(container.querySelector('.optional')).toBeNull();
  });

  it('renders error text and prefers it over hint/description', () => {
    const { container } = render(Field, {
      props: { label: 'X', error: 'Required', hint: 'optional hint', description: 'desc' },
    });
    expect(container.querySelector('.err')?.textContent).toBe('Required');
    expect(container.querySelector('.hint')).toBeNull();
    expect(container.querySelector('.desc')).toBeNull();
  });

  it('renders hint when there is no error', () => {
    const { container } = render(Field, {
      props: { label: 'X', hint: 'press ⌘P', description: 'shouldnt show' },
    });
    expect(container.querySelector('.hint')?.textContent).toBe('press ⌘P');
    expect(container.querySelector('.desc')).toBeNull();
  });

  it('falls back to description when there is no error or hint', () => {
    const { container } = render(Field, {
      props: { label: 'X', description: 'shown' },
    });
    expect(container.querySelector('.desc')?.textContent).toBe('shown');
  });
});
