#!/usr/bin/env node

/**
 * Quantumn Code - Post-install script
 * Downloads the appropriate binary for the current platform
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const VERSION = '0.1.0';
const REPO = 'Akatsuki2r/QuantumCode';

// Platform mapping
const PLATFORMS = {
  'darwin-x64': 'quantumn-x86_64-apple-darwin.tar.gz',
  'darwin-arm64': 'quantumn-aarch64-apple-darwin.tar.gz',
  'linux-x64': 'quantumn-x86_64-unknown-linux-gnu.tar.gz',
  'linux-arm64': 'quantumn-aarch64-unknown-linux-gnu.tar.gz',
  'win32-x64': 'quantumn-x86_64-pc-windows-gnu.zip',
  'win32-arm64': 'quantumn-aarch64-pc-windows-gnu.zip',
};

function getPlatform() {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;

  if (!PLATFORMS[key]) {
    console.error(`Unsupported platform: ${key}`);
    console.error('Supported platforms:', Object.keys(PLATFORMS).join(', '));
    process.exit(1);
  }

  return key;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`Downloading from ${url}...`);

    const file = fs.createWriteStream(dest);
    let redirects = 0;

    function makeRequest(currentUrl) {
      if (redirects++ > 10) {
        reject(new Error('Too many redirects'));
        return;
      }

      https.get(currentUrl, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          makeRequest(res.headers.location);
          return;
        }

        if (res.statusCode !== 200) {
          reject(new Error(`Failed to download: ${res.statusCode}`));
          return;
        }

        res.pipe(file);

        file.on('finish', () => {
          file.close();
          resolve();
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    }

    makeRequest(url);
  });
}

function extractTarGz(archive, destDir) {
  return new Promise((resolve, reject) => {
    const tar = spawn('tar', ['-xzf', archive, '-C', destDir]);

    tar.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`tar extraction failed with code ${code}`));
      }
    });

    tar.on('error', reject);
  });
}

function extractZip(archive, destDir) {
  return new Promise((resolve, reject) => {
    // Use python or unzip if available, otherwise use jszip
    try {
      const unzipper = require('unzipper');
      fs.createReadStream(archive)
        .pipe(unzipper.Extract({ path: destDir }))
        .on('close', resolve)
        .on('error', reject);
    } catch (e) {
      // Fallback: try system unzip command
      const unzip = spawn('unzip', ['-o', archive, '-d', destDir]);
      unzip.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`unzip failed with code ${code}`));
        }
      });
      unzip.on('error', reject);
    }
  });
}

function extract(archive, destDir, platform) {
  if (archive.endsWith('.tar.gz')) {
    return extractTarGz(archive, destDir);
  } else if (archive.endsWith('.zip')) {
    return extractZip(archive, destDir);
  } else {
    return Promise.reject(new Error(`Unknown archive format: ${archive}`));
  }
}

async function main() {
  console.log('Quantumn Code - Post-install\n');

  const platform = getPlatform();
  const assetName = PLATFORMS[platform];
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryName = process.platform === 'win32' ? 'quantumn.exe' : 'quantumn';
  const binaryPath = path.join(binDir, binaryName);

  // Create bin directory
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // Check if binary already exists and works
  if (fs.existsSync(binaryPath)) {
    try {
      fs.chmodSync(binaryPath, '755');
      console.log('Binary already exists, skipping download.');
      return;
    } catch (e) {
      // Need to re-download
    }
  }

  // Download URL
  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;
  const tempDir = fs.mkdtempSync(path.join(require('os').tmpdir(), 'quantumn-'));
  const archivePath = path.join(tempDir, assetName);

  try {
    // Download
    await downloadFile(downloadUrl, archivePath);

    // Extract
    console.log('Extracting...');
    await extract(archivePath, tempDir, platform);

    // Find binary
    let extractedBinary = path.join(tempDir, binaryName);
    if (!fs.existsSync(extractedBinary)) {
      // Try to find it in subdirectory
      const files = fs.readdirSync(tempDir);
      for (const file of files) {
        const fullPath = path.join(tempDir, file);
        if (fs.statSync(fullPath).isDirectory()) {
          const subBinary = path.join(fullPath, binaryName);
          if (fs.existsSync(subBinary)) {
            extractedBinary = subBinary;
            break;
          }
        }
      }
    }

    if (!fs.existsSync(extractedBinary)) {
      console.error('Failed to find extracted binary');
      console.error('Contents of temp dir:', fs.readdirSync(tempDir));
      process.exit(1);
    }

    // Copy binary to bin directory
    fs.copyFileSync(extractedBinary, binaryPath);
    fs.chmodSync(binaryPath, '755');

    console.log('Installation complete!');
    console.log('Run: quantumn --version');
  } catch (err) {
    console.error('Installation failed:', err.message);
    console.error('\nAlternative install methods:');
    console.error('  cargo install quantumn');
    console.error('  curl -sSL https://get.quantumn.dev | bash');
    process.exit(1);
  } finally {
    // Cleanup
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

main().catch((err) => {
  console.error('Post-install failed:', err.message);
  process.exit(1);
});
