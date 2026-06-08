#!/usr/bin/env npx tsx
// scripts/check-i18n-consistency.ts
// 检查所有 locale 文件与 types.ts 的翻译键是否一致

import * as fs from 'fs';
import * as path from 'path';

const LOCALES_DIR = path.resolve(__dirname, '../src/locales');
const LOCALE_FILES = [
  'zh-CN.ts', 'en-US.ts', 'de-DE.ts', 'es-ES.ts', 'fr-FR.ts',
  'ja-JP.ts', 'ko-KR.ts', 'ms-MY.ts', 'ru-RU.ts', 'th-TH.ts', 'vi-VN.ts',
];

function extractKeys(content: string): Set<string> {
  const keys = new Set<string>();
  // 匹配 object property keys: keyName: 'value' or keyName: "value"
  const regex = /^\s+(\w+):\s*['"`{]/gm;
  let match;
  while ((match = regex.exec(content)) !== null) {
    keys.add(match[1]);
  }
  return keys;
}

function main() {
  const allKeys = new Map<string, Set<string>>();
  let hasErrors = false;

  for (const file of LOCALE_FILES) {
    const filePath = path.join(LOCALES_DIR, file);
    if (!fs.existsSync(filePath)) {
      console.error(`❌ Missing locale file: ${file}`);
      hasErrors = true;
      continue;
    }
    const content = fs.readFileSync(filePath, 'utf-8');
    allKeys.set(file, extractKeys(content));
  }

  // Use en-US as reference
  const refKeys = allKeys.get('en-US.ts');
  if (!refKeys) {
    console.error('❌ en-US.ts not found');
    process.exit(1);
  }

  console.log(`\nReference: en-US.ts has ${refKeys.size} keys\n`);

  for (const [file, keys] of allKeys) {
    if (file === 'en-US.ts') continue;
    const missing = [...refKeys].filter(k => !keys.has(k));
    const extra = [...keys].filter(k => !refKeys.has(k));

    if (missing.length > 0 || extra.length > 0) {
      hasErrors = true;
      console.log(`⚠️  ${file}: ${keys.size} keys`);
      if (missing.length > 0) console.log(`   Missing ${missing.length}: ${missing.slice(0, 10).join(', ')}${missing.length > 10 ? '...' : ''}`);
      if (extra.length > 0) console.log(`   Extra ${extra.length}: ${extra.slice(0, 5).join(', ')}`);
    } else {
      console.log(`✅ ${file}: ${keys.size} keys (matches reference)`);
    }
  }

  if (hasErrors) {
    console.log('\n❌ i18n keys are inconsistent. Please sync missing keys.');
    process.exit(1);
  } else {
    console.log('\n✅ All locale files have consistent keys.');
  }
}

main();