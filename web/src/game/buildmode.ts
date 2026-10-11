/**
 * Buy and Build mode input: placing and moving objects on the home lot (Buy), and drawing
 * walls and rooms and putting doors and windows into them (Build). The simulation is the
 * authority (it re-checks every edit and reports problems as errors); this module only shows
 * previews and sends commands.
 *
 * Walls, Room and Remove work by press on a corner → drag → release (with a live preview and its
 * cost); a press and release without moving sets the start corner instead and the next click
 * finishes (click-then-click). Dragging at about 45° draws a diagonal run across tiles; the Room
 * tool draws the four walls of the rectangle between the two corners. Esc or a right-click while
 * drawing cancels. While drawing, left-drag doesn't turn the camera.
 *
 * New walls, doors and windows take the looks picked in Build mode (`game.buildLook`: covering
 * and form, door and window style); drawing a wall in another form over a standing one rebuilds
 * it, and a door or window of another style replaces the one there. The Paint tool covers the
 * wall face on the pointer's side: click, or drag along walls; with Shift, every face of the
 * room (or, outdoors, the outside of the house) at once. The Floor tool covers the floor of the
 * tile under the pointer: click, or drag out a rectangle of tiles; with Shift, the whole room.
 * Remove on doors and windows alone walls
 * them up again (the wall stays); a run that takes in walls tears everything on it down.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content, ObjectDef } from '../content/content';
import { groundDepth, MAX_HEAD, MAX_OFFSET, MAX_SHORTER, MAX_SIZE, MAX_TALLER, MIN_DOOR_HEAD, MIN_SIZE, MIN_WINDOW, objectCentre, type Command, type EdgeEdit, type FacePaint, type FloorPaint, type ObjectPlacement, type PlotInfo, type WorldStructure } from '../core/protocol';
import type { BuildEffect, ObjectResize, Renderer, ResizeHandle, ViewRect } from '../render/types';
import { styledModel } from '../ui/buy/catalog';
import type { Sound } from '../ui/sfx';
import { game, type BuildTool } from '../ui/state.svelte';
import { approachCandidates, covers, facedTable, fitOnTop, frontTile, ownSize, spotAt, topOf, type Top } from './surfaces';

/** Preview calls the renderer offers in Build mode. */
interface BuildPreview {
  /** Paint tool: the wall faces a click would cover, dressed in wall covering `cover` (null: hidden). */
  setPaintPreview(faces: readonly PaintFace[], cover: number | null): void;
  /** Floor tool: the tiles a release would cover, laid with floor covering `floor` (null: hidden). */
  setFloorPreview(tiles: readonly { x: number; z: number }[], floor: number | null): void;
  setHeldObject(id: number | null, moved?: boolean): void;
  /** Where the pointer's ray meets the level `height` metres above the floor in view (a table top). */
  pickPlane(height: number): Point | null;
  setPlacementGhost(
    ghost: {
      model: string;
      x: number;
      z: number;
      rot: number;
      turn?: number;
      offset?: [number, number];
      lift?: number;
      w: number;
      d: number;
      resize?: ObjectResize;
      front?: { x: number; z: number };
      valid: boolean;
    } | null,
  ): void;
  setResizeHandles(handles: readonly ResizeHandle[] | null, hot?: string | null): void;
  handleAt(x: number, y: number): string | null;
  pickVertical(x: number, z: number): number | null;
  pickOnWall(axis: 'h' | 'v', line: number, storeyRow: number): { along: number; y: number } | null;
  objectBox(id: number): { height: number; bottom: number } | null;
  setEdgePreview(edges: EdgeEdit[], valid: boolean): void;
  setBuildGrid(rect: ViewRect | null): void;
  /** Whether a left-drag turns the camera (off while drawing walls). */
  setLeftDragCamera(on: boolean): void;
}

type Point = { x: number; z: number };
/** A wall face to paint; `half` for a half wall (the preview's height). */
export type PaintFace = FacePaint & { half: boolean };
/** A wall's look as the structure sends it (doors and windows resized: sill and top, decimetres). */
type Look = { faces?: [number, number]; form?: number; style?: number; sill?: number; head?: number };

/**
 * Doors or windows side by side in one straight wall, the same style and size: one wide window
 * or a double door (as the renderer draws them). `line`: the wall's z (`h`) or x (`v`); the run
 * covers edges `start .. start + len - 1` along it. Heights in centimetres.
 */
interface OpeningRun {
  axis: 'h' | 'v';
  line: number;
  start: number;
  len: number;
  kind: 'door' | 'window';
  style: number;
  sill: number;
  head: number;
  /** Whether the player set them (else the style's own). */
  sized: boolean;
}
/** The widest window (edges) and door (a double door). */
const MAX_WINDOW_RUN = 6;
const MAX_DOOR_RUN = 2;
/** A door's own top (centimetres; house.ts `DOOR_HEIGHT`). */
const DOOR_HEAD = 210;

/** What the eyedropper picked up: a build tool with its look set, or an object to buy again. */
export type Picked = { kind: 'tool'; tool: BuildTool } | { kind: 'object'; def: string; style: number; rot: number; turn: number; size?: [number, number]; tall?: number };

/**
 * Things stand anywhere on their tiles (all but stairs): their middle snaps to quarter tiles (the
 * build grid's finer lines), or with Shift held to the centimetre.
 */
const OFFSET_STEP = 0.25;
const FREE_STEP = 0.01;
/** Things on a top snap finer than that: tops are small. */
const TOP_STEP = 0.05;

/** The eyedropper takes a wall (rather than the floor) within this distance of it (tiles). */
const PICK_WALL = 0.4;
type EdgeState = EdgeEdit['kind'];
type Axis = EdgeEdit['axis'];

/** A drag counts as diagonal within this angle of 45° (tan 22.5°: halfway to straight). */
const DIAGONAL_SLOPE = Math.tan(Math.PI / 8);

/** Resize arrows sit this far (m) outside the edge they move, and the height arrow this far above the top. */
const HANDLE_OUT = 0.3;

type Point2 = { x: number; y: number };

/** An owned object being resized by one of its arrows (see `press`). */
interface Resizing {
  id: number;
  def: ObjectDef;
  handle: string;
  /** As it was: its middle (tiles) and size along x and z (m), facing, turn and height (cm). */
  cx: number;
  cz: number;
  ex: number;
  ez: number;
  rot: number;
  turn: number;
  tall: number;
  lift: number;
  box: { height: number; bottom: number };
  /** Its tiles' corner and offset as it was. */
  was: { x: number; z: number; offset?: [number, number] };
  /** Height arrow: where the drag started (m above the floor). */
  from: number | null;
  /** What letting go now would make it: its middle and size, height, tiles (corner, turned size), offset and own size (cm before turning). */
  now: { cx: number; cz: number; ex: number; ez: number; tall: number; x: number; z: number; w: number; d: number; offset?: [number, number]; size?: [number, number]; valid: boolean };
}

/** An object's offset (centimetres off the middle of its footprint) as a key; `0:0` when centred. */
function offsetKey(offset: [number, number] | undefined): string {
  return `${offset?.[0] ?? 0}:${offset?.[1] ?? 0}`;
}

export class BuildBuyInput {
  private readonly preview: Partial<BuildPreview>;
  /** What stands on each wall edge now (`h:x:z` / `v:x:z` / `dp:x:z` / `dn:x:z`), for previews and costs. */
  private edges = new Map<string, EdgeState>();
  /** Direction of the diagonal across each tile (`x:z`), if any. */
  private diagonals = new Map<string, 'dp' | 'dn'>();
  /** Looks of the walls that have one (keys as `edges`). */
  private looks = new Map<string, Look>();
  /** Room of each tile, and of each half of a split tile (`x:z` → rooms of halves 0 and 1). */
  private rooms: number[] = [];
  private halves = new Map<string, [number, number]>();
  private width = 0;
  /** Paint tool: faces collected by the drag in progress. */
  private painting: PaintFace[] | null = null;
  /** Floor coverings laid (`x:z` → covering), and the Floor tool's drag start tile. */
  private floors = new Map<string, number>();
  private floorStart: Point | null = null;
  private shift = false;
  /** Alt (Option) held: a click picks up a look (the eyedropper). */
  private alt = false;
  private readonly onKey = (e: KeyboardEvent) => {
    if (this.alt !== e.altKey) {
      this.alt = e.altKey;
      if (!this.alt && !game.eyedropper) game.pickHint = null;
    }
    if (this.shift === e.shiftKey) return;
    this.shift = e.shiftKey;
    if (this.lastHover && this.building() && (game.buildTool === 'paint' || game.buildTool === 'floor')) this.hoverBuild(this.lastHover);
    // Shift lets go of the quarter-tile snap, and of whole tiles and 10 cm steps while resizing.
    const held = game.placing && this.content.object(game.placing.def);
    if (held && !held.stairs) this.rehover();
    if (this.resizing) this.hoverResize(this.lastHover);
    if (this.openingDrag) this.hoverOpening(this.lastHover);
  };
  private lastHover: Point | null = null;
  /** A wall/remove press is held (the gesture belongs to drawing until release). */
  private pressed = false;
  /** That press started a new run (rather than finishing a click-then-click one). */
  private fresh = false;
  /** Esc or right-click cancelled the held press: ignore its release. */
  private dropped = false;
  /** Last ground point seen while drawing (used when the pointer leaves the lot). */
  private lastGround: Point | null = null;
  /** Move room: the room being carried (its tiles), and the tile it was picked up by. */
  private carrying: { tiles: Point[]; at: Point } | null = null;
  /** Buy: an owned object carried by a drag (from where its middle was grabbed), until release. */
  private dragging: { id: number } | null = null;
  private grab: { dx: number; dz: number } | null = null;
  /** Buy: an owned object being resized by an arrow, until release. */
  private resizing: Resizing | null = null;
  /** After an edit renumbers the picked object: what to pick again once the new structure arrives. */
  private reselect: { def: string; x: number; z: number; rot: number } | null = null;
  /** What the shown arrows were made from (redrawn only when it changes). */
  private handlesKey = '';
  /** The arrow under the pointer. */
  private hotHandle: string | null = null;
  /** Build: the door or window picked with its tool (all of a wide one), and its resize in progress. */
  private openingSel: OpeningRun | null = null;
  private openingDrag: { handle: string; from: number | null; now: OpeningRun; valid: boolean } | null = null;

  constructor(
    renderer: Renderer,
    private readonly content: Content,
    private readonly assets: AssetRegistry,
    private readonly send: (command: Command) => void,
  ) {
    this.preview = renderer as unknown as Partial<BuildPreview>;
    window.addEventListener('keydown', this.onKey);
    window.addEventListener('keyup', this.onKey);
  }

  dispose(): void {
    window.removeEventListener('keydown', this.onKey);
    window.removeEventListener('keyup', this.onKey);
  }

  /** The lot arrays the maps below were made from (unchanged ones arrive as the same arrays). */
  private lotRefs: unknown[] = [];

  /** Keeps the current walls, doors and windows (from each new world structure). */
  setStructure(world: WorldStructure): void {
    const refs = [world.walls, world.openings, world.diagonals, world.fences, world.floors, world.rooms];
    if (refs.every((r, i) => r === this.lotRefs[i])) return;
    this.lotRefs = refs;
    this.edges = new Map();
    this.diagonals = new Map();
    this.looks = new Map();
    this.halves = new Map();
    this.floors = new Map((world.floors ?? []).map(([x, z, c]) => [`${x}:${z}`, c]));
    this.rooms = world.rooms;
    this.width = world.width;
    this.storeyDepth = groundDepth(world);
    const look = (key: string, l: Look) => this.looks.set(key, { ...this.looks.get(key), ...l });
    for (const e of world.walls ?? []) {
      this.edges.set(`${e.axis}:${e.x}:${e.z}`, 'wall');
      if (e.faces || e.form) look(`${e.axis}:${e.x}:${e.z}`, { faces: e.faces, form: e.form });
    }
    for (const o of world.openings ?? []) {
      this.edges.set(`${o.axis}:${o.x}:${o.z}`, o.kind);
      if (o.style || o.sill !== undefined || o.head !== undefined) look(`${o.axis}:${o.x}:${o.z}`, { style: o.style, sill: o.sill, head: o.head });
    }
    // A picked door or window that changed (or went) is found again from where it starts.
    const sel = this.openingSel;
    if (sel) {
      const [x, z] = sel.axis === 'h' ? [sel.start, sel.line] : [sel.line, sel.start];
      this.openingSel = this.openingRun(sel.axis, x, z);
      this.handlesKey = '';
    }
    for (const f of world.fences ?? []) {
      this.edges.set(`${f.axis}:${f.x}:${f.z}`, f.kind);
      if (f.style) look(`${f.axis}:${f.x}:${f.z}`, { style: f.style });
    }
    for (const d of world.diagonals ?? []) {
      this.edges.set(`${d.axis}:${d.x}:${d.z}`, d.kind);
      this.diagonals.set(`${d.x}:${d.z}`, d.axis);
      this.halves.set(`${d.x}:${d.z}`, d.rooms);
      if (d.faces || d.form || d.style) look(`${d.axis}:${d.x}:${d.z}`, { faces: d.faces, form: d.form, style: d.style });
    }
  }

