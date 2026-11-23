// Fixed situations of the run page, opened from disk in a headless browser
// (.kiro/specs/run-dashboard, tasks 4.2 and 4.3). For each one a page is
// written from run.html and the library beside this file, with the
// situation's data inside, as the tool writes a report. The page says which
// state it is in, how many failures its code had, and how wide it is.
//
// Run with verification/playwright.config.mjs; see there.
import { test, expect } from '../../verification/node_modules/@playwright/test/index.mjs';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const SIZES = [{ width: 1280, height: 800 }, { width: 400, height: 800 }];

// The page as the tool builds it: the library in its place, then one marker filled.
function put(text, marker, value) {
  const at = text.indexOf(marker);
  if (at < 0) throw new Error(`the page has no ${marker}`);
  return text.slice(0, at) + value + text.slice(at + marker.length);
}
const template = put(readFileSync(join(here, 'run.html'), 'utf8'), '/*LIB*/', readFileSync(join(here, 'htm-preact-standalone.umd.js'), 'utf8'));
function written(out, name, data, events) {
  let page = template;
  if (data !== undefined) page = put(page, '/*DATA*/null', JSON.stringify(data).replaceAll('<', '\\u003c'));
  if (events) page = put(page, '/*EVENTS*/null', JSON.stringify(events));
  mkdirSync(join(out, 'pages'), { recursive: true });
  const path = join(out, 'pages', `${name}.html`);
  writeFileSync(path, page);
  return pathToFileURL(path).href;
}

const ago = (seconds) => new Date(Date.now() - seconds * 1000).toISOString();

// A run of `step plan`, midway and ended, in the shape the tool writes.
function stage(id, title, status, done, total, seconds) {
  return { id, title, what: `What the stage ${id} does.`, status, done, total,
    seconds: status === 'ended' ? seconds : null, seconds_so_far: status === 'going on' ? seconds : null,
    seconds_left: status === 'going on' && done >= 5 ? 40 : null, reason: null };
}
function tally(file, name, noticed, unnoticed, left) {
  return { file, function: name, noticed, unnoticed, not_built: 0, timed_out: 0, not_judged: 0, left,
    unprotected: noticed === 0 && unnoticed > 0 && left === 0, reaches_out: [] };
}
function run(ended, heardSecondsAgo) {
  const left = ended ? 0 : 4;
  return {
    schema: 1, sequence: 14, tool: { name: 'lesscode-core', version: '2.1.0' }, command: 'step plan',
    started: ago(heardSecondsAgo + 90), heartbeat: ago(heardSecondsAgo), ended,
    outcome: ended ? 'The plan is written' : null, outcome_detail: ended ? '1 of 3 functions with planted faults are unprotected.' : null,
    exit_code: ended ? 0 : null, seconds: 90.5,
    subject: { repository: 'D:/Source/example', branch: 'main', commit: '0123456789abcdef0123456789abcdef01234567', dirty: false, on_main_branch: true },
    copies: { directory: 'D:/BuildArtifacts/example/work', planned: 2, working: 2, dropped: [], bytes: null },
    commands: [['cargo', 'test']], writes_in_repository: [],
    tests: { command_given: true, passed_unchanged: true, tests_run: 12, reason: null },
    stages: [
      stage('prepare', 'Copy the project and run your tests on the unchanged code', 'ended', null, null, 30),
      stage('faults', 'Plant faults', ended ? 'ended' : 'going on', 12 - left, 12, 55),
      stage('close', 'Check the copies and write the plan', ended ? 'ended' : 'not begun', null, null, 5)],
    faults: {
      total: 12, done: 12 - left, noticed: 5, unnoticed: 7 - left, not_built: 0, timed_out: 0, not_judged: 0, not_compiled_here: null,
      example: { file: 'src/app.rs', line: 2, what_was_changed: 'negate', text: 'a < b', outcome: 'noticed: a test failed' },
      files: [tally('src/app.rs', null, 5, 3, 0), tally('src/draw.rs', null, 0, 4 - left, left)],
      functions: [tally('src/app.rs', 'step', 5, 1, 0), tally('src/app.rs', 'parse', 0, 2, 0), tally('src/draw.rs', 'paint', 0, 4 - left, left)],
    },
    next: ['Read the tab Unprotected.', 'Write tests for a function before you change it.'],
  };
}
function large() {
  const one = run(true, 5), part = (i) => `src/renderer/part_${String(i).padStart(2, '0')}.rs`;
  one.faults.files = Array.from({ length: 20 }, (_, i) => tally(part(i), null, 200, 100, 0));
  one.faults.functions = Array.from({ length: 2000 }, (_, i) => ({
    ...tally(part(i % 20), `${'a_function_with_a_very_long_name'.repeat(4)}_${i}`.slice(0, 120), i % 5, i % 3, 0),
    reaches_out: ['create_surface', 'write_texture'] }));
  return one;
}
function noTests() {
  const one = run(true, 5);
  one.tests = { command_given: false, passed_unchanged: null, tests_run: null, reason: null };
  Object.assign(one, { outcome: 'Could not judge', outcome_detail: 'No test command was given, so nothing below was measured.', exit_code: 2 });
  one.stages = one.stages.map((s) => ({ ...s, status: 'not begun', seconds: null, done: null }));
  one.faults = { total: 0, done: 0, noticed: 0, unnoticed: 0, not_built: 0, files: [], functions: [] };
  return one;
}
function refused() {
  const one = run(true, 5);
  Object.assign(one, { command: 'step check', outcome: 'Refused', outcome_detail: 'The step is not accepted as it stands.', exit_code: 1 });
  one.report = {
    lenses: [
      { lens: 'Size', measure: 'tokens of the file', before: 4495, after: 4567, expected: 'may rise by 60', moved: false, measured: true },
      { lens: 'Protection', measure: 'planted faults unnoticed', before: '', after: '', expected: 'does not rise', measured: false }],
    refusal: { reasons: ['the scope grows from 4495 to 4567 tokens; the declaration allows 60 more'] } };
  return one;
}
// Every field of another kind than the page expects.
const wrong = () => ({
  schema: 1, heartbeat: 7, ended: true, tool: 'a text', command: [1, 2], seconds: 'long', exit_code: 'none', subject: 'a text',
  copies: { dropped: [null, 5, 'x'], planned: 'two' }, commands: [5, null, 'x'], writes_in_repository: 3, tests: [],
  stages: [null, 5, 'x', {}, { id: 'faults', status: 7, done: 'x' }],
  faults: { total: 'many', files: [null, {}, { file: 9 }], functions: 'none', example: 7 },
  report: { lenses: [null, 3, {}], refusal: 'yes' }, next: 'x', outcome: 5, outcome_detail: {} });

