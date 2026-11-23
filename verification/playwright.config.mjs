// The run page in a headless browser (.kiro/specs/run-dashboard, tasks 4.2 and 4.3).
// Not one of the unit tests, and nothing here is part of the tool or of its build.
//
//   $env:LESSCODE_PAGE_CHECK_OUT = 'D:\BuildArtifacts\lesscode\run-dashboard\states-1'
//   node verification/node_modules/@playwright/test/cli.js test --config verification/playwright.config.mjs
import { defineConfig } from '@playwright/test';
import { isAbsolute, join, parse, relative, resolve } from 'node:path';

function outputRoot() {
  const value = process.env.LESSCODE_PAGE_CHECK_OUT;
  if (!value || !isAbsolute(value)) throw new Error('LESSCODE_PAGE_CHECK_OUT must name an absolute directory for the record');
  const path = resolve(value);
  if (path === parse(path).root) throw new Error('the record cannot go into the root of a drive');
  if (process.platform === 'win32') {
    const inside = relative('D:/BuildArtifacts', path);
    if (!inside || inside.startsWith('..') || isAbsolute(inside)) throw new Error('on Windows the record goes beneath D:/BuildArtifacts');
    if (!process.env.PLAYWRIGHT_BROWSERS_PATH) throw new Error('PLAYWRIGHT_BROWSERS_PATH must be set on Windows');
  }
  return path;
}
const out = outputRoot();

export default defineConfig({
  testDir: '../native/page',
  testMatch: 'states.spec.mjs',
  workers: 1,
  retries: 0,
  reporter: [['line'], ['json', { outputFile: join(out, 'states.json') }]],
  outputDir: join(out, 'test-results'),
  metadata: { out },
  use: { browserName: 'chromium', headless: true, trace: 'off', video: 'off', screenshot: 'off' },
});
