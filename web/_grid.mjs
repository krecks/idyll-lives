import sharp from 'sharp';
const [,, out, l, t, w, h, cols, k, ...srcs] = process.argv;
const W = Math.round(+w * +k), H = Math.round(+h * +k);
const tiles = await Promise.all(srcs.map((s) => sharp(s).extract({ left: +l, top: +t, width: +w, height: +h }).resize({ width: W, height: H }).png().toBuffer()));
const rows = Math.ceil(tiles.length / +cols);
await sharp({ create: { width: (W + 6) * +cols, height: (H + 6) * rows, channels: 3, background: '#ff00ff' } }).composite(tiles.map((input, i) => ({ input, left: (i % +cols) * (W + 6), top: Math.floor(i / +cols) * (H + 6) }))).png().toFile(out);
