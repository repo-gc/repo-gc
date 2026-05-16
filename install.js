#!/usr/bin/env node
'use strict';
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

const bin = os.platform() === 'win32' ? 'repo-gc.exe' : 'repo-gc';
const binPath = path.join(__dirname, 'target', 'release', bin);

if (fs.existsSync(binPath)) {
  console.log('repo-gc-rust: already built.');
  process.exit(0);
}

const check = spawnSync('cargo', ['--version'], { encoding: 'utf8' });
if (check.error || check.status !== 0) {
  console.error(
    '\nrepo-gc-rust requires Rust/Cargo.\n' +
    'Install from https://rustup.rs/ then: npm install -g repo-gc-rust\n'
  );
  process.exit(1);
}

console.log('repo-gc-rust: building release binary (~30s first time)...');
const build = spawnSync('cargo', ['build', '--release'], {
  cwd: __dirname,
  stdio: 'inherit',
});
if (build.status !== 0) {
  console.error('repo-gc-rust: build failed.');
  process.exit(build.status || 1);
}
console.log('repo-gc-rust: done.');
