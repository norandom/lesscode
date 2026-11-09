// Counted TypeScript/JavaScript observation and tagged value codec.
// Governed by:
// - .kiro/specs/native-deconvolution-closure/adapter-contract.md
// - .kiro/specs/typescript-deconvolution-adapter/design.md

export function toTaggedValue(val) {
  if (val === null || val === undefined) {
    return { type: "null" };
  }
  if (typeof val === "boolean") {
    return { type: "bool", value: val };
  }
  if (typeof val === "number") {
    if (Number.isInteger(val) && !Object.is(val, -0) && val >= -9007199254740991 && val <= 9007199254740991) {
      return { type: "int64", value: val };
    }
    const buf = new ArrayBuffer(8);
    const view = new DataView(buf);
    view.setFloat64(0, val, false);
    const bits = view.getBigUint64(0, false);
    return { type: "float_bits", value: bits.toString() };
  }
  if (typeof val === "string") {
    return { type: "string", value: val };
  }
  if (Array.isArray(val)) {
    return { type: "array", value: val.map(toTaggedValue) };
  }
  if (typeof val === "object") {
    const keys = Object.keys(val).sort();
    return {
      type: "object",
      value: keys.map((k) => [k, toTaggedValue(val[k])]),
    };
  }
  return { type: "string", value: String(val) };
}

export class SinkBuffer {
  constructor(name) {
    this.name = name;
    this.events = [];
    this.totalBytes = 0;
  }

  write(chunk) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk), "utf-8");
    this.totalBytes += bytes.length;
    this.events.push({
      sink: this.name,
      byteCount: bytes.length,
      hex: bytes.toString("hex"),
      timestamp: Date.now(),
    });
  }

  flush() {
    const captured = [...this.events];
    this.events = [];
    return captured;
  }

  reset() {
    this.events = [];
    this.totalBytes = 0;
  }
}
