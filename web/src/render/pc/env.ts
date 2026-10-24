/**
 * The sky's image-based lighting on PlayCanvas: the manifest's `environment.sky`, a `.pcenv` that
 * `cargo envconv` makes from the HDRI (see `crates/tools/src/envconv.rs`): six RGBE cube faces,
 * already turned and mirrored into PlayCanvas's space (the world hangs under a root mirrored in z,
 * `space.ts`), and the sky's irradiance as PlayCanvas's nine ambient harmonics (the ones the game's
 * look was calibrated on, see envconv.rs).
 *
 * Built once per device: the faces become a lighting source and then PlayCanvas's environment atlas
 * (its own GGX prefiltering, on the GPU), which every scene of that device can share. The download
 * `index.html` started with the page (`earlyEnvironment` in vite.config.ts) is used when it is the
 * same file.
 */

import * as pc from './engine';

/** What lights a scene from the sky: reflections (`scene.envAtlas`) and ambient (`material.ambientSH`). */
export interface SkyLighting {
  atlas: pc.Texture;
  /** PlayCanvas's `ambientSH`: 9 × RGB. */
  sh: Float32Array;
}

const MAGIC = 'IDLLENV1';

const built = new WeakMap<pc.GraphicsDevice, Map<string, Promise<SkyLighting | null>>>();

/**
 * The sky's lighting from `url` (a `.pcenv`) for `device`, built on its first request; null when
 * it can't be loaded (the scene is then lit without it).
 */
export function skyLighting(device: pc.GraphicsDevice, url: string): Promise<SkyLighting | null> {
  const file = new URL(url, location.href).href;
  let byUrl = built.get(device);
  if (!byUrl) built.set(device, (byUrl = new Map()));
  let job = byUrl.get(file);
  if (!job) {
    job = load(file)
      .then((bytes) => build(device, bytes))
      .catch((err: unknown) => {
        console.warn(`[render] environment ${file} unavailable; using lights only`, err);
        return null;
      });
    byUrl.set(file, job);
  }
  return job;
}

async function load(url: string): Promise<ArrayBuffer> {
  const bytes = (await earlySky(url)) ?? (await fetch(url).then((res) => (res.ok ? res.arrayBuffer() : null)));
  if (!bytes) throw new Error('not loaded');
  return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer();
}

function build(device: pc.GraphicsDevice, buffer: ArrayBuffer): SkyLighting {
  const bytes = new Uint8Array(buffer);
  if (new TextDecoder().decode(bytes.subarray(0, 8)) !== MAGIC) throw new Error('not a .pcenv');
  const length = new DataView(buffer).getUint32(8, true);
  const header = JSON.parse(new TextDecoder().decode(bytes.subarray(12, 12 + length))) as { version: number; size: number; encoding: string; sh: number[] };
  if (header.version !== 1 || header.encoding !== 'rgbe' || header.sh.length !== 27) throw new Error('unknown .pcenv');
  const size = header.size;
  const faceBytes = size * size * 4;
  const start = 12 + length;
  if (bytes.length !== start + faceBytes * 6) throw new Error('truncated .pcenv');
  const faces = Array.from({ length: 6 }, (_, f) => bytes.slice(start + f * faceBytes, start + (f + 1) * faceBytes));
  const source = new pc.Texture(device, {
    name: 'skySource',
    cubemap: true,
    width: size,
    height: size,
    format: pc.PIXELFORMAT_RGBA8,
    type: pc.TEXTURETYPE_RGBE,
    mipmaps: false,
    // (Encoded texels don't filter: the reprojection takes many samples instead.)
    minFilter: pc.FILTER_NEAREST,
    magFilter: pc.FILTER_NEAREST,
    addressU: pc.ADDRESS_CLAMP_TO_EDGE,
    addressV: pc.ADDRESS_CLAMP_TO_EDGE,
    levels: [faces],
  });
  const lighting = pc.EnvLighting.generateLightingSource(source);
  const atlas = pc.EnvLighting.generateAtlas(lighting);
  atlas.name = 'skyAtlas';
  source.destroy();
  lighting.destroy();
  return { atlas, sh: new Float32Array(header.sh) };
}

/**
 * The sky's lighting as `index.html` started fetching it with the page (`earlyEnvironment` in
 * vite.config.ts), if that's `url`. Taken once: later devices (the previews') build from the
 * browser's cache.
 */
function earlySky(url: string): Promise<ArrayBuffer | null> | null {
  const page = self as { __skyEnv?: { url: string; bytes: Promise<ArrayBuffer | null> } };
  const early = page.__skyEnv;
  if (!early || new URL(early.url, location.href).href !== url) return null;
  delete page.__skyEnv;
  return early.bytes;
}
