import { describe, it, expect } from 'vitest';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { scan } from '../src/runner';
import { getThresholds } from '../src/shared/threshold';
import { typescriptPlugin } from 'repo-gc-typescript';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixturesDir = path.join(__dirname, 'fixtures', 'simple-project');

const plugins = [typescriptPlugin];

describe('integration: scan simple-project', () => {
  it('produces a valid report', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('strict'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);

    expect(report.files_analyzed).toBeGreaterThan(0);
    expect(report.total_lines).toBeGreaterThan(0);
    expect(report.total_estimated_tokens).toBeGreaterThan(0);
    expect(typeof report.global_score.ai_friction_score).toBe('number');
    expect(typeof report.global_score.context_waste_ratio).toBe('number');
    expect(Array.isArray(report.findings)).toBe(true);
  });

  it('detects context bombs (oversized files)', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('strict'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);
    const cb = report.findings.filter((f: any) => f.kind === 'context-bomb');
    expect(cb.length).toBeGreaterThan(0);
    expect(cb[0].path).toContain('big-file');
  });

  it('detects barrel file re-export entropy', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('strict'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);
    const rx = report.findings.filter((f: any) => f.kind === 'reexport-entropy');
    expect(rx.length).toBeGreaterThan(0);
    expect(rx[0].path).toContain('barrel');
  });

  it('detects code duplication', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('normal'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);
    const dup = report.findings.filter((f: any) => f.kind === 'code-duplication');
    expect(dup.length).toBeGreaterThan(0);
  });

  it('does not flag entry point as dead weight', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('normal'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);
    const dw = report.findings.filter((f: any) => f.kind === 'dead-weight');
    const hasIndex = dw.some((f: any) => f.path.includes('index.ts'));
    expect(hasIndex).toBe(false);
  });

  it('emits JSON matching Rust schema', async () => {
    const output = await scan({
      path: fixturesDir,
      format: 'json',
      threshold: getThresholds('strict'),
      includeTests: false,
      color: false,
    }, plugins);

    const report = JSON.parse(output);

    expect(report).toHaveProperty('findings');
    expect(report).toHaveProperty('global_score');
    expect(report).toHaveProperty('files_analyzed');
    expect(report).toHaveProperty('files_skipped');
    expect(report).toHaveProperty('total_lines');
    expect(report).toHaveProperty('total_estimated_tokens');

    for (const f of report.findings) {
      expect(f).toHaveProperty('id');
      expect(f).toHaveProperty('kind');
      expect(f).toHaveProperty('severity');
      expect(f).toHaveProperty('confidence');
      expect(f).toHaveProperty('path');
      expect(f).toHaveProperty('summary');
      expect(f).toHaveProperty('reasons');
      expect(f).toHaveProperty('evidence');
      expect(f).toHaveProperty('suggested_next_step');
      expect(f).toHaveProperty('estimated_tokens');
    }

    const gs = report.global_score;
    expect(gs).toHaveProperty('ai_friction_score');
    expect(gs).toHaveProperty('context_waste_score');
    expect(gs).toHaveProperty('structural_entropy_score');
    expect(gs).toHaveProperty('context_waste_ratio');
    expect(gs).toHaveProperty('estimated_waste_pct');
  });
});
