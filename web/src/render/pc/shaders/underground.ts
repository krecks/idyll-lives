/**
 * Daylight kept out of a basement in view and out of the rooms without windows on the storey in
 * view: there the sun, the sky's fill and the environment are scaled down to `game_undergroundLit`
 * (x, y, z), so only the lamps light them, day and night.
 *
 * `gameUnder` is worked out once a fragment (`litUserMainStartPS`); the sun reads it where its
 * colour is read (an engine patch after `lightColor = light{i}_color`, `chunks.ts`), the sky's
 * fill in `gameHemi` and the environment (ambient and reflections) in `combineColor`
 * (`softLook.ts`).
 *
 * - The basement: inside `game_undergroundRect` (x0, z0, x1, z1) while `game_undergroundLit.w`.
 * - Windowless rooms: `game_windowlessMask` has a byte per tile of `game_windowlessArea` (x0, z0 in
 *   metres, then w, d in tiles; `WINDOWLESS_BITS`) in its corner, read only between the heights
 *   of `game_windowlessBand` (y0, y1). A tile split by a diagonal wall has its halves' bits; in
 *   tile-local u = x - x0, v = z - z0 a 'dp' wall's half 0 is v < u, a 'dn' wall's u + v < 1.
 */

import type { ShaderFeature } from './chunks';

/** The bits of a windowless mask's tile (`DigState.darken`), as `materials.ts` exports them. */
const B = { tile: 1, half0: 2, half1: 4, dn: 8 } as const;

export const UNDERGROUND: ShaderFeature = {
  name: 'underground',
  priority: 212,
  glsl: {
    psDecl: `
#ifdef GAME_UNDERGROUND
	uniform vec4 game_undergroundRect;
	uniform vec4 game_undergroundLit;
	uniform vec4 game_windowlessBand;
	uniform vec4 game_windowlessArea;
	uniform highp sampler2D game_windowlessMask;
	vec3 gameUnder = vec3(1.0);
	vec3 gameUnderground(vec3 p) {
		vec3 under = vec3(1.0);
		if (game_undergroundLit.w > 0.5 && p.x > game_undergroundRect.x && p.x < game_undergroundRect.z && p.z > game_undergroundRect.y && p.z < game_undergroundRect.w) { under = game_undergroundLit.xyz; }
		// (Only at the storey's heights: the rest of the scene doesn't read the mask.)
		if (p.y > game_windowlessBand.x && p.y < game_windowlessBand.y) {
			vec2 wlT = p.xz - game_windowlessArea.xy;
			if (wlT.x >= 0.0 && wlT.y >= 0.0 && wlT.x < game_windowlessArea.z && wlT.y < game_windowlessArea.w) {
				vec2 wlC = floor(wlT);
				int wlM = int(texelFetch(game_windowlessMask, ivec2(wlC), 0).r * 255.0 + 0.5);
				vec2 wlF = wlT - wlC;
				bool wlDark = (wlM & ${B.tile}) != 0;
				if (!wlDark && wlM > 1) {
					bool wlOne = (wlM & ${B.dn}) != 0 ? wlF.x + wlF.y > 1.0 : wlF.y > wlF.x;
					wlDark = (wlM & (wlOne ? ${B.half1} : ${B.half0})) != 0;
				}
				if (wlDark) { under = game_undergroundLit.xyz; }
			}
		}
		return under;
	}
#endif`,
    psStart: `
#if defined(GAME_UNDERGROUND) && !defined(SHADOW_PASS)
	gameUnder = gameUnderground(gamePos(vPositionW));
#endif`,
  },
  wgsl: {
    psDecl: `
#ifdef GAME_UNDERGROUND
	uniform game_undergroundRect: vec4f;
	uniform game_undergroundLit: vec4f;
	uniform game_windowlessBand: vec4f;
	uniform game_windowlessArea: vec4f;
	var game_windowlessMask: texture_2d<f32>;
	var<private> gameUnder: vec3f = vec3f(1.0);
	fn gameUnderground(p: vec3f) -> vec3f {
		var under = vec3f(1.0);
		if (uniform.game_undergroundLit.w > 0.5 && p.x > uniform.game_undergroundRect.x && p.x < uniform.game_undergroundRect.z && p.z > uniform.game_undergroundRect.y && p.z < uniform.game_undergroundRect.w) { under = uniform.game_undergroundLit.xyz; }
		if (p.y > uniform.game_windowlessBand.x && p.y < uniform.game_windowlessBand.y) {
			let wlT = p.xz - uniform.game_windowlessArea.xy;
			if (wlT.x >= 0.0 && wlT.y >= 0.0 && wlT.x < uniform.game_windowlessArea.z && wlT.y < uniform.game_windowlessArea.w) {
				let wlC = floor(wlT);
				let wlM = u32(textureLoad(game_windowlessMask, vec2<i32>(wlC), 0).r * 255.0 + 0.5);
				let wlF = wlT - wlC;
				var wlDark = (wlM & ${B.tile}u) != 0u;
				if (!wlDark && wlM > 1u) {
					let wlOne = select(wlF.y > wlF.x, wlF.x + wlF.y > 1.0, (wlM & ${B.dn}u) != 0u);
					wlDark = (wlM & select(${B.half0}u, ${B.half1}u, wlOne)) != 0u;
				}
				if (wlDark) { under = uniform.game_undergroundLit.xyz; }
			}
		}
		return under;
	}
#endif`,
    psStart: `
#if defined(GAME_UNDERGROUND) && !defined(SHADOW_PASS)
	gameUnder = gameUnderground(gamePos(vPositionW));
#endif`,
  },
};
