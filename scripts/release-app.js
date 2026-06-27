#!/usr/bin/env node

/**
 * Automates app release: bumps version, syncs, commits, tags, and pushes.
 *
 * Usage:
 *   node scripts/release.js --version "26.8.0"
 *   node scripts/release.js --bump patch    # auto-increments from package.json
 *   node scripts/release.js --bump minor
 *   node scripts/release.js --bump major
 */

import { readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'
import { createInterface } from 'node:readline'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

// ── Parse CLI args ───────────────────────────────────────────────────
const args = process.argv.slice(2)
let version = null
let bump = null

for (let i = 0; i < args.length; i++) {
  if (args[i] === '--version' && args[i + 1]) {
    version = args[++i]
  } else if (args[i] === '--bump' && args[i + 1]) {
    bump = args[++i]
  }
}

if (bump && !['patch', 'minor', 'major'].includes(bump)) {
  console.error('Invalid --bump value. Use "patch", "minor", or "major".')
  process.exit(1)
}

// ── 1. Determine target version ──────────────────────────────────────
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'))
const currentVersion = pkg.version
console.log(`\x1b[36mCurrent version: ${currentVersion}\x1b[0m`)

if (bump) {
  const parts = currentVersion.split('.').map(Number)
  switch (bump) {
    case 'major': parts[0]++; parts[1] = 0; parts[2] = 0; break
    case 'minor': parts[1]++; parts[2] = 0; break
    case 'patch': parts[2]++; break
  }
  version = parts.join('.')
}

if (!version) {
  console.error('Provide --version or --bump')
  process.exit(1)
}

console.log(`\x1b[32mTarget version: ${version}\x1b[0m`)

// ── 2. Update package.json version ───────────────────────────────────
pkg.version = version
writeFileSync(resolve(root, 'package.json'), JSON.stringify(pkg, null, 2) + '\n', 'utf-8')
console.log('  Updated package.json')

// ── 3. Sync version to all files ─────────────────────────────────────
execSync(`node "${resolve(root, 'scripts/sync-version.js')}"`, { cwd: root, stdio: 'inherit' })
console.log('  Synced versions')

// ── 4. Git commit ────────────────────────────────────────────────────
execSync('git add -A', { cwd: root, stdio: 'inherit' })
execSync(`git commit -m "release: v${version}"`, { cwd: root, stdio: 'inherit' })
console.log('  Committed')

// ── 5. Git tag ───────────────────────────────────────────────────────
execSync(`git tag "v${version}"`, { cwd: root, stdio: 'inherit' })
console.log(`  Tagged v${version}`)

// ── 6. Push ──────────────────────────────────────────────────────────
const rl = createInterface({ input: process.stdin, output: process.stdout })

rl.question('Push commit and tag to origin? (y/n) ', (answer) => {
  rl.close()
  if (answer.trim().toLowerCase() === 'y') {
    execSync('git push origin main', { cwd: root, stdio: 'inherit' })
    execSync(`git push origin "v${version}"`, { cwd: root, stdio: 'inherit' })
    console.log('\x1b[32m  Pushed!\x1b[0m')
  } else {
    console.log('\x1b[33m  Skipped push. Run manually:\x1b[0m')
    console.log('\x1b[33m    git push origin main\x1b[0m')
    console.log(`\x1b[33m    git push origin v${version}\x1b[0m`)
  }
  console.log('\n\x1b[32mDone. GitHub Actions will build the release.\x1b[0m')
})
