/**
 * The finishes (`material.finish.*` in the base manifest, written by `cargo tex materials`) as the game draws them:
 * - a plain one's roughness and metalness survive the vertex colour's alpha (`encodeFinish`, read back as
 *   `PLAIN_FINISH` reads it), in a street mesh's 8 bits as in a model's floats, so the plain finishes added
 *   since (enamel, ceramic, plastic, rubber, brushed, iron) cost no draw and no material;
 * - cloth's sheen and a lacquer's clear coat (`MaterialEntry.sheenColor`, `clearcoat`) are layers, drawn where
 *   the graphics preset has `finishLayers` and left out on Low, where those finishes are the surface under them;
 * - the game's chunks carry the layers' reflections in both shader languages.
 */
import { afterAll, describe, expect, mock, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import type { AssetRegistry } from '../src/assets/registry';
import type { Manifest, MaterialEntry } from '../src/assets/types';
import * as pc from '../src/render/pc/engine';
import { QUALITY } from '../src/render/quality';

const manifest = JSON.parse(readFileSync(new URL('../public/assets/manifest.json', import.meta.url), 'utf8')) as Manifest;
const finishes = Object.entries(manifest.entries).filter((e): e is [string, MaterialEntry] => e[0].startsWith('material.finish.') && e[1].type === 'material');
const finish = (name: string): MaterialEntry => {
  const entry = manifest.entries[`material.finish.${name}`];
  if (entry?.type !== 'material') throw new Error(`no finish ${name}`);
  return entry;
};

// (The page's flags and the player's settings, which the materials read: a page without flags, Medium.)
const realLocation = globalThis.location;
Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
afterAll(() => Object.assign(globalThis, { location: realLocation }));
mock.module('../src/settings/settings.svelte', () => ({ settings: { shading: 'soft', graphics: 'medium' } }));
const { encodeFinish, MaterialLibrary } = await import('../src/render/pc/materials');
const { gameChunks } = await import('../src/render/pc/shaders/chunks');
const { PLAIN_FINISH } = await import('../src/render/pc/shaders/surface');

/** The plain finish's alpha as `PLAIN_FINISH` reads it back. */
function decode(alpha: number): { roughness: number; metal: boolean } {
  const metal = alpha >= 0.5;
  return { roughness: Math.min(1, Math.max(0.03, (alpha - (metal ? 0.5 : 0)) * 2)), metal };
}

/** An app on the null device (PlayCanvas's own chunks are an app's device's). */
function app(): pc.AppBase {
  const canvas = { width: 16, height: 16, style: {}, addEventListener() {}, removeEventListener() {}, getBoundingClientRect: () => ({ width: 16, height: 16 }) } as unknown as HTMLCanvasElement;
  const made = new pc.AppBase(canvas);
  const options = new pc.AppOptions();
  options.graphicsDevice = new pc.NullGraphicsDevice(canvas);
  options.componentSystems = [];
  options.resourceHandlers = [];
  made.init(options);
  return made;
}

/** A library with the base manifest's materials (their textures are never asked for). */
function library(layers: boolean) {
  // (As the registry has them: a texture's URL whole.)
  const entry = (key: string) => {
    const e = manifest.entries[key];
    return e?.type === 'material' && e.texture ? { ...e, texture: `http://localhost/assets/${e.texture}` } : e;
  };
  const assets = { get: entry, has: (key: string) => key in manifest.entries } as unknown as AssetRegistry;
  const made = app();
  Object.assign(made.assets, { add() {}, load() {} });
  const lib = MaterialLibrary.for(made).setAssets(assets);
  lib.finishLayers = layers;
  return lib;
}

describe('the plain finishes in the vertex colour', () => {
  const plain = finishes.filter(([, e]) => !e.texture && (e.alpha ?? 1) >= 1);

  test('the manifest has the plain finishes a thing can be made of', () => {
    const names = plain.map(([key]) => key.replace('material.finish.', ''));
    for (const name of ['matte', 'satin', 'gloss', 'metal', 'chrome', 'enamel', 'ceramic', 'plastic', 'rubber', 'brushed', 'iron']) expect(names).toContain(name);
  });

  test('the shader reads the alpha as it is decoded here', () => {
    for (const code of [PLAIN_FINISH.glsl, PLAIN_FINISH.wgsl]) {
      expect(code.metalness).toContain('dMetalness = step(0.5, gameFinish());');
      expect(code.gloss).toContain('clamp((finish - 0.5 * step(0.5, finish)) * 2.0, 0.03, 1.0)');
    }
  });

  test("each one's roughness and metalness come back, from floats and from 8 bits", () => {
    for (const [key, e] of plain) {
      const [roughness, metallic] = [e.roughness ?? 0.8, e.metallic ?? 0];
      const alpha = encodeFinish(roughness, metallic);
      const exact = decode(Math.fround(alpha));
      expect(exact.metal, key).toBe(metallic >= 0.5);
      expect(Math.abs(exact.roughness - roughness), key).toBeLessThan(1e-6);
      // (A street mesh's colours are bytes: a step of 1/255 of the alpha is 0.008 of roughness.)
      const byte = decode(Math.round(alpha * 255) / 255);
      expect(byte.metal, key).toBe(metallic >= 0.5);
      expect(Math.abs(byte.roughness - roughness), key).toBeLessThan(0.005);
    }
  });

  test('a metal never reads as a dielectric in 8 bits, however polished', () => {
    for (let roughness = 0; roughness <= 1; roughness += 0.01) {
      expect(decode(Math.round(encodeFinish(roughness, 1) * 255) / 255).metal).toBe(true);
      expect(decode(Math.round(encodeFinish(roughness, 0) * 255) / 255).metal).toBe(false);
    }
  });
});

describe("the finishes' layers", () => {
  test('the manifest: sheen on cloth, a clear coat on lacquer, the same maps as the finish under them', () => {
    const layered = finishes.filter(([, e]) => e.sheenColor !== undefined || e.clearcoat !== undefined).map(([key]) => key.replace('material.finish.', ''));
    expect(layered.sort()).toEqual(['fabric', 'lacquer', 'velvet']);
    // No download for velvet or lacquer: the fabric's and the wood's files.
    for (const [layer, under] of [['velvet', 'fabric'], ['lacquer', 'wood']] as const) {
      for (const map of ['texture', 'normal', 'arm', 'uvScale', 'roughness', 'metallic'] as const) expect(finish(layer)[map], `${layer} ${map}`).toBe(finish(under)[map]!);
    }
    expect(finish('lacquer').clearcoat).toBe(1);
    // (Velvet's sheen is the brighter: grey levels, compared as written.)
    expect(finish('velvet').sheenColor! > finish('fabric').sheenColor!).toBe(true);
  });

  test('off on Low, on from Medium', () => {
    expect([QUALITY.low.finishLayers, QUALITY.medium.finishLayers, QUALITY.high.finishLayers]).toEqual([false, true, true]);
  });

  test('where the preset has them, cloth has its sheen and lacquer its coat', () => {
    const lib = library(true);
    const fabric = lib.finish('material.finish.fabric');
    expect(fabric.useSheen).toBe(true);
    expect(fabric.sheenGloss).toBe(finish('fabric').sheenRoughness!);
    expect(fabric.getDefine('GAME_FINISH_LAYERS')).toBe(true);
    const velvet = lib.finish('material.finish.velvet');
    expect(velvet.useSheen).toBe(true);
    expect(velvet.sheen.r).toBeGreaterThan(fabric.sheen.r);
    const lacquer = lib.finish('material.finish.lacquer');
    expect(lacquer.clearCoat).toBe(1);
    expect(lacquer.clearCoatGlossInvert).toBe(true);
    expect(lacquer.clearCoatGloss).toBe(finish('lacquer').clearcoatRoughness!);
    expect(lacquer.useSheen).toBe(false);
    // The finishes without a layer are as they were: no lobe, no define, and so the same shader.
    for (const key of ['material.finish.wood', 'material.finish.leather', 'material.finish.matte']) {
      const mat = lib.finish(key);
      expect([mat.useSheen, mat.clearCoat, mat.getDefine('GAME_FINISH_LAYERS')], key).toEqual([false, 0, false]);
    }
    expect(lib.plainFinish().getDefine('GAME_FINISH_LAYERS')).toBe(false);
  });

  test('on Low the layered finishes are the surface under them', () => {
    const lib = library(false);
    for (const key of ['material.finish.fabric', 'material.finish.velvet', 'material.finish.lacquer']) {
      const mat = lib.finish(key);
      expect([mat.useSheen, mat.clearCoat, mat.getDefine('GAME_FINISH_LAYERS')], key).toEqual([false, 0, false]);
    }
    // Lacquer is then the wood, to the number.
    const [lacquer, wood] = [lib.finish('material.finish.lacquer'), lib.finish('material.finish.wood')];
    expect([lacquer.gloss, lacquer.metalness, lacquer.diffuseMapTiling.x]).toEqual([wood.gloss, wood.metalness, wood.diffuseMapTiling.x]);
  });

  test("the game's chunks have the layers' reflections of the room, in GLSL and in WGSL", () => {
    const chunks = gameChunks(app().graphicsDevice);
    for (const lang of ['glsl', 'wgsl'] as const) {
      for (const chunk of ['reflectionSheenPS', 'reflectionCCPS']) {
        const code = chunks[lang][chunk];
        expect(code, `${lang} ${chunk}`).toContain('#ifdef GAME_FINISH_LAYERS');
        expect(code).toContain('gameRoomHere > 1.5');
        // The engine's own line stays for every other material.
        expect(code.split('calcReflection(').length - 1).toBe(2);
      }
      expect(chunks[lang].combinePS).toContain('#if defined(GAME_FINISH_LAYERS) && defined(LIT_REFLECTIONS)');
    }
  });
});
