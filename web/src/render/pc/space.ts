/**
 * Game space and PlayCanvas space. The game's world is Babylon's left-handed one, as the simulation,
 * the generators, the residents' files and the camera's angles have it (x east, y up, z north, a
 * yaw clockwise from above); PlayCanvas is right-handed. Everything in the world hangs under one
 * root mirrored in z (`worldRoot`), so it keeps its game-space numbers, and PlayCanvas, seeing the
 * root's negative scale, turns the faces' winding round with it (`worldScaleSign`). What lives
 * outside the root (cameras, lights) or crosses over (picking rays, points projected on screen,
 * mirror planes) is converted here and only here.
 */

import * as pc from './engine';

/** The world's root: game space, mirrored in z. */
export function createWorldRoot(app: pc.AppBase): pc.Entity {
  const root = new pc.Entity('world', app);
  root.setLocalScale(1, 1, -1);
  app.root.addChild(root);
  return root;
}

/** A game-space point or direction in PlayCanvas space (and back: the same mirror). */
export function toEngine(x: number, y: number, z: number, out: pc.Vec3): pc.Vec3 {
  return out.set(x, y, -z);
}

export function toGame(v: pc.Vec3, out: { x: number; y: number; z: number }): { x: number; y: number; z: number } {
  out.x = v.x;
  out.y = v.y;
  out.z = -v.z;
  return out;
}