  private lookOf(e: { axis: Axis; x: number; z: number }): Look {
    return this.looks.get(`${e.axis}:${e.x}:${e.z}`) ?? {};
  }

  private edgeState(e: { axis: Axis; x: number; z: number }): EdgeState {
    return this.edges.get(`${e.axis}:${e.x}:${e.z}`) ?? 'open';
  }

  /** Rows per storey of the lot (see `WorldStructure.storeys`). */
  private storeyDepth = 0;

  /**
   * The player's home plot, where everything in Build mode happens: on the storey in
   * view, in its rows of the lot (the renderer lifts them to that storey).
   */
  home(): PlotInfo | null {
    const plot = game.households.find((h) => h.player)?.plot;
    const p = plot == null ? null : (game.plots[plot] ?? null);
    return p && game.storey > 0 ? { ...p, z: p.z + game.storey * this.storeyDepth } : p;
  }

  /** Called when the mode changes: shows the grid and clears old previews. */
  modeChanged(): void {
    this.openingSel = null;
    this.openingDrag = null;
    // Never leave the camera's left-drag switched off once drawing can't continue.
    if (!this.pressed) this.preview.setLeftDragCamera?.(true);
    const home = this.home();
    this.preview.setBuildGrid?.(game.mode === 'build' && home ? { x: home.x, z: home.z, w: home.w, d: home.d } : null);
    this.clearPreviews();
  }

  clearPreviews(): void {
    this.preview.setPlacementGhost?.(null);
    this.preview.setEdgePreview?.([], true);
    game.buildCost = 0;
    this.preview.setPaintPreview?.([], null);
    this.preview.setFloorPreview?.([], null);
    game.paintFaces = 0;
    game.floorTiles = 0;
    game.buildWallUp = false;
    game.buildEdges = 0;
    game.buildValid = true;
    game.buildRoom = null;
    game.placeValid = true;
    game.placeHint = null;
  }

  /** Model key for an object in a style, falling back to the plain model. */
  styledModel(def: ObjectDef, style: number): string {
    return styledModel(this.content, this.assets, def, style);
  }

  /** Build mode's Build tab: walls, rooms, paint, doors, windows and removing them. */
  private building(): boolean {
    return game.mode === 'build' && game.buildTab === 'build';
  }

  /** Build mode's Paint tool. */
  private paintTool(): boolean {
    return this.building() && game.buildTool === 'paint';
  }

  /** Build mode's Floor tool. */
  private floorTool(): boolean {
    return this.building() && game.buildTool === 'floor';
  }

  /** Build mode's Stairs tool: places and moves staircases as the catalog places furniture. */
  stairsTool(): boolean {
    return this.building() && game.buildTool === 'stairs';
  }

  hover(ground: Point | null, objectId: number | null = null, screen: Point2 | null = null): void {
    if (this.openingDrag) {
      if (ground) this.lastHover = ground;
      return this.hoverOpening(ground ?? this.lastHover);
    }
    if (this.resizing) {
      if (ground) this.lastHover = ground;
      return this.hoverResize(ground ?? this.lastHover);
    }
    // The arrow under the pointer lights up.
    const picked = (game.furnishing && game.buySelection !== null && !game.placing) || (this.openingTool() && this.openingSel !== null);
    const hot = screen && !this.dragging && picked ? (this.preview.handleAt?.(screen.x, screen.y) ?? null) : null;
    if (hot !== this.hotHandle) {
      this.hotHandle = hot;
      this.handlesKey = '';
      this.syncHandles();
    }
    if (this.picking()) {
      this.clearPreviews();
      game.pickHint = this.pickLabel(ground, objectId);
      return;
    }
    if (game.pickHint !== null) game.pickHint = null;
    // While drawing, keep the last preview when the pointer strays off the lot.
    if (!ground && this.pressed) return;
    this.lastHover = ground;
    if (!ground) return this.clearPreviews();
    if (this.pressed) this.lastGround = ground;
    if (this.stairsTool() || game.furnishing) this.hoverBuy(ground);
    else if (this.building()) this.hoverBuild(ground);
  }

  /** Redraw the Buy ghost where the pointer last was (after a turn, without waiting for the pointer to move). */
  rehover(): void {
    if (this.lastHover && (game.furnishing || this.stairsTool()) && !this.picking()) this.hoverBuy(this.lastHover);
  }

  /** The next click picks up a look (E, or Alt held) instead of building or placing. */
  picking(): boolean {
    return game.mode === 'build' && (game.eyedropper || this.alt);
  }

  /**
   * The eyedropper: what's under the pointer, as something to build or buy again. A wall gives
   * its covering and form (a door or window, its style), a floor its covering; an object, another
   * of it in the same style and at the same angle. Null: nothing to pick there.
   */
  pick(ground: Point | null, objectId: number | null, apply = true): Picked | null {
    const look = game.buildLook;
    const obj = objectId === null ? undefined : game.objects.find((o) => o.id === objectId);
    const def = obj && this.content.object(obj.def);
    if (obj && def?.stairs) return { kind: 'tool', tool: 'stairs' };
    if (obj && def?.price !== undefined) {
      return { kind: 'object', def: obj.def, style: obj.style, rot: obj.rot, turn: obj.turn ?? 0, size: ownSize(this.content, obj), tall: obj.tall ?? 0 };
    }
    if (!ground) return null;
    const e = this.nearestEdge(ground);
    const state = this.edgeState(e);
    const away = e.axis === 'h' ? Math.abs(ground.z - e.z) : e.axis === 'v' ? Math.abs(ground.x - e.x) : 0;
    if (state !== 'open' && away < PICK_WALL) {
      const l = this.lookOf(e);
      if (state === 'door' || state === 'window') {
        if (apply) game.buildLook = { ...look, [state]: l.style ?? 0 };
        return { kind: 'tool', tool: state };
      }
      if (state === 'fence' || state === 'gate') {
        if (apply) game.buildLook = { ...look, fence: l.style ?? 0 };
        return { kind: 'tool', tool: state };
      }
      const face = this.faceAt(ground);
      if (apply) game.buildLook = { ...look, cover: l.faces?.[face?.side ?? 0] ?? 0, form: l.form ?? 0 };
      return { kind: 'tool', tool: game.buildTool === 'wall' || game.buildTool === 'room' ? game.buildTool : 'paint' };
    }
    const t = this.tileOf(ground);
    if (this.indoors(t.x, t.z)) {
      if (apply) game.buildLook = { ...look, floor: this.floors.get(`${t.x}:${t.z}`) ?? 0 };
      return { kind: 'tool', tool: 'floor' };
    }
    return null;
  }

  /** What the eyedropper would pick up here, for the cursor tag. */
  private pickLabel(ground: Point | null, objectId: number | null): string {
    const p = this.pick(ground, objectId, false);
    if (!p) return 'Nothing to pick up here';
    if (p.kind === 'object') return `Another ${this.content.object(p.def)?.name ?? p.def}`;
    if (p.tool === 'stairs') return 'Another staircase';
    if (p.tool === 'floor') {
      const t = this.tileOf(ground!);
      const c = this.floors.get(`${t.x}:${t.z}`) ?? 0;
      return c ? (this.content.floorCoverings[c - 1]?.label ?? 'This floor') : 'Room default floor';
    }
    const e = this.nearestEdge(ground!);
    const l = this.lookOf(e);
    if (p.tool === 'door') return this.content.doorStyles[l.style ?? 0]?.label ?? 'This door';
    if (p.tool === 'window') return this.content.windowStyles[l.style ?? 0]?.label ?? 'This window';
    if (p.tool === 'fence' || p.tool === 'gate') return `${this.content.fenceStyles[l.style ?? 0]?.label ?? 'This'} ${p.tool}`;
    const c = l.faces?.[this.faceAt(ground!)?.side ?? 0] ?? 0;
    const label = c ? (this.content.wallCoverings[c - 1]?.label ?? 'This wall') : 'House default wall';
    return l.form ? `${label} · half wall` : label;
  }

  /** Returns true when the click was used. */
  click(ground: Point | null, objectId: number | null): boolean {
    if (this.paintTool() || this.floorTool()) return true;
    if (this.stairsTool()) return this.clickStairs(ground, objectId);
    if (this.building()) return ground ? this.clickBuild(ground) : true;
    if (game.furnishing) return this.clickBuy(ground, objectId);
    return false;
  }

  // ---- Buy -------------------------------------------------------------------------

  private placement(ground: Point) {
    const placing = game.placing;
    const def = placing && this.content.object(placing.def);
    if (!placing || !def) return null;
    // Its own size (cm) and the tiles it takes (rounded up).
    const kind = def.footprint ?? [1, 1];
    const size: [number, number] = placing.size ? [placing.size[0], placing.size[1]] : [kind[0] * 100, kind[1] * 100];
    const tall = placing.tall ?? 0;
    if (def.layer === 'surface') return { ...this.placementOnTop(ground, def, placing), size, tall };
    const [fw, fd] = size.map((v) => Math.max(1, Math.ceil(v / 100 - 1e-6)));
    // The middle goes under the pointer (or where it was grabbed), off the tile grid in quarter
    // tiles (Shift: centimetres); stairs stay square on it.
    const step = def.stairs ? 0 : this.shift ? FREE_STEP : OFFSET_STEP;
    const [gx, gz] = [ground.x + (this.grab?.dx ?? 0), ground.z + (this.grab?.dz ?? 0)];
    const cx = step ? Math.round(gx / step) * step : gx;
    const cz = step ? Math.round(gz / step) * step : gz;
    const at = (rot: number) => {
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const x = Math.floor(cx - w / 2 + 0.5);
      const z = Math.floor(cz - d / 2 + 0.5);
      const cm = (c: number) => Math.max(-MAX_OFFSET, Math.min(MAX_OFFSET, Math.round(c * 100)));
      let offset: [number, number] | undefined = step ? [cm(cx - x - w / 2), cm(cz - z - d / 2)] : undefined;
      // Things on a wall slide along it, never off it.
      if (offset && def.layer === 'wall') offset = rot % 2 === 0 ? [offset[0], 0] : [0, offset[1]];
      const fit = this.fits(x, z, w, d, rot, placing.objectId, def, [fw, fd]);
      return { def, rot, turn: placing.turn, offset, x, z, w, d, valid: fit === true, reason: fit === true ? null : fit, lift: 0, size, tall };
    };
    const p = at(placing.rot);
    // Things on a wall turn to face away from the wall they're pointed at.
    if (!p.valid && def.layer === 'wall') {
      for (let k = 1; k < 4; k++) {
        const q = at((placing.rot + k) % 4);
        if (q.valid) return q;
      }
    }
    // Chairs turn to face a table they're put beside.
    if (def.seatAt?.length && !this.facesTable(p)) {
      for (let k = 1; k < 4; k++) {
        const q = at((placing.rot + k) % 4);
        if (q.valid && this.facesTable(q)) return q;
      }
    }
    return p;
  }

  /** Whether seat `objectId` stands pulled up to a table (its `needsTable` interactions are offered). */
  atTable(objectId: number): boolean {
    const seat = game.objects.find((o) => o.id === objectId);
    return !!seat && facedTable(this.content, game.objects, seat, (ax, az, bx, bz) => this.wallBetween(ax, az, bx, bz), this.storeyDepth) !== null;
  }

