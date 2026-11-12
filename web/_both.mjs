import sharp from 'sharp';
const [,, a, b] = process.argv;
const A = await sharp(a).raw().toBuffer({ resolveWithObject: true }), B = await sharp(b).raw().toBuffer({ resolveWithObject: true });
const ch = A.info.channels, n = A.info.width * A.info.height;
const lum = (d, i) => 0.2126 * d[i] + 0.7152 * d[i + 1] + 0.0722 * d[i + 2];
const la = new Float32Array(n), lb = new Float32Array(n);
for (let i = 0; i < n; i++) { la[i] = lum(A.data, i * ch); lb[i] = lum(B.data, i * ch); }
const pct = (l, p) => Float32Array.from(l).sort()[Math.floor(n * p)];
const tA = pct(la, 0.95), tB = pct(lb, 0.95);
let k = [0, 0, 0], lost = [0, 0, 0];
for (let i = 0; i < n; i++) if (la[i] >= tA) { const s = lb[i] >= tB ? k : lost; s[0]++; s[1] += la[i]; s[2] += lb[i]; }
console.log(a.split('/').pop(), `still brightest: ${(100 * k[0] / n).toFixed(2)} % of frame, was ${(k[1] / k[0]).toFixed(0)} now ${(k[2] / k[0]).toFixed(0)}; no longer: ${(100 * lost[0] / n).toFixed(2)} %, was ${(lost[1] / lost[0]).toFixed(0)} now ${(lost[2] / lost[0]).toFixed(0)}`);
