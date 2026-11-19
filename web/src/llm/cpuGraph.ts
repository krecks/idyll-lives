/**
 * The language model's graph for the processor. Its 4-bit export reads the tokens with
 * `GatherBlockQuantized`, which ONNX Runtime Web's WebGPU build (the one the game loads) has no
 * processor kernel for: the session would not start there. `forCpu` puts standard operators in
 * its place that compute the same embedding: gather the token's packed row, its scales and zero
 * points, unpack the two 4-bit values in each byte (the first in the low half), and scale them.
 * Only the token's own rows are unpacked, so nothing more is kept in memory.
 *
 * Works on the protobuf wire format, as `crates/tools/src/llmslim.rs` does: the node is cut out
 * of the graph and the new nodes and their small constants are added at its end (ONNX Runtime
 * sorts the nodes itself); everything else is copied byte for byte. Handles what the export has:
 * 4 bits, a block along the row, token ids of shape [batch, sequence].
 */

/** ModelProto.graph; GraphProto.node and .initializer; NodeProto's fields; AttributeProto's. */
const MODEL_GRAPH = 7;
const GRAPH_NODE = 1;
const GRAPH_INITIALIZER = 5;
const NODE = { input: 1, output: 2, name: 3, opType: 4, attribute: 5, domain: 7 } as const;
const ATTR = { name: 1, i: 3, type: 20 } as const;
/** AttributeProto.type INT; TensorProto data types. */
const INT = 2;
const FLOAT = 1;
const INT32 = 6;
const INT64 = 7;
/** Wire types: varint, length-delimited. */
const VARINT = 0;
const BYTES = 2;

interface Field {
  no: number;
  wire: number;
  /** The whole field (tag included), and a length-delimited one's payload. */
  start: number;
  end: number;
  body: number;
  /** A varint's value. */
  value: bigint;
}

function readVarint(buf: Uint8Array, at: number): { value: bigint; next: number } {
  let value = 0n;
  let shift = 0n;
  for (;;) {
    if (at >= buf.length) throw new Error('model graph: truncated varint');
    const b = buf[at++];
    value |= BigInt(b & 0x7f) << shift;
    if (b < 0x80) return { value, next: at };
    shift += 7n;
  }
}

/** A message's fields, in order. */
function fields(buf: Uint8Array): Field[] {
  const out: Field[] = [];
  let at = 0;
  while (at < buf.length) {
    const start = at;
    const tag = readVarint(buf, at);
    at = tag.next;
    const no = Number(tag.value >> 3n);
    const wire = Number(tag.value & 7n);
    let body = at;
    let value = 0n;
    if (wire === VARINT) {
      ({ value, next: at } = readVarint(buf, at));
    } else if (wire === BYTES) {
      const len = readVarint(buf, at);
      body = len.next;
      at = body + Number(len.value);
    } else if (wire === 1) {
      at += 8;
    } else if (wire === 5) {
      at += 4;
    } else {
      throw new Error(`model graph: wire type ${wire}`);
    }
    if (at > buf.length) throw new Error('model graph: truncated field');
    out.push({ no, wire, start, end: at, body, value });
  }
  return out;
}

const text = new TextDecoder();
const utf8 = new TextEncoder();
const payload = (buf: Uint8Array, f: Field) => buf.subarray(f.body, f.end);
const str = (buf: Uint8Array, f: Field) => text.decode(payload(buf, f));

function varint(v: bigint | number): number[] {
  let n = BigInt.asUintN(64, BigInt(v));
  const out: number[] = [];
  while (n >= 0x80n) {
    out.push(Number(n & 0x7fn) | 0x80);
    n >>= 7n;
  }
  out.push(Number(n));
  return out;
}