// The state expected, the data inside, the address of a stream, and what the page must and must not show.
const SITUATIONS = [
  { name: 'waiting', state: 'waiting', shows: ['Waiting for lesscode'] },
  { name: 'waiting-no-answer', state: 'waiting', events: 'http://127.0.0.1:9/none/events', shows: ['The tool does not answer'] },
  { name: 'running', state: 'running', data: () => run(false, 2), shows: ['Running for', '8 of 12 done'] },
  { name: 'quiet', state: 'quiet', data: () => run(false, 30), shows: ['No word from the tool for'] },
  { name: 'stopped', state: 'stopped', data: () => run(false, 120),
    shows: ['The tool stopped without finishing', 'Stopped after', 'was going on when the tool stopped'], hides: ['Running for', '▶'] },
  { name: 'ended', state: 'ended', data: () => run(true, 5), shows: ['The plan is written', 'Took 1 min 31 s'], hides: ['Running for', '▶'], folds: true },
  { name: 'cannot-read', state: 'cannot-read', data: () => ({ schema: 99 }), shows: ['cannot read', 'version 99', 'reads version 1'] },
  { name: 'damaged', state: 'cannot-read', data: () => 'this is not a run', shows: ['cannot read', 'found a text'] },
  { name: 'sparse', state: 'running', data: () => ({ schema: 1, heartbeat: ago(1) }), shows: ['not known'] },
  { name: 'wrong', state: 'ended', data: wrong },
  { name: 'large', state: 'ended', data: large, folds: true },
  { name: 'void', state: 'ended', data: noTests, shows: ['Could not judge', 'No test command was given'], hides: ['✓'] },
  { name: 'refused', state: 'ended', data: refused, shows: ['Refused', 'Fix the step'] },
];

const marks = (page) => page.evaluate(() => ({ ...document.documentElement.dataset }));
// Opens a situation; failures the browser reports are collected beside those the page counts.
async function open(page, testInfo, name, data, events) {
  const failures = [];
  page.on('pageerror', (error) => failures.push(String(error)));
  // Written just before it is opened: the states depend on how old the last word is.
  await page.goto(written(testInfo.config.metadata.out, name, data, events));
  await expect(page.locator('html')).toHaveAttribute('data-state', /./);
  return failures;
}
async function picture(page, testInfo, name) {
  await page.screenshot({ path: join(testInfo.config.metadata.out, 'pictures', `${name}.png`) });
}

for (const size of SIZES) {
  for (const s of SITUATIONS) {
    test(`${s.name} at ${size.width}`, async ({ page }, testInfo) => {
      await page.setViewportSize(size);
      const failures = await open(page, testInfo, s.name, s.data ? s.data() : undefined, s.events);
      for (const text of s.shows ?? []) await expect(page.locator('body')).toContainText(text);
      for (const text of s.hides ?? []) await expect(page.locator('body')).not.toContainText(text);
      await expect(page.locator('html')).toHaveAttribute('data-state', s.state);
      if (s.folds) {
        // Folding one file: its functions go, the other files keep theirs.
        const before = await page.locator('.d2').count();
        await page.locator('button.d1').first().click();
        await expect(page.locator('button.d1').first()).toHaveAttribute('aria-expanded', 'false');
        await expect(page.locator('button.d1').nth(1)).toHaveAttribute('aria-expanded', 'true');
        const after = await page.locator('.d2').count();
        expect(after).toBeGreaterThan(0);
        expect(after).toBeLessThan(before);
      }
      await picture(page, testInfo, `${s.name}-${size.width}`);
      const seen = await marks(page);
      expect(seen.failures, seen.failure).toBe('0');
      expect(failures).toEqual([]);
      expect(await page.evaluate(() => window.innerWidth)).toBe(size.width);
      // The page as a whole does not scroll sideways.
      expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
      expect((await page.locator('body').innerText()).trim()).not.toBe('');
    });
  }
}

test('drawing that fails gives the state "cannot read", not a blank page', async ({ page }, testInfo) => {
  await page.setViewportSize(SIZES[0]);
  await open(page, testInfo, 'drawing-fails', run(true, 5));
  await expect(page.locator('html')).toHaveAttribute('data-state', 'ended');
  // The status line draws a time of day; the page draws again once a second.
  await page.evaluate(() => { Date.prototype.toLocaleTimeString = () => { throw new Error('made to fail by the check'); }; });
  await expect(page.locator('html')).toHaveAttribute('data-state', 'cannot-read');
  for (const text of ['cannot read', 'Drawing the page failed', 'made to fail by the check']) await expect(page.locator('body')).toContainText(text);
  expect(Number((await marks(page)).failures)).toBeGreaterThanOrEqual(1);
  await picture(page, testInfo, 'drawing-fails');
});
