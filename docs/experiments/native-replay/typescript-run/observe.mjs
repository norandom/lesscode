// Frozen task-local observations through the real public module exports.
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';
const [work, output] = process.argv.slice(2);
const api = await import(pathToFileURL(`${work}/src/degradation.ts`).href);
const occurredAt = '2026-11-01T00:00:00.000Z';
const alphabet = ['a', 'Z', '0', '-', '_', ' ', '\t', '\n', 'é', 'İ', 'ß', '😀', '\ud800', '\u0307'];
const inputs = ['', ' --- ', 'ABC_def 12', 'é', 'İ', 'ẞ', 'Σς', 'ＡＢＣ', 'a\u0000b'];
for (const a of alphabet) for (const b of alphabet) for (const c of alphabet) inputs.push(a + b + c);
const outcomes = [];
for (const value of inputs) {
  outcomes.push(api.createUnsupportedCapabilityNotice({ capability: value, reason: 'reason', occurredAt }));
  outcomes.push(api.createPermissionDeniedNotice({ operation: value, reason: 'reason', occurredAt, phase: 'execution' }));
}
const notices = [
  api.createDisabledWorkflowNotice({ reason: 'disabled', occurredAt }),
  api.createFailedPhaseNotice({ phase: 'verification', reason: 'failure', occurredAt }),
  api.createHighEffortLimitationNotice({ reason: 'limited', occurredAt }),
  api.createMultiAgentUnavailableNotice({ reason: 'unavailable', occurredAt })
];
outcomes.push(...notices);
for (const enabled of [true, false]) for (let mask = 0; mask < 16; mask++) {
  outcomes.push(api.summarizeWorkflowDegradation({ enabled, notices: notices.filter((_, i) => mask & (1 << i)) }));
}
const digests = outcomes.map(value => createHash('sha256').update(JSON.stringify(value)).digest('hex'));
writeFileSync(output, JSON.stringify({ cases: outcomes.length, digests }, null, 2) + '\n');
