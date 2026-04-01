#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const PLATFORM = os.platform();
const ARCH = os.arch();

// Map Node arch to Rust target
const TARGET_MAP = {
  'darwin': {
    'x64': 'x86_64-apple-darwin',
    'arm64': 'aarch64-apple-darwin'
  },
  'linux': {
    'x64': 'x86_64-unknown-linux-gnu',
    'arm64': 'aarch64-unknown-linux-gnu'
  },
  'win32': {
    'x64': 'x86_64-pc-windows-gnu'
  }
};

async function getLatestVersion() {
  return new Promise((resolve, reject) => {
    https.get('https://api.github.com/repos/Akatsuki2r/QuantumCode/releases/latest', {
      headers: { 'User-Agent': 'quantumn-code-installer' }
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          resolve(json.tag_name);
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

async function downloadBinary(version) {
  const target = TARGET_MAP[PLATFORM]?.[ARCH];
  if (!target) {
    console.error(`Unsupported platform: ${PLATFORM}-${ARCH}`);
    process.exit(1);
  }

  const ext = PLATFORM === 'win32' ? 'zip' : 'tar.gz';
  const url = `https://github.com/Akatsuki2r/QuantumCode/releases/download/${version}/quantumn-${target}.${ext}`;
  const binaryDir = path.join(__dirname, '..', 'binaries', `${PLATFORM}-${ARCH}`);
  const binaryName = PLATFORM === 'win32' ? 'quantumn.exe' : 'quantumn';
  const binaryPath = path.join(binaryDir, binaryName);

  fs.mkdirSync(binaryDir, { recursive: true });

  console.log(`Downloading Quantumn Code ${version} for ${PLATFORM}-${ARCH}...`);

  // Download and extract
  const tmpFile = path.join(os.tmpdir(), `quantumn-${Date.now()}.${ext}`);

  await new Promise((resolve, reject) => {
    https.get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        // Follow redirect
        https.get(res.headers.location, (res2) => {
          const file = fs.createWriteStream(tmpFile);
          res2.pipe(file);
          file.on('finish', () => {
            file.close();
            resolve();
          });
        }).on('error', reject);
      } else {
        const file = fs.createWriteStream(tmpFile);
        res.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
      }
    }).on('error', reject);
  });

  // Extract
  console.log('Extracting...');
  if (PLATFORM === 'win32') {
    // For Windows, we'd need a zip extractor, but for now just copy
    // In production, use adm-zip or similar
    console.log('Please extract manually on Windows');
  } else {
    execSync(`tar -xzf "${tmpFile}" -C "${binaryDir}" --strip-components=1`);
  }

  // Make executable
  if (PLATFORM !== 'win32') {
    fs.chmodSync(binaryPath, '755');
  }

  // Cleanup
  fs.unlinkSync(tmpFile);

  console.log('Quantumn Code installed successfully!');
}

async function main() {
  try {
    const version = await getLatestVersion();
    await downloadBinary(version);
  } catch (err) {
    console.error('Installation failed:', err.message);
    console.log('\nYou can also build from source:');
    console.log('  git clone https://github.com/Akatsuki2r/QuantumCode.git');
    console.log('  cd QuantumCode');
    console.log('  cargo build --release');
    process.exit(1);
  }
}

main();