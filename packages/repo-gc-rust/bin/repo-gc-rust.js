#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoGcCli = resolve(__dirname, '../../repo-gc/dist/cli.js');

const rawArgs = process.argv.slice(2);
const subcommand = rawArgs[0] || 'scan';
const restArgs = rawArgs.slice(1);
const args = [subcommand, '--lang', 'rust', ...restArgs];

const child = spawn(process.execPath, [repoGcCli, ...args], { stdio: 'inherit' });
child.on('error', (err) => {
  console.error('repo-gc runner not found. Install repo-gc alongside repo-gc-rust.');
  process.exit(1);
});
child.on('exit', (code) => process.exit(code || 0));
