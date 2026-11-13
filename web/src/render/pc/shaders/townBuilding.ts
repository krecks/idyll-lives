/**
 * The town view's buildings (`townView.ts`): every
 * backdrop building and every far lot's house as copies of a few unit meshes (a box of walls, and a
 * gable, hip, flat or mansard roof), one instanced draw each, in one material (`GAME_TOWN_BUILDING`).
 *
 * Each copy carries its look beside its matrix (`TOWN_WALL`, `TOWN_ROOF`, `TOWN_INFO`):
 *
 * - its wall and roof colours (sRGB); a mesh's vertex colour picks between them (red: 1 the wall's, 0
 *   the roof's: a gable's ends are wall), its green a tone over it (a hip's ends a touch darker);
 * - how far its walls go below its floor (its plinth, m: walls start under the lowest corner's ground);
 * - its windows (`TOWN_STYLE`, `shared/townBuildings.ts`), its height to the eaves (m), a number of its own (which windows are
 *   lit at night) and the share of them lit.
 *
 * The windows are not geometry: the fragment shader draws them over the walls, in bands a storey
 * (3 m) apart and in bays along each face, from the face's own metres (the copy's scale undone), so
 * a 6 m terrace and a 30 m block of flats get windows of one size. Where a bay would be under a pixel
 * or two the pattern fades to its average (no shimmer from far off). A shop's ground floor has a shopfront on its
 * long faces: wide, paler glass under a fascia in the shop's own colour. By day the glass is dark; by
 * night a share of the windows glows warm (`game_townGlow`, the street's glow by the lamps' level).
 * The whole draw fades in by a dither (`game_townFade`) as the town view comes in, so nothing pops.
 */

import { float, registerShaderFeature, type Language, type ShaderFeature } from './chunks';
import * as pc from '../engine';

/** Per copy: the wall's colour (sRGB) and its plinth, the metres of wall under its floor. */
export const TOWN_WALL_SEMANTIC = pc.SEMANTIC_ATTR10;
/** Per copy: the roof's colour (sRGB) and its window style (`TOWN_STYLE`). */
export const TOWN_ROOF_SEMANTIC = pc.SEMANTIC_ATTR8;
/** Per copy: its number (0..1), its height from floor to eaves (m), its shopfront's number (0..1: which fascia), the share of its windows lit at night. */
export const TOWN_INFO_SEMANTIC = pc.SEMANTIC_ATTR9;

export { TOWN_STYLE } from '../../shared/townBuildings';

/** The glass by day (linear), and how much darker a window's frame line is. */
const GLASS = [0.045, 0.052, 0.062] as const;
/** A storey (m), and a window's sill and head within it (m over its floor); a shopfront's. */
const STOREY = 3;
const SILL = 0.95;
const HEAD = 2.15;
const SHOP = [0.35, 2.45] as const;
/** Bays (m) along a face: a house's, and flats' and offices'; the window's share of its bay. */
const BAY = [2.7, 2.3] as const;
const PANE = 0.4;
/** A shopfront: its glass's share of its bay, its glass by day (linear; paler than a window's: a lit shop behind it), and the fascias over it (linear: burgundy, bottle green, navy, ochre), one by the building's own number. */
const SHOP_PANE = 0.8;
const SHOP_GLASS = [0.1, 0.135, 0.16] as const;
const FASCIA = [
  [0.2, 0.03, 0.035],
  [0.03, 0.11, 0.055],
  [0.03, 0.055, 0.15],
  [0.42, 0.26, 0.06],
] as const;

