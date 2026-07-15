import { execFileSync } from 'child_process';
import { copyFileSync, mkdirSync, existsSync } from 'fs';
import { join, resolve } from 'path';

const extension = process.platform === 'win32' ? '.exe' : '';
const targetTriple = execFileSync('rustc', ['--print', 'host-tuple']).toString().trim();

const srcTauriDir = resolve(import.meta.dirname, '..', 'src-tauri');
const profile = process.argv.includes('--release') ? 'release' : 'debug';
const sourcePath = join(srcTauriDir, 'target', profile, `meridian-daemon${extension}`);
const destDir = join(srcTauriDir, 'binaries');
const destPath = join(destDir, `meridian-daemon-${targetTriple}${extension}`);

if (!existsSync(sourcePath)) {
  console.error(`ERROR: Daemon binary not found at ${sourcePath}`);
  console.error('Run: cd src-tauri && cargo build -p meridian-daemon');
  process.exit(1);
}

mkdirSync(destDir, { recursive: true });
copyFileSync(sourcePath, destPath);
console.log(`Copied daemon sidecar → ${destPath}`);
