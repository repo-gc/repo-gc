import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { existsSync } from 'node:fs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoGcCli = resolve(__dirname, '../../repo-gc/dist/cli.js');

if (!existsSync(repoGcCli)) {
  console.error('repo-gc umbrella runner not found. Install repo-gc alongside repo-gc-typescript.');
  process.exit(1);
}

// Rewrite argv: insert --lang <name> after the subcommand
// e.g. "repo-gc-typescript scan --path ." → "repo-gc scan --lang typescript --path ."
const rawArgs = process.argv.slice(2);
const subcommand = rawArgs[0] || 'scan';
const restArgs = rawArgs.slice(1);
const args = [subcommand, '--lang', 'typescript', ...restArgs];

const child = spawn(process.execPath, [repoGcCli, ...args], { stdio: 'inherit' });
child.on('exit', (code) => process.exit(code || 0));
