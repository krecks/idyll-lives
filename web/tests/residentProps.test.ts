// Held props (`render/pc/residentProps.ts` `show`): only the rows that hold a prop (a matrix set) are copied into its
// draws, packed into the first copies; the rest of the town's rows (all zeros) are neither uploaded nor drawn.
import { afterAll, expect, mock, test } from 'bun:test';

const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
// (Building props isn't what is tested: the models' and materials' modules, with the settings they read, are left out.)
const none = () => {};
mock.module('../src/render/pc/materials', () => ({ INSTANCE_COLOR_SEMANTIC: 'ATTR10' }));
mock.module('../src/render/pc/models', () => ({ buildModel: none, disposeTemplate: none, readMesh: none, releaseTemplateFile: none, retellModelFileArrived: none, uploadData: none }));
const { ResidentProps } = await import('../src/render/pc/residentProps');
type Prop = import('../src/render/pc/residentProps').Prop;

test('a prop draws one copy per holder, none for the rows that hold nothing', () => {
  // A draw as `Instances` is, its copies' x as written.
  const draw = { enabled: false, count: 0, xs: [] as number[], uploads: 0 };
  const fake = Object.assign(draw, {
    setCount: (n: number) => void (draw.count = n),
    setMatrix: (i: number, m: Float32Array, at: number) => void (draw.xs[i] = m[at + 12]),
    update: () => void draw.uploads++,
  });
  const rows = 8;
  const matrices = new Float32Array(rows * 16);
  // Rows 1 and 6 hold it, placed 1 and 6 m along x.
  for (const i of [1, 6]) matrices.set([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, i, 0, 0, 1], i * 16);
  const prop = { draws: [fake], matrices } as unknown as Prop;
  const show = (count: number) => ResidentProps.prototype.show.call(null as never, prop, count);

  show(rows);
  expect([draw.enabled, draw.count, draw.xs.slice(0, draw.count)]).toEqual([true, 2, [1, 6]]);
  // (Row 6 past the rows counted: one copy.)
  show(4);
  expect([draw.enabled, draw.count]).toEqual([true, 1]);
  // Nobody holds it: not drawn, nothing uploaded.
  matrices.fill(0);
  const uploads = draw.uploads;
  show(rows);
  expect([draw.enabled, draw.uploads]).toEqual([false, uploads]);
});
