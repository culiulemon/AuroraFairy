import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const BINARIES_DIR = resolve(ROOT, "src-tauri", "binaries");

const arch = process.argv[2] || "x86_64";
const BINARY_NAME = "fairy-action.exe";

const archBinary = resolve(BINARIES_DIR, arch, BINARY_NAME);
const rootBinary = resolve(BINARIES_DIR, BINARY_NAME);

if (existsSync(archBinary)) {
  console.log(`[arch] fairy-action.exe found at: ${archBinary}`);
} else {
  console.warn(
    `\n[arch] fairy-action.exe not found at: ${archBinary}\n` +
    `   Please place the ${arch} binary in src-tauri/binaries/${arch}/\n`
  );
}

if (existsSync(rootBinary)) {
  console.log(`[root] fairy-action.exe found at: ${rootBinary}`);
} else {
  console.warn(
    `\n[root] fairy-action.exe not found at: ${rootBinary}\n` +
    `   Run "node scripts/prepare-binaries.mjs ${arch}" first.\n`
  );
}
