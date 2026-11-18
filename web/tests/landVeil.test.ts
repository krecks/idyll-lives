/**
 * The menus' land's haze at its edge (`render/shared/landVeil.ts`, `render/pc/landHaze.ts`): the veil clear at the
 * rectangle `VEIL_FADE` m in from the landscape's edge and whole at the edge and past it, by distance from that
 * rectangle alone (round at its corners), as the old veil's rings had it, and clear near the eye; the curtain closing
 * the landscape's cut edge, over the ground there (the ground tiles' coarser levels too) and under it and the sea's
 * sheet; and both shader languages' haze taking the veil in.
 */
import { describe, expect, test } from 'bun:test';
import { curtain, VEIL_FADE, VEIL_NEAR, veilAt, veilRect } from '../src/render/shared/landVeil';
import { gameFog } from '../src/render/pc/shaders/fog';

/** A medium land's landscape: 832 m and its 260 m ring each way. */
const BOX = { x0: -260, z0: -260, x1: 1092, z1: 1092 };

describe('the veil', () => {
  const rect = veilRect(BOX);

  test('clear over the land, whole at the landscape edge and past it', () => {
    expect(rect.fade).toBe(VEIL_FADE);
    // (The land itself: 0..832, clear right up to 20 m past its edge.)
    for (const [x, z] of [
      [416, 416],
      [0, 0],
      [832, 400],
      [-20, 500],
    ])
      expect(veilAt(rect, x, z)).toBe(0);
    for (const [x, z] of [
      [1092, 400],
      [-260, -260],
      [5000, 400],
      [400, -3000],
    ])
      expect(veilAt(rect, x, z)).toBe(1);
  });

  test('as the old rings had it across the fade (30 % at four tenths of the way, 80 % at three quarters)', () => {
    const at = (share: number) => veilAt(rect, 416, BOX.z0 + VEIL_FADE * (1 - share));
    expect(Math.abs(at(0.4) - 0.3)).toBeLessThan(0.06);
    expect(Math.abs(at(0.75) - 0.8)).toBeLessThan(0.06);
    // (Thicker all the way out.)
    for (let s = 0.05; s <= 1; s += 0.05) expect(at(s)).toBeGreaterThan(at(s - 0.05));
  });

  test('round at the corners: as thick as along a side at the same distance from where it begins', () => {
    const [cx, cz] = [BOX.x0 + VEIL_FADE, BOX.z0 + VEIL_FADE];
    for (const d of [30, 100, 180]) {
      const side = veilAt(rect, 500, cz - d);
      const corner = veilAt(rect, cx - d / Math.SQRT2, cz - d / Math.SQRT2);
      expect(corner).toBeCloseTo(side, 6);
    }
  });

  test('clear at the eye, whole from VEIL_NEAR off: a camera out in it sees what is round it', () => {
    expect(veilAt(rect, 1092, 400, 0)).toBe(0);
    expect(veilAt(rect, 1092, 400, VEIL_NEAR / 2)).toBeCloseTo(0.5, 6);
    expect(veilAt(rect, 1092, 400, VEIL_NEAR)).toBe(1);
    expect(veilAt(rect, 416, 416, 10)).toBe(0);
  });

  test('a landscape too small for the fade is whole at its edge all the same', () => {
    const small = veilRect({ x0: 0, z0: 0, x1: 300, z1: 200 });
    expect(small.fade).toBe(100);
    expect(veilAt(small, 150, 100)).toBe(0);
    expect(veilAt(small, 0, 100)).toBe(1);
  });
});

