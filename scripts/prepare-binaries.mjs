import { existsSync, copyFileSync, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const BINARIES_DIR = resolve(ROOT, "src-tauri", "binaries");

const VALID_ARCHS = ["x86_64", "aarch64"];
const ARCH_DIRS = new Set(VALID_ARCHS);

const arch = process.argv[2];
if (!arch || !VALID_ARCHS.includes(arch)) {
  console.error(`Usage: node prepare-binaries.mjs <${VALID_ARCHS.join(" | ")}>\n`);
  process.exit(1);
}

const archDir = resolve(BINARIES_DIR, arch);

if (!existsSync(archDir)) {
  console.error(`Architecture directory not found: ${archDir}`);
  console.error(`Please create the directory and place the ${arch} binaries there.`);
  process.exit(1);
}

function cleanRootBinaries() {
  const entries = readdirSync(BINARIES_DIR);
  for (const entry of entries) {
    if (ARCH_DIRS.has(entry)) continue;
    const fullPath = join(BINARIES_DIR, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      rmSync(fullPath, { recursive: true, force: true });
    } else {
      rmSync(fullPath, { force: true });
    }
  }
}

function copyRecursive(src, dest) {
  const stat = statSync(src);
  if (stat.isDirectory()) {
    if (!existsSync(dest)) mkdirSync(dest, { recursive: true });
    const entries = readdirSync(src);
    for (const entry of entries) {
      copyRecursive(join(src, entry), join(dest, entry));
    }
  } else {
    copyFileSync(src, dest);
  }
}

console.log(`Preparing binaries for architecture: ${arch}`);

cleanRootBinaries();

const archEntries = readdirSync(archDir);
for (const entry of archEntries) {
  const srcPath = join(archDir, entry);
  const destPath = join(BINARIES_DIR, entry);
  copyRecursive(srcPath, destPath);
  console.log(`  Copied: ${entry}`);
}

const requiredFiles = [
  resolve(BINARIES_DIR, "fairy-action.exe"),
  resolve(BINARIES_DIR, "qdrant.exe"),
];

let allFound = true;
for (const file of requiredFiles) {
  if (!existsSync(file)) {
    console.error(`Missing required file: ${file}`);
    allFound = false;
  }
}

if (allFound) {
  console.log(`All required binaries prepared for ${arch}.`);
} else {
  console.error(`Some required binaries are missing for ${arch}.`);
  process.exit(1);
}
