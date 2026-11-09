// Counted Node runtime entrypoint; reserved protocol writer and process-output interception.
// Governed by:
// - .kiro/specs/native-deconvolution-closure/adapter-contract.md
// - .kiro/specs/typescript-deconvolution-adapter/design.md

import fs from "node:fs";
import { ScopeHarness } from "./scope.mjs";

function writeEvent(eventObj) {
  process.stdout.write(JSON.stringify(eventObj) + "\n");
}

function logDiag(msg) {
  process.stderr.write(`[ts-node-v1] ${msg}\n`);
}

async function main() {
  const args = process.argv.slice(2);
  let requestJson = null;

  if (args.includes("--request") && args[args.indexOf("--request") + 1]) {
    const p = args[args.indexOf("--request") + 1];
    requestJson = fs.readFileSync(p, "utf-8");
  } else if (!process.stdin.isTTY) {
    const chunks = [];
    for await (const chunk of process.stdin) {
      chunks.push(chunk);
    }
    requestJson = Buffer.concat(chunks).toString("utf-8");
  }

  if (!requestJson) {
    logDiag("No request provided via stdin or --request");
    process.exit(1);
  }

  let request;
  try {
    request = JSON.parse(requestJson);
  } catch (err) {
    logDiag(`Malformed JSON request: ${err.message}`);
    process.exit(2);
  }

  const { request_id = "req-0", module_path, function_name, cases = [] } = request;

  writeEvent({
    event: "start",
    protocol_version: 1,
    request_id,
    profile_id: "ts-node-v1",
    timestamp: Date.now(),
  });

  const harness = new ScopeHarness();
  let executedCount = 0;

  for (const c of cases) {
    const caseId = c.case_id || `case-${executedCount}`;
    try {
      const frame = await harness.invoke(module_path, function_name, c.args || []);
      writeEvent({
        event: "state",
        request_id,
        case_id: caseId,
        state: frame,
      });
      executedCount++;
    } catch (err) {
      writeEvent({
        event: "case_failed",
        request_id,
        case_id: caseId,
        error: err.message,
      });
    }
  }

  writeEvent({
    event: "complete",
    request_id,
    status: "success",
    case_count: executedCount,
    timestamp: Date.now(),
  });
}

main().catch((err) => {
  logDiag(`Uncaught runtime error: ${err.stack || err.message}`);
  process.exit(3);
});