  /** Whether a seat placed as `p` faces a table it tucks under. */
  private facesTable(p: { def: ObjectDef; x: number; z: number; rot: number }): boolean {
    const others = game.objects.filter((o) => o.id !== game.placing?.objectId);
    return facedTable(this.content, others, { def: p.def.id, x: p.x, z: p.z, rot: p.rot }, (ax, az, bx, bz) => this.wallBetween(ax, az, bx, bz), this.storeyDepth) !== null;
  }

  /**
   * Something small going on a top: the highest top on the home plot under the pointer takes it
   * (the pointer's ray meets the higher tops first), its middle snapped to 5 cm (with Shift, to
   * the centimetre). Elsewhere it's shown on the floor, refused.
   */
  private placementOnTop(ground: Point, def: ObjectDef, placing: NonNullable<typeof game.placing>) {
    const home = this.home();
    const moving = placing.objectId;
    const others = game.objects.filter((o) => o.id !== moving);
    const onPlot = (o: { x: number; z: number }) => !!home && o.x >= home.x && o.z >= home.z && o.x < home.x + home.w && o.z < home.z + home.d;
    const tops = others.filter(onPlot).map((o) => topOf(this.content, o)).filter((t): t is Top => t !== null).sort((a, b) => b.height - a.height);
    let under: { top: Top; at: Point } | null = null;
    for (const top of tops) {
      const at = this.preview.pickPlane?.(top.height);
      if (at && covers(top, at.x, at.z)) {
        under = { top, at };
        break;
      }
    }
    const at = under?.at ?? ground;
    const step = this.shift ? FREE_STEP : TOP_STEP;
    const spot = spotAt(Math.round(at.x / step) * step, Math.round(at.z / step) * step);
    const [cx, cz] = [spot.x + 0.5 + spot.offset[0] / 100, spot.z + 0.5 + spot.offset[1] / 100];
    const fit = fitOnTop(this.content, others, cx, cz, this.content.radius(def));
    const affordable = moving !== null || (def.price !== undefined && game.affords(def.price));
    const valid = typeof fit !== 'string' && onPlot(fit.host) && onPlot(spot) && affordable;
    const reason = valid ? null : fit === 'noTop' ? 'Goes on a table, counter, desk or shelf' : fit === 'overEdge' ? 'Too near the edge' : null;
    return { def, rot: placing.rot, turn: placing.turn, offset: spot.offset, x: spot.x, z: spot.z, w: 1, d: 1, valid, reason, lift: typeof fit !== 'string' ? fit.height : (under?.top.height ?? 0) };
  }

  /** Whether a tile is indoors (a tile split by a diagonal wall is when either half is, as in the simulation). */
  private indoors(tx: number, tz: number): boolean {
    return (this.rooms[tz * this.width + tx] ?? 0) !== 0;
  }

  /**
   * Same checks as the simulation, minus reachability: on the home plot, free tiles, outdoors for
   * garden things, affordable. True, or why not (a short hint; null for the usual "doesn't fit").
   */
  private fits(x: number, z: number, w: number, d: number, rot: number, moving: number | null, def: ObjectDef, size?: readonly [number, number]): true | string | null {
    const home = this.home();
    if (!home) return null;
    const onPlot = (tx: number, tz: number) => tx >= home.x && tz >= home.z && tx < home.x + home.w && tz < home.z + home.d;
    // Only things on the same layer take each other's place (a picture over a sofa, a rug under it).
    const layer = this.content.layer(def);
    const takenOn = (on: string, tx: number, tz: number) =>
      game.objects.some((o) => o.id !== moving && tx >= o.x && tz >= o.z && tx < o.x + o.w && tz < o.z + o.d && this.content.layer(this.content.object(o.def)) === on);
    for (let tz = z; tz < z + d; tz++) for (let tx = x; tx < x + w; tx++) if (!onPlot(tx, tz) || takenOn(layer, tx, tz)) return null;
    if (def.outdoors) {
      for (let tz = z; tz < z + d; tz++) for (let tx = x; tx < x + w; tx++) if (this.indoors(tx, tz)) return 'Goes outdoors';
    }
    if (layer === 'ceiling') {
      for (let tz = z; tz < z + d; tz++) for (let tx = x; tx < x + w; tx++) if (!this.indoors(tx, tz)) return 'Hangs from a ceiling';
    }
    if (layer === 'wall') {
      const backs: { axis: Axis; x: number; z: number }[] = [];
      for (let tx = x; tx < x + w; tx++) if (rot % 4 === 0 || rot % 4 === 2) backs.push({ axis: 'h', x: tx, z: rot % 4 === 0 ? z : z + d });
      for (let tz = z; tz < z + d; tz++) if (rot % 4 === 1 || rot % 4 === 3) backs.push({ axis: 'v', x: rot % 4 === 1 ? x : x + w, z: tz });
      if (!backs.every((e) => ['wall', 'window'].includes(this.edgeState(e)))) return 'Hangs on a wall';
    }
    // Something nobody uses needn't be reached (a picture, a rug, a table chairs go all round);
    // a chair pulled up to a table is used from any free side.
    const unused = def.interactions.length === 0;
    const front = frontTile(x, z, w, d, rot);
    if (!unused && !onPlot(front[0], front[1])) return null;
    const free = ({ tile: [tx, tz], inner: [ix, iz] }: { tile: [number, number]; inner: [number, number] }) =>
      onPlot(tx, tz) && !takenOn('floor', tx, tz) && !this.diagonals.has(`${tx}:${tz}`) && !this.wallBetween(ix, iz, tx, tz);
    if (!unused && !approachCandidates(def, x, z, rot, size).some(free)) return def.approach === 'around' ? 'No free side to sit down from' : null;
    // As the simulation: no wall through the footprint, or between it and where it's used from.
    for (let tz = z; tz < z + d; tz++) {
      for (let tx = x; tx < x + w; tx++) {
        if (this.diagonals.has(`${tx}:${tz}`)) return null;
        if ((tx + 1 < x + w && this.wallBetween(tx, tz, tx + 1, tz)) || (tz + 1 < z + d && this.wallBetween(tx, tz, tx, tz + 1))) return 'Blocked by a wall';
      }
    }
    const [fx, fz] = front;
    const back = [
      [fx, fz - 1],
      [fx - 1, fz],
      [fx, fz + 1],
      [fx + 1, fz],
    ][rot % 4];
    if (this.wallBetween(back[0], back[1], fx, fz)) return 'Faces a wall';
    return moving !== null || (def.price !== undefined && game.affords(def.price)) ? true : null;
  }

  /** Whether a wall (with or without a door or window) stands between two neighbouring tiles. */
  private wallBetween(ax: number, az: number, bx: number, bz: number): boolean {
    const edge =
      bx !== ax ? { axis: 'v' as const, x: Math.max(ax, bx), z: az } : { axis: 'h' as const, x: ax, z: Math.max(az, bz) };
    return this.edgeState(edge) !== 'open';
  }

  private hoverBuy(ground: Point): void {
    const p = this.placement(ground);
    if (!p) return this.preview.setPlacementGhost?.(null);
    const style = p && game.placing?.objectId != null ? (game.objects.find((o) => o.id === game.placing!.objectId)?.style ?? 0) : (game.placing?.style ?? game.householdStyle);
    // The arrow marks where it's used from: nothing for things nobody uses (or on a top).
    const used = p.def.interactions.length > 0 && p.def.layer !== 'surface';
    const [fx, fz] = frontTile(p.x, p.z, p.w, p.d, p.rot);
    const resize = this.resizeOf(p.def, p.size, p.tall);
    this.preview.setPlacementGhost?.({ model: this.styledModel(p.def, style), x: p.x, z: p.z, rot: p.rot, turn: p.turn, offset: p.offset, lift: p.lift, w: p.w, d: p.d, resize, front: used ? { x: fx, z: fz } : undefined, valid: p.valid });
    if (game.placeValid !== p.valid) game.placeValid = p.valid;
    if (game.placeHint !== p.reason) game.placeHint = p.reason;
  }

  private clickBuy(ground: Point | null, objectId: number | null): boolean {
    const placing = game.placing;
    if (placing) {
      const p = ground && this.placement(ground);
      if (!p) return true;
      // Doesn't fit: keep it in hand rather than ask the simulation to refuse it.
      if (!p.valid) {
        game.placeRefused++;
        return true;
      }
      if (placing.objectId !== null) {
        // Put down where it was: nothing to send (a move renumbers it and is an undo step).
        const was = game.objects.find((o) => o.id === placing.objectId);
        const same = was && was.x === p.x && was.z === p.z && was.rot === p.rot && (was.turn ?? 0) === p.turn && offsetKey(was.offset) === offsetKey(p.offset);
        if (!same) this.send({ type: 'moveObject', household: game.home, object: placing.objectId, x: p.x, z: p.z, rot: p.rot, turn: p.turn, offset: p.offset });
        // Moved: the original stays hidden until it shows up in its new place.
        this.preview.setHeldObject?.(null, !same);
        game.placing = null;
        this.preview.setPlacementGhost?.(null);
      } else {
        // New purchases stay on the cursor so several can be placed in a row.
        const resized = this.resizeOf(p.def, p.size, p.tall);
        this.send({ type: 'buy', household: game.home, object: p.def.id, at: [p.x, p.z, p.rot], style: placing.style ?? game.householdStyle, ...(p.turn ? { turn: p.turn } : {}), ...(offsetKey(p.offset) !== '0:0' ? { offset: p.offset } : {}), ...(resized ? { size: resized.size, tall: resized.tall } : {}) });
      }
      return true;
    }
    const home = this.home();
    const obj = objectId === null ? undefined : game.objects.find((o) => o.id === objectId);
    game.buySelection = null;
    // Stairs are moved in Build mode (Stairs tool).
    if (!obj || !home || this.content.object(obj.def)?.stairs || obj.x < home.x || obj.z < home.z || obj.x >= home.x + home.w || obj.z >= home.z + home.d) return true;
    // Clicking something at home picks it: its panel (move, turn, restyle, sell) and its resize arrows.
    game.buySelection = obj.id;
    this.syncHandles();
    return true;
  }

  /** How an object of type `def` is resized (undefined: as its kind comes). */
  private resizeOf(def: ObjectDef, size: readonly [number, number] | undefined, tall: number | undefined): ObjectResize | undefined {
    const from = def.footprint ?? [1, 1];
    // Its own size in metres (`size`: centimetres).
    const own: [number, number] = size ? [size[0] / 100, size[1] / 100] : [from[0], from[1]];
    if (own[0] === from[0] && own[1] === from[1] && !tall) return undefined;
    return { from: [from[0], from[1]], size: own, tall: tall ?? 0, ceiling: def.layer === 'ceiling' };
  }

  /** The owned object at home a press can pick up (not stairs: they move in Build mode). */
  private ownedAt(objectId: number | null): ObjectPlacement | null {
    const home = this.home();
    const obj = objectId === null ? undefined : game.objects.find((o) => o.id === objectId);
    if (!obj || !home || this.content.object(obj.def)?.stairs) return null;
    return obj.x >= home.x && obj.z >= home.z && obj.x < home.x + home.w && obj.z < home.z + home.d ? obj : null;
  }

  /** The object being resized: it's hidden while its preview shows (null: none). */
  heldId(): number | null {
    return this.resizing?.id ?? null;
  }

