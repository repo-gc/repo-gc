import { defineConfig } from 'tsup';

export default defineConfig({
  entry: { plugin: 'src/plugin.ts', cli: 'src/cli.ts' },
  format: ['esm'],
  target: 'node18',
  clean: true,
  sourcemap: true,
  dts: true,
  banner: {
    js: '#!/usr/bin/env node',
  },
  external: ['oxc-parser', 'unrs-resolver', 'repo-gc-shared'],
});
