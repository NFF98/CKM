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
    errors.push('CKM_BASE_SHA 必須是 40 位 Git Commit SHA');
    return [];
  }
  try {
    return git('diff','--name-only','--diff-filter=ACMR',base,'HEAD')
      .split(/\r?\n/).map(norm).filter(Boolean);
  } catch (e) {
    errors.push('無法取得 Git diff；請確認 checkout 包含 base commit：'+e.message);
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

if (policy.implementation_enabled) {
  if (!/^B\d{2}$/.test(String(currentTask||''))) errors.push('已啟動時必須有唯一 active B-ID Task');
  else {
    const manifestPath = resolve(repoRoot,'harness/tasks/'+currentTask+'.json');
    if (!existsSync(manifestPath)) errors.push('找不到 active Task Manifest：'+currentTask);
    else {
      const task = JSON.parse(readFileSync(manifestPath,'utf8'));
      if (task.task_id !== currentTask || task.status !== 'ACTIVE') errors.push('active Task Manifest 必須設為 ACTIVE 並與 Policy 一致');
      const productChanges = changed.filter(isImplementation);
      for (const p of productChanges) {
        if (!inRoots(p, task.allowed_write_prefixes || [])) errors.push('Task 未授權的程式修改路徑：'+p);
      }
      if (productChanges.length && process.env.CKM_PR_TITLE && !process.env.CKM_PR_TITLE.includes(currentTask)) {
        errors.push('產品實作 PR 標題必須明示 active Task：'+currentTask);
      }
    }
  }
} else if (changed.some(isImplementation)) {
  errors.push('Implementation HOLD：未經 Human 啟動 Task 不得修改程式實作');
}
if (changed.some(isImplementation) && changed.some(isProtected)) {
  errors.push('產品程式 PR 不可同時修改 Bible／AGENTS／正式設計來源；須獨立審核');
}
for (const path of files.filter(p=>isImplementation(p)&&isInspectable(p))) {
  const fullPath = resolve(repoRoot,path);
  if (!existsSync(fullPath)||statSync(fullPath).size>1_500_000) continue;
  const code=readFileSync(fullPath,'utf8');
  if (bannedCloudAiPatterns.some(rx=>rx.test(code))) {
    errors.push('疑似違反 No Cloud AI 禁令：'+path);
  }
}
if (errors.length) {
  for (const e of errors) process.stderr.write('CKM GATE FAIL: '+e+'\n');
  process.exit(1);
}
process.stdout.write('CKM GATE PASS：Bible/Agent Policy、單一 Task Scope、Cloud AI 靜態檢查（非真機驗證）\n');
