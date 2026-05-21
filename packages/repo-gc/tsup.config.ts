import { defineConfig } from 'tsup';

export default defineConfig({
  entry: { index: 'src/index.ts', cli: 'src/cli.ts' },
  format: ['esm'],
  target: 'node18',
  clean: true,
  sourcemap: true,
  dts: { entry: { index: 'src/index.ts' } },
  banner: {
    js: '#!/usr/bin/env node',
  },
  external: ['repo-gc-typescript', 'repo-gc-python'],
});
