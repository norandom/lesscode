// The page that lists several reports, opened from disk in a headless browser
// (.kiro/specs/run-dashboard, task 7.1). The lists are those the tool wrote for the fixture
// steps of native/tests/run_page_list.rs: LESSCODE_CHECK_REPORTS names the directory that holds
// `list` and `list-without-names`. Without it only the page's own states are checked.
//
// Run with verification/playwright.config.mjs; see there.
import { test, expect } from '../../verification/node_modules/@playwright/test/index.mjs';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const SIZES = [{ width: 1280, height: 800 }, { width: 400, height: 800 }];
const kept = process.env.LESSCODE_CHECK_REPORTS;
const listAt = (name) => join(kept ?? '', name, 'index.html');
const have = (name) => Boolean(kept) && existsSync(listAt(name));
// Names of the fixture project that no sentence of the tool has.
const NAMES = ['app.py', 'expect.txt', 'helper', 'toy_check'];

const marks = (page) => page.evaluate(() => ({ ...document.documentElement.dataset }));
const sideways = (page) => page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
async function picture(page, testInfo, name) {
  await page.screenshot({ path: join(testInfo.config.metadata.out, 'pictures', `${name}.png`), fullPage: true });
}
// What a block is built of: its parts in order, without what they say.
const build = (card) => card.evaluate((node) => [...node.children].map((part) => `${part.tagName}.${part.className.replace(/ (ok|bad|warn)$/, '')}`).join(' '));

for (const size of SIZES) {
  test(`the list of three fixture reports shows all three alike at ${size.width}`, async ({ page }, testInfo) => {
    test.skip(!have('list'), 'LESSCODE_CHECK_REPORTS names no directory with a list');
    const failures = [];
    page.on('pageerror', (error) => failures.push(String(error)));
    await page.setViewportSize(size);
    await page.goto(pathToFileURL(listAt('list')).href);
    await expect(page.locator('html')).toHaveAttribute('data-state', 'list');
    const cards = page.locator('.card');
    await expect(cards).toHaveCount(3);
    await expect(page.locator('.head')).toContainText('3 reports');
    const builds = [];
    for (let i = 0; i < 3; i += 1) {
      const card = cards.nth(i);
      builds.push(await build(card));
      // Requirement 8.1: the project, the scope, the date and the lenses before and after.
      await expect(card.locator('h2')).toHaveText('repo');
      await expect(card.locator('.facts')).toContainText('src/app.py');
      await expect(card.locator('header')).toContainText(/\d{2}\/\d{2}\/\d{4}/);
      await expect(card.locator('thead th')).toHaveText(['Lens', 'Measure', 'Before', 'After', 'Expected', 'Moved that way']);
      for (const lens of ['Size', 'Structure', 'State', 'Local complexity', 'Behaviour kept', 'Protection']) await expect(card.locator('tbody')).toContainText(lens);
      await expect(card.locator('tbody')).toContainText('not measured');
      // The outcome has a mark and a word, not a colour alone.
      const kind = await card.getAttribute('data-outcome'), said = await card.locator('.pill').innerText();
      expect({ ok: '✓ Accepted', bad: '✗ Refused' }[kind]).toBe(said.trim());
      await expect(card.locator('.verdict')).toHaveClass(new RegExp(kind));
      await expect(card.locator('.verdict')).not.toBeEmpty();
    }
    // Requirement 8.3: a refusal, and a step whose size grew, are built like the acceptance.
    expect(new Set(builds).size, builds.join('\n')).toBe(1);
    expect(await cards.evaluateAll((all) => all.map((card) => card.dataset.outcome))).toEqual(['ok', 'bad', 'bad']);
    await expect(cards.nth(1).locator('.verdict li').first()).toContainText('unnoticed faults rise');
    await expect(cards.nth(2).locator('tbody tr').first()).toContainText('no');
    expect(await sideways(page)).toBeLessThanOrEqual(1);
    expect(await page.evaluate(() => window.innerWidth)).toBe(size.width);
    await picture(page, testInfo, `list-${size.width}`);
    expect((await marks(page)).failures).toBe('0');
    expect(failures).toEqual([]);
  });
}

test('each report of the list opens from its link', async ({ page }, testInfo) => {
  test.skip(!have('list'), 'LESSCODE_CHECK_REPORTS names no directory with a list');
  await page.setViewportSize(SIZES[0]);
  const outcomes = [];
  for (let i = 0; i < 3; i += 1) {
    await page.goto(pathToFileURL(listAt('list')).href);
    // Requirement 8.2. The name of the project is the link; by keyboard it is the first stop of its block.
    const link = page.locator('.card').nth(i).locator('h2 a');
    await link.focus();
    await page.keyboard.press('Enter');
    await expect(page.locator('html')).toHaveAttribute('data-state', 'ended');
    expect(page.url()).toContain(`report-0${i + 1}.html`);
    outcomes.push(await page.getByRole('tabpanel').locator('.verdict h2').innerText());
    expect((await marks(page)).failures).toBe('0');
  }
  expect(outcomes).toEqual(['Accepted', 'Refused', 'Refused']);
});