  /**
   * The resize arrows for an object as it is (or as a resize would make it): one per side it
   * can grow towards (along its wall, for things on a wall; none for things on a top or stairs)
   * and one for its height (none for rugs). `f`: its middle (tiles), its size along x and z (m),
   * facing, and top and bottom (m above its floor).
   */
  private handlesFor(def: ObjectDef, f: { cx: number; cz: number; ex: number; ez: number; rot: number; top: number; bottom: number }): ResizeHandle[] {
    const layer = this.content.layer(def);
    if (def.stairs) return [];
    const out: ResizeHandle[] = [];
    const [x0, x1, z0, z1] = [f.cx - f.ex / 2, f.cx + f.ex / 2, f.cz - f.ez / 2, f.cz + f.ez / 2];
    if (layer === 'wall') {
      // In the wall's plane, a little in front of it, halfway up.
      const y = (f.top + f.bottom) / 2;
      const r = f.rot % 4;
      const front = 0.12;
      // Its back is on its tile's edge (facing +z: the tile's near edge, -z: its far one).
      if (r % 2 === 0) {
        const z = r === 0 ? Math.floor(f.cz) + front : Math.floor(f.cz) + 1 - front;
        out.push({ id: 'x-', x: x0 - 0.1, y, z, dir: [-1, 0, 0], wall: true }, { id: 'x+', x: x1 + 0.1, y, z, dir: [1, 0, 0], wall: true });
      } else {
        const x = r === 1 ? Math.floor(f.cx) + front : Math.floor(f.cx) + 1 - front;
        out.push({ id: 'z-', x, y, z: z0 - 0.1, dir: [0, 0, -1], wall: true }, { id: 'z+', x, y, z: z1 + 0.1, dir: [0, 0, 1], wall: true });
      }
    } else if (layer !== 'surface') {
      // On the floor around it (from the ceiling: at its underside).
      const y = layer === 'ceiling' ? f.bottom : 0.04;
      out.push(
        { id: 'x-', x: x0 - HANDLE_OUT, y, z: f.cz, dir: [-1, 0, 0] },
        { id: 'x+', x: x1 + HANDLE_OUT, y, z: f.cz, dir: [1, 0, 0] },
        { id: 'z-', x: f.cx, y, z: z0 - HANDLE_OUT, dir: [0, 0, -1] },
        { id: 'z+', x: f.cx, y, z: z1 + HANDLE_OUT, dir: [0, 0, 1] },
      );
    }
    if (layer !== 'rug') {
      const ceiling = layer === 'ceiling';
      out.push({ id: 'up', x: f.cx, y: ceiling ? f.bottom - HANDLE_OUT - 0.1 : f.top + HANDLE_OUT, z: f.cz, dir: [0, ceiling ? -1 : 1, 0] });
    }
    return out;
  }

  /** A placed object's middle (tiles) and its size along x and z (m). */
  private frameOf(o: { x: number; z: number; w: number; d: number; rot: number; offset?: [number, number]; size?: [number, number] }): { cx: number; cz: number; ex: number; ez: number } {
    const c = objectCentre(o);
    const [sx, sz] = o.size ? [o.size[0] / 100, o.size[1] / 100] : o.rot % 2 === 0 ? [o.w, o.d] : [o.d, o.w];
    const [ex, ez] = o.rot % 2 === 0 ? [sx, sz] : [sz, sx];
    return { cx: c.x, cz: c.z, ex, ez };
  }

  /**
   * Shows the resize arrows of the object picked in the catalog (or of the resize in progress),
   * when what they're made from changed. Cheap enough to call every frame.
   */
  syncHandles(): void {
    if (this.openingTool() || this.openingDrag) return this.syncOpeningHandles();
    const r = this.resizing;
    const sel = game.furnishing && !game.placing && game.buySelection !== null ? (game.objects.find((o) => o.id === game.buySelection) ?? null) : null;
    const n = r?.now;
    const key = r && n ? `r:${r.id}:${n.cx}:${n.cz}:${n.ex}:${n.ez}:${n.tall}:${r.handle}` : sel ? `s:${sel.id}:${sel.x}:${sel.z}:${sel.w}:${sel.d}:${sel.size}:${sel.tall ?? 0}:${offsetKey(sel.offset)}:${this.hotHandle}:${game.objects.length}` : '';
    if (key === this.handlesKey) return;
    this.handlesKey = key;
    if (r && n) {
      const top = r.lift + r.box.height + (n.tall - r.tall) / 100;
      const handles = this.handlesFor(r.def, { cx: n.cx, cz: n.cz, ex: n.ex, ez: n.ez, rot: r.rot, top, bottom: r.lift + r.box.bottom });
      return this.preview.setResizeHandles?.(handles, r.handle);
    }
    const def = sel && this.content.object(sel.def);
    const box = sel && this.preview.objectBox?.(sel.id);
    if (!sel || !def || !box) {
      // Picked but not drawn yet (the renderer is still placing the new structure): try again next time.
      if (sel) this.handlesKey = '';
      return this.preview.setResizeHandles?.(null);
    }
    const lift = sel.lift ?? 0;
    this.preview.setResizeHandles?.(this.handlesFor(def, { ...this.frameOf(sel), rot: sel.rot, top: lift + box.height, bottom: lift + box.bottom }), this.hotHandle);
  }

  /** A new structure arrived: picks again what an edit renumbered. */
  objectsChanged(): void {
    const r = this.reselect;
    if (r) {
      const obj = game.objects.find((o) => o.def === r.def && o.x === r.x && o.z === r.z && o.rot === r.rot);
      this.reselect = null;
      if (obj && game.furnishing && !game.placing) game.buySelection = obj.id;
    }
    this.handlesKey = '';
    this.syncHandles();
  }

  /** Buy: pressing on an arrow of the picked object resizes it; on something at home, carries it. */
  private pressBuy(ground: Point | null, objectId: number | null, screen: Point2 | null): boolean {
    if (game.placing || this.picking()) return false;
    const handle = screen && game.buySelection !== null ? (this.preview.handleAt?.(screen.x, screen.y) ?? null) : null;
    const sel = handle ? game.objects.find((o) => o.id === game.buySelection) : undefined;
    const selDef = sel && this.content.object(sel.def);
    const box = sel && this.preview.objectBox?.(sel.id);
    if (handle && sel && selDef && box) {
      const tall = sel.tall ?? 0;
      const frame = this.frameOf(sel);
      this.resizing = {
        id: sel.id,
        def: selDef,
        handle,
        ...frame,
        rot: sel.rot,
        turn: sel.turn ?? 0,
        tall,
        lift: sel.lift ?? 0,
        box,
        from: null,
        was: { x: sel.x, z: sel.z, offset: sel.offset },
        now: { ...frame, tall, x: sel.x, z: sel.z, w: sel.w, d: sel.d, offset: sel.offset, valid: true },
      };
      this.preview.setLeftDragCamera?.(false);
      this.preview.setHeldObject?.(sel.id);
      this.hoverResize(ground);
      return true;
    }
    const obj = this.ownedAt(objectId);
    if (!obj || !ground) return false;
    // Carried by the point it was grabbed at.
    const c = objectCentre(obj);
    this.grab = { dx: c.x - ground.x, dz: c.z - ground.z };
    this.dragging = { id: obj.id };
    game.buySelection = null;
    game.placing = { def: obj.def, rot: obj.rot, turn: obj.turn ?? 0, objectId: obj.id, size: ownSize(this.content, obj), tall: obj.tall ?? 0 };
    this.preview.setLeftDragCamera?.(false);
    this.hoverBuy(ground);
    return true;
  }

  /**
   * Resizing: the size or height the arrow has been dragged to, previewed. The edge opposite the
   * arrow stays where it is; the size snaps to whole tiles and the height to 10 cm, or with
   * Shift held to the centimetre. The object takes its size's tiles rounded up and stands in
   * their middle, moved by its offset.
   */
  private hoverResize(ground: Point | null): void {
    const r = this.resizing;
    if (!r) return;
    const clampN = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
    let [x0, x1, z0, z1] = [r.cx - r.ex / 2, r.cx + r.ex / 2, r.cz - r.ez / 2, r.cz + r.ez / 2];
    let tall = r.tall;
    // Things on a wall follow the pointer on the wall's plane (the ground behind it would run away).
    const layer = this.content.layer(r.def);
    if (layer === 'wall') {
      const rr = r.rot % 4;
      const axis = rr % 2 === 0 ? 'h' : 'v';
      const line = rr === 0 ? Math.floor(r.cz - r.ez / 2) : rr === 2 ? Math.ceil(r.cz + r.ez / 2) : rr === 1 ? Math.floor(r.cx - r.ex / 2) : Math.ceil(r.cx + r.ex / 2);
      const hit = this.preview.pickOnWall?.(axis, line, r.was.z) ?? null;
      if (hit) ground = axis === 'h' ? { x: hit.along, z: ground?.z ?? r.cz } : { x: ground?.x ?? r.cx, z: hit.along };
    }
    const step = this.shift ? 0.01 : 1;
    const [least, most] = [MIN_SIZE / 100, MAX_SIZE];
    // The size from the fixed edge to the pointer, in steps.
    const span = (from: number, to: number) => clampN(Math.round(Math.abs(to - from) / step) * step, least, most);
    if (ground && r.handle === 'x+') x1 = x0 + span(x0, Math.max(ground.x, x0));
    if (ground && r.handle === 'x-') x0 = x1 - span(x1, Math.min(ground.x, x1));
    if (ground && r.handle === 'z+') z1 = z0 + span(z0, Math.max(ground.z, z0));
    if (ground && r.handle === 'z-') z0 = z1 - span(z1, Math.min(ground.z, z1));
    if (r.handle === 'up') {
      const h = this.preview.pickVertical?.(r.cx, r.cz) ?? null;
      if (h !== null) {
        r.from ??= h;
        const sign = layer === 'ceiling' ? -1 : 1;
        const cm = this.shift ? 1 : 10;
        // Not so short that it folds up (the model shrinks above 40% of its height).
        const lowest = Math.max(-MAX_SHORTER, r.tall - Math.floor((r.box.height - r.box.bottom) * 50));
        tall = clampN(r.tall + sign * Math.round(((h - r.from) * 100) / cm) * cm, lowest, MAX_TALLER);
      }
    }
    // Its tiles: its size rounded up, around its middle; what's left over is its offset.
    const [cx, cz, ex, ez] = [(x0 + x1) / 2, (z0 + z1) / 2, x1 - x0, z1 - z0];
    const w = Math.max(1, Math.ceil(ex - 1e-6));
    const d = Math.max(1, Math.ceil(ez - 1e-6));
    const x = Math.round(cx - w / 2);
    const z = Math.round(cz - d / 2);
    const cm = (v: number) => clampN(Math.round(v * 100), -MAX_OFFSET, MAX_OFFSET);
    const offset: [number, number] = [cm(cx - x - w / 2), cm(cz - z - d / 2)];
    const size: [number, number] = r.rot % 2 === 0 ? [Math.round(ex * 100), Math.round(ez * 100)] : [Math.round(ez * 100), Math.round(ex * 100)];
    const tiles: [number, number] = r.rot % 2 === 0 ? [w, d] : [d, w];
    const fit = this.fits(x, z, w, d, r.rot, r.id, r.def, tiles);
    r.now = { cx, cz, ex, ez, tall, x, z, w, d, offset, size, valid: fit === true };
    const obj = game.objects.find((o) => o.id === r.id);
    const model = this.styledModel(r.def, obj?.style ?? 0);
    this.preview.setPlacementGhost?.({ model, x, z, rot: r.rot, turn: r.turn, offset, lift: r.lift, w, d, resize: this.resizeOf(r.def, size, tall), valid: r.now.valid });
    if (game.placeValid !== r.now.valid) game.placeValid = r.now.valid;
    const hint = fit === true ? null : (fit ?? "There's no room for it that size");
    if (game.placeHint !== hint) game.placeHint = hint;
    this.syncHandles();
  }

