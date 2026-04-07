import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const TARGET_DIR = resolve(ROOT, "src-tauri", "binaries");

const BINARY_NAME = "fairy-action.exe";
const srcBinary = resolve(TARGET_DIR, BINARY_NAME);

if (!existsSync(srcBinary)) {
  console.warn(
    `\n⚠️  fairy-action.exe not found at: ${srcBinary}\n` +
    `   Please manually copy the binary to this location.\n`
  );
} else {
  console.log(`✓ fairy-action.exe found`);
}
