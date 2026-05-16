#!/usr/bin/env node
'use strict';
const path = require('path');
const { spawnSync } = require('child_process');
const os = require('os');
const fs = require('fs');

const bin = os.platform() === 'win32' ? 'repo-gc.exe' : 'repo-gc';
const binPath = path.join(__dirname, '..', 'target', 'release', bin);

if (!fs.existsSync(binPath)) {
  console.error(
    'repo-gc-rust: binary not found at ' + binPath + '\n' +
    'Run: cargo build --release'
  );
  process.exit(1);
}

const r = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit' });
if (r.error) {
  console.error('repo-gc-rust: ' + r.error.message);
  process.exit(1);
}
process.exit(r.status || 0);