  /** End of a Buy press: drops what was carried, or makes the resize. True if it was one of those. */
  private releaseBuy(ground: Point | null, moved: boolean): boolean {
    const r = this.resizing;
    if (r) {
      this.resizing = null;
      this.preview.setLeftDragCamera?.(true);
      const n = r.now;
      const changed = n.size !== undefined && (n.cx !== r.cx || n.cz !== r.cz || n.ex !== r.ex || n.ez !== r.ez || n.tall !== r.tall);
      if (changed && n.valid && n.size) {
        this.send({ type: 'moveObject', household: game.home, object: r.id, x: n.x, z: n.z, rot: r.rot, offset: n.offset, size: n.size, tall: n.tall });
        this.reselect = { def: r.def.id, x: n.x, z: n.z, rot: r.rot };
      }
      this.preview.setHeldObject?.(null, changed && n.valid);
      this.preview.setPlacementGhost?.(null);
      game.placeHint = null;
      game.placeValid = true;
      this.handlesKey = '';
      this.syncHandles();
      return true;
    }
    const drag = this.dragging;
    if (!drag) return false;
    this.dragging = null;
    this.preview.setLeftDragCamera?.(true);
    const p = moved ? this.placement(ground ?? this.lastHover ?? { x: 0, z: 0 }) : null;
    this.grab = null;
    game.placing = null;
    this.preview.setPlacementGhost?.(null);
    if (!moved) {
      // A press without a drag picks it.
      game.buySelection = drag.id;
      this.syncHandles();
      return true;
    }
    const was = game.objects.find((o) => o.id === drag.id);
    const same = !!p && !!was && was.x === p.x && was.z === p.z && was.rot === p.rot && offsetKey(was.offset) === offsetKey(p.offset);
    if (p?.valid && was && !same) {
      this.send({ type: 'moveObject', household: game.home, object: drag.id, x: p.x, z: p.z, rot: p.rot, turn: p.turn, offset: p.offset });
      this.reselect = { def: was.def, x: p.x, z: p.z, rot: p.rot };
      this.preview.setHeldObject?.(null, true);
    } else {
      // Dropped where it can't go (or where it was): it goes back.
      if (p && !p.valid) game.placeRefused++;
      this.preview.setHeldObject?.(null);
      game.buySelection = drag.id;
    }
    this.syncHandles();
    return true;
  }

  // ---- Doors and windows: resizing -------------------------------------------------

  /** Build mode's Door or Window tool (they pick and resize doors and windows). */
  private openingTool(): boolean {
    return this.building() && (game.buildTool === 'door' || game.buildTool === 'window');
  }

  /** Whether doors or windows on two neighbouring edges make one (house.ts `joins`). */
  private joined(axis: 'h' | 'v', x: number, z: number, step: 1 | -1): boolean {
    const kind = this.edgeState({ axis, x, z });
    if (kind !== 'door' && kind !== 'window') return false;
    const [nx, nz] = axis === 'h' ? [x + step, z] : [x, z + step];
    const [px, pz] = axis === 'h' ? [step > 0 ? x + 1 : x, z] : [x, step > 0 ? z + 1 : z];
    const crossed = axis === 'h'
      ? this.edgeState({ axis: 'v', x: px, z: pz }) !== 'open' || this.edgeState({ axis: 'v', x: px, z: pz - 1 }) !== 'open'
      : this.edgeState({ axis: 'h', x: px, z: pz }) !== 'open' || this.edgeState({ axis: 'h', x: px - 1, z: pz }) !== 'open';
    if (crossed || this.edgeState({ axis, x: nx, z: nz }) !== kind) return false;
    const [a, b] = [this.lookOf({ axis, x, z }), this.lookOf({ axis, x: nx, z: nz })];
    return (a.style ?? 0) === (b.style ?? 0) && a.sill === b.sill && a.head === b.head;
  }

  /** The door or window on an edge with its neighbours of the same kind, style and size (null: none there). */
  private openingRun(axis: 'h' | 'v', x: number, z: number): OpeningRun | null {
    const kind = this.edgeState({ axis, x, z });
    if (kind !== 'door' && kind !== 'window') return null;
    let [cx, cz] = [x, z];
    while (this.joined(axis, cx, cz, -1)) [cx, cz] = axis === 'h' ? [cx - 1, cz] : [cx, cz - 1];
    const start = axis === 'h' ? cx : cz;
    let len = 1;
    while (this.joined(axis, cx, cz, 1)) {
      len++;
      [cx, cz] = axis === 'h' ? [cx + 1, cz] : [cx, cz + 1];
    }
    const look = this.lookOf({ axis, x, z });
    const style = look.style ?? 0;
    const spec = kind === 'window' ? this.content.windowStyles[style] : undefined;
    const sill = look.sill ?? Math.round((spec?.sill ?? 0.9) * 100);
    const head = look.head ?? (kind === 'door' ? DOOR_HEAD : Math.round((spec?.head ?? 2.1) * 100));
    return { axis, line: axis === 'h' ? z : x, start, len, kind, style, sill: kind === 'door' ? 0 : sill, head, sized: look.sill !== undefined || look.head !== undefined };
  }

  /** The edges of a run (lot coordinates). */
  private runEdges(r: OpeningRun): { axis: 'h' | 'v'; x: number; z: number }[] {
    return Array.from({ length: r.len }, (_, k) => (r.axis === 'h' ? { axis: r.axis, x: r.start + k, z: r.line } : { axis: r.axis, x: r.line, z: r.start + k }));
  }

  /** Arrows for a door or window: its two ends along the wall, its top, and a window's sill. */
  private openingHandles(r: OpeningRun): ResizeHandle[] {
    const at = (a: number, y: number) => (r.axis === 'h' ? { x: a, y, z: r.line } : { x: r.line, y, z: a });
    const [sill, head] = [r.sill / 100, r.head / 100];
    const mid = r.kind === 'door' ? head / 2 : (sill + head) / 2;
    const dir = (s: number): [number, number, number] => (r.axis === 'h' ? [s, 0, 0] : [0, 0, s]);
    const out: ResizeHandle[] = [
      { id: 'a-', ...at(r.start - 0.2, mid), dir: dir(-1), wall: true },
      { id: 'a+', ...at(r.start + r.len + 0.2, mid), dir: dir(1), wall: true },
      { id: 'up', ...at(r.start + r.len / 2, head + 0.25), dir: [0, 1, 0] },
    ];
    if (r.kind === 'window') out.push({ id: 'down', ...at(r.start + r.len / 2, Math.max(0.15, sill - 0.25)), dir: [0, -1, 0] });
    return out;
  }

  private syncOpeningHandles(): void {
    const d = this.openingDrag;
    const r = d?.now ?? this.openingSel;
    const key = r ? `o:${r.axis}:${r.line}:${r.start}:${r.len}:${r.sill}:${r.head}:${d?.handle ?? this.hotHandle}` : '';
    if (key === this.handlesKey) return;
    this.handlesKey = key;
    this.preview.setResizeHandles?.(r ? this.openingHandles(r) : null, d?.handle ?? this.hotHandle);
  }

  /**
   * Door or Window tool: a press on an arrow of the picked one resizes it; on a door or window
   * of the tool's kind in the style picked, picks it (another style is replaced, as a click does).
   */
  private pressOpening(ground: Point | null, screen: Point2 | null): boolean {
    const handle = screen && this.openingSel ? (this.preview.handleAt?.(screen.x, screen.y) ?? null) : null;
    if (handle && this.openingSel) {
      this.openingDrag = { handle, from: null, now: { ...this.openingSel }, valid: true };
      this.preview.setLeftDragCamera?.(false);
      this.hoverOpening(ground);
      return true;
    }
    if (!ground) return false;
    const e = this.nearestWall(ground);
    if (!e || (e.axis !== 'h' && e.axis !== 'v')) return false;
    const run = this.openingRun(e.axis, e.x, e.z);
    const style = game.buildTool === 'door' ? game.buildLook.door : game.buildLook.window;
    if (!run || run.kind !== game.buildTool || run.style !== style) return false;
    this.openingSel = run;
    this.clearPreviews();
    this.handlesKey = '';
    this.syncHandles();
    return true;
  }

  /** Resizing a door or window: the run and heights its arrow has been dragged to, previewed. */
  private hoverOpening(ground: Point | null): void {
    const d = this.openingDrag;
    const sel = this.openingSel;
    if (!d || !sel) return;
    const now = { ...sel };
    const most = sel.kind === 'window' ? MAX_WINDOW_RUN : MAX_DOOR_RUN;
    const clampN = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
    // Where the pointer is on the wall's own plane (the ground behind it would run away).
    void ground;
    const row = sel.axis === 'h' ? sel.line : sel.start;
    const hit = this.preview.pickOnWall?.(sel.axis, sel.line, row) ?? null;
    if (hit && d.handle === 'a+') now.len = clampN(Math.round(hit.along) - sel.start, 1, most);
    if (hit && d.handle === 'a-') {
      const end = sel.start + sel.len;
      now.start = clampN(Math.round(hit.along), end - most, end - 1);
      now.len = end - now.start;
    }
    if (d.handle === 'up' || d.handle === 'down') {
      const h = hit?.y ?? null;
      if (h !== null) {
        d.from ??= h;
        // In 10 cm steps, or with Shift held to the centimetre.
        const cm = this.shift ? 1 : 10;
        const steps = Math.round(((h - d.from) * 100) / cm) * cm;
        if (d.handle === 'up') now.head = clampN(sel.head + steps, sel.kind === 'door' ? MIN_DOOR_HEAD : sel.sill + MIN_WINDOW, MAX_HEAD);
        else now.sill = clampN(sel.sill + steps, 0, sel.head - MIN_WINDOW);
        now.sized = true;
      }
    }
    d.now = now;
    const edits = this.openingEdits(sel, now);
    // Every edge it takes needs a full-height wall (or the same kind of opening) already there.
    const fits = this.runEdges(now).every((e) => {
      const state = this.edgeState(e);
      return (state === 'wall' && !this.lookOf(e).form) || state === sel.kind;
    });
    d.valid = fits && this.edgesOnHome(edits) && game.affords(game.creative ? 0 : this.cost(edits));
    game.buildCost = game.creative ? 0 : this.cost(edits);
    game.buildValid = d.valid;
    this.preview.setEdgePreview?.(this.runEdges(now).map((e) => ({ ...e, kind: sel.kind, style: sel.style, ...(now.sized ? { sill: sel.kind === 'window' ? now.sill : undefined, head: now.head } : {}) })), d.valid);
    this.syncHandles();
  }

  /** The build edits that turn run `from` into `to`: its edges with the new size, walls where it no longer reaches. */
  private openingEdits(from: OpeningRun, to: OpeningRun): EdgeEdit[] {
    const size = to.sized ? { ...(to.kind === 'window' ? { sill: to.sill } : {}), head: to.head } : {};
    const kept = new Set(this.runEdges(to).map((e) => `${e.x}:${e.z}`));
    return [
      ...this.runEdges(to).map((e) => ({ ...e, kind: to.kind, style: to.style, ...size })),
      ...this.runEdges(from).filter((e) => !kept.has(`${e.x}:${e.z}`)).map((e) => ({ ...e, kind: 'wall' as const })),
    ];
  }

  /** End of an arrow drag on a door or window: makes the change. True if it was one. */
  private releaseOpening(): boolean {
    const d = this.openingDrag;
    const sel = this.openingSel;
    if (!d) return false;
    this.openingDrag = null;
    this.preview.setLeftDragCamera?.(true);
    const n = d.now;
    const changed = !!sel && (n.start !== sel.start || n.len !== sel.len || n.sill !== sel.sill || n.head !== sel.head);
    if (sel && changed && d.valid) {
      this.send({ type: 'build', household: game.home, edits: this.openingEdits(sel, n) });
      this.openingSel = n;
    }
    this.clearPreviews();
    this.handlesKey = '';
    this.syncHandles();
    return true;
  }

  /** The staircase the Stairs tool builds. */
  stairsDef(): ObjectDef | undefined {
    return this.content.shop.find((d) => d.stairs);
  }

  /**
   * Stairs tool: places the staircase in hand (it stays there for another). With nothing in
   * hand, a click on stairs at home picks them up to move them; anywhere else, takes a new one.
   */
  private clickStairs(ground: Point | null, objectId: number | null): boolean {
    if (game.placing) return this.clickBuy(ground, objectId);
    const home = this.home();
    const obj = objectId === null ? undefined : game.objects.find((o) => o.id === objectId);
    if (obj && home && this.content.object(obj.def)?.stairs && obj.x >= home.x && obj.z >= home.z && obj.x < home.x + home.w && obj.z < home.z + home.d) {
      game.placing = { def: obj.def, rot: obj.rot, turn: obj.turn ?? 0, objectId: obj.id };
    } else {
      const def = this.stairsDef();
      if (def) game.placing = { def: def.id, rot: 0, turn: 0, objectId: null };
    }
    if (ground) this.hoverBuy(ground);
    return true;
  }

