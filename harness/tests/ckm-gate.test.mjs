import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, dirname } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';

const originalScript = resolve(import.meta.dirname, '../scripts/ckm-gate.mjs');

function git(root, ...args) {
  return execFileSync('git', args, { cwd:root, encoding:'utf8' }).trim();
}
function fixture() {
  const root=mkdtempSync(resolve(tmpdir(),'ckm-guard-'));
  const w=(name,content)=>{
    const full=resolve(root,name);
    mkdirSync(dirname(full),{recursive:true});
    writeFileSync(full,content);
  };
  w('build/BACKLOG.md','# Bible\n');
  w('AGENTS.md','# Agents\n');
  w('harness/scripts/ckm-gate.mjs',readFileSync(originalScript,'utf8'));
  const policy={
    repository:'NFF98/CKM', phase_1_cloud_ai_allowed:false,
    implementation_enabled:false, current_task:null,
    implementation_roots:['build/app/','build/core/','build/mobile/','src/'],
    immutability:['build/BACKLOG.md','AGENTS.md','design/doc/']
  };
  const task={
    task_id:'B01',status:'PLANNED',
    allowed_write_prefixes:['build/app/','build/core/','build/mobile/']
  };
  const savePolicy=()=>w('harness/policy/ckm-policy.json',JSON.stringify(policy));
  const saveTask=()=>w('harness/tasks/B01.json',JSON.stringify(task));
  savePolicy();saveTask();
  git(root,'init','-q');
  git(root,'config','user.email','ckm-test@example.invalid');
  git(root,'config','user.name','CKM Test');
  git(root,'add','.');
  git(root,'commit','-qm','Initial');
  const base=git(root,'rev-parse','HEAD');
  const commit=()=>{
    git(root,'add','.');
    git(root,'commit','-qm','Test change');
  };
  const run=(title='[B01] Android Bridge')=>spawnSync(process.execPath,['harness/scripts/ckm-gate.mjs'],{
    cwd:root,encoding:'utf8',
    env:{...process.env,CKM_BASE_SHA:base,CKM_PR_TITLE:title}
  });
  return {root,w,policy,task,savePolicy,saveTask,commit,run};
}
function withFixture(fn) {
  const f=fixture();
  try{ fn(f); }finally{ rmSync(f.root,{recursive:true,force:true}); }
}
test('HOLD 狀態：只有治理文件的 PR 可以通過',()=>withFixture(f=>{
  f.w('harness/new-rule.md','調整治理規則');
  f.commit();
  const r=f.run('治理：新增文件');
  assert.equal(r.status,0,r.stderr);
}));
test('HOLD 治理狀態：提醒但不阻擋合法 Coding',()=>withFixture(f=>{
  f.w('build/app/index.ts','export const app = true;');
  f.commit();
  const r=f.run();
  assert.equal(r.status,0,r.stderr);
  assert.match(r.stdout,/Policy 標記 HOLD/);
}));
test('B01 ACTIVE：授權路徑與標題吻合才可過 Gate',()=>withFixture(f=>{
  f.policy.current_task='B01';f.policy.implementation_enabled=true;f.task.status='ACTIVE';
  f.savePolicy();f.saveTask();
  f.w('build/app/index.ts','export const app = true;');
  f.commit();
  const r=f.run();
  assert.equal(r.status,0,r.stderr);
}));
test('B01 ACTIVE：跨 Task 範圍寫入只提醒，不造成治理死鎖',()=>withFixture(f=>{
  f.policy.current_task='B01';f.policy.implementation_enabled=true;f.task.status='ACTIVE';
  f.savePolicy();f.saveTask();
  f.w('src/other-task.ts','export const x = true;');
  f.commit();
  const r=f.run();
  assert.equal(r.status,0,r.stderr);
  assert.match(r.stdout,/Task 範圍之外/);
}));
test('Phase 1：雲端 AI API 呼叫不得通過',()=>withFixture(f=>{
  f.policy.current_task='B01';f.policy.implementation_enabled=true;f.task.status='ACTIVE';
  f.savePolicy();f.saveTask();
  f.w('build/app/request.ts','export const endpoint = "https://api.openai.com/v1/chat/completions";');
  f.commit();
  const r=f.run();
  assert.equal(r.status,1,r.stdout);
  assert.match(r.stderr,/No Cloud AI/);
}));
test('程式與 Bible 同 PR 修改提醒人工 Review、不鎖住 CI',()=>withFixture(f=>{
  f.policy.current_task='B01';f.policy.implementation_enabled=true;f.task.status='ACTIVE';
  f.savePolicy();f.saveTask();
  f.w('build/app/index.ts','export const ok = true;');
  f.w('build/BACKLOG.md','# 私自改 Bible\n');
  f.commit();
  const r=f.run();
  assert.equal(r.status,0,r.stderr);
  assert.match(r.stdout,/請在 Review 時人工確認/);
}));

test('未啟動 Task 的情況下，仍可開發 iOS／Android 共用核心',()=>withFixture(f=>{
  f.policy.implementation_enabled=true;
  f.savePolicy();
  f.w('build/core/shared.ts','export const platform = "cross-platform";');
  f.commit();
  const r=f.run('共用核心');
  assert.equal(r.status,0,r.stderr);
}));
test('真正違反 No Cloud AI 原則仍會 FAIL',()=>withFixture(f=>{
  f.policy.phase_1_cloud_ai_allowed=true;
  f.savePolicy();
  f.commit();
  const r=f.run('違反產品原則');
  assert.equal(r.status,1,r.stdout);
  assert.match(r.stderr,/Phase 1 必須禁止雲端 AI/);
}));
