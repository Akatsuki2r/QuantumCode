#!/usr/bin/env node

/**
 * Quantumn Code - Pre-uninstall script
 * Cleans up any installed binaries
 */

const fs = require('fs');
const path = require('path');

function main() {
  console.log('Quantumn Code - Uninstalling...\n');

  const binDir = path.join(__dirname, '..', 'bin');

  // Remove bin directory
  if (fs.existsSync(binDir)) {
    try {
      fs.rmSync(binDir, { recursive: true, force: true });
      console.log('Removed binary directory');
    } catch (err) {
      console.warn('Could not remove binary directory:', err.message);
    }
  }

  console.log('Quantumn Code has been uninstalled.');
  console.log('Note: If you installed the binary separately, you may need to remove it manually.');
}

main();