  // ---- Build -----------------------------------------------------------------------

  private corner(ground: Point): Point {
    return { x: Math.round(ground.x), z: Math.round(ground.z) };
  }

  /** The wall edge nearest to a ground point: a grid edge, or the diagonal across its tile. */
  private nearestEdge(ground: Point): { axis: Axis; x: number; z: number } {
    const tx = Math.floor(ground.x);
    const tz = Math.floor(ground.z);
    const fx = ground.x - tx;
    const fz = ground.z - tz;
    const toGrid = Math.min(fz, 1 - fz, fx, 1 - fx);
    const diagonal = this.diagonals.get(`${tx}:${tz}`);
    if (diagonal) {
      // Distance to the diagonal line across this tile.
      const toDiagonal = (diagonal === 'dp' ? Math.abs(fx - fz) : Math.abs(fx + fz - 1)) / Math.SQRT2;
      if (toDiagonal < toGrid) return { axis: diagonal, x: tx, z: tz };
    }
    return Math.min(fz, 1 - fz) <= Math.min(fx, 1 - fx) ? { axis: 'h', x: tx, z: Math.round(ground.z) } : { axis: 'v', x: Math.round(ground.x), z: tz };
  }

  /** The nearest full-height wall (with or without an opening) round the pointer's tile, if any. */
  private nearestWall(ground: Point): { axis: Axis; x: number; z: number } | null {
    const tx = Math.floor(ground.x);
    const tz = Math.floor(ground.z);
    const fx = ground.x - tx;
    const fz = ground.z - tz;
    const diagonal = this.diagonals.get(`${tx}:${tz}`);
    const around: [number, { axis: Axis; x: number; z: number }][] = [
      [fz, { axis: 'h', x: tx, z: tz }],
      [1 - fz, { axis: 'h', x: tx, z: tz + 1 }],
      [fx, { axis: 'v', x: tx, z: tz }],
      [1 - fx, { axis: 'v', x: tx + 1, z: tz }],
    ];
    if (diagonal) around.push([(diagonal === 'dp' ? Math.abs(fx - fz) : Math.abs(fx + fz - 1)) / Math.SQRT2, { axis: diagonal, x: tx, z: tz }]);
    const walls = around.filter(([, e]) => ['wall', 'door', 'window'].includes(this.edgeState(e)) && !(this.edgeState(e) === 'wall' && this.lookOf(e).form));
    return walls.length ? walls.reduce((a, b) => (b[0] < a[0] ? b : a))[1] : null;
  }

  /**
   * The run of edges drawn from corner `a` towards the pointer: straight along the longer
   * direction, or diagonal (one tile per step, corner to corner) when the drag is near 45°.
   */
  private run(a: Point, ground: Point, kind: EdgeEdit['kind']): EdgeEdit[] {
    const edges: EdgeEdit[] = [];
    const vx = ground.x - a.x;
    const vz = ground.z - a.z;
    const ax = Math.abs(vx);
    const az = Math.abs(vz);
    // Fences run along the grid only.
    if (kind !== 'fence' && Math.min(ax, az) > Math.max(ax, az) * DIAGONAL_SLOPE) {
      const n = Math.round((ax + az) / 2);
      const sx = Math.sign(vx);
      const sz = Math.sign(vz);
      const axis = sx === sz ? 'dp' : 'dn';
      for (let k = 0; k < n; k++) {
        edges.push({ axis, x: sx > 0 ? a.x + k : a.x - k - 1, z: sz > 0 ? a.z + k : a.z - k - 1, kind });
      }
      return edges;
    }
    const b = this.corner(ground);
    if (ax >= az) {
      for (let x = Math.min(a.x, b.x); x < Math.max(a.x, b.x); x++) edges.push({ axis: 'h', x, z: a.z, kind });
    } else {
      for (let z = Math.min(a.z, b.z); z < Math.max(a.z, b.z); z++) edges.push({ axis: 'v', x: a.x, z, kind });
    }
    return edges;
  }

  /** Doors and windows go into an existing wall with a single click; gates onto any edge without a wall. */
  private opening(): 'door' | 'window' | 'gate' | null {
    return game.buildTool === 'door' || game.buildTool === 'window' || game.buildTool === 'gate' ? game.buildTool : null;
  }

  /** Walls, Room, Fence and Remove: drawn by dragging (or click-then-click). */
  private drawing(): boolean {
    return this.building() && (game.buildTool === 'wall' || game.buildTool === 'room' || game.buildTool === 'fence' || game.buildTool === 'remove');
  }

  /** The walls around the rectangle between corner `a` and the pointer (a line while it's flat). */
  private room(a: Point, ground: Point): EdgeEdit[] {
    const b = this.corner(ground);
    if (a.x === b.x || a.z === b.z) return this.run(a, ground, 'wall');
    const [x0, x1] = [Math.min(a.x, b.x), Math.max(a.x, b.x)];
    const [z0, z1] = [Math.min(a.z, b.z), Math.max(a.z, b.z)];
    const edges: EdgeEdit[] = [];
    for (let x = x0; x < x1; x++) edges.push({ axis: 'h', x, z: z0, kind: 'wall' }, { axis: 'h', x, z: z1, kind: 'wall' });
    for (let z = z0; z < z1; z++) edges.push({ axis: 'v', x: x0, z, kind: 'wall' }, { axis: 'v', x: x1, z, kind: 'wall' });
    return edges;
  }

  private edits(ground: Point): EdgeEdit[] {
    const look = game.buildLook;
    const opening = this.opening();
    if (opening) {
      // Doors and windows keep to walls, so passing from one wall tile to the next stays on the wall.
      const edge = (opening !== 'gate' && this.nearestWall(ground)) || this.nearestEdge(ground);
      // Gates stand on grid edges (a diagonal under the pointer gives way to the nearest one).
      const e = opening === 'gate' && (edge.axis === 'dp' || edge.axis === 'dn') ? this.gridEdge(ground) : edge;
      return [{ ...e, kind: opening, style: opening === 'door' ? look.door : opening === 'window' ? look.window : look.fence }];
    }
    const kind = game.buildTool === 'remove' ? 'open' : game.buildTool === 'fence' ? 'fence' : 'wall';
    let edges: EdgeEdit[];
    if (!game.buildStart) {
      // Before the first click: highlight the edge under the cursor.
      edges = [{ ...(kind === 'fence' ? this.gridEdge(ground) : this.nearestEdge(ground)), kind }];
    } else edges = game.buildTool === 'room' ? this.room(game.buildStart, ground) : this.run(game.buildStart, ground, kind);
    // Removing only doors, windows and gates closes them up again (a wall, or the fence with the
    // gate's style); a run with walls or fences in it tears it all down.
    const opened = (e: EdgeEdit) => ['door', 'window', 'gate'].includes(this.edgeState(e));
    if (kind === 'open' && edges.length && edges.every(opened)) {
      return edges.map((e) => (this.edgeState(e) === 'gate' ? { ...e, kind: 'fence' } : { ...e, kind: 'wall' }));
    }
    if (kind === 'fence') return edges.map((e) => ({ ...e, style: look.fence }));
    return kind === 'wall' ? edges.map((e) => ({ ...e, cover: look.cover, form: look.form })) : edges;
  }

  /** The straight grid edge nearest a ground point (fences and gates ignore diagonals). */
  private gridEdge(ground: Point): { axis: Axis; x: number; z: number } {
    const fx = ground.x - Math.floor(ground.x);
    const fz = ground.z - Math.floor(ground.z);
    return Math.min(fz, 1 - fz) <= Math.min(fx, 1 - fx)
      ? { axis: 'h', x: Math.floor(ground.x), z: Math.round(ground.z) }
      : { axis: 'v', x: Math.round(ground.x), z: Math.floor(ground.z) };
  }

  private edgesOnHome(edges: EdgeEdit[]): boolean {
    const home = this.home();
    if (!home) return false;
    return edges.every((e) =>
      e.axis === 'h'
        ? e.x >= home.x && e.x < home.x + home.w && e.z >= home.z && e.z <= home.z + home.d
        : e.axis === 'v'
          ? e.x >= home.x && e.x <= home.x + home.w && e.z >= home.z && e.z < home.z + home.d
          : e.x >= home.x && e.x < home.x + home.w && e.z >= home.z && e.z < home.z + home.d,
    );
  }

  /** Diagonal walls can't cross furniture, nor a tile that has a diagonal the other way. */
  private diagonalsFit(edges: EdgeEdit[]): boolean {
    return edges.every((e) => {
      if ((e.axis !== 'dp' && e.axis !== 'dn') || e.kind === 'open') return true;
      const other = this.diagonals.get(`${e.x}:${e.z}`);
      if (other && other !== e.axis) return false;
      return !game.objects.some((o) => e.x >= o.x && e.z >= o.z && e.x < o.x + o.w && e.z < o.z + o.d);
    });
  }

  /** Price of a door or window style (the plain price without styles in content). */
  private stylePrice(kind: 'door' | 'window', style: number | undefined): number {
    const prices = game.catalog?.build;
    const list = kind === 'door' ? this.content.doorStyles : this.content.windowStyles;
    return list[style ?? 0]?.price ?? (kind === 'door' ? (prices?.door ?? 0) : (prices?.window ?? prices?.door ?? 0));
  }

  /** A metre of fence in a style; a gate adds the gate price. */
  private fencePrice(style: number | undefined, gate = false): number {
    const prices = game.catalog?.build;
    const metre = this.content.fenceStyles[style ?? 0]?.price ?? prices?.fence ?? 0;
    return metre + (gate ? (prices?.gate ?? 0) : 0);
  }

  private coverPrice(cover: number | undefined): number {
    return cover ? (this.content.wallCoverings[cover - 1]?.price ?? 0) : 0;
  }

  /** What the simulation will charge (see `World::build`): only edges that change cost anything. */
  private cost(edges: EdgeEdit[]): number {
    const prices = game.catalog?.build;
    if (!prices) return 0;
    const diagonalWall = prices.diagonalWall ?? Math.round(prices.wall * 1.414);
    return edges.reduce((sum, e) => {
      const state = this.edgeState(e);
      const wall = e.axis === 'dp' || e.axis === 'dn' ? diagonalWall : prices.wall;
      // Removing a diagonal that runs the other way (or isn't there) changes nothing.
      if (e.kind === 'open') return sum + (state === 'open' ? 0 : prices.remove);
      const old = this.lookOf(e);
      if (state === e.kind) {
        // Rebuilt in another form, or replaced by another style.
        if (e.kind === 'wall') return sum + ((e.form ?? 0) !== (old.form ?? 0) && e.form !== undefined ? wall : 0);
        if (e.kind === 'fence' || e.kind === 'gate') return sum + ((e.style ?? 0) !== (old.style ?? 0) ? this.fencePrice(e.style, e.kind === 'gate') : 0);
        return sum + ((e.style ?? old.style ?? 0) !== (old.style ?? 0) ? this.stylePrice(e.kind, e.style) : 0);
      }
      if (e.kind === 'wall') return sum + wall + (state === 'open' ? 2 * this.coverPrice(e.cover) : 0);
      if (e.kind === 'fence' || e.kind === 'gate') return sum + this.fencePrice(e.style, e.kind === 'gate');
      return sum + this.stylePrice(e.kind, e.style);
    }, 0);
  }

  // ---- Paint ----------------------------------------------------------------------

  /** Room on side `side` (0: -z / -x / half 0) of a wall (outdoors: 0). */
  private roomBeside(e: { axis: Axis; x: number; z: number }, side: 0 | 1): number {
    if (e.axis === 'dp' || e.axis === 'dn') return this.halves.get(`${e.x}:${e.z}`)?.[side] ?? 0;
    const [tx, tz] = e.axis === 'h' ? [e.x, side ? e.z : e.z - 1] : [side ? e.x : e.x - 1, e.z];
    const halves = this.halves.get(`${tx}:${tz}`);
    if (halves) {
      // The half of a split tile that touches this edge.
      const d = this.diagonals.get(`${tx}:${tz}`);
      const half = e.axis === 'h' ? (side ? 0 : 1) : side ? (d === 'dp' ? 1 : 0) : d === 'dp' ? 0 : 1;
      return halves[half];
    }
    if (tx < 0 || tz < 0 || tx >= this.width) return 0;
    return this.rooms[tz * this.width + tx] ?? 0;
  }

