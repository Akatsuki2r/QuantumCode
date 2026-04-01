#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

// Determine the binary path based on platform
const platform = os.platform();
const arch = os.arch();

let binaryName = 'quantumn';
if (platform === 'win32') {
  binaryName = 'quantumn.exe';
}

const binaryPath = path.join(__dirname, '..', 'binaries', `${platform}-${arch}`, binaryName);

// Spawn the binary with all arguments
const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to start quantumn:', err.message);
  console.error('\nTry reinstalling:');
  console.error('  npm install -g @quantumn/code');
  process.exit(1);
});