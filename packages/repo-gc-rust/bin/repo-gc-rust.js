#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoGcCli = resolve(__dirname, '../../repo-gc/dist/cli.js');

const rawArgs = process.argv.slice(2);
let subcommand = rawArgs[0];
let restArgs = rawArgs.slice(1);

// Route bare --help/-h to the default subcommand so --lang is visible in help
if (!subcommand || subcommand === '--help' || subcommand === '-h') {
  subcommand = 'scan';
  if (rawArgs[0] === '--help' || rawArgs[0] === '-h') {
    restArgs = ['--help'];
  }
}

const args = [subcommand, '--lang', 'rust', ...restArgs];

const child = spawn(process.execPath, [repoGcCli, ...args], { stdio: 'inherit' });
child.on('error', (err) => {
  console.error('repo-gc runner not found. Install repo-gc alongside repo-gc-rust.');
  process.exit(1);
});
child.on('exit', (code) => process.exit(code || 0));
