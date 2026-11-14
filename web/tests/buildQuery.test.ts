/**
 * The query a built game asks for its data with (`assets/build.ts`): a file the build hashed by its
 * content hash (the same URL from release to release while it doesn't change), anything else by the
 * build's id; and the keys that outlive a build without either.
 */
import { afterAll, beforeAll, describe, expect, test } from 'bun:test';

type Build = typeof import('../src/assets/build');
let build: Build;
const realLocation = globalThis.location;
const site = 'https://example.org/game/';

beforeAll(async () => {
  // As Vite defines it in a build (the module reads it once, when first imported).
  Object.assign(globalThis, { __BUILD_ID__: 'abc1234', location: { href: `${site}index.html` } });
  build = await import('../src/assets/build');
});
afterAll(() => {
  Object.assign(globalThis, { location: realLocation });
  delete (globalThis as { __BUILD_ID__?: string }).__BUILD_ID__;
});

describe('data URLs', () => {
  test("by the build's id until the bundle's hashes are known, and for files without one", () => {
    expect(build.withBuild(`${site}assets/models/chair.glb`)).toBe(`${site}assets/models/chair.glb?v=abc1234`);
    build.useHashes({ 'assets/opt/models/chair.glb': '0123456789ab', 'content/talk/en.json': 'ba9876543210', 'assets/a%20b.png': 'aaaaaaaaaaaa' }, new URL(site));
    expect(build.withBuild(`${site}assets/models/sofa.glb`)).toBe(`${site}assets/models/sofa.glb?v=abc1234`);
    expect(build.withBuild(`${site}bundle.json`)).toBe(`${site}bundle.json?v=abc1234`);
  });

  test('by content hash where the build has one, relative or absolute, once', () => {
    expect(build.withBuild(`${site}assets/opt/models/chair.glb`)).toBe(`${site}assets/opt/models/chair.glb?h=0123456789ab`);
    expect(build.withBuild('/game/content/talk/en.json')).toBe('/game/content/talk/en.json?h=ba9876543210');
    expect(build.withBuild(`${site}assets/a b.png`)).toBe(`${site}assets/a b.png?h=aaaaaaaaaaaa`);
    const hashed = build.withBuild(`${site}assets/opt/models/chair.glb`);
    expect(build.withBuild(hashed)).toBe(hashed);
    // Another site's file, or one with a query of its own (a pack's), isn't the build's.
    expect(build.withBuild('https://other.example/game/assets/opt/models/chair.glb')).toBe('https://other.example/game/assets/opt/models/chair.glb?v=abc1234');
    expect(build.withBuild(`${site}assets/opt/models/chair.glb?pack=1`)).toBe(`${site}assets/opt/models/chair.glb?pack=1&v=abc1234`);
    expect(build.withBuild('data:,x')).toBe('data:,x');
  });

  test('keys that outlive a build carry neither query', () => {
    const entry = JSON.stringify({ url: `${site}assets/opt/models/chair.glb?h=0123456789ab`, far: `${site}assets/models/far.glb?v=abc1234` });
    expect(build.withoutBuild(entry)).toBe(JSON.stringify({ url: `${site}assets/opt/models/chair.glb`, far: `${site}assets/models/far.glb` }));
    expect(build.withoutBuild(`${site}x.glb?pack=1&h=0123456789ab`)).toBe(`${site}x.glb?pack=1`);
  });
});
