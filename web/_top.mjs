import sharp from 'sharp';
const [,, a, b, outPng] = process.argv;
const A = await sharp(a).raw().toBuffer({ resolveWithObject: true }), B = await sharp(b).raw().toBuffer({ resolveWithObject: true });
const ch = A.info.channels, n = A.info.width * A.info.height;
const lum = (d, i) => 0.2126 * d[i] + 0.7152 * d[i + 1] + 0.0722 * d[i + 2];
const la = new Float32Array(n), lb = new Float32Array(n);
for (let i = 0; i < n; i++) { la[i] = lum(A.data, i * ch); lb[i] = lum(B.data, i * ch); }
const pct = (l, p) => Float32Array.from(l).sort()[Math.floor(n * p)];
const tA = pct(la, 0.95), tB = pct(lb, 0.95);
const cls = (d, i) => { const [r, g, bl] = [d[i], d[i + 1], d[i + 2]]; return g > r * 1.12 && g > bl * 1.25 ? 'green' : r > g * 1.15 && g > bl * 1.1 ? 'warm' : 'neutral'; };
const sum = {};
const mask = Buffer.alloc(n * 3);
for (let i = 0; i < n; i++) {
  if (la[i] >= tA) { const c = cls(A.data, i * ch); (sum[c] ??= { n: 0, a: 0, b: 0 }); sum[c].n++; sum[c].a += la[i]; sum[c].b += lb[i]; mask[i * 3] = 255; }
  if (lb[i] >= tB) mask[i * 3 + 1] = 255;
}
console.log('p95', tA.toFixed(0), '->', tB.toFixed(0));
for (const [c, s] of Object.entries(sum)) console.log(c.padEnd(8), `${(100 * s.n / n).toFixed(2)} % of frame`, 'was', (s.a / s.n).toFixed(0), 'now', (s.b / s.n).toFixed(0));
if (outPng) await sharp(mask, { raw: { width: A.info.width, height: A.info.height, channels: 3 } }).resize({ width: 960 }).png().toFile(outPng);
