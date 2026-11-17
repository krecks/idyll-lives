/**
 * The menus' land's haze at its edge (`render/shared/landVeil.ts`, `render/pc/landHaze.ts`): the veil clear at the
 * rectangle `VEIL_FADE` m in from the landscape's edge and whole at the edge and past it, by distance from that
 * rectangle alone (round at its corners), as the old veil's rings had it, and clear near the eye; and both shader
 * languages' haze taking the veil in.
 */
import { describe, expect, test } from 'bun:test';
import { VEIL_FADE, VEIL_NEAR, veilAt, veilRect } from '../src/render/shared/landVeil';
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
