import * as fs from 'node:fs';
import * as path from 'node:path';
import { globbySync } from 'globby';

const EXCLUDED_DIRS = ['node_modules', 'dist', 'build', '.next', '.git', 'target', '__pycache__'];

import type { SourceFile, LanguagePlugin } from '../plugin';

function isTestFile(relativePath: string, patterns: RegExp[]): boolean {
  return patterns.some((p) => p.test(relativePath));
}

function getGlobPatterns(sourceRoots: string[], extensions: string[]): string[] {
  if (sourceRoots.length === 0) return [];
  const extList = extensions.map((e) => e.replace('.', '')).join(',');
  const extGlob = `**/*.{${extList}}`;
  return sourceRoots.map((root) => path.join(root, extGlob).replace(/\\/g, '/'));
}

export function enumerateFiles(
  sourceRoots: string[],
  workspaceRoot: string,
  plugin: LanguagePlugin,
  includeTests: boolean,
): { files: SourceFile[]; skipped: number } {
  const seen = new Set<string>();
  const files: SourceFile[] = [];
  let skipped = 0;

  const patterns = getGlobPatterns(sourceRoots, plugin.fileExtensions);
  if (patterns.length === 0) return { files, skipped };

  const matches = globbySync(patterns, {
    gitignore: true,
    ignore: EXCLUDED_DIRS.map((d) => `**/${d}/**`),
    absolute: true,
    onlyFiles: true,
  });

  for (const filePath of matches) {
    const canonical = fs.realpathSync(filePath);
    if (seen.has(canonical)) continue;
    seen.add(canonical);

    const relativePath = path.relative(workspaceRoot, filePath);
    if (!includeTests && isTestFile(relativePath, plugin.testFilePatterns)) {
      skipped++;
      continue;
    }

    if (filePath.endsWith('.d.ts')) {
      skipped++;
      continue;
    }

    let sizeBytes = 0;
    let lineCount = 0;
    try {
      const content = fs.readFileSync(filePath, 'utf-8');
      sizeBytes = Buffer.byteLength(content, 'utf-8');
      lineCount = content.split('\n').length;
    } catch {
      skipped++;
      continue;
    }

    files.push({
      path: canonical,
      relativePath,
      sizeBytes,
      lineCount,
      language: plugin.name,
      metadata: {},
    });
  }

  return { files, skipped };
}