  /** The wall face nearest the pointer, on the pointer's side of the wall (null: no wall there). */
  private faceAt(ground: Point): PaintFace | null {
    const e = this.nearestEdge(ground);
    if (this.edgeState(e) === 'open') return null;
    const fx = ground.x - e.x;
    const fz = ground.z - e.z;
    // Face 0 looks towards -z (h), -x (v), or the tile's half 0 (the -z side of a diagonal).
    const side: 0 | 1 = e.axis === 'h' ? (ground.z < e.z ? 0 : 1) : e.axis === 'v' ? (ground.x < e.x ? 0 : 1) : e.axis === 'dp' ? (fz < fx ? 0 : 1) : fz < 1 - fx ? 0 : 1;
    return { ...e, side, covering: game.buildLook.cover, half: this.half(e) };
  }

  private half(e: { axis: Axis; x: number; z: number }): boolean {
    return this.edgeState(e) === 'wall' && !!this.lookOf(e).form;
  }

  /** Every face on the home lot that looks into the same room as `face` (outdoors: the house's outside). */
  private roomFaces(face: PaintFace): PaintFace[] {
    const home = this.home();
    const room = this.roomBeside(face, face.side);
    const out: PaintFace[] = [];
    for (const [key, state] of this.edges) {
      if (state === 'open') continue;
      const [axis, xs, zs] = key.split(':') as [Axis, string, string];
      const e = { axis, x: Number(xs), z: Number(zs) };
      if (!this.edgesOnHome([{ ...e, kind: 'wall' }]) || !home) continue;
      for (const side of [0, 1] as const) {
        if (this.roomBeside(e, side) === room) out.push({ ...e, side, covering: face.covering, half: this.half(e) });
      }
    }
    return out;
  }

  private paintCost(faces: readonly PaintFace[]): number {
    return faces.reduce((sum, f) => sum + ((this.lookOf(f).faces?.[f.side] ?? 0) !== f.covering ? this.coverPrice(f.covering) : 0), 0);
  }

  private hoverPaint(ground: Point): void {
    const face = this.faceAt(ground);
    let faces: PaintFace[] = [];
    if (this.painting) {
      if (face && !this.painting.some((f) => f.axis === face.axis && f.x === face.x && f.z === face.z && f.side === face.side)) this.painting.push(face);
      faces = this.painting;
    } else if (face) faces = this.shift ? this.roomFaces(face) : [face];
    faces = faces.filter((f) => this.edgesOnHome([{ ...f, kind: 'wall' }]));
    this.preview.setPaintPreview?.(faces, faces.length ? game.buildLook.cover : null);
    game.buildCost = game.creative ? 0 : this.paintCost(faces);
    game.paintFaces = faces.length;
    game.buildEdges = faces.length;
    game.buildValid = faces.length > 0 && game.affords(game.buildCost);
  }

  private commitPaint(ground: Point | null): void {
    const faces = this.painting ?? [];
    this.painting = null;
    if (!faces.length && ground) {
      const face = this.faceAt(ground);
      if (face) faces.push(...(this.shift ? this.roomFaces(face) : [face]));
    }
    const onHome = faces.filter((f) => this.edgesOnHome([{ ...f, kind: 'wall' }]));
    if (onHome.length) this.send({ type: 'paint', household: game.home, faces: onHome.map(({ axis, x, z, side, covering }) => ({ axis, x, z, side, covering })) });
  }

  // ---- Floor ----------------------------------------------------------------------

  private tileOf(ground: Point): Point {
    return { x: Math.floor(ground.x), z: Math.floor(ground.z) };
  }

  private onHomeTile(t: Point): boolean {
    const home = this.home();
    return !!home && t.x >= home.x && t.z >= home.z && t.x < home.x + home.w && t.z < home.z + home.d;
  }

  /**
   * Tiles the Floor tool covers: the rectangle from the drag's start tile to the pointer's, or
   * the tile under the pointer (Shift: every tile of its room). Only indoor tiles on the home lot.
   */
  private floorTiles(ground: Point): FloorPaint[] {
    const covering = game.buildLook.floor;
    const at = this.tileOf(ground);
    const out: FloorPaint[] = [];
    const add = (x: number, z: number) => {
      if (this.indoors(x, z) && this.onHomeTile({ x, z })) out.push({ x, z, covering });
    };
    if (this.floorStart) {
      const s = this.floorStart;
      for (let z = Math.min(s.z, at.z); z <= Math.max(s.z, at.z); z++) for (let x = Math.min(s.x, at.x); x <= Math.max(s.x, at.x); x++) add(x, z);
    } else if (this.shift && this.indoors(at.x, at.z)) {
      const room = this.rooms[at.z * this.width + at.x];
      const home = this.home();
      if (home) for (let z = home.z; z < home.z + home.d; z++) for (let x = home.x; x < home.x + home.w; x++) if (this.rooms[z * this.width + x] === room) add(x, z);
    } else add(at.x, at.z);
    return out;
  }

  private floorCost(tiles: readonly FloorPaint[]): number {
    return tiles.reduce((sum, t) => sum + ((this.floors.get(`${t.x}:${t.z}`) ?? 0) !== t.covering && t.covering ? (this.content.floorCoverings[t.covering - 1]?.price ?? 0) : 0), 0);
  }

  private hoverFloor(ground: Point): void {
    const tiles = this.floorTiles(ground);
    this.preview.setFloorPreview?.(tiles, tiles.length ? game.buildLook.floor : null);
    game.buildCost = game.creative ? 0 : this.floorCost(tiles);
    game.floorTiles = tiles.length;
    game.buildEdges = tiles.length;
    game.buildValid = tiles.length > 0 && game.affords(game.buildCost);
  }

  private commitFloor(ground: Point | null): void {
    const tiles = ground ? this.floorTiles(ground) : [];
    this.floorStart = null;
    if (tiles.length) this.send({ type: 'paintFloor', household: game.home, tiles });
  }

  // ---- Move room ------------------------------------------------------------------

  private moveTool(): boolean {
    return this.building() && game.buildTool === 'move';
  }

  /** The room on a tile of the home lot (0: outdoors, or not on the lot). */
  private roomAt(t: Point): number {
    if (!this.onHomeTile(t)) return 0;
    const halves = this.halves.get(`${t.x}:${t.z}`);
    return halves ? halves[0] || halves[1] : (this.rooms[t.z * this.width + t.x] ?? 0);
  }

  /** Every tile of a room on the home lot (a split tile counts if either half is the room). */
  private roomTiles(room: number): Point[] {
    const home = this.home();
    if (!home || !room) return [];
    const tiles: Point[] = [];
    for (let z = home.z; z < home.z + home.d; z++) {
      for (let x = home.x; x < home.x + home.w; x++) {
        const halves = this.halves.get(`${x}:${z}`);
        if (halves ? halves.includes(room) : this.rooms[z * this.width + x] === room) tiles.push({ x, z });
      }
    }
    return tiles;
  }

  /** The walls, doors and windows of a room (edges touching its tiles), moved by `(dx, dz)`. */
  private roomEdges(tiles: Point[], dx: number, dz: number): EdgeEdit[] {
    const inRoom = new Set(tiles.map((t) => `${t.x}:${t.z}`));
    const edges: EdgeEdit[] = [];
    for (const [key, kind] of this.edges) {
      if (kind !== 'wall' && kind !== 'door' && kind !== 'window') continue;
      const [axis, xs, zs] = key.split(':') as [Axis, string, string];
      const [x, z] = [Number(xs), Number(zs)];
      const sides = axis === 'h' ? [`${x}:${z - 1}`, `${x}:${z}`] : axis === 'v' ? [`${x - 1}:${z}`, `${x}:${z}`] : [`${x}:${z}`];
      if (sides.some((s) => inRoom.has(s))) edges.push({ axis, x: x + dx, z: z + dz, kind, form: this.lookOf({ axis, x, z }).form });
    }
    return edges;
  }

  /** Move room: the room under the pointer, or the one in hand where it would land. */
  private hoverMove(ground: Point): void {
    const t = this.tileOf(ground);
    const c = this.carrying;
    game.buildCost = 0;
    if (!c) {
      const tiles = this.roomTiles(this.roomAt(t));
      this.preview.setFloorPreview?.(tiles, tiles.length ? 0 : null);
      this.preview.setEdgePreview?.([], true);
      game.buildValid = tiles.length > 0;
      return;
    }
    const [dx, dz] = [t.x - c.at.x, t.z - c.at.z];
    const own = new Set(c.tiles.map((p) => `${p.x}:${p.z}`));
    const moved = c.tiles.map((p) => ({ x: p.x + dx, z: p.z + dz }));
    // The simulation re-checks everything (furniture in the way, nobody shut in).
    const valid = moved.every((p) => this.onHomeTile(p) && (own.has(`${p.x}:${p.z}`) || (this.roomAt(p) === 0 && !this.diagonals.has(`${p.x}:${p.z}`))));
    this.preview.setFloorPreview?.(moved, 0);
    this.preview.setEdgePreview?.(this.roomEdges(c.tiles, dx, dz), valid);
    game.buildValid = valid;
  }

  private hoverBuild(ground: Point): void {
    // The Roof tool is used from its panel, and so are blueprints.
    if (game.buildTool === 'roof' || game.buildTool === 'blueprint') return this.clearPreviews();
    if (this.moveTool()) return this.hoverMove(ground);
    if (this.paintTool()) return this.hoverPaint(ground);
    if (this.floorTool()) return this.hoverFloor(ground);
    const edges = this.edits(ground);
    game.buildCost = game.creative ? 0 : this.cost(edges);
    // Doors and windows need a full-height wall to go into (the simulation re-checks everything).
    // Fences and gates never go into walls; doors and windows only into full-height ones.
    const walled = (e: EdgeEdit) => ['wall', 'door', 'window'].includes(this.edgeState(e));
    const fenced = game.buildTool === 'fence' || game.buildTool === 'gate';
    const inWall = fenced
      ? edges.every((e) => !walled(e))
      : !this.opening() || edges.every((e) => this.edgeState(e) !== 'open' && !(this.edgeState(e) === 'wall' && this.lookOf(e).form));
    const valid = inWall && this.edgesOnHome(edges) && this.diagonalsFit(edges) && game.affords(game.buildCost);
    this.preview.setEdgePreview?.(edges, valid);
    game.buildEdges = edges.length;
    game.buildValid = valid;
    game.buildWallUp = game.buildTool === 'remove' && edges.length > 0 && edges.every((e) => e.kind === 'wall' || e.kind === 'fence');
    const start = game.buildStart;
    const end = this.corner(ground);
    game.buildRoom = game.buildTool === 'room' && start && start.x !== end.x && start.z !== end.z ? [Math.abs(end.x - start.x), Math.abs(end.z - start.z)] : null;
  }

  private commit(ground: Point): void {
    const edges = this.edits(ground);
    game.buildStart = null;
    if (edges.length) this.send({ type: 'build', household: game.home, edits: edges });
    this.clearPreviews();
  }

  private clickBuild(ground: Point): boolean {
    if (game.buildTool === 'roof' || game.buildTool === 'blueprint' || game.buildTool === 'move') return true;
    if (this.opening()) {
      this.openingSel = null;
      this.send({ type: 'build', household: game.home, edits: this.edits(ground) });
      return true;
    }
    if (!game.buildStart) {
      game.buildStart = this.corner(ground);
      this.hoverBuild(ground);
      return true;
    }
    this.commit(ground);
    return true;
  }

