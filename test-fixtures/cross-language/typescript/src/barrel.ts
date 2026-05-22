// Barrel and utility functions
import fs from 'node:fs';

export function helper(): string {
  return 'helper';
}

export function readFileSilent(path: string): void {
  try {
    fs.readFileSync(path);
  } catch {
    // silently ignore — error-swallow
  }
}

export function parseNumberSilent(input: string): number {
  const n = parseInt(input, 10);
  if (isNaN(n)) {
    return 0;
  }
  return n;
}
