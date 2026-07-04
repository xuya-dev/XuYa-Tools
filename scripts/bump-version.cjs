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

// 注意:SettingsView.vue 的版本号现已运行时从 @tauri-apps/api/app getVersion() 动态读取,
// 不再需要在此维护;只需同步 3 个权威源 + 标题栏硬编码。
const targets = [
  { file: 'package.json', regex: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
  { file: 'src-tauri/Cargo.toml', regex: /^version\s*=\s*"[^"]*"/m, replacement: `version = "${newVersion}"` },
  { file: 'src-tauri/tauri.conf.json', regex: /"version":\s*"[^"]*"/, replacement: `"version": "${newVersion}"` },
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
