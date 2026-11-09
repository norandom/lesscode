// Counted TypeScript/JavaScript scope entrypoints and reset adapters.
// Governed by:
// - .kiro/specs/native-deconvolution-closure/adapter-contract.md
// - .kiro/specs/typescript-deconvolution-adapter/design.md

import { pathToFileURL } from "node:url";
import path from "node:path";
import { toTaggedValue, SinkBuffer } from "./observe.mjs";

export class ScopeHarness {
  constructor() {
    this.modules = new Map();
    this.stdoutSink = new SinkBuffer("stdout");
    this.stderrSink = new SinkBuffer("stderr");
  }

  async loadModule(filePath) {
    const url = filePath.startsWith("node:")
      ? filePath
      : pathToFileURL(path.resolve(filePath)).href;
    if (!this.modules.has(url)) {
      const mod = await import(url);
      this.modules.set(url, mod);
    }
    return this.modules.get(url);
  }

  async invoke(filePath, exportName, args = []) {
    this.stdoutSink.reset();
    this.stderrSink.reset();

    const mod = await this.loadModule(filePath);
    const fn = mod[exportName];
    if (typeof fn !== "function") {
      throw new Error(`Export "${exportName}" in ${filePath} is not a function`);
    }

    let result = null;
    let error = null;
    try {
      result = fn(...args);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }

    const stateFrame = {
      result: error === null ? toTaggedValue(result) : null,
      error: error,
      flushedStdout: this.stdoutSink.flush(),
      flushedStderr: this.stderrSink.flush(),
    };

    return stateFrame;
  }
}
