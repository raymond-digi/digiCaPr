#!/usr/bin/env node

/**
 * Creates a config release tag and pushes to GitHub.
 *
 * Usage:
 *   node scripts/release-config.js --year 2026
 */

import { existsSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'
import { createInterface } from 'node:readline'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

// ── Parse CLI args ───────────────────────────────────────────────────
const args = process.argv.slice(2)
let year = null

for (let i = 0; i < args.length; i++) {
  if (args[i] === '--year' && args[i + 1]) {
    year = parseInt(args[++i], 10)
  }
}

if (!year || isNaN(year)) {
  console.error('Provide --year (e.g. --year 2026)')
  process.exit(1)
}

const tagName = `config-${year}`

// ── 1. Verify config file exists ─────────────────────────────────────
const configFile = resolve(root, `config/tax_rates_${year}.json`)
if (!existsSync(configFile)) {
  console.error(`Config file not found: ${configFile}`)
  process.exit(1)
}

console.log(`\x1b[36mConfig file: ${configFile}\x1b[0m`)

// ── 2. Git commit if there are changes ───────────────────────────────
execSync('git add -A', { cwd: root, stdio: 'inherit' })
const status = execSync('git status --porcelain', { cwd: root, encoding: 'utf-8' }).trim()
if (status) {
  execSync(`git commit -m "config: update tax rates for ${year}"`, { cwd: root, stdio: 'inherit' })
  console.log('  Committed changes')
}

// ── 3. Git tag ───────────────────────────────────────────────────────
execSync(`git tag ${tagName}`, { cwd: root, stdio: 'inherit' })
console.log(`  Tagged ${tagName}`)

// ── 4. Push ──────────────────────────────────────────────────────────
const rl = createInterface({ input: process.stdin, output: process.stdout })

rl.question('Push commit and tag to origin? (y/n) ', (answer) => {
  rl.close()
  if (answer.trim().toLowerCase() === 'y') {
    execSync('git push origin main', { cwd: root, stdio: 'inherit' })
    execSync(`git push origin ${tagName}`, { cwd: root, stdio: 'inherit' })
    console.log('\x1b[32m  Pushed!\x1b[0m')
  } else {
    console.log('\x1b[33m  Skipped push. Run manually:\x1b[0m')
    console.log('\x1b[33m    git push origin main\x1b[0m')
    console.log(`\x1b[33m    git push origin ${tagName}\x1b[0m`)
  }
  console.log('\n\x1b[32mDone. GitHub Actions will create the config release.\x1b[0m')
})
