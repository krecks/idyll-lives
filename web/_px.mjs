import sharp from 'sharp';
const [,, ...rest] = process.argv;
const srcs = rest.filter((a) => a.endsWith('.png')), pts = rest.filter((a) => !a.endsWith('.png'));
const imgs = await Promise.all(srcs.map((s) => sharp(s).raw().toBuffer({ resolveWithObject: true })));
for (const p of pts) {
  const [name, xy] = p.split('='); const [x, y] = xy.split(',').map(Number);
  const cols = imgs.map(({ data, info }) => { const m = [0, 0, 0]; for (let dy = -3; dy <= 3; dy++) for (let dx = -3; dx <= 3; dx++) { const i = ((y + dy) * info.width + x + dx) * info.channels; for (let k = 0; k < 3; k++) m[k] += data[i + k] / 49; } return m.map((v) => v.toFixed(0).padStart(3)).join(' '); });
  console.log(name.padEnd(10), cols.join('  |  '));
}
