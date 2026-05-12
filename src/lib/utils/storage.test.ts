import { describe, it, expect, beforeEach } from 'vitest';
import { loadStorageInt } from './storage';

describe('loadStorageInt', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('returns the fallback when the key is missing', () => {
    expect(loadStorageInt('missing', 100, 0, 1000)).toBe(100);
  });

  it('returns the parsed integer when in range', () => {
    window.localStorage.setItem('w', '320');
    expect(loadStorageInt('w', 100, 0, 1000)).toBe(320);
  });

  it('returns the fallback when the value is not numeric', () => {
    window.localStorage.setItem('w', 'not-a-number');
    expect(loadStorageInt('w', 100, 0, 1000)).toBe(100);
  });

  it('returns the fallback when the value is below min', () => {
    window.localStorage.setItem('w', '50');
    expect(loadStorageInt('w', 200, 100, 500)).toBe(200);
  });

  it('returns the fallback when the value is above max', () => {
    window.localStorage.setItem('w', '999');
    expect(loadStorageInt('w', 200, 100, 500)).toBe(200);
  });

  it('accepts the bounds inclusively', () => {
    window.localStorage.setItem('lo', '100');
    window.localStorage.setItem('hi', '500');
    expect(loadStorageInt('lo', 200, 100, 500)).toBe(100);
    expect(loadStorageInt('hi', 200, 100, 500)).toBe(500);
  });

  it('parses leading-int strings (parseInt semantics)', () => {
    window.localStorage.setItem('w', '320px');
    expect(loadStorageInt('w', 100, 0, 1000)).toBe(320);
  });
});
