/**
 * Asset manifest schema. Code and content refer to assets only by key
 * (e.g. `model.fridge`, `icon.need.hunger`); the manifest maps keys to files.
 * Swapping art = editing the manifest or adding a pack.
 */

export type Vec3 = [number, number, number];

/** A part of a model the player may colour: a material of the file, and its name in the game. */
export interface PaintSlot {
  material: string;
  label: string;
}

/** A primitive part drawn in code: a character's limbs, or the few parts a file can't carry. */
export interface PrimitivePart {
  /** `blob` is a lumpy sphere for foliage and rocks. */
  shape: 'box' | 'cylinder' | 'cone' | 'sphere' | 'capsule' | 'blob';
  size: Vec3;
  /** Centre of the part. Local space: footprint centred on origin, y up, front is +z. */
  at: Vec3;
  color: string;
  /** Parts tinted per instance from a Sim's appearance (outfit, skin, hair). */
  tint?: 'body' | 'skin' | 'hair';
  /**
   * Surface finish: key of a `material` entry (e.g. `material.finish.fabric`). The part colour
   * tints that material's texture. Defaults to a plain matte finish.
   */
  material?: string;
  /** Edge rounding radius in metres for boxes (default: a small bevel; 0 = sharp). */
  radius?: number;
  /** Cylinders/cones: top diameter as a fraction of the bottom (1 = straight). */
  taper?: number;
  /** Blobs: surface noise amplitude as a fraction of the size (default 0.18). */
  noise?: number;
  /** Rotation in degrees around x, y, z (applied before `at`). */
  rotate?: Vec3;
  /** Tessellation override for round shapes (cylinder/cone sides, sphere segments, blob subdivisions: 1 = 20 triangles, 2 = 80, 3 = 180). */
  segments?: number;
  /**
   * Character parts only: the limb that moves this part (walk cycle, sitting, gestures).
   * Pivots are fixed (hips at 0.9 m, knees at 0.48 m, shoulders at 1.36 m, neck at 1.46 m;
   * x = ±0.09 for legs, ±0.21 for arms); shins follow their thigh. Parts without a bone move
   * with the body.
   */
  bone?: 'head' | 'armL' | 'armR' | 'legL' | 'legR' | 'shinL' | 'shinR';
  /**
   * Mirror glass (a thin box, or a disc: a flat cylinder): the face towards the model's front
   * (+z) or back reflects the room live (see `render/babylon/mirrors.ts`).
   */
  mirror?: 'front' | 'back';
}

export interface ModelEntry {
  type: 'model';
  /**
   * glTF/GLB file. A key without one, or a file that fails to load, draws a plain box (things placed,
   * held, carried or pictured; `BuildOptions.standIn`) or nothing (the landscape, foliage, the
   * selection marker). A file that fails is tried again in the background and drawn once it comes.
   */
  url?: string;
  /**
   * Its far version (`<id>.far.glb`, about a quarter of the triangles, no moving parts), drawn
   * where it is only seen from afar: the town overview, the far woods (`BuildOptions.far`, `lod` 1).
   */
  far?: string;
  scale?: number;
  /** Degrees. */
  rotationY?: number;
  offset?: Vec3;
  /**
   * Natural variation for placed objects (plants): each one gets its own turn and a size up to
   * ±`vary` (0.1 = 10%) larger or smaller, fixed by where it stands.
   */
  vary?: number;
  /**
   * The parts the player may colour: the file's materials by name, each with what the player
   * calls it (the double vanity's `body` is its Cabinet, `stone` its Worktop). The first is the
   * main part, the one the quick colour row paints. A bare name is its own label. Absent: it
   * comes as made.
   */
  paint?: (string | PaintSlot)[];
}

export interface MaterialEntry {
  type: 'material';
  color: string;
  /** Used for the sides of slab-like meshes. */
  sideColor?: string;
  texture?: string;
  /** Texture repeats per metre. */
  uvScale?: number;
  /** Tangent-space normal map (OpenGL convention), resolved relative to `texture`. */
  normal?: string;
  /** Packed ambient occlusion (R), roughness (G), metalness (B), resolved relative to `texture`. */
  arm?: string;
  /** 0 (mirror) to 1 (matte); multiplies the `arm` roughness when present. Default 0.8. */
  roughness?: number;
  /** 0 (dielectric) to 1 (metal). Default 0. */
  metallic?: number;
  /** Normal map strength (default 1). */
  normalStrength?: number;
  /** `xz`: texture by world position (metres) instead of the mesh UVs, for flat ground meshes. */
  projection?: 'uv' | 'xz';
  /** Opacity below 1 makes the surface see-through (glass). Default 1. */
  alpha?: number;
}

export interface IconEntry {
  type: 'icon';
  url: string;
  /** `mask` (default) recolours a monochrome icon with CSS; `image` shows it as-is. */
  mode?: 'mask' | 'image';
}

export interface ImageEntry {
  type: 'image';
  url: string;
}

export interface SpriteEntry {
  type: 'sprite';
  url: string;
  frameWidth: number;
  frameHeight: number;
  frames: number;
  fps: number;
}

/** Colour choices offered in character creation (`palette.skin`, `palette.hair`, `palette.outfit`). */
export interface PaletteEntry {
  type: 'palette';
  colors: string[];
}

/**
 * Rigged characters (`model.sim`, `model.hair.*`): `url` is a character set (`rig.json`: skeleton,
 * bodies, hairstyles and animation clips, see `render/babylon/characters`). `part` picks a mesh
 * part of each body (e.g. `hair.long`); without it the entry is the set itself.
 */
export interface CharacterEntry {
  type: 'character';
  url: string;
  part?: string;
}

export type AssetEntry = ModelEntry | MaterialEntry | IconEntry | ImageEntry | SpriteEntry | PaletteEntry | CharacterEntry;
export type AssetType = AssetEntry['type'];
export type EntryOf<T extends AssetType> = Extract<AssetEntry, { type: T }>;

export interface Manifest {
  version: number;
  /** Further manifests (relative URLs) whose entries override this one's. */
  packs?: string[];
  entries: Record<string, AssetEntry>;
}
