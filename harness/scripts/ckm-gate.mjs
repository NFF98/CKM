#!/usr/bin/env node
/**
 * CKM single-agent guard. Portable baseline adapted from the generic CursorBuildMachine
 * governance concepts, NOT a copy of its Cursor-specific execution protocol.
 *
 * This catches common dangerous edits. It does NOT prove absence of cloud AI at runtime,
 * cannot validate Android device acceptance, and cannot independently verify human approval.
 */
import { readFileSync, existsSync, statSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '../../');
const policy = JSON.parse(readFileSync(resolve(repoRoot, 'harness/policy/ckm-policy.json'), 'utf8'));
const errors = [];
const warnings = [];

const norm = (p) => p.replaceAll('\\', '/').replace(/^\.\/+/, '');
const inRoots = (p, roots) => roots.some((root) => p.startsWith(root));
const isImplementation = (p) => inRoots(p, policy.implementation_roots);
const isProtected = (p) => policy.immutability.some((x) => x.endsWith('/') ? p.startsWith(x) : p === x);
const isInspectable = (p) => /\.(ts|tsx|js|mjs|rs|kt|java|json|toml|yaml|yml)$/i.test(p);
const bannedCloudAiPatterns = [
  /api\.openai\.com/i,
  /api\.anthropic\.com/i,
  /generativelanguage\.googleapis\.com/i,
  /aiplatform\.googleapis\.com/i,
  /api\.replicate\.com/i,
  /api\.together\.xyz/i,
  /["']@google\/generative-ai["']/i,
  /["']@anthropic-ai\/sdk["']/i,
  /(?:from\s*["']openai["']|require\(\s*["']openai["']\s*\))/i
];

function git(...args) {
  return execFileSync('git', args, {cwd:repoRoot, encoding:'utf8', stdio:['ignore','pipe','pipe']}).trim();
}
function listFiles() {
  return git('ls-files').split(/\r?\n/).map(norm).filter(Boolean);
}
function changedPaths() {
  const base = process.env.CKM_BASE_SHA;
  if (!base) return [];
  if (!/^[0-9a-f]{40}$/i.test(base)) {
    warnings.push('缺少有效的 PR base SHA；本次無法做變更範圍比對，但不阻止開發');
    return [];
  }
  try {
    return git('diff','--name-only','--diff-filter=ACMR',base,'HEAD')
      .split(/\r?\n/).map(norm).filter(Boolean);
  } catch (e) {
    warnings.push('無法取得 Git diff；本次略過變更範圍比對：'+e.message);
    return [];
  }
}
const files = listFiles();
const changed = changedPaths();
const currentTask = policy.current_task;

if (policy.repository !== 'NFF98/CKM') errors.push('Harness 不可使用錯誤的 Repository Policy');
if (!existsSync(resolve(repoRoot,'build/BACKLOG.md'))) errors.push('找不到 Build Bible：build/BACKLOG.md');
if (!existsSync(resolve(repoRoot,'AGENTS.md'))) errors.push('找不到 CKM AGENTS.md');
if (policy.phase_1_cloud_ai_allowed !== false) errors.push('Phase 1 必須禁止雲端 AI');

if (policy.implementation_enabled && currentTask) {
  if (!/^B\d{2}$/.test(String(currentTask||''))) warnings.push('Task ID 格式不符，請檢查紀錄');
  else {
    const manifestPath = resolve(repoRoot,'harness/tasks/'+currentTask+'.json');
    if (!existsSync(manifestPath)) warnings.push('找不到 Task Manifest：'+currentTask);
    else {
      const task = JSON.parse(readFileSync(manifestPath,'utf8'));
      if (task.task_id !== currentTask || task.status !== 'ACTIVE') warnings.push('Task 尚未 ACTIVE／或 ID 不一致（提示，不阻止 Coding）');
      const productChanges = changed.filter(isImplementation);
      for (const p of productChanges) {
        if (!inRoots(p, task.allowed_write_prefixes || [])) warnings.push('Task 範圍之外的程式修改，請自行確認：'+p);
      }
      if (productChanges.length && process.env.CKM_PR_TITLE && !process.env.CKM_PR_TITLE.includes(currentTask)) {
        warnings.push('建議在 PR 標題標示 Task：'+currentTask);
      }
    }
  }
} else if (!policy.implementation_enabled && changed.some(isImplementation)) {
  warnings.push('Policy 標記 HOLD，但本次僅提示：不以治理程序阻止 Coding');
}
if (changed.some(isImplementation) && changed.some(isProtected)) {
  warnings.push('程式與 Bible／正式設計同 PR，請在 Review 時人工確認規格變更');
}
for (const path of files.filter(p=>isImplementation(p)&&isInspectable(p))) {
  const fullPath = resolve(repoRoot,path);
  if (!existsSync(fullPath)||statSync(fullPath).size>1_500_000) continue;
  const code=readFileSync(fullPath,'utf8');
  if (bannedCloudAiPatterns.some(rx=>rx.test(code))) {
    errors.push('疑似違反 No Cloud AI 禁令：'+path);
  }
}
for (const w of warnings) process.stdout.write('CKM 建議：'+w+'\n');
if (errors.length) {
  for (const e of errors) process.stderr.write('CKM GATE FAIL: '+e+'\n');
  process.exit(1);
}
process.stdout.write('CKM GATE PASS：治理提醒不阻止 Coding；只對明顯違反 No Cloud AI／基礎安全設定報錯（非真機驗證）\n');
