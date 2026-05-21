import { describe, it, expect } from 'vitest';
import {
  analyzeContextBombs,
  analyzeDeadWeight,
  analyzeCoupling,
  analyzeReexportEntropy,
  analyzeDuplication,
  analyzeUnusedImports,
  Severity,
} from '../src/shared/index';

function makeFileData(overrides: Partial<import('../src/shared/types').FileData> = {}): import('../src/shared/types').FileData {
  return {
    path: '/root/src/foo.ts',
    relativePath: 'src/foo.ts',
    lineCount: 200,
    sizeBytes: 4000,
    modulePath: 'pkg::src::foo',
    isEntryPoint: false,
    isPackageInit: false,
    functionCount: 3,
    classCount: 0,
    implBlockCount: 0,
    imports: ['./bar'],
    importedNames: ['bar'],
    allIdentifiers: new Set(['bar', 'x', 'y']),
    moduleDeclarations: [],
    allExport: null,
    exports: [],
    functionBodies: [],
    ...overrides,
  };
}

// ─── context-bombs ────────────────────────────────────────────

describe('analyzeContextBombs', () => {
  it('returns null when under line limit', () => {
    const data = makeFileData({ lineCount: 100, sizeBytes: 2000 });
    const result = analyzeContextBombs(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).toBeNull();
  });

  it('returns Medium severity at 1x limit', () => {
    const data = makeFileData({ lineCount: 500, sizeBytes: 20000 });
    const result = analyzeContextBombs(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).not.toBeNull();
    expect(result!.severity).toBe(Severity.Medium);
    expect(result!.confidence).toBe(0.75);
  });

  it('returns High severity at 2x limit with confidence 0.85', () => {
    const data = makeFileData({ lineCount: 1000, sizeBytes: 40000 });
    const result = analyzeContextBombs(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).not.toBeNull();
    expect(result!.severity).toBe(Severity.High);
    expect(result!.confidence).toBe(0.85);
  });

  it('returns Critical severity at 4x limit with confidence 0.95', () => {
    const data = makeFileData({ lineCount: 2000, sizeBytes: 80000 });
    const result = analyzeContextBombs(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).not.toBeNull();
    expect(result!.severity).toBe(Severity.Critical);
    expect(result!.confidence).toBe(0.95);
  });

  it('includes function_count and class_count in evidence when thresholds met', () => {
    const data = makeFileData({ lineCount: 600, sizeBytes: 24000, functionCount: 15, classCount: 5 });
    const result = analyzeContextBombs(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result!.evidence.some(e => e.includes('function_count: 15'))).toBe(true);
    expect(result!.evidence.some(e => e.includes('class_count: 5'))).toBe(true);
  });
});

// ─── dead-weight ──────────────────────────────────────────────

