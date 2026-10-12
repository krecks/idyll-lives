/**
 * An ONNX model with its weights stored as 16-bit floats: every float32 weight of 1,024 values
 * or more becomes float16 (rounded to nearest, ties to even), and a `Cast` back to float32 is
 * put in front of the graph under the weight's own name, so every operator still sees float32
 * and the GPU does the same arithmetic as before. Half the download; ONNX Runtime folds the
 * casts once when the session starts. A weight whose values don't fit in float16 stays as it is.
 *
 * Works on the protobuf wire format directly (no dependencies): only the graph's initializers
 * and nodes are touched, every other field is copied byte for byte. Deterministic, so the output
 * can be pinned by its SHA-256. Used by `fetch.mjs`.
 */

/** Weights smaller than this stay float32 (biases, norms: little to gain). */
const MIN_VALUES = 1024;

const FLOAT = 1;
const FLOAT16 = 10;
/** `AttributeProto.type` INT. */
const ATTR_INT = 2;

/** Reads a varint at `at`: [value, next position]. Values here stay below 2^53. */
function varint(buf, at) {
  let value = 0;
  let scale = 1;
  for (;;) {
    const b = buf[at++];
    value += (b & 0x7f) * scale;
    if (b < 0x80) return [value, at];
    scale *= 128;
  }
}

function encodeVarint(value) {
  const out = [];
  do {
    let b = value % 128;
    value = Math.floor(value / 128);
    if (value > 0) b |= 0x80;
    out.push(b);
  } while (value > 0);
  return Buffer.from(out);
}

/** A message's fields in order: number, wire type, the payload (length-delimited: its bytes) and the whole field's bytes. */
function fields(buf) {
  const out = [];
  let at = 0;
  while (at < buf.length) {
    const start = at;
    let key;
    [key, at] = varint(buf, at);
    const field = Math.floor(key / 8);
    const wire = key % 8;
    let payload;
    if (wire === 0) {
      let v;
      [v, at] = varint(buf, at);
      payload = v;
    } else if (wire === 1) {
      payload = buf.subarray(at, at + 8);
      at += 8;
    } else if (wire === 2) {
      let len;
      [len, at] = varint(buf, at);
      payload = buf.subarray(at, at + len);
      at += len;
    } else if (wire === 5) {
      payload = buf.subarray(at, at + 4);
      at += 4;
    } else {
      throw new Error(`protobuf: wire type ${wire} not supported`);
    }
    out.push({ field, wire, payload, bytes: buf.subarray(start, at) });
  }
  return out;
}

const bytesField = (field, payload) => Buffer.concat([encodeVarint(field * 8 + 2), encodeVarint(payload.length), payload]);
const varintField = (field, value) => Buffer.concat([encodeVarint(field * 8), encodeVarint(value)]);
const stringField = (field, s) => bytesField(field, Buffer.from(s, 'utf8'));

/** float32 bits to float16 bits, rounded to nearest, ties to even (as NumPy's `astype(float16)`). */
function half(bits) {
  const sign = (bits >>> 16) & 0x8000;
  const exp = (bits >>> 23) & 0xff;
  let mant = bits & 0x7fffff;
  if (exp === 0xff) return sign | 0x7c00 | (mant ? 0x200 : 0);
  const e = exp - 127 + 15;
  if (e >= 0x1f) return sign | 0x7c00;
  if (e <= 0) {
    if (e < -10) return sign;
    mant |= 0x800000;
    const shift = 14 - e;
    let h = mant >>> shift;
    const rest = mant & ((1 << shift) - 1);
    const mid = 1 << (shift - 1);
    if (rest > mid || (rest === mid && h & 1)) h++;
    return sign | h;
  }
  let h = (e << 10) | (mant >>> 13);
  const rest = mant & 0x1fff;
  if (rest > 0x1000 || (rest === 0x1000 && h & 1)) h++;
  return sign | h;
}

/** A float32 tensor's values as float16 bytes, or null if one doesn't fit (it would become infinite). */
function toHalf(values) {
  const bits = new Uint32Array(values.buffer, values.byteOffset, values.length);
  const out = new Uint16Array(values.length);
  for (let i = 0; i < values.length; i++) {
    const h = half(bits[i]);
    if ((h & 0x7c00) === 0x7c00 && (bits[i] & 0x7f800000) !== 0x7f800000) return null;
    out[i] = h;
  }
  return Buffer.from(out.buffer);
}

/** A `TensorProto`'s float32 values (raw or packed), its name and data type. */
function readTensor(buf) {
  let name = '';
  let type = 0;
  let raw = null;
  const floats = [];
  for (const f of fields(buf)) {
    if (f.field === 8) name = f.payload.toString('utf8');
    else if (f.field === 2) type = f.payload;
    else if (f.field === 9) raw = f.payload;
    else if (f.field === 4 && f.wire === 2) for (let i = 0; i < f.payload.length; i += 4) floats.push(f.payload.readFloatLE(i));
    else if (f.field === 4 && f.wire === 5) floats.push(f.payload.readFloatLE(0));
    else if (f.field === 13 || f.field === 14) return { name, type: -1, values: null }; // external data: left alone
  }
  let values = null;
  if (type === FLOAT) {
    if (raw) {
      const copy = Buffer.from(raw);
      values = new Float32Array(copy.buffer, copy.byteOffset, copy.length / 4);
    } else {
      values = Float32Array.from(floats);
    }
  }
  return { name, type, values };
}

/** The tensor with float16 data under `name`: its dims copied as they are. */
function halfTensor(buf, name, data) {
  const dims = fields(buf).filter((f) => f.field === 1).map((f) => f.bytes);
  return Buffer.concat([...dims, varintField(2, FLOAT16), stringField(8, name), bytesField(9, data)]);
}

/** `Cast(from) -> to` as float32. */
function castNode(from, to) {
  const attribute = Buffer.concat([stringField(1, 'to'), varintField(3, FLOAT), varintField(20, ATTR_INT)]);
  return Buffer.concat([stringField(1, from), stringField(2, to), stringField(3, `fp16_cast_${to}`), stringField(4, 'Cast'), bytesField(5, attribute)]);
}

/** The model with its big float32 weights stored as float16; and how many and how many bytes were saved. */
export function weightsToFp16(model) {
  const top = fields(model);
  const graphField = top.find((f) => f.field === 7);
  if (!graphField) throw new Error('ONNX: no graph');
  const casts = [];
  const graphParts = [];
  let saved = 0;
  for (const f of fields(graphField.payload)) {
    if (f.field === 5) {
      const t = readTensor(f.payload);
      const data = t.type === FLOAT && t.values && t.values.length >= MIN_VALUES ? toHalf(t.values) : null;
      if (data) {
        const name = `${t.name}__fp16`;
        graphParts.push(bytesField(5, halfTensor(f.payload, name, data)));
        casts.push(bytesField(1, castNode(name, t.name)));
        saved += t.values.length * 2;
        continue;
      }
    }
    graphParts.push(f.bytes);
  }
  // The casts before every other node (nodes are listed in the order they run).
  const graph = Buffer.concat([...casts, ...graphParts]);
  const out = Buffer.concat(top.map((f) => (f === graphField ? bytesField(7, graph) : f.bytes)));
  return { model: out, weights: casts.length, saved };
}
