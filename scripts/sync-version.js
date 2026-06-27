#!/usr/bin/env node

/**
 * Reads the version from package.json and syncs it to:
 *   - src-tauri/tauri.conf.json
 *   - src-tauri/Cargo.toml
 *   - Cargo.toml (workspace)
 *
 * Usage:
 *   node scripts/sync-version.js
 *
 * Typically run automatically via `npm run version:sync` or as part of the
 * build pipeline so that a single `npm version` / manual edit in package.json
 * propagates to every place that needs it.
 */

import { readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

// 1. Read version from package.json
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'))
const version = pkg.version

console.log(`Syncing version "${version}" from package.json …`)

// 2. Update src-tauri/tauri.conf.json
const tauriConfPath = resolve(root, 'src-tauri/tauri.conf.json')
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'))
tauriConf.version = version
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf-8')
console.log(`  ✓ tauri.conf.json`)

// 3. Update src-tauri/Cargo.toml
updateCargoVersion(resolve(root, 'src-tauri/Cargo.toml'), version)
console.log(`  ✓ src-tauri/Cargo.toml`)

// 4. Update root Cargo.toml (workspace.package.version)
updateCargoVersion(resolve(root, 'Cargo.toml'), version, true)
console.log(`  ✓ Cargo.toml (workspace)`)

console.log('Done.')

// ---------- helpers ----------

function updateCargoVersion(filePath, newVersion, isWorkspace = false) {
  let content = readFileSync(filePath, 'utf-8')

  if (isWorkspace) {
    // Replace `version = "x.y.z"` inside [workspace.package]
    content = content.replace(
      /(\[workspace\.package\][\s\S]*?version\s*=\s*")([\d.]+)(")/,
      `$1${newVersion}$3`
    )
  } else {
    // Replace the top-level `version = "x.y.z"` under [package]
    content = content.replace(
      /(\[package\][\s\S]*?version\s*=\s*")([\d.]+)(")/,
      `$1${newVersion}$3`
    )
  }

  writeFileSync(filePath, content, 'utf-8')
}
