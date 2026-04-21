import { readFileSync, writeFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(__dirname, '..')

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf-8'))
}

function writeJson(filePath, data) {
  writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n')
}

function incrementVersion(version) {
  const parts = version.split('.').map(Number)
  if (parts.length !== 3) {
    throw new Error(`Invalid version format: ${version}`)
  }
  parts[2] += 1
  return parts.join('.')
}

const packageJsonPath = resolve(rootDir, 'package.json')
const tauriConfPath = resolve(rootDir, 'src-tauri', 'tauri.conf.json')
const aboutPagePath = resolve(rootDir, 'src', 'components', 'AboutPage.vue')

const packageJson = readJson(packageJsonPath)
const newVersion = incrementVersion(packageJson.version)
packageJson.version = newVersion
writeJson(packageJsonPath, packageJson)
console.log(`package.json version updated to ${newVersion}`)

const tauriConf = readJson(tauriConfPath)
tauriConf.version = newVersion
writeJson(tauriConfPath, tauriConf)
console.log(`tauri.conf.json version updated to ${newVersion}`)

const aboutPageContent = readFileSync(aboutPagePath, 'utf-8')
const updatedAboutPage = aboutPageContent.replace(
  /const appVersion = ref\(['"]([^'"]+)['"]\)/,
  `const appVersion = ref('${newVersion}')`
)
writeFileSync(aboutPagePath, updatedAboutPage)
console.log(`AboutPage.vue default version updated to ${newVersion}`)