test('with the names left out, neither the list nor its reports show a name of the project', async ({ page }, testInfo) => {
  test.skip(!have('list-without-names'), 'LESSCODE_CHECK_REPORTS names no directory with a list without names');
  await page.setViewportSize(SIZES[0]);
  const failures = [];
  page.on('pageerror', (error) => failures.push(String(error)));
  const nameless = async (what) => {
    const said = await page.locator('body').innerText();
    for (const name of NAMES) expect(said.includes(name), `${name} in ${what}`).toBe(false);
  };
  await page.goto(pathToFileURL(listAt('list-without-names')).href);
  await expect(page.locator('.card')).toHaveCount(3);
  await expect(page.locator('.card').first().locator('.facts')).toContainText('file 1');
  await expect(page.locator('.card').first().locator('.facts')).toContainText('the names of files and functions of this project are left out');
  await expect(page.locator('.card').nth(1).locator('.verdict')).toContainText('file 1: function 2: unnoticed faults rise');
  await nameless('the list');
  await picture(page, testInfo, 'list-without-names');
  for (let i = 1; i <= 3; i += 1) {
    await page.goto(pathToFileURL(join(kept, 'list-without-names', `report-0${i}.html`)).href);
    await expect(page.locator('html')).toHaveAttribute('data-state', 'ended');
    // The report says what was done to it, where it names the repository.
    for (const tab of await page.getByRole('tab').all()) {
      await tab.click();
      await nameless(`report ${i}, tab ${await tab.innerText()}`);
    }
    await page.getByRole('tab', { name: /^Your code/ }).click();
    await expect(page.getByRole('tabpanel')).toContainText('the names of its files and functions are left out');
    await expect(page.getByRole('tabpanel')).toContainText('a command of the project, left out');
    await page.getByRole('tab', { name: /^Result/ }).click();
    // The drawing of functions and calls is still there, with numbers for names.
    await expect(page.getByRole('tabpanel').locator('.bowl svg')).toHaveCount(2);
    await picture(page, testInfo, `report-without-names-${i}`);
    expect((await marks(page)).failures).toBe('0');
  }
  expect(failures).toEqual([]);
});

// The page's own states, written here from list.html as the tool writes it.
function written(out, name, data) {
  const lib = readFileSync(join(here, 'htm-preact-standalone.umd.js'), 'utf8');
  let page = readFileSync(join(here, 'list.html'), 'utf8');
  for (const [marker, value] of [['/*LIB*/', lib], ['/*DATA*/null', JSON.stringify(data).replaceAll('<', '\\u003c')]]) {
    const at = page.indexOf(marker);
    if (at < 0) throw new Error(`the page has no ${marker}`);
    page = page.slice(0, at) + value + page.slice(at + marker.length);
  }
  mkdirSync(join(out, 'pages'), { recursive: true });
  const path = join(out, 'pages', `${name}.html`);
  writeFileSync(path, page);
  return pathToFileURL(path).href;
}
const row = (more) => ({ link: 'report-01.html', project: 'example', names_left_out: false, scope: ['src/app.rs'], date: '2026-11-23T18:56:09.472Z',
  tool: { name: 'lesscode-core', version: '2.1.0' }, command: 'step check', branch: 'step/smaller', commit: '0123456789abcdef', compared_with: 'fedcba9876543210',
  outcome: 'Accepted', outcome_detail: 'Every check passed.', exit_code: 0, reasons: null, lenses: [], ...more });
const STATES = [
  { name: 'list-another-version', data: { schema: 2, reports: [] }, state: 'cannot-read', shows: ['This page cannot read its list of reports', 'version 2'] },
  { name: 'list-of-nothing', data: { schema: 1, reports: [] }, state: 'list', shows: ['0 reports', 'The list has no report.'] },
  // A run that judged nothing, one that could not judge, and fields that are missing or of another kind.
  { name: 'list-of-odd-rows', state: 'list', shows: ['compared nothing before and after', 'Not judged', '</script><b>', 'not known'],
    data: { schema: 1, written: 'never', tool: null, reports: [
      row({ command: 'step plan', compared_with: null, outcome: 'The plan is written', lenses: null }),
      row({ outcome: 'Not judged', exit_code: 2, outcome_detail: '</script><b>', project: '</script><b>' }),
      { link: 7, lenses: [null, { lens: 'Size', measured: true, before: 4, after: null, moved: null }], scope: 'all', reasons: 'none' }, null, 'a text'] } },
];
for (const s of STATES) {
  test(`${s.name}`, async ({ page }, testInfo) => {
    const failures = [];
    page.on('pageerror', (error) => failures.push(String(error)));
    await page.setViewportSize(SIZES[1]);
    await page.goto(written(testInfo.config.metadata.out, s.name, s.data));
    await expect(page.locator('html')).toHaveAttribute('data-state', s.state);
    for (const text of s.shows) await expect(page.locator('body')).toContainText(text);
    expect(await sideways(page)).toBeLessThanOrEqual(1);
    await picture(page, testInfo, s.name);
    expect((await marks(page)).failures).toBe('0');
    expect(failures).toEqual([]);
  });
}