describe('the curtain', () => {
  // Rolling ground rising to a rim at the edge, with a notch (a river's way out) on one side.
  const ground = (x: number, z: number) => 60 + 40 * Math.sin(x * 0.013) * Math.cos(z * 0.021) - (Math.abs(z - 420) < 12 && x > 1000 ? 64 : 0);
  const { positions, indices } = curtain(BOX, ground, -0.35);
  const n = positions.length / 6;

  test('a closed band just past the edge, all the way round', () => {
    expect(indices.length).toBe(n * 6);
    expect(Math.max(...indices)).toBe(2 * n - 1);
    for (let i = 0; i < 2 * n; i++) {
      const [x, z] = [positions[i * 3], positions[i * 3 + 2]];
      const out = Math.max(BOX.x0 - x, x - BOX.x1, BOX.z0 - z, z - BOX.z1);
      expect(out).toBeCloseTo(0.5, 6);
    }
  });

  test('its top over the ground along the edge, the ground tiles coarsest level (straight every 8 m) too', () => {
    let lowest = Infinity;
    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      const [ax, at, az] = [positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]];
      const [bx, bt, bz] = [positions[j * 3], positions[j * 3 + 1], positions[j * 3 + 2]];
      // (Each metre between two samples, on the edge itself.)
      const len = Math.hypot(bx - ax, bz - az);
      for (let s = 0; s <= len; s += 1) {
        const t = len > 0 ? s / len : 0;
        const [x, z] = [Math.min(BOX.x1, Math.max(BOX.x0, ax + (bx - ax) * t)), Math.min(BOX.z1, Math.max(BOX.z0, az + (bz - az) * t))];
        const top = at + (bt - at) * t;
        // The ground as the coarsest level draws it: straight between its points every 8 m from the box's corner.
        const along = Math.abs(z - BOX.z0) < 1e-6 || Math.abs(z - BOX.z1) < 1e-6 ? x - BOX.x0 : z - BOX.z0;
        const [p0, p1] = [Math.floor(along / 8) * 8, Math.floor(along / 8) * 8 + 8];
        const point = (a: number) => (Math.abs(z - BOX.z0) < 1e-6 || Math.abs(z - BOX.z1) < 1e-6 ? ground(BOX.x0 + a, z) : ground(x, BOX.z0 + a));
        const coarse = point(p0) + ((point(p1) - point(p0)) * (along - p0)) / 8;
        expect(top).toBeGreaterThan(Math.max(ground(x, z), coarse) - 1e-6);
        lowest = Math.min(lowest, ground(x, z));
      }
    }
    // Its foot well under the lowest ground at the edge and under the sea's sheet.
    for (let i = n; i < 2 * n; i++) expect(positions[i * 3 + 1]).toBeLessThan(Math.min(lowest, -0.35) - 10);
  });
});

describe("the haze's shaders", () => {
  test('both languages take the veil in, in the fragment shader alone', () => {
    const engine = {
      glsl: '#ifdef VERTEXSHADER\n#else\nfloat depth = gl_FragCoord.z / gl_FragCoord.w;\n#endif\nfogFactor = exp(-depth * depth * fog_density * fog_density);\n',
      wgsl: '#ifdef VERTEXSHADER\n#else\nlet depth = pcPosition.z / pcPosition.w;\n#endif\nreturn evaluateFogFactorExp2(depth, uniform.fog_density);\n',
    };
    const once = (code: string, _name: string, anchor: string, replacement: string) => {
      expect(code.split(anchor).length).toBe(2);
      return code.replace(anchor, replacement);
    };
    const glsl = gameFog(engine.glsl, 'glsl', once);
    expect(glsl).toContain('uniform vec4 game_landVeil;');
    expect(glsl).toContain('uniform vec4 game_veilFade;');
    expect(glsl).toContain('smoothstep(0.0, game_veilFade.z, distance(vPositionW, view_position))');
    expect(glsl).toMatch(/#ifndef VERTEXSHADER\s+fogFactor \*= 1\.0 - game_veilFade\.y \* smoothstep\(0\.0, game_veilFade\.x/);
    const wgsl = gameFog(engine.wgsl, 'wgsl', once);
    expect(wgsl).toContain('uniform game_landVeil: vec4f;');
    expect(wgsl).toContain('uniform game_veilFade: vec4f;');
    expect(wgsl).toContain('smoothstep(0.0, uniform.game_veilFade.z, distance(vPositionW, uniform.view_position))');
    expect(wgsl).toMatch(/#ifdef VERTEXSHADER[\s\S]*#else\s+let veilOut = length\(max\(abs\(vPositionW\.xz - uniform\.game_landVeil\.xy\)/);
  });
});
