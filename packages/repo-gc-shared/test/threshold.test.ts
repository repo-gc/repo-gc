import { describe, it, expect } from 'vitest';
import { getThresholds } from '../src/threshold';

describe('getThresholds', () => {
  it('returns strict preset', () => {
    const t = getThresholds('strict');
    expect(t.lineCountLimit).toBe(300);
    expect(t.fanInLimit).toBe(5);
    expect(t.fanOutLimit).toBe(10);
    expect(t.reexportLimit).toBe(5);
  });

  it('returns normal preset', () => {
    const t = getThresholds('normal');
    expect(t.lineCountLimit).toBe(500);
    expect(t.fanInLimit).toBe(10);
    expect(t.fanOutLimit).toBe(15);
    expect(t.reexportLimit).toBe(10);
  });

  it('returns relaxed preset', () => {
    const t = getThresholds('relaxed');
    expect(t.lineCountLimit).toBe(1000);
    expect(t.fanInLimit).toBe(20);
    expect(t.fanOutLimit).toBe(25);
    expect(t.reexportLimit).toBe(20);
  });

  it('strict is tighter than normal', () => {
    const s = getThresholds('strict');
    const n = getThresholds('normal');
    expect(s.lineCountLimit).toBeLessThan(n.lineCountLimit);
    expect(s.fanInLimit).toBeLessThan(n.fanInLimit);
    expect(s.fanOutLimit).toBeLessThan(n.fanOutLimit);
    expect(s.reexportLimit).toBeLessThan(n.reexportLimit);
  });
});
