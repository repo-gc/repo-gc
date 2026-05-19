import { describe, it, expect } from 'vitest';
import { estimateTokens } from '../src/token-estimate';

describe('estimateTokens', () => {
  it('returns 0 for zero bytes', () => {
    expect(estimateTokens(0)).toBe(0);
  });

  it('returns 1 for 4 bytes', () => {
    expect(estimateTokens(4)).toBe(1);
  });

  it('returns 1024 for 4096 bytes', () => {
    expect(estimateTokens(4096)).toBe(1024);
  });

  it('rounds to nearest', () => {
    expect(estimateTokens(2)).toBe(1);
    expect(estimateTokens(1)).toBe(0);
    expect(estimateTokens(6)).toBe(2);
  });
});