describe('analyzeDeadWeight', () => {
  it('skips entry points', () => {
    const data = makeFileData({ isEntryPoint: true });
    const graph = { fanIn: new Map(), fanOut: new Map() };
    const result = analyzeDeadWeight([data], graph, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('skips package init files', () => {
    const data = makeFileData({ isPackageInit: true });
    const graph = { fanIn: new Map(), fanOut: new Map() };
    const result = analyzeDeadWeight([data], graph, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('skips files with fan-in > 0', () => {
    const data = makeFileData();
    const graph = { fanIn: new Map([['/root/src/foo.ts', 1]]), fanOut: new Map() };
    const result = analyzeDeadWeight([data], graph, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('flags orphaned files', () => {
    const data = makeFileData({ lineCount: 200, sizeBytes: 4000 });
    const graph = { fanIn: new Map(), fanOut: new Map() };
    const result = analyzeDeadWeight([data], graph, { value: 1 });
    expect(result).toHaveLength(1);
    expect(result[0].severity).toBe(Severity.Medium);
    expect(result[0].confidence).toBe(0.6);
  });

  it('escalates to Critical for large orphaned files', () => {
    const data = makeFileData({ lineCount: 1200, sizeBytes: 48000 });
    const graph = { fanIn: new Map(), fanOut: new Map() };
    const result = analyzeDeadWeight([data], graph, { value: 1 });
    expect(result[0].severity).toBe(Severity.Critical);
  });
});

// ─── coupling ─────────────────────────────────────────────────

describe('analyzeCoupling', () => {
  it('returns empty when under both limits', () => {
    const data = makeFileData();
    const graph = { fanIn: new Map([['/root/src/foo.ts', 3]]), fanOut: new Map([['/root/src/foo.ts', 5]]) };
    const result = analyzeCoupling(data, graph, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('classifies high fan-in as api pattern', () => {
    const data = makeFileData();
    const graph = { fanIn: new Map([['/root/src/foo.ts', 15]]), fanOut: new Map() };
    const result = analyzeCoupling(data, graph, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).toHaveLength(1);
    expect(result[0].evidence.some(e => e.includes('pattern: api'))).toBe(true);
    expect(result[0].confidence).toBe(0.65);
  });

  it('uses ch- prefix', () => {
    const data = makeFileData();
    const graph = { fanIn: new Map([['/root/src/foo.ts', 15]]), fanOut: new Map() };
    const result = analyzeCoupling(data, graph, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result[0].id).toMatch(/^ch-\d{3}$/);
  });
});

// ─── reexport-entropy ─────────────────────────────────────────

describe('analyzeReexportEntropy', () => {
  it('returns null with too few re-exports', () => {
    const data = makeFileData({
      exports: [{ sourcePath: './a', itemCount: 1, isWildcard: false }],
    });
    const result = analyzeReexportEntropy(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).toBeNull();
  });

  it('flags wildcard re-exports as High', () => {
    const data = makeFileData({
      exports: [
        { sourcePath: './a', itemCount: 3, isWildcard: false },
        { sourcePath: './b', itemCount: 5, isWildcard: false },
        { sourcePath: './c', itemCount: 4, isWildcard: true },
      ],
    });
    const result = analyzeReexportEntropy(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result).not.toBeNull();
    expect(result!.severity).toBe(Severity.High);
  });

  it('uses re- prefix', () => {
    const data = makeFileData({
      exports: [
        { sourcePath: './a', itemCount: 5, isWildcard: false },
        { sourcePath: './b', itemCount: 5, isWildcard: false },
        { sourcePath: './c', itemCount: 5, isWildcard: false },
      ],
    });
    const result = analyzeReexportEntropy(data, { lineCountLimit: 500, fanInLimit: 10, fanOutLimit: 15, reexportLimit: 10 }, { value: 1 });
    expect(result!.id).toMatch(/^re-\d{3}$/);
  });
});

// ─── duplication ──────────────────────────────────────────────

describe('analyzeDuplication', () => {
  it('returns empty when no duplicates found', () => {
    const files = [
      makeFileData({
        relativePath: 'src/a.ts',
        functionBodies: [{ name: 'foo', rawBody: '{}', normalizedBody: '_unique_body_that_is_long_enough_to_pass_min_length_check' }],
      }),
      makeFileData({
        relativePath: 'src/b.ts',
        functionBodies: [{ name: 'bar', rawBody: '{}', normalizedBody: '_different_body_that_is_also_long_enough_to_pass' }],
      }),
    ];
    const result = analyzeDuplication(files, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('detects duplicate function bodies across files', () => {
    const dupBody = '_shared_body_that_is_definitely_long_enough_to_pass_min_length';
    const files = [
      makeFileData({
        relativePath: 'src/a.ts',
        functionBodies: [{ name: 'compute', rawBody: '{ let x = 1; }', normalizedBody: dupBody }],
      }),
      makeFileData({
        relativePath: 'src/b.ts',
        functionBodies: [{ name: 'compute', rawBody: '{ let y = 2; }', normalizedBody: dupBody }],
      }),
    ];
    const result = analyzeDuplication(files, { value: 1 });
    expect(result).toHaveLength(1);
    expect(result[0].evidence.some(e => e.includes('file_count: 2'))).toBe(true);
  });

  it('skips test-only function names', () => {
    const dupBody = '_shared_body_that_is_definitely_long_enough_to_pass_min_length';
    const files = [
      makeFileData({
        relativePath: 'src/a.ts',
        functionBodies: [{ name: 'setup_for_test', rawBody: '{ let x = 1; }', normalizedBody: dupBody }],
      }),
      makeFileData({
        relativePath: 'src/b.ts',
        functionBodies: [{ name: 'setup_for_test', rawBody: '{ let y = 2; }', normalizedBody: dupBody }],
      }),
    ];
    const result = analyzeDuplication(files, { value: 1 });
    expect(result).toHaveLength(0);
  });

  it('uses dup- prefix', () => {
    const dupBody = '_shared_body_that_is_definitely_long_enough_to_pass_min_length';
    const files = [
      makeFileData({ relativePath: 'src/a.ts', functionBodies: [{ name: 'f', rawBody: '{}', normalizedBody: dupBody }] }),
      makeFileData({ relativePath: 'src/b.ts', functionBodies: [{ name: 'f', rawBody: '{}', normalizedBody: dupBody }] }),
    ];
    const result = analyzeDuplication(files, { value: 1 });
    expect(result[0].id).toMatch(/^dup-\d{3}$/);
  });
});

// ─── unused-imports ───────────────────────────────────────────

describe('analyzeUnusedImports', () => {
  it('returns null when all imports are used', () => {
    const data = makeFileData({
      importedNames: ['foo', 'bar'],
      allIdentifiers: new Set(['foo', 'bar', 'baz']),
    });
    const result = analyzeUnusedImports(data, { value: 1 });
    expect(result).toBeNull();
  });

  it('requires minimum 2 unused imports', () => {
    const data = makeFileData({
      importedNames: ['foo', 'bar', 'baz'],
      allIdentifiers: new Set(['foo']),
    });
    const result = analyzeUnusedImports(data, { value: 1 });
    expect(result).not.toBeNull();
    expect(result!.evidence.some(e => e.includes('unused_count: 2'))).toBe(true);
  });

  it('escalates to Medium at 5+ unused', () => {
    const data = makeFileData({
      importedNames: ['a', 'b', 'c', 'd', 'e'],
      allIdentifiers: new Set([]),
    });
    const result = analyzeUnusedImports(data, { value: 1 });
    expect(result!.severity).toBe(Severity.Medium);
  });

  it('excludes compiler names', () => {
    const data = makeFileData({
      importedNames: ['React', 'useState'],
      allIdentifiers: new Set([]),
    });
    const result = analyzeUnusedImports(data, { value: 1 }, { compilerNames: new Set(['React']) });
    expect(result).toBeNull();
  });

  it('uses ui- prefix', () => {
    const data = makeFileData({
      importedNames: ['a', 'b', 'c'],
      allIdentifiers: new Set([]),
    });
    const result = analyzeUnusedImports(data, { value: 1 });
    expect(result!.id).toMatch(/^ui-\d{3}$/);
  });
});
