#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const newVersion = process.argv[2];

if (!newVersion || !/^\d+\.\d+\.\d+/.test(newVersion)) {
  console.error('Usage: npm run version <version>');
  console.error('Example: npm run version 1.2.0');
  process.exit(1);
}

const root = path.resolve(__dirname, '..');

const targets = [
  { file: 'package.json', regex: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
  { file: 'src-tauri/Cargo.toml', regex: /^version\s*=\s*"[^"]*"/m, replacement: `version = "${newVersion}"` },
  { file: 'src-tauri/tauri.conf.json', regex: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
  { file: 'src/views/SettingsView.vue', regex: /v\d+\.\d+\.\d+\s*·\s*程序员/, replacement: `v${newVersion} · 程序员` },
  { file: 'src/views/SettingsView.vue', regex: /data\['__version'\]:\s*'[^']*'/, replacement: `data['__version'] = '${newVersion}'` },
  { file: 'src/components/layout/ToolMasterDetail.vue', regex: /开发工具箱 · v[\d.]+/g, replacement: `开发工具箱 · v${newVersion}` },
];

let changed = 0;

for (const t of targets) {
  const filePath = path.join(root, t.file);
  if (!fs.existsSync(filePath)) {
    console.warn(`  SKIP ${t.file} (not found)`);
    continue;
  }
  let content = fs.readFileSync(filePath, 'utf8');
  const before = content;
  content = content.replace(t.regex, t.replacement);
  if (content !== before) {
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`  UPDATE ${t.file}`);
    changed++;
  }
}

console.log(`\nDone: ${changed} files updated to ${newVersion}`);