function code(lang: Language): ShaderFeature['glsl'] {
  const wgsl = lang === 'wgsl';
  const v2 = wgsl ? 'vec2f' : 'vec2';
  const v3 = wgsl ? 'vec3f' : 'vec3';
  const v4 = wgsl ? 'vec4f' : 'vec4';
  const f = float;
  const u = (n: string) => (wgsl ? `uniform.${n}` : n);
  const out = (n: string) => (wgsl ? `output.${n}` : n);
  const let_ = (n: string, t: string, mut = false) => (wgsl ? `${mut ? 'var' : 'let'} ${n}` : `${t} ${n}`);

  const vsDecl = wgsl
    ? `
#ifdef GAME_TOWN_BUILDING
	attribute game_townWall: vec4f;
	attribute game_townRoof: vec4f;
	attribute game_townInfo: vec4f;
	uniform game_townPart: f32;
	varying vTownCol: vec4f;
	varying vTownWin: vec4f;
	varying vTownSeed: vec3f;
#endif`
    : `
#ifdef GAME_TOWN_BUILDING
	attribute vec4 game_townWall;
	attribute vec4 game_townRoof;
	attribute vec4 game_townInfo;
	uniform float game_townPart;
	varying vec4 vTownCol;
	varying vec4 vTownWin;
	varying vec3 vTownSeed;
#endif`;

  // The colour picked and toned (the vertex colour linear already: PlayCanvas converts it), and the
  // face's own metres: along it (x or z by its normal) and up from the floor.
  const vsEnd = `
#ifdef GAME_TOWN_BUILDING
	{
		${let_('tvc', v4)} = ${out('vVertexColor')};
		${let_('tWall', v3)} = pow(max(game_townWall.rgb, ${v3}(0.0)), ${v3}(2.2));
		${let_('tRoof', v3)} = pow(max(game_townRoof.rgb, ${v3}(0.0)), ${v3}(2.2));
		${let_('tSel', 'float')} = step(0.5, tvc.r);
		${let_('tSc', v3, true)} = ${v3}(1.0);
		#ifdef INSTANCING
			tSc = ${v3}(length(instance_line1.xyz), length(instance_line2.xyz), length(instance_line3.xyz));
		#endif
		${let_('tLp', v3)} = vertex_position.xyz;
		${let_('tAlong', 'float')} = ${wgsl ? 'select(tLp.x * tSc.x, tLp.z * tSc.z, abs(vertex_normal.x) > 0.5)' : '(abs(vertex_normal.x) > 0.5 ? tLp.z * tSc.z : tLp.x * tSc.x)'};
		// (Windows only on the walls' draw, on its sides: not its gables, nor any roof. 1 on its long faces, 0.6 on its ends: a shopfront is on the long ones.)
		${let_('tWin', 'float')} = tSel * (1.0 - ${u('game_townPart')}) * (1.0 - step(0.5, abs(vertex_normal.y))) * (0.6 + 0.4 * step(0.5, abs(vertex_normal.z)));
		${out('vTownCol')} = ${v4}(mix(tRoof, tWall, tSel) * tvc.g, tWin);
		${out('vTownWin')} = ${v4}(tAlong, tLp.y * tSc.y - game_townWall.a, game_townInfo.y, game_townRoof.a);
		${out('vTownSeed')} = ${v3}(game_townInfo.x, game_townInfo.w, game_townInfo.z);
	}
#endif`;

  const psDecl = wgsl
    ? `
#ifdef GAME_TOWN_BUILDING
	varying vTownCol: vec4f;
	varying vTownWin: vec4f;
	varying vTownSeed: vec3f;
	uniform game_townFade: f32;
	uniform game_townGlow: vec3f;
	var<private> gameTownLit: f32 = 0.0;
	fn gameTownHash(p: vec3f) -> f32 {
		return fract(sin(dot(p, vec3f(12.9898, 78.233, 37.719))) * 43758.5453);
	}
	fn gameTownPulse(x: f32, lo: f32, hi: f32, w: f32) -> f32 {
		let p = smoothstep(lo - w, lo + w, x) - smoothstep(hi - w, hi + w, x);
		return mix(p, hi - lo, smoothstep(0.12, 0.45, w));
	}
#endif`
    : `
#ifdef GAME_TOWN_BUILDING
	varying vec4 vTownCol;
	varying vec4 vTownWin;
	varying vec3 vTownSeed;
	uniform float game_townFade;
	uniform vec3 game_townGlow;
	float gameTownLit = 0.0;
	float gameTownHash(vec3 p) {
		return fract(sin(dot(p, vec3(12.9898, 78.233, 37.719))) * 43758.5453);
	}
	float gameTownPulse(float x, float lo, float hi, float w) {
		float p = smoothstep(lo - w, lo + w, x) - smoothstep(hi - w, hi + w, x);
		return mix(p, hi - lo, smoothstep(0.12, 0.45, w));
	}
#endif`;

  const frag = wgsl ? 'pcPosition' : 'gl_FragCoord';
  const fw = wgsl ? 'fwidth' : 'fwidth';
  const albedo = `
#ifdef GAME_TOWN_BUILDING
	{
		// (Derivatives first, outside any branch.)
		${let_('tw', v4)} = vTownWin;
		${let_('tStyle', 'float')} = tw.w;
		${let_('tBay', 'float')} = ${wgsl ? `select(${f(BAY[0])}, ${f(BAY[1])}, tStyle > 2.5)` : `(tStyle > 2.5 ? ${f(BAY[1])} : ${f(BAY[0])})`};
		${let_('tU', 'float')} = tw.x / tBay + 0.5;
		${let_('tWu', 'float')} = ${fw}(tU);
		${let_('tWy', 'float')} = ${fw}(tw.y) / ${f(STOREY)};
		// Fading in: a dither over the picture, whole by the time the town view is.
		${let_('tFp', v2)} = floor(${frag}.xy);
		if (fract(52.9829189 * fract(dot(tFp, ${v2}(0.06711056, 0.00583715)))) >= ${u('game_townFade')}) { discard; }
		${let_('tStorey', 'float')} = floor(tw.y / ${f(STOREY)});
		${let_('tFy', 'float')} = tw.y / ${f(STOREY)} - tStorey;
		${let_('tFx', 'float')} = fract(tU);
		${let_('tShopStyle', 'float')} = step(1.5, tStyle) * step(tStyle, 2.5);
		${let_('tShop', 'float')} = tShopStyle * (1.0 - step(0.5, tStorey)) * step(0.8, vTownCol.a);
		${let_('tLo', 'float')} = mix(${f(SILL / STOREY)}, ${f(SHOP[0] / STOREY)}, tShop);
		${let_('tHi', 'float')} = mix(${f(HEAD / STOREY)}, ${f(SHOP[1] / STOREY)}, tShop);
		${let_('tPane', 'float')} = mix(${f(PANE)}, ${f(SHOP_PANE)}, tShop);
		${let_('tWin', 'float', true)} = gameTownPulse(tFx, 0.5 - tPane * 0.5, 0.5 + tPane * 0.5, tWu) * gameTownPulse(tFy, tLo, tHi, tWy);
		// (Walls only, from the floor to under the eaves, and none on a barn.)
		${let_('tOn', 'float')} = step(0.3, vTownCol.a) * step(0.0, tw.y) * step(tw.y, tw.z - 0.35);
		tWin = tWin * tOn * step(0.5, tStyle);
		// (A shopfront's glass is paler than a window's, and a fascia in the shop's own colour runs over it.)
		dAlbedo = mix(vTownCol.rgb, mix(${v3}(${GLASS.map(f).join(', ')}), ${v3}(${SHOP_GLASS.map(f).join(', ')}), tShop), tWin);
		${let_('tAcc', 'float')} = vTownSeed.z;
		${let_('tFascia', v3)} = mix(mix(${v3}(${FASCIA[0].map(f).join(', ')}), ${v3}(${FASCIA[1].map(f).join(', ')}), step(0.25, tAcc)), mix(${v3}(${FASCIA[2].map(f).join(', ')}), ${v3}(${FASCIA[3].map(f).join(', ')}), step(0.75, tAcc)), step(0.5, tAcc));
		dAlbedo = mix(dAlbedo, tFascia, tShop * tOn * gameTownPulse(tFy, ${f(SHOP[1] / STOREY + 0.02)}, ${f(2.95 / STOREY)}, tWy));
		// Lit at night: a share of the windows, by building, storey and bay (a shopfront's all of a piece; the windows over a shop half as often).
		${let_('tBayN', 'float')} = mix(floor(tU), 0.0, tShop);
		gameTownLit = tWin * step(gameTownHash(${v3}(vTownSeed.x * 97.0, tStorey, tBayN)), vTownSeed.y * mix(1.0, mix(0.45, 1.0, tShop), tShopStyle));
	}
#endif`;

  const emissive = `
#ifdef GAME_TOWN_BUILDING
	dEmission = dEmission + ${u('game_townGlow')} * gameTownLit;
#endif`;

  return { vsDecl, vsEnd, psDecl, albedo, emissive };
}

/** The town view's buildings (see the file's header), before the soft look (300) and after the grass (235). */
export const TOWN_BUILDING: ShaderFeature = { name: 'townBuilding', priority: 236, glsl: code('glsl'), wgsl: code('wgsl') };

registerShaderFeature(TOWN_BUILDING);

/** Makes `material` (a copy of the plain finish) draw the town's buildings: its define, its streams' attributes and its uniforms' first values. */
export function townBuildingMaterial(material: pc.StandardMaterial): void {
  material.setDefine('GAME_TOWN_BUILDING', '');
  material.setAttribute('game_townWall', TOWN_WALL_SEMANTIC);
  material.setAttribute('game_townRoof', TOWN_ROOF_SEMANTIC);
  material.setAttribute('game_townInfo', TOWN_INFO_SEMANTIC);
  material.setParameter('game_townPart', 0);
  material.setParameter('game_townFade', 1);
  material.setParameter('game_townGlow', new Float32Array(3));
  material.update();
}