  /**
   * Left press with a wall or remove tool on the lot: starts a run at the nearest corner (or,
   * with a start already set by a click, will finish it on release). Returns true when the
   * gesture belongs to drawing; the camera then doesn't turn until the release.
   */
  press(ground: Point | null, objectId: number | null = null, screen: Point2 | null = null): boolean {
    if (game.furnishing) return this.pressBuy(ground, objectId, screen);
    if (this.openingTool() && this.pressOpening(ground, screen)) return true;
    if (this.floorTool() && ground) {
      // Floor: a drag covers the rectangle of tiles it spans; a click, one tile (Shift: the room).
      this.pressed = true;
      this.dropped = false;
      this.floorStart = this.shift ? null : this.tileOf(ground);
      this.preview.setLeftDragCamera?.(false);
      this.hoverFloor(ground);
      return true;
    }
    if (this.paintTool() && ground) {
      // Paint: a drag covers every face it passes (on the side it's on); a click, one (Shift: a room).
      this.pressed = true;
      this.dropped = false;
      this.painting = this.shift ? null : [];
      this.preview.setLeftDragCamera?.(false);
      this.hoverPaint(ground);
      return true;
    }
    if (this.moveTool() && ground) {
      // Move room: pick the room up; it lands where the pointer is let go.
      const at = this.tileOf(ground);
      const room = this.roomAt(at);
      if (!room) return false;
      this.carrying = { tiles: this.roomTiles(room), at };
      this.pressed = true;
      this.dropped = false;
      this.preview.setLeftDragCamera?.(false);
      this.hoverMove(ground);
      return true;
    }
    if (!this.drawing() || !ground) return false;
    this.pressed = true;
    this.dropped = false;
    this.fresh = !game.buildStart;
    if (this.fresh) game.buildStart = this.corner(ground);
    this.lastGround = ground;
    this.preview.setLeftDragCamera?.(false);
    this.hoverBuild(ground);
    return true;
  }

  /** End of a drawing press: a drag builds the run; a click without moving only sets the start. */
  release(ground: Point | null, moved: boolean): void {
    if (this.releaseOpening()) return;
    if (this.releaseBuy(ground, moved)) return;
    if (!this.pressed) return;
    this.pressed = false;
    this.preview.setLeftDragCamera?.(true);
    if (this.dropped) {
      this.dropped = false;
      this.painting = null;
      this.floorStart = null;
      this.carrying = null;
      return;
    }
    if (this.carrying) {
      const c = this.carrying;
      this.carrying = null;
      const g = ground ?? this.lastHover;
      if (g) {
        const t = this.tileOf(g);
        const [dx, dz] = [t.x - c.at.x, t.z - c.at.z];
        if (dx || dz) this.send({ type: 'moveRoom', household: game.home, x: c.at.x, z: c.at.z, dx, dz });
      }
      this.clearPreviews();
      if (ground) this.hoverMove(ground);
      return;
    }
    if (this.floorTool()) {
      this.commitFloor(ground ?? this.lastHover);
      if (ground) this.hoverFloor(ground);
      return;
    }
    if (this.paintTool()) {
      this.commitPaint(ground);
      if (ground) this.hoverPaint(ground);
      return;
    }
    const g = ground ?? this.lastGround;
    const start = game.buildStart;
    if (!g || !start || !this.drawing()) return;
    const end = this.corner(g);
    if (this.fresh && !moved && game.buildTool === 'remove') {
      // Remove: a click on a door or window walls it up at once.
      const e = this.nearestEdge(g);
      const state = this.edgeState(e);
      if (state === 'door' || state === 'window') {
        game.buildStart = null;
        this.send({ type: 'build', household: game.home, edits: [{ ...e, kind: 'wall' }] });
        this.clearPreviews();
        return this.hoverBuild(g);
      }
    }
    // A press without a drag (or back onto the start corner) keeps the start for click-then-click.
    if (this.fresh && (!moved || (end.x === start.x && end.z === start.z))) return this.hoverBuild(g);
    this.commit(g);
    this.hoverBuild(g);
  }

  /** Right-click: stops drawing (a held drag or a click-then-click start). */
  cancelDrawing(): boolean {
    this.painting = null;
    this.floorStart = null;
    this.carrying = null;
    if (!this.pressed && !game.buildStart) return false;
    if (this.pressed) {
      this.dropped = true;
      this.preview.setLeftDragCamera?.(true);
    }
    game.buildStart = null;
    this.clearPreviews();
    return true;
  }

  /** Escape: drop what's in hand. Returns true if there was something to cancel. */
  cancel(): boolean {
    this.carrying = null;
    if (this.resizing || this.dragging) {
      const id = this.resizing?.id ?? this.dragging?.id ?? null;
      this.resizing = null;
      this.dragging = null;
      this.grab = null;
      game.placing = null;
      this.preview.setHeldObject?.(null);
      this.preview.setLeftDragCamera?.(true);
      this.clearPreviews();
      game.buySelection = id;
      this.handlesKey = '';
      this.syncHandles();
      return true;
    }
    if (this.pressed) {
      this.dropped = true;
      this.preview.setLeftDragCamera?.(true);
    }
    if (game.placing || game.buildStart || game.buySelection !== null) {
      game.placing = null;
      game.buildStart = null;
      game.buySelection = null;
      this.clearPreviews();
      return true;
    }
    return false;
  }
}

/** More changed objects than this at once is a different view, not an edit. */
const MAX_EDIT_OBJECTS = 8;
/** Dust is shown on at most this many changed wall edges. */
const MAX_EDIT_EDGES = 40;

/**
 * What changed between two world structures of the same lot, as effects to play and the sound
 * that goes with them: furniture bought, moved, restyled, upgraded or sold; walls, doors and
 * windows put in or taken out. Feedback only follows edits the simulation accepted.
 */
export function editFeedback(prev: WorldStructure, next: WorldStructure): { effects: BuildEffect[]; sound: Sound | null } {
  const effects: BuildEffect[] = [];
  const sounds = new Set<Sound>();
  // An object's id is its place in the list, and edits reorder it (selling shifts later ids
  // down, moving re-adds an object at the end, undo puts things back), so match objects by what
  // they are: first those unchanged (same look, same place), then the same kind of object among
  // the rest (moved, turned, restyled or upgraded); the same id wins a tie.
  const before = new Map<number, ObjectPlacement>();
  const left = new Set(prev.objects);
  const key = (o: ObjectPlacement) => `${o.def}:${o.x}:${o.z}:${offsetKey(o.offset)}:${o.rot}:${o.turn ?? 0}:${o.style}:${o.quality}`;
  const byKey = Map.groupBy(prev.objects, key);
  for (const o of next.objects) {
    const same = byKey.get(key(o));
    if (!same?.length) continue;
    const old = same.splice(Math.max(0, same.findIndex((p) => p.id === o.id)), 1)[0];
    before.set(o.id, old);
    left.delete(old);
  }
  for (const o of next.objects) {
    if (before.has(o.id)) continue;
    const kind = [...left].filter((p) => p.def === o.def);
    const old = kind.find((p) => p.id === o.id) ?? kind[0];
    if (!old) continue;
    before.set(o.id, old);
    left.delete(old);
  }
  // Around where it stands (off the middle of its footprint, for things that stand anywhere on their tile).
  const rect = (o: ObjectPlacement) => ({ x: o.x + (o.offset?.[0] ?? 0) / 100, z: o.z + (o.offset?.[1] ?? 0) / 100, w: o.w, d: o.d });
  const added = next.objects.filter((o) => !before.has(o.id));
  const removed = [...left];
  if (added.length + removed.length <= MAX_EDIT_OBJECTS) {
    for (const o of added) {
      effects.push({ kind: 'place', ...rect(o), objectId: o.id });
      sounds.add('place');
    }
    for (const o of removed) {
      effects.push({ kind: 'sell', ...rect(o) });
      sounds.add('sell');
    }
    for (const o of next.objects) {
      const old = before.get(o.id);
      if (!old) continue;
      if (o.quality > old.quality) {
        effects.push({ kind: 'upgrade', ...rect(o), objectId: o.id });
        sounds.add('upgrade');
      } else if (o.x !== old.x || o.z !== old.z || offsetKey(o.offset) !== offsetKey(old.offset) || o.rot !== old.rot || o.turn !== old.turn || o.style !== old.style) {
        effects.push({ kind: 'place', ...rect(o), objectId: o.id });
        sounds.add(o.x !== old.x || o.z !== old.z || offsetKey(o.offset) !== offsetKey(old.offset) ? 'place' : 'rotate');
      }
    }
  }

  const edges = (w: WorldStructure) => {
    const map = new Map<string, { axis: Axis; x: number; z: number }>();
    for (const e of w.walls ?? []) map.set(`${e.axis}:${e.x}:${e.z}:wall`, e);
    for (const o of w.openings ?? []) map.set(`${o.axis}:${o.x}:${o.z}:${o.kind}`, o);
    for (const d of w.diagonals ?? []) map.set(`${d.axis}:${d.x}:${d.z}:${d.kind}`, d);
    for (const f of w.fences ?? []) map.set(`${f.axis}:${f.x}:${f.z}:${f.kind}`, f);
    return map;
  };
  /** Looks by edge: coverings (painting), and form and style (rebuilt or replaced in place). */
  const looks = (w: WorldStructure) => {
    const faces = new Map<string, string>();
    const shape = new Map<string, string>();
    for (const e of [...(w.walls ?? []), ...(w.diagonals ?? [])]) {
      faces.set(`${e.axis}:${e.x}:${e.z}`, String(e.faces ?? ''));
      shape.set(`${e.axis}:${e.x}:${e.z}`, `${e.form ?? 0}:${'style' in e ? (e.style ?? 0) : ''}`);
    }
    for (const o of w.openings ?? []) shape.set(`${o.axis}:${o.x}:${o.z}`, `0:${o.style ?? 0}`);
    for (const f of w.fences ?? []) shape.set(`${f.axis}:${f.x}:${f.z}`, `${f.kind}:${f.style ?? 0}`);
    return { faces, shape };
  };
  // Only furniture changed: the lot arrived as the same arrays.
  const sameLot = prev.walls === next.walls && prev.openings === next.openings && prev.diagonals === next.diagonals && prev.fences === next.fences;
  const was = sameLot ? new Map() : edges(prev);
  const now = sameLot ? new Map() : edges(next);
  const edgeRect = (e: { axis: Axis; x: number; z: number }) =>
    e.axis === 'h' ? { x: e.x, z: e.z - 0.1, w: 1, d: 0.2 } : e.axis === 'v' ? { x: e.x - 0.1, z: e.z, w: 0.2, d: 1 } : { x: e.x + 0.3, z: e.z + 0.3, w: 0.4, d: 0.4 };
  const built = [...now].filter(([k]) => !was.has(k));
  const torn = [...was].filter(([k]) => !now.has(k));
  for (const [, e] of built.slice(0, MAX_EDIT_EDGES)) effects.push({ kind: 'build', ...edgeRect(e) });
  // A door or window replacing a wall shows up as both; only the new opening needs dust.
  const edge = (key: string) => key.slice(0, key.lastIndexOf(':'));
  const rebuilt = new Set(built.map(([k]) => edge(k)));
  for (const [k, e] of torn.slice(0, MAX_EDIT_EDGES)) if (!rebuilt.has(edge(k))) effects.push({ kind: 'remove', ...edgeRect(e) });
  if (built.length) sounds.add('build');
  else if (torn.length) sounds.add('remove');
  const none = { faces: new Map<string, string>(), shape: new Map<string, string>() };
  const [lookWas, lookNow] = sameLot ? [none, none] : [looks(prev), looks(next)];
  // (Walls that became a door or window already have their dust.)
  const restyled = [...lookNow.shape].filter(([k, v]) => !rebuilt.has(k) && lookWas.shape.has(k) && lookWas.shape.get(k) !== v);
  for (const [k] of restyled.slice(0, MAX_EDIT_EDGES)) {
    const [axis, x, z] = k.split(':');
    effects.push({ kind: 'build', ...edgeRect({ axis: axis as Axis, x: Number(x), z: Number(z) }) });
  }
  if (restyled.length) sounds.add('build');
  if ([...lookNow.faces].some(([k, v]) => lookWas.faces.has(k) && lookWas.faces.get(k) !== v)) sounds.add('paint');
  const floorKey = (w: WorldStructure) => (w.floors ?? []).map((f) => f.join(':')).join(',');
  if (prev.floors !== next.floors && floorKey(prev) !== floorKey(next)) sounds.add('paint');

  const order: Sound[] = ['upgrade', 'place', 'sell', 'build', 'remove', 'paint', 'rotate'];
  return { effects, sound: order.find((s) => sounds.has(s)) ?? null };
}
