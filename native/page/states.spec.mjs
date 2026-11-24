// Fixed situations of the run page, opened from disk in a headless browser
// (.kiro/specs/run-dashboard, tasks 4.2 and 4.3). For each one a page is
// written from run.html and the library beside this file, with the
// situation's data inside, as the tool writes a report. The page says which
// state it is in, how many failures its code had, and how wide it is.
//
// Run with verification/playwright.config.mjs; see there.
import { test, expect } from '../../verification/node_modules/@playwright/test/index.mjs';
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
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
    outcome: ended ? 'The plan is written' : null, outcome_detail: ended ? '2 of 3 functions with planted faults are unprotected.' : null,
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
      example: { file: 'src/app.rs', function: 'step', line: 2, what_was_changed: 'negate', text: 'a < b', outcome: 'noticed: a test failed' },
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
// What the run touches (task 5.1): the copies measured at the end; and a run on a branch of
// its own, with uncommitted work, whose plan and copies lie in the repository.
function measured() {
  const one = run(true, 5);
  one.copies.bytes = 2684354560;
  return one;
}
function onABranch() {
  const one = run(false, 2);
  Object.assign(one.subject, { branch: 'rework/shared-frame', on_main_branch: false, dirty: true });
  one.writes_in_repository = ['plan.json', 'work'];
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
// Without tests nothing is protected (task 5.4): a test command that runs no test, tests that
// fail on the unchanged code in every copy, and one copy of two that drops out.
function noTestRan() {
  const one = noTests();
  one.tests = { command_given: true, passed_unchanged: true, tests_run: 0, reason: null };
  one.outcome_detail = 'The test command ran no test, so nothing below was measured.';
  return one;
}
function testsFail() {
  const one = noTests();
  one.tests = { command_given: true, passed_unchanged: false, tests_run: null, reason: 'cargo test ended with 101: test parse::tests::empty ... FAILED' };
  one.copies.working = 0;
  one.outcome_detail = 'Your tests do not pass on the unchanged code, so nothing below was measured.';
  return one;
}
function copyDropped() {
  const one = run(false, 2);
  one.copies.working = 1;
  one.copies.dropped = [{ copy: 2, at: ago(60), reason: 'cargo test ended with 101' }];
  return one;
}
// Faults in code that is not built on this system (task 5.3): one file, one function, four faults.
function notBuiltHere() {
  const one = run(true, 5);
  Object.assign(one.faults, { total: 16, done: 16, not_compiled_here: 4 });
  one.faults.files.push({ ...tally('src/wayland.rs', null, 0, 0, 0), not_compiled_here: 4 });
  one.faults.functions.push({ ...tally('src/wayland.rs', 'attach', 0, 0, 0), not_compiled_here: 4 });
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
// A run of `step check` (tasks 6.1 and 6.2) in the shape the tool writes: accepted, refused by
// what the commits show before anything ran, or refused by the faults of its two runs.
const BASE = 'e4d3b2c0aa11bb22cc33dd44ee55ff6677889900';
function checked(kind) {
  const one = run(true, 5), early = kind === 'refused-early', accepted = kind === 'accepted';
  const reasons = early ? ['the scope grows from 4495 to 4567 tokens; the declaration allows 60 more']
    : ['src/frame.rs: take_damage: unnoticed faults rise from 1 to 3: #7 line 41 negate, #9 line 44 drop_statement'];
  Object.assign(one, { command: 'step check', outcome: accepted ? 'Accepted' : 'Refused', exit_code: accepted ? 0 : 1,
    outcome_detail: accepted ? 'Every check passed. In no function are more faults unnoticed after the step than before. No test noticed any fault before the step in: mark_all. There the acceptance means only that nothing got worse; nothing was protected before.' : reasons.join('; '),
    next: accepted ? ['Put the step on the ledger with `step record`.', 'Tests come first: write tests for the functions named above before another step changes them.'] : ['Change the step so that the reasons no longer hold, and run this again. Fix the step, not the limit.'] });
  Object.assign(one.subject, { branch: 'step/shared-frame-pending', on_main_branch: false, compared_with: BASE });
  one.writes_in_repository = ['.kiro/specs/shared-frame-pending/verdict.json'];
  const status = (i) => (early ? (i === 0 ? 'failed' : 'left out') : 'ended');
  one.stages = [['checks', 'Check what the commits of the step show'], ['tests-before', 'Copy the code before the step and run your tests on it'],
    ['faults-before', 'Plant faults in the code before the step'], ['tests-after', 'Copy the code after the step and run your tests on it'],
    ['faults-after', 'Plant faults in the code after the step'], ['verdict', 'Compare and write the verdict']]
    .map(([id, title], i) => ({ ...stage(id, title, status(i), null, null, 12), reason: early && i === 0 ? reasons[0] : null }));
  const sided = (side, row) => ({ ...row, side });
  one.faults = early ? { total: 0, done: 0, noticed: 0, unnoticed: 0, not_built: 0, timed_out: 0, not_judged: 0, not_compiled_here: null, example: null, files: [], functions: [] } : {
    total: 30, done: 30, noticed: 20, unnoticed: 10, not_built: 0, timed_out: 0, not_judged: 0, not_compiled_here: null,
    // No source text unless the person asked for it.
    example: { file: 'src/frame.rs', function: 'take_damage', line: 41, what_was_changed: 'negate', text: '', outcome: 'noticed: a test failed' },
    files: [sided('before', tally('src/frame.rs', null, 11, 5, 0)), sided('after', tally('src/frame.rs', null, 9, 5, 0))],
    functions: [sided('before', tally('src/frame.rs', 'take_damage', 11, 1, 0)), sided('before', tally('src/frame.rs', 'mark_all', 0, 3, 0)), sided('before', tally('src/frame.rs', 'release', 0, 1, 0)),
      sided('after', tally('src/frame.rs', 'take_damage', 9, accepted ? 1 : 3, 0)), sided('after', tally('src/frame.rs', 'mark_all', 0, accepted ? 3 : 1, 0))] };
  const lens = (name, measure, before, after, expected, moved, command) => ({ lens: name, measure, before, after, expected, moved, measured: true, command, handed_in: false, note: null });
  const none = (name, measure, expected, note) => ({ lens: name, measure, before: null, after: null, expected, moved: null, measured: false, command: null, handed_in: false, note });
  const handIn = 'This run does not measure it. Numbers measured elsewhere are shown when a file with them is named with --measures.';
  one.report = {
    lenses: [
      lens('Size', 'tokens of the files in scope', 4495, early ? 4567 : 4452, 'falls', !early, 'lesscode-core step check'),
      lens('Structure', 'functions', 15, 17, 'falls', false, 'lesscode-core deconvolute measure-scope'),
      accepted ? { ...lens('State', 'ways SharedFrame can be written', 16, 6, 'falls', true, 'state-holders --type SharedFrame'), handed_in: true }
        : none('State', 'ways the state can be written, is reached and is seen by the tests', 'falls', handIn),
      none('Local complexity', 'per routine: decisions, nesting, parameters and length', 'no routine above its limit gets worse', handIn),
      early ? none('Behaviour kept', 'your tests on the code, with no fault planted', 'stays', 'The tests were not run, or could not be run.')
        : lens('Behaviour kept', 'your tests on the code, with no fault planted', 'pass', 'pass', 'stays', true, 'cargo test --lib'),
      early ? none('Protection', 'planted faults in the functions of the step: noticed; unnoticed; neither', 'unnoticed faults do not rise in any function', 'The faults of the two sides were not compared.')
        : lens('Protection', 'planted faults in the functions of the step: noticed; unnoticed; neither', '11; 5; 0', accepted ? '9; 4; 1' : '9; 4; 0', 'unnoticed faults do not rise in any function', accepted, 'lesscode-core step check')],
    functions: early ? [] : [
      { file: 'src/frame.rs', function: 'take_damage', before: { noticed: 11, unnoticed: 1, neither: 0 }, after: { noticed: 9, unnoticed: accepted ? 1 : 3, neither: 0 }, out_of_reach: null },
      { file: 'src/frame.rs', function: 'mark_all', before: { noticed: 0, unnoticed: 3, neither: 0 }, after: { noticed: 0, unnoticed: accepted ? 3 : 1, neither: 0 }, out_of_reach: 'needs a window to be tested' },
      { file: 'src/frame.rs', function: 'release', before: { noticed: 0, unnoticed: 1, neither: 0 }, after: null, out_of_reach: null }],
    refusal: accepted ? null : { reasons }, graph: null, shapes: [] };
  return one;
}
// The two drawings (task 6.3), from the numbers of SharedFrame: see shared-frame.json beside this file.
const sharedFrame = JSON.parse(readFileSync(join(here, 'shared-frame.json'), 'utf8'));
function checkedWithDrawings() {
  const one = checked('accepted');
  Object.assign(one.report, { graph: sharedFrame.graph, shapes: sharedFrame.shapes });
  return one;
}
// The tests fail after the step: the faults of that side were planned and never run.
function checkedTestsFailAfter() {
  const one = checked('refused');
  one.stages = one.stages.map((s, i) => ({ ...s, status: i < 3 ? 'ended' : i === 3 ? 'failed' : 'left out', reason: i === 3 ? 'the project\'s tests do not pass on the step' : null }));
  const unrun = (row) => (row.side === 'after' ? { ...row, noticed: 0, unnoticed: 0, left: 4, unprotected: false } : row);
  one.faults.files = one.faults.files.map(unrun);
  one.faults.functions = one.faults.functions.map(unrun);
  return one;
}
function checkedOnMain() {
  const one = checked('accepted');
  Object.assign(one.subject, { branch: 'main', on_main_branch: true });
  return one;
}
const LENSES = ['Size', 'Structure', 'State', 'Local complexity', 'Behaviour kept', 'Protection'];
// Every field of another kind than the page expects.
const wrong = () => ({
  schema: 1, heartbeat: 7, ended: true, tool: 'a text', command: [1, 2], seconds: 'long', exit_code: 'none', subject: 'a text',
  copies: { dropped: [null, 5, 'x'], planned: 'two' }, commands: [5, null, 'x'], writes_in_repository: 3, tests: [],
  stages: [null, 5, 'x', {}, { id: 'faults', status: 7, done: 'x' }],
  faults: { total: 'many', files: [null, {}, { file: 9 }], functions: 'none', example: 7 },
  report: { lenses: [null, 3, {}], refusal: 'yes' }, next: 'x', outcome: 5, outcome_detail: {} });

// The four notices on tests and copies; a situation shows its own and no other.
const NOTICE = { command: 'No test command was given', none: 'ran no test', fail: 'did not get through on your unchanged code', dropped: 'of your project stopped' };

// The state expected, the data inside, the address of a stream, and what the page must and must not show.
const SITUATIONS = [
  { name: 'waiting', state: 'waiting', shows: ['Waiting for lesscode'] },
  { name: 'waiting-no-answer', state: 'waiting', events: 'http://127.0.0.1:9/none/events', shows: ['The tool does not answer'] },
  { name: 'running', state: 'running', data: () => run(false, 2),
    shows: ['Running for', '8 of 12 done', 'Which of your functions do your tests protect?', 'No file of your project is changed', 'Of 12 faults planted in the copies', '4 still to run', 'Nothing is needed from you', 'Your functions so far', 'going on now', 'works in copies of your project under D:/BuildArtifacts/example/work', 'this is the main branch', 'writes nothing into your repository'],
    hides: ['The copies take', 'uncommitted changes', 'and nothing else'],
    tabs: { 'Your code': { shows: ['cargo test', 'in the copies only', 'reads the main branch', 'Files written into your repository'], hides: ['To remove the copies', 'Space the copies take'] },
      // The explanation, with the first fault the run judged (task 5.2).
      'Planted faults': { shows: ['makes one on purpose', 'One mistake, on purpose', 'An unnoticed fault can mean three things', 'never reaches your repository', 'One from this run', 'src/app.rs, line 2, in step', 'a < b', 'negate',
        'noticed: a test failed', 'sharp at that place', 'does not show that two versions of the code behave the same',
        'A test is missing', 'changes nothing a caller can see', 'your tests cannot observe'], hides: ['No fault of this run has been judged yet'] } } },
  { name: 'on-a-branch', state: 'running', data: onABranch,
    shows: ['rework/shared-frame', 'writes plan.json, work into your repository and nothing else', 'had uncommitted changes when the run began'],
    hides: ['main branch', 'writes nothing'],
    tabs: { 'Your code': { shows: ['plan.json', 'the only kind of file'], hides: ['main branch'] } } },
  { name: 'measured', state: 'ended', data: measured, shows: ['The copies take 2.5 GiB'],
    tabs: { 'Your code': { shows: ['2.5 GiB', 'To remove the copies, delete the directory D:/BuildArtifacts/example/work'] } } },
  { name: 'quiet', state: 'quiet', data: () => run(false, 30), shows: ['Quiet', 'No word from the tool since'] },
  { name: 'stopped', state: 'stopped', data: () => run(false, 120),
    shows: ['The tool stopped without finishing', 'Stopped after', 'was going on when the tool stopped'], hides: ['Running for', '▶'] },
  { name: 'ended', state: 'ended', data: () => run(true, 5), shows: ['The plan is written', 'Took 1 min 31 s', '3 functions: 2 unprotected, 1 with some faults unnoticed.', 'Done. All 12 planted faults were run:'], hides: ['Running for', '▶', 'still to run', 'Nothing is needed from you', 'The copies take', 'not built here', 'not built on this system', ...Object.values(NOTICE)], folds: true,
    signs: ['not built on this system: nothing is known', 0],
    // The share of unprotected functions comes before any other result of the faults.
    order: ['Unprotected functions', '2 of 3 (67%)', 'Faults planted'],
    tabs: { 'Your code': { shows: ['not measured', 'To remove the copies, delete the directory'] } } },
  { name: 'not-built-here', state: 'ended', data: notBuiltHere, shows: ['4 not built here', 'In code not built on this system'],
    signs: ['not built on this system: nothing is known', 2],
    tabs: { 'Planted faults': { shows: ['4 faults lie in code that is not built on this system', 'they tell nothing'] },
      Unprotected: { shows: ['2 of 3 functions are unprotected', '1 more lies wholly in code that is not built on this system'], hides: ['attach'] } } },
  { name: 'cannot-read', state: 'cannot-read', data: () => ({ schema: 99 }), shows: ['cannot read', 'version 99', 'reads version 1'] },
  { name: 'damaged', state: 'cannot-read', data: () => 'this is not a run', shows: ['cannot read', 'found a text'] },
  { name: 'sparse', state: 'running', data: () => ({ schema: 1, heartbeat: ago(1) }), shows: ['not known', 'works in copies of your project.'],
    hides: ['your project under', 'main branch', 'writes nothing', 'and nothing else', 'uncommitted changes', 'The copies take'],
    tabs: { 'Your code': { shows: ['None is known', 'not known'], hides: ['main branch', 'To remove the copies', 'the only kind of file', 'Files written into your repository'] },
      'Planted faults': { shows: ['makes one on purpose', 'never reaches your repository', 'No fault of this run has been judged yet', 'A test is missing'], hides: ['One from this run'] } } },
  { name: 'wrong', state: 'ended', data: wrong, tabs: { 'Planted faults': { shows: ['No fault of this run has been judged yet'], hides: ['One from this run'] } } },
  { name: 'large', state: 'ended', data: large, folds: true,
    tabs: { Unprotected: { shows: ['create_surface, write_texture', 'a test cannot come first', 'What to do with them', 'You do not plan to change it', 'the brownfield test stage', 'make a spec whose tasks are the tests'] } } },
  { name: 'void', state: 'ended', data: noTests, shows: ['Could not judge', 'No test command was given', 'every result of this run is empty'],
    hides: ['✓', NOTICE.none, NOTICE.fail, NOTICE.dropped, 'Unprotected functions'],
    tabs: { Unprotected: { shows: ['No fault was run'], hides: ['is known when their faults have been run'] } } },
  { name: 'no-test-ran', state: 'ended', data: noTestRan, shows: [NOTICE.none, 'every result of this run is empty'], hides: [NOTICE.command, NOTICE.fail, NOTICE.dropped] },
  { name: 'tests-fail', state: 'ended', data: testsFail, shows: [NOTICE.fail, 'no planted fault could be judged', 'test parse::tests::empty ... FAILED'], hides: [NOTICE.command, NOTICE.none, NOTICE.dropped] },
  { name: 'copy-dropped', state: 'running', data: copyDropped, shows: ['Working copy 2 of your project stopped', 'cargo test ended with 101', 'The run goes on with 1 of 2 copies', 'Your repository is not affected'],
    hides: [NOTICE.command, NOTICE.none, NOTICE.fail],
    tabs: { Unprotected: { shows: ['1 of 3 functions are unprotected', 'every test still passes', 'What to do with them', 'leave it'], hides: ['a test cannot come first'] } } },
  { name: 'refused', state: 'ended', data: refused, shows: ['Refused', 'Fix the step'] },
  // The report of a judged step (task 6.2): each lens or "not measured", what was compared and by
  // which command; the faults of each side under the stage of that side.
  { name: 'check-accepted', state: 'ended', data: () => checked('accepted'), folds: true,
    shows: ['Accepted', 'Does this change keep what your tests protect?', 'How much code there is', '✓ moved as expected', '! did not move as expected', '4,495 to 4,452', 'The change that is judged is on branch step/shared-frame-pending', 'nothing was protected before', ...LENSES, 'not measured',
      'lesscode-core deconvolute measure-scope', 'cargo test --lib', 'state-holders --type SharedFrame handed in, not measured by this run', 'named with --measures',
      BASE, 'lesscode-core 2.1.0', 'Nothing was protected before. Left unprotected on purpose: needs a window to be tested', 'not there',
      'Unprotected functions after the change', '1 of 2 (50%)', 'Faults planted, on both sides'],
    hides: ['Refused', 'Fix the step', 'this is the main branch', 'compared nothing', 'Did not build'],
    order: ['Accepted', 'What to do next', 'In short', 'What was compared', 'Before and after', 'Planted faults per function', 'Unprotected functions after the change'],
    tabs: { 'Your code': { shows: ['Compared with commit', BASE, 'verdict.json', 'the only kind of file'], hides: ['main branch'] },
      'Planted faults': { shows: ['One from this run', 'src/frame.rs, line 41, in take_damage', 'left out', '--page-source'] },
      Unprotected: { shows: ['1 of 2 functions are unprotected after the change', 'mark_all'], hides: ['release'] } } },
  { name: 'check-on-main', state: 'ended', data: checkedOnMain, shows: ['this is the main branch'],
    tabs: { 'Your code': { shows: ['The change that is judged is on the main branch', 'A change belongs on a branch of its own'], hides: ['Make the change you plan'] } } },
  { name: 'check-refused-early', state: 'ended', data: () => checked('refused-early'),
    shows: ['Refused', 'the declaration allows 60 more', 'Fix the step', ...LENSES, 'The tests were not run', 'The faults of the two sides were not compared'],
    hides: ['Accepted', 'Planted faults per function', 'Unprotected functions'],
    signs: ['failed', 1],
    order: ['Refused', 'the scope grows from 4495 to 4567', 'Fix the step', 'What to do next', 'In short', 'What was compared', 'Before and after'] },
  { name: 'check-refused', state: 'ended', data: () => checked('refused'),
    shows: ['Refused', 'unnoticed faults rise from 1 to 3', ...LENSES, '1 of 2 (50%)'], hides: ['Accepted', 'handed in'],
    order: ['Refused', 'unnoticed faults rise', 'What was compared', 'Before and after', 'Planted faults per function'] },
  // The two drawings (task 6.3): shown where their numbers are in the snapshot, and "not measured" where not.
  { name: 'check-drawings', state: 'ended', data: checkedWithDrawings,
    shows: ['Functions and calls', 'Before: 15 functions, 8 calls, 0 of them across files', 'After: 17 functions, 12 calls, 0 of them across files', 'drawn to one scale', 'need not happen when the code runs',
      'Shapes of the state', 'SharedFrame, before the change', 'Can be written: 8. Reached by the code: 6. Seen by the tests: 6.', 'dirty no, damage none, full yes', 'never reached', 'seen by no test',
      'research programs, handed in, not measured by this run'],
    hides: ['Not measured: the functions and calls', 'is counted outside this run'],
    order: ['What to do next', 'Before and after', 'Planted faults per function', 'Functions and calls', 'Shapes of the state'] },
  { name: 'check-no-drawings', state: 'ended', data: () => checked('refused'),
    shows: ['Functions and calls', 'Not measured: the functions and calls of the scope were not counted in this run', 'Shapes of the state', 'is counted outside this run', '--measures'],
    hides: ['drawn to one scale', 'Can be written:'] },
  // After the end nothing is shown as going on: faults that were never run were left out.
  { name: 'check-tests-fail-after', state: 'ended', data: checkedTestsFailAfter, shows: ['Refused'], hides: ['▶', 'Unprotected functions'], signs: ['going on', 0] },
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
      if (s.signs) await expect(page.locator(`.mark[aria-label="${s.signs[0]}"]`)).toHaveCount(s.signs[1]);
      if (s.order) {
        const said = await page.locator('body').innerText(), at = s.order.map((text) => said.indexOf(text));
        expect(at.every((x, i) => x >= 0 && (i === 0 || x > at[i - 1])), `${s.order.join(' < ')}: ${at}`).toBeTruthy();
      }
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
      // What a tab says in this situation, and what it must not say.
      for (const [name, says] of Object.entries(s.tabs ?? {})) {
        await page.getByRole('tab', { name: new RegExp(`^${name}`) }).click();
        for (const text of says.shows ?? []) await expect(page.getByRole('tabpanel')).toContainText(text);
        for (const text of says.hides ?? []) await expect(page.getByRole('tabpanel')).not.toContainText(text);
        await picture(page, testInfo, `${s.name}-${size.width}-${name.toLowerCase().replaceAll(' ', '-')}`);
      }
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

// Requirement 7.7: a refusal is shown as prominently as an acceptance. The outcome is the first
// thing of the result in both, at the same place, in the same type.
test('a refusal stands where an acceptance stands, in the same size', async ({ page }, testInfo) => {
  await page.setViewportSize(SIZES[0]);
  const outcome = async (kind) => {
    await open(page, testInfo, `prominence-${kind}`, checked(kind));
    const panel = page.getByRole('tabpanel'), verdict = panel.locator('.verdict'), head = verdict.locator('h2');
    await expect(verdict).toHaveCount(1);
    const first = await panel.evaluate((el) => el.querySelector(':scope > div > :first-child').className);
    const type = await head.evaluate((el) => { const s = getComputedStyle(el); return `${s.fontSize} ${s.fontWeight}`; });
    const at = await verdict.boundingBox();
    return { said: await head.innerText(), first, type, x: at.x, y: at.y, width: at.width };
  };
  const yes = await outcome('accepted'), no = await outcome('refused-early');
  expect([yes.said, no.said]).toEqual(['Accepted', 'Refused']);
  expect(yes.first).toContain('verdict');
  expect(no.first).toContain('verdict');
  expect({ ...no, said: '', first: '' }).toEqual({ ...yes, said: '', first: '' });
});

// Task 9.1: short first. The answer and what to do stand before the numbers; a lens has a tile in
// plain words; a function has a square; the marks of the tree are explained; what explains more
// is closed until it is opened.
test('the short version stands first, and what explains more can be opened', async ({ page }, testInfo) => {
  await page.setViewportSize(SIZES[0]);
  await open(page, testInfo, 'short-check', checked('accepted'));
  const panel = page.getByRole('tabpanel');
  // The outcome is the name of the run at the top, in the colour of the result.
  await expect(page.locator('.head .pill')).toHaveText('Accepted');
  await expect(page.locator('.head .pill')).toHaveClass(/ok/);
  const tiles = panel.locator('.tile');
  await expect(tiles).toHaveCount(LENSES.length);
  expect(await tiles.evaluateAll((all) => all.map((t) => [t.querySelector('b').innerText, t.className.replace('tile ', ''), t.querySelector('.verd').innerText])))
    .toEqual([['Size', 'ok', '✓ moved as expected'], ['Structure', 'warn', '! did not move as expected'], ['State', 'ok', '✓ moved as expected'],
      ['Local complexity', 'off', '– not measured'], ['Behaviour kept', 'ok', '✓ moved as expected'], ['Protection', 'ok', '✓ moved as expected']]);
  // The functions after the change, one square each, with what the squares say in a sentence.
  await expect(panel.locator('.strip i')).toHaveText(['!', '✗']);
  await expect(panel.locator('.strip')).toHaveAttribute('aria-label', 'One square per function. 2 functions: 1 unprotected, 1 with some faults unnoticed.');
  // Only the marks the tree shows are explained under it.
  await expect(page.locator('.key div')).toHaveText(['✓done', '!some faults were not noticed', '✗unprotected: no fault was noticed']);
  await picture(page, testInfo, 'short-check');

  await open(page, testInfo, 'short-refused', checked('refused'));
  await expect(page.locator('.head .pill')).toHaveText('Refused');
  await expect(page.locator('.head .pill')).toHaveClass(/bad/);

  // While a run goes on: one sentence on the stage, the functions so far, and the four steps closed.
  await open(page, testInfo, 'short-running', run(false, 2));
  await expect(panel.locator('.lead')).toHaveText('What the stage faults does.');
  // A function in which no fault was judged yet has not begun; it is not shown as going on.
  await expect(panel.locator('.strip i')).toHaveText(['!', '✗', '·']);
  const how = panel.locator('details');
  await expect(how).toHaveCount(1);
  expect(await how.evaluate((d) => d.open)).toBe(false);
  await expect(panel.locator('.how li').first()).toBeHidden();
  await how.locator('summary').focus();
  await page.keyboard.press('Enter');
  await expect(panel.locator('.how li')).toHaveCount(4);
  await expect(panel.locator('.how li').first()).toBeVisible();
  await picture(page, testInfo, 'short-running-how');
  await page.getByRole('tab', { name: /^Planted faults/ }).click();
  await expect(panel.locator('.how li').first()).toBeVisible();
  expect(await panel.locator('details').evaluateAll((all) => all.map((d) => d.open))).toEqual([true, false, false]);
  await picture(page, testInfo, 'short-planted-faults');
  expect((await marks(page)).failures).toBe('0');
});

// Task 6.3: both sides of the drawing of functions and calls have every function and every call of
// their side, and are drawn to one scale; the grid has one cell per shape that can be written.
test('the two drawings show what was counted, to one scale', async ({ page }, testInfo) => {
  await page.setViewportSize(SIZES[0]);
  await open(page, testInfo, 'drawings', checkedWithDrawings());
  const panel = page.getByRole('tabpanel'), bowls = panel.locator('.bowl svg');
  await expect(bowls).toHaveCount(2);
  const drawn = await bowls.evaluateAll((all) => all.map((svg) => ({ box: svg.getAttribute('viewBox'), name: svg.getAttribute('aria-label'),
    dots: [...svg.querySelectorAll('.fn')].map((d) => [d.getAttribute('cx'), d.getAttribute('cy'), d.querySelector('title').textContent]), calls: svg.querySelectorAll('.call').length,
    rim: svg.querySelector('.rim').getAttribute('r'), wide: svg.getBoundingClientRect().width })));
  expect(drawn.map((d) => [d.dots.length, d.calls])).toEqual([[15, 8], [17, 12]]);
  expect(drawn[0].name).toContain('Before: 15 functions');
  // One scale: the same frame, the same rim, the same width on the screen, and a function that is
  // on both sides at the same place of its file takes the same place on the rim.
  expect([drawn[1].box, drawn[1].rim, drawn[1].wide]).toEqual([drawn[0].box, drawn[0].rim, drawn[0].wide]);
  expect(drawn[1].dots[0]).toEqual(drawn[0].dots[0]);
  expect(drawn[0].dots[0][2]).toBe('src-tauri/src/renderer/shared_frame.rs: begin_write');
  await expect(panel.locator('.cell')).toHaveCount(8);
  await expect(panel.locator('.cell.on')).toHaveCount(6);
  await picture(page, testInfo, 'drawings');
  await panel.locator('.bowls').scrollIntoViewIfNeeded();
  await picture(page, testInfo, 'drawings-bowls');
  await panel.locator('.cells').scrollIntoViewIfNeeded();
  await picture(page, testInfo, 'drawings-grid');
});

// Reports the tool itself wrote, for the fixture steps of native/tests/run_page_check.rs (task
// 6.2). LESSCODE_CHECK_REPORTS names a directory of such pages; without it this is not run.
const reports = process.env.LESSCODE_CHECK_REPORTS;
test('the reports the tool wrote for the fixture steps show each lens or "not measured"', async ({ page }, testInfo) => {
  test.skip(!reports, 'LESSCODE_CHECK_REPORTS names no directory of reports');
  await page.setViewportSize(SIZES[0]);
  const files = readdirSync(reports).filter((file) => file.endsWith('.html')), outcomes = [];
  expect(files.length).toBeGreaterThan(0);
  const failures = [];
  page.on('pageerror', (error) => failures.push(String(error)));
  for (const file of files) {
    await page.goto(pathToFileURL(join(reports, file)).href);
    await expect(page.locator('html')).toHaveAttribute('data-state', 'ended');
    const panel = page.getByRole('tabpanel'), verdict = panel.locator('.verdict');
    for (const lens of LENSES) await expect(panel).toContainText(lens);
    await expect(panel).toContainText('not measured');
    await expect(panel).toContainText('What was compared');
    // Task 6.3: the functions and calls of the two commits are drawn; the shapes only where handed in.
    await expect(panel.locator('.bowl svg')).toHaveCount(2);
    await expect(panel).toContainText('Shapes of the state');
    await expect(verdict).toHaveCount(1);
    const said = await verdict.locator('h2').innerText();
    expect(await verdict.getAttribute('class')).toContain({ Accepted: 'ok', Refused: 'bad', 'Not judged': 'warn' }[said]);
    if (said === 'Refused') await expect(verdict.locator('li').first()).not.toBeEmpty();
    outcomes.push(said);
    expect((await marks(page)).failures).toBe('0');
    await picture(page, testInfo, `report-${file.replace(/\.html$/, '')}`);
  }
  expect(failures).toEqual([]);
  // A refusal and an acceptance are both among them.
  expect(outcomes).toEqual(expect.arrayContaining(['Accepted', 'Refused']));
});

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

// The walk through the page by keyboard (task 4.4). What is met is written
// down as it goes, into walk.txt of the record, and the tree a screen reader
// is given into aria.txt. No screen reader is run here.
const focused = (page) => page.evaluate(() => {
  const el = document.activeElement;
  if (!el || el === document.body) return null;
  const style = getComputedStyle(el), say = (name) => el.getAttribute(name);
  return { what: say('role') || el.tagName.toLowerCase(), name: say('aria-label') || el.innerText.replace(/\s+/g, ' ').trim().slice(0, 70),
    labelled: say('aria-labelledby'), pressed: say('aria-pressed'), expanded: say('aria-expanded'), selected: say('aria-selected'),
    outline: `${style.outlineStyle} ${style.outlineWidth}` };
});
const line = (at) => `${at.what} "${at.name}"` + ['pressed', 'expanded', 'selected'].filter((k) => at[k] !== null).map((k) => `, ${k} ${at[k]}`).join('') + `, focus ${at.outline}`;

test('a walk through the page by keyboard', async ({ page }, testInfo) => {
  const out = testInfo.config.metadata.out, walk = [];
  await page.setViewportSize(SIZES[0]);
  await open(page, testInfo, 'walk', run(true, 5));

  walk.push('Tab from the top of the page, one press per line:');
  const stops = [];
  for (let i = 0; i < 60; i++) {
    await page.keyboard.press('Tab');
    const at = await focused(page);
    if (!at) break;
    stops.push(at);
    walk.push(`${String(stops.length).padStart(3)}. ${line(at)}`);
  }
  // The lane of three stages and the result, nine tools, the files of the faults and the two files, the tabs as one stop, and the pane.
  expect(stops.map((s) => s.what)).toEqual([...Array(16).fill('button'), 'tab', 'tabpanel']);
  for (const stop of stops) {
    expect(stop.outline, stop.name).toBe('solid 2px');
    expect(stop.name || stop.labelled, 'every stop has a name').toBeTruthy();
  }
  // Every control of the page is one of the stops, or a tab that the arrows reach.
  expect(await page.locator('button:not([role=tab])').count()).toBe(16);

  const press = async (key, note) => { await page.keyboard.press(key); const at = await focused(page); walk.push(`   ${key}: ${line(at)}${note ? `; ${note}` : ''}`); return at; };
  walk.push('', 'The first tool, Follow, with Space and with Enter:');
  await page.getByRole('button', { name: 'Follow the stage that is going on' }).focus();
  expect((await press('Space')).pressed).toBe('false');
  expect((await press('Enter')).pressed).toBe('true');

  walk.push('', 'A file of the tree, with Enter and with Space:');
  await page.locator('button.d1').first().focus();
  const rows = await page.locator('.d2').count();
  expect((await press('Enter', 'its functions are hidden')).expanded).toBe('false');
  expect(await page.locator('.d2').count()).toBeLessThan(rows);
  expect((await press('Space', 'they are shown again')).expanded).toBe('true');
  expect(await page.locator('.d2').count()).toBe(rows);

  walk.push('', 'A step of the lane, with Enter:');
  await page.locator('.lane .stop').first().focus();
  await press('Enter', 'the stage is the current one and Follow is switched off');
  await expect(page.locator('.lane .stop').first()).toHaveAttribute('aria-current', 'true');
  await expect(page.getByRole('button', { name: 'Follow the stage that is going on' })).toHaveAttribute('aria-pressed', 'false');

  walk.push('', 'The tabs, with the arrows, End and Home:');
  await page.getByRole('tab', { selected: true }).focus();
  expect((await press('ArrowRight')).name).toBe('Your code');
  await expect(page.getByRole('tabpanel')).toContainText('What happens to your code');
  expect((await press('End')).name).toBe('Result');
  expect((await press('Home')).name).toBe('Now');
  expect((await press('ArrowLeft', 'from the first tab the arrow goes round to the last')).name).toBe('Result');
  expect((await press('Tab', 'one press leaves the tabs for the pane')).what).toBe('tabpanel');
  expect((await press('Shift+Tab', 'and one goes back to the tab that is shown')).name).toBe('Result');

  walk.push('', 'The tool Copy, with Enter:');
  await page.getByRole('button', { name: 'Copy the place of the copies' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.said')).toHaveText(/Copied|Could not copy/);
  walk.push(`   Enter: the page says "${await page.locator('.said').innerText()}"`);

  walk.push('', 'The tool for the explanation, with Enter:');
  await page.getByRole('button', { name: 'What a planted fault is' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('tab', { selected: true })).toHaveText('Planted faults');
  walk.push('   Enter: the tab "Planted faults" is shown');

  // Names for a screen reader, and no meaning by colour alone: every mark is a sign with a name.
  const signs = await page.locator('.mark').evaluateAll((all) => all.map((m) => ({ sign: m.innerText.trim(), name: m.getAttribute('aria-label'), role: m.getAttribute('role') })));
  expect(signs.length).toBeGreaterThan(0);
  for (const sign of signs) expect(sign.sign && sign.name && sign.role === 'img', JSON.stringify(sign)).toBeTruthy();
  expect((await page.locator('.pill').first().innerText()).trim()).not.toBe('');
  walk.push('', `Marks in the tree: ${[...new Set(signs.map((m) => `${m.sign} = ${m.name}`))].join('; ')}`);

  writeFileSync(join(out, 'walk.txt'), walk.join('\n') + '\n');
  writeFileSync(join(out, 'aria.txt'), await page.locator('body').ariaSnapshot() + '\n');
  await picture(page, testInfo, 'walk-end');
  expect(Number((await marks(page)).failures)).toBe(0);
});

// Group 8, from the owner's watching of a real run: the steps as a lane, and nothing shown as going on
// before it has begun.
test('the steps of a run stand in one row, and what has not begun says so', async ({ page }, testInfo) => {
  await page.setViewportSize(SIZES[0]);
  const waiting = run(false, 2);
  Object.assign(waiting.stages[0], { status: 'going on', seconds: null, seconds_so_far: 61 });
  Object.assign(waiting.stages[1], { status: 'not begun', done: 0, seconds_so_far: null, seconds_left: null });
  waiting.faults = { ...waiting.faults, done: 0, noticed: 0, unnoticed: 0, example: null,
    files: [tally('src/app.rs', null, 0, 0, 8), tally('src/draw.rs', null, 0, 0, 4)],
    functions: [tally('src/app.rs', 'step', 0, 0, 6), tally('src/app.rs', 'parse', 0, 0, 2), tally('src/draw.rs', 'paint', 0, 0, 4)] };
  await open(page, testInfo, 'lane-waiting', waiting);
  const stops = page.locator('.lane .stop');
  await expect(stops).toHaveText([/1. Copy the project and run your tests on the unchanged codes*going on for 1 min 1 s/, /2. Plant faultss*not begun yet/,
    /3. Check the copies and write the plans*not begun yet/, /4. Results*when the run has ended/]);
  // One thing is going on: the first step. No file and no function is.
  await expect(page.locator('.mark[aria-label="going on"]')).toHaveCount(1);
  await expect(page.getByRole('tabpanel').locator('.strip')).toHaveAttribute('aria-label', 'One square per function. 3 functions: 3 not begun.');
  // The files of a step that has not begun are shut, and open on request.
  const group = page.locator('button.grp');
  await expect(group).toContainText('Plant faults: 2 files');
  await expect(group).toHaveAttribute('aria-expanded', 'false');
  await expect(page.locator('button.d1')).toHaveCount(0);
  await group.click();
  await expect(group).toHaveAttribute('aria-expanded', 'true');
  await expect(page.locator('button.d1')).toHaveCount(2);
  await expect(page.locator('.d2 .count')).toHaveText(['6 to run', '2 to run', '4 to run']);
  await picture(page, testInfo, 'lane-waiting');
  expect(Number((await marks(page)).failures)).toBe(0);

  // Once the faults are being planted their files are open, and the lane counts them.
  await open(page, testInfo, 'lane-running', run(false, 2));
  await expect(page.locator('button.grp')).toHaveAttribute('aria-expanded', 'true');
  await expect(page.locator('.lane .stop').nth(1)).toContainText('8 of 12 done');
  await expect(page.locator('.lane .stop').first()).toContainText('done in 30 s');
  await page.locator('.lane .stop').last().click();
  await expect(page.getByRole('tab', { selected: true })).toHaveText('Result');
  await picture(page, testInfo, 'lane-running');
  expect(Number((await marks(page)).failures)).toBe(0);

  // At its end the run stands at the result; a step that is picked is shown, and Follow goes back.
  await open(page, testInfo, 'lane-ended', run(true, 5));
  await expect(page.locator('.lane .stop').last()).toHaveAttribute('aria-current', 'true');
  await expect(page.locator('.lane .stop[aria-current="true"]')).toHaveCount(1);
  await page.locator('.lane .stop').nth(1).click();
  await expect(page.locator('.lane .stop').nth(1)).toHaveAttribute('aria-current', 'true');
  await expect(page.getByRole('tabpanel').locator('h2')).toHaveText('Plant faults');
  await page.getByRole('button', { name: 'Follow the stage that is going on' }).click();
  await expect(page.getByRole('tabpanel').locator('h2').first()).toHaveText('The plan is written');

  // A run that never planted a fault: the files are shut, and a function says how many faults were not run.
  const never = run(true, 5);
  never.stages[1].status = 'left out'; never.stages[2].status = 'left out'; never.stages[0].status = 'failed';
  never.exit_code = 3; never.outcome = 'Could not judge';
  never.faults = { ...never.faults, done: 0, noticed: 0, unnoticed: 0, files: [tally('src/app.rs', null, 0, 0, 8)], functions: [tally('src/app.rs', 'step', 0, 0, 8)] };
  await open(page, testInfo, 'lane-never', never);
  await expect(page.locator('button.grp')).toHaveAttribute('aria-expanded', 'false');
  await page.locator('button.grp').click();
  await expect(page.locator('.d2 .count')).toHaveText(['8 not run']);

  // A copy that stopped: the tool's sentence is read at once, the end of the output behind a line that opens.
  never.copies.dropped = [{ copy: 2, at: ago(20), reason: 'the unmutated run ended before any test ran in this copy (test step, exit Some(101), ended by the system as happens when memory runs out; the whole output is in D:/work/first-run-w2-2.log)\nerror: could not compile `windows_numerics`' }];
  never.copies.working = 0;
  await open(page, testInfo, 'lane-never-why', never);
  const notice = page.locator('.notice', { hasText: 'Working copy 2 of your project stopped' });
  await expect(notice).toContainText('The tool says: the unmutated run ended before any test ran in this copy');
  await expect(notice).toContainText('first-run-w2-2.log');
  await expect(notice.locator('details')).toHaveCount(1);
  expect(await notice.locator('details').evaluate((d) => d.open)).toBe(false);
  await expect(notice.locator('.raw')).toHaveText('error: could not compile `windows_numerics`');
  await picture(page, testInfo, 'lane-never-why');
  expect(Number((await marks(page)).failures)).toBe(0);
});

test('reduced motion stops the moving bar', async ({ page }, testInfo) => {
  const building = run(false, 2);
  building.stages[0].status = 'going on';
  building.stages[1].status = 'not begun';
  await open(page, testInfo, 'building', building);
  const moving = () => page.locator('.meter.busy i').evaluate((bar) => getComputedStyle(bar).animationName);
  expect(await moving()).toBe('slide');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  expect(await moving()).toBe('none');
});