function concat(parts: (Uint8Array | number[])[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

const tagged = (no: number, wire: number) => varint((no << 3) | wire);
const bytesField = (no: number, data: Uint8Array) => concat([tagged(no, BYTES), varint(data.length), data]);
const stringField = (no: number, s: string) => bytesField(no, utf8.encode(s));
const intField = (no: number, v: number) => concat([tagged(no, VARINT), varint(v)]);

/** The graph's nodes and constants to add. */
class Builder {
  readonly nodes: Uint8Array[] = [];
  readonly constants: Uint8Array[] = [];
  constructor(private readonly prefix: string) {}

  /** A constant: int64 (shapes, axes) or int32 values, `dims` its shape ([] a scalar). */
  constant(name: string, type: typeof INT64 | typeof INT32 | typeof FLOAT, values: number[], dims: number[] = [values.length]): string {
    const full = this.prefix + name;
    const raw = type === INT64 ? new Uint8Array(BigInt64Array.from(values, BigInt).buffer) : type === INT32 ? new Uint8Array(Int32Array.from(values).buffer) : new Uint8Array(Float32Array.from(values).buffer);
    this.constants.push(concat([...dims.map((d) => intField(1, d)), intField(2, type), stringField(8, full), bytesField(9, raw)]));
    return full;
  }

  /** A node writing `output` (prefixed unless `exact`). */
  node(op: string, inputs: string[], output: string, attrs: Record<string, number> = {}, exact = false): string {
    const out = exact ? output : this.prefix + output;
    const attributes = Object.entries(attrs).map(([k, v]) => bytesField(NODE.attribute, concat([stringField(ATTR.name, k), intField(ATTR.i, v), intField(ATTR.type, INT)])));
    this.nodes.push(concat([...inputs.map((i) => stringField(NODE.input, i)), stringField(NODE.output, out), stringField(NODE.name, out), stringField(NODE.opType, op), ...attributes]));
    return out;
  }
}

/**
 * The graph with every `GatherBlockQuantized` replaced by standard operators (the same graph if
 * there is none). Throws for a variant it doesn't handle.
 */
export function forCpu(model: Uint8Array): Uint8Array {
  const top = fields(model);
  const graphField = top.find((f) => f.no === MODEL_GRAPH && f.wire === BYTES);
  if (!graphField) throw new Error('model graph: no graph');
  const graph = payload(model, graphField);
  const graphFields = fields(graph);
  const found = graphFields.filter((f) => f.no === GRAPH_NODE && f.wire === BYTES && nodeOp(graph, f) === 'GatherBlockQuantized');
  if (!found.length) return model;
  const added: Uint8Array[] = [];
  for (const [k, f] of found.entries()) added.push(...replace(payload(graph, f), `/cpu_gather_${k}/`));
  const kept = graphFields.filter((f) => !found.includes(f)).map((f) => graph.subarray(f.start, f.end));
  const newGraph = concat([...kept, ...added]);
  return concat([model.subarray(0, graphField.start), bytesField(MODEL_GRAPH, newGraph), model.subarray(graphField.end)]);
}

function nodeOp(graph: Uint8Array, f: Field): string {
  const op = fields(payload(graph, f)).find((x) => x.no === NODE.opType);
  return op ? str(payload(graph, f), op) : '';
}

/** The fields (nodes, then constants) standing in for one `GatherBlockQuantized` node. */
function replace(node: Uint8Array, prefix: string): Uint8Array[] {
  const nf = fields(node);
  const inputs = nf.filter((f) => f.no === NODE.input).map((f) => str(node, f));
  const output = nf.find((f) => f.no === NODE.output);
  if (!output) throw new Error('model graph: GatherBlockQuantized without an output');
  const attrs: Record<string, number> = {};
  for (const f of nf.filter((x) => x.no === NODE.attribute)) {
    const af = fields(payload(node, f));
    const name = af.find((x) => x.no === ATTR.name);
    const i = af.find((x) => x.no === ATTR.i);
    if (name && i) attrs[str(payload(node, f), name)] = Number(BigInt.asIntN(64, i.value));
  }
  const bits = attrs.bits ?? 4;
  const block = attrs.block_size ?? 128;
  if (bits !== 4 || (attrs.gather_axis ?? 0) !== 0 || (attrs.quantize_axis ?? 1) !== 1 || inputs.length < 3) {
    throw new Error(`model graph: GatherBlockQuantized ${JSON.stringify(attrs)} with ${inputs.length} inputs isn't handled on the processor`);
  }
  const [data, ids, scales, zeroPoints] = inputs;
  const b = new Builder(prefix);
  const sixteen = b.constant('16', INT32, [16], []);
  const last = b.constant('last', INT64, [-1]);
  /** Bytes of two 4-bit values each, as int32 pairs (low half first) along a new last axis. */
  const unpack = (bytes: string, tag: string) => {
    const wide = b.node('Cast', [bytes], `${tag}_i32`, { to: INT32 });
    const high = b.node('Div', [wide, sixteen], `${tag}_high`);
    const low = b.node('Sub', [wide, b.node('Mul', [high, sixteen], `${tag}_high16`)], `${tag}_low`);
    return b.node('Concat', [b.node('Unsqueeze', [low, last], `${tag}_low1`), b.node('Unsqueeze', [high, last], `${tag}_high1`)], `${tag}_pairs`, { axis: -1 });
  };
  // [batch, sequence, blocks, block] values, and per block a zero point and a scale.
  const values = b.node('Cast', [b.node('Reshape', [unpack(b.node('Gather', [data, ids], 'rows', { axis: 0 }), 'rows'), b.constant('blocks', INT64, [0, 0, -1, block])], 'rows_blocks')], 'rows_f', { to: FLOAT });
  const zero = zeroPoints
    ? b.node('Unsqueeze', [b.node('Cast', [b.node('Reshape', [unpack(b.node('Gather', [zeroPoints, ids], 'zp', { axis: 0 }), 'zp'), b.constant('flat3', INT64, [0, 0, -1])], 'zp_flat')], 'zp_f', { to: FLOAT }), last], 'zp_1')
    : b.constant('zp8', FLOAT, [8], []);
  const scale = b.node('Unsqueeze', [b.node('Gather', [scales, ids], 'scales', { axis: 0 }), last], 'scales_1');
  const scaled = b.node('Mul', [b.node('Sub', [values, zero], 'centred'), scale], 'scaled');
  b.node('Reshape', [scaled, b.constant('flat', INT64, [0, 0, -1])], str(node, output), {}, true);
  return [...b.nodes.map((n) => bytesField(GRAPH_NODE, n)), ...b.constants.map((c) => bytesField(GRAPH_INITIALIZER, c))];
}
