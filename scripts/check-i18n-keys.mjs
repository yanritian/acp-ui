#!/usr/bin/env node
/**
 * 翻译键一致性检查脚本
 * 检查所有语言文件是否与 en-US.ts 保持一致的键结构
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const localesDir = path.resolve(__dirname, '../src/locales');

// 递归获取所有键
function getAllKeys(obj, prefix = '') {
  const keys = [];
  for (const key of Object.keys(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
      keys.push(...getAllKeys(obj[key], fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

// 加载语言文件
function loadLocaleFile(filename) {
  const filePath = path.join(localesDir, filename);
  if (!fs.existsSync(filePath)) {
    return null;
  }
  const content = fs.readFileSync(filePath, 'utf-8');
  // 提取 export const xxx: MessageSchema = { ... } 中的对象
  const match = content.match(/export const \w+:\s*MessageSchema\s*=\s*({[\s\S]*})\s*;?\s*$/);
  if (!match) {
    console.error(`Failed to parse ${filename}`);
    return null;
  }
  // 使用 Function 构造器安全解析（比 eval 更安全）
  try {
    const obj = new Function('return ' + match[1])();
    return obj;
  } catch (e) {
    console.error(`Failed to evaluate ${filename}:`, e.message);
    return null;
  }
}

// 主函数
function main() {
  const files = fs.readdirSync(localesDir).filter(f => f.endsWith('.ts') && f !== 'types.ts');

  // 加载 en-US 作为基准
  const enUS = loadLocaleFile('en-US.ts');
  if (!enUS) {
    console.error('Failed to load en-US.ts');
    process.exit(1);
  }
  const enUSKeys = getAllKeys(enUS);
  console.log(`en-US.ts: ${enUSKeys.length} keys`);

  let hasIssues = false;
  const report = [];

  for (const file of files) {
    if (file === 'en-US.ts') continue;

    const locale = loadLocaleFile(file);
    if (!locale) continue;

    const localeKeys = getAllKeys(locale);
    const missingKeys = enUSKeys.filter(k => !localeKeys.includes(k));
    const extraKeys = localeKeys.filter(k => !enUSKeys.includes(k));

    if (missingKeys.length > 0 || extraKeys.length > 0) {
      hasIssues = true;
      report.push({
        file,
        total: localeKeys.length,
        missing: missingKeys.length,
        extra: extraKeys.length,
        missingKeys: missingKeys.slice(0, 10), // 只显示前10个
        extraKeys: extraKeys.slice(0, 10),
      });
    }

    console.log(`${file}: ${localeKeys.length} keys (missing: ${missingKeys.length}, extra: ${extraKeys.length})`);
  }

  if (hasIssues) {
    console.log('\n=== Detailed Report ===');
    for (const item of report) {
      console.log(`\n${item.file}:`);
      if (item.missingKeys.length > 0) {
        console.log(`  Missing keys (showing ${item.missingKeys.length}/${item.missing}):`);
        item.missingKeys.forEach(k => console.log(`    - ${k}`));
      }
      if (item.extraKeys.length > 0) {
        console.log(`  Extra keys (showing ${item.extraKeys.length}/${item.extra}):`);
        item.extraKeys.forEach(k => console.log(`    + ${k}`));
      }
    }

    console.log('\n❌ Translation keys are not consistent!');
    process.exit(1);
  } else {
    console.log('\n✅ All translation keys are consistent!');
    process.exit(0);
  }
}

main();