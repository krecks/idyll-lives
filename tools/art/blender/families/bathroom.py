# The bathroom sinks: the base sink (`model.sink`), the pedestal (`model.sinkPedestal`), the double
# vanity (`model.vanityDouble`), the vessel sink (`model.sinkVessel`) and the console sink
# (`model.sinkConsole`), over their primitive-part placeholders (`catalog.py`, `furniture_prims.py`),
# which stay the fallback; and the five toilets (`model.toilet`, `model.toiletCompact`, `model.toiletSmart`,
# `model.toiletClassic`, `model.toiletWallHung`). Notes: bathroom.md.
#
# Each sink is one still body (cabinet / basin / pedestal / legs) plus one moving part: a stream of
# water (`water`) that grows from the tap's spout into the basin while a resident washes (`motions`).
# Each toilet is one still body plus its lid, which lifts while a resident uses it.
# Built to the placeholder's size, in its numbers: the parts are placed in game axes (`F`).
#
# Run:  python3 -B tools/art/blender/build.py bathroom [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#EDEBE6', 'satin'),       # cabinet / vanity / pedestal / basin; the main part the player colours
    porcelain=('#F6F6F4', 'gloss'),  # white ceramic
    stone=('#DCDDDA', 'gloss'),      # worktops and vanity tops
    wood=('#B98A5A', 'wood'),        # wooden tops, shelves and legs
    trim=('#B9BCBF', 'satin'),       # plinths and seals
    metal=('#8E9196', 'metal'),      # basin interiors, grey metal
    chrome=('#D0D3D6', 'chrome'),    # chrome taps
    brass=('#C9A54A', 'metal'),      # brass taps and legs
    blackmetal=('#2F3236', 'metal'),  # black taps and legs
    fabric=('#E4B8AE', 'fabric'),    # towels (a woven texture: its own draw call)
    towel=('#E4B8AE', 'satin'),      # towels on the vessel sink, kept plain to save a draw call
    dark=('#2F3236', 'satin'),       # drains and seals
    teal=('#4E8A7A', 'gloss'),       # soap and cup accents
    water=('#9EC9DD', 'gloss'),      # the stream from the tap
    # the toilets
    seat=('#EFEFEC', 'gloss'),       # a toilet's seat and lid
    seat_wood=('#8A5A36', 'wood'),   # the classic toilet's wooden seat and lid
    cistern=('#F6F6F4', 'gloss'),    # a cistern on the bowl
    panel=('#E4E3DF', 'satin'),      # the smart toilet's boxed-in cistern
    boxing=('#C9C6C0', 'matte'),     # the wall-hung toilet's boxed-in cistern
    screen=('#2A2C30', 'gloss'),     # flush plates and the smart toilet's display
    led=('#5EC28B', 'gloss'),        # the smart toilet's ready light
    paper=('#FBFBF8', 'matte'),      # toilet rolls
)

# The parts the player may colour, the biggest first (the cabinet, vanity, pedestal, console basin or
# vessel vanity: the one the quick colour row paints). Drains, water, soap and the basins' metal
# insides keep their own material.


def _tube(F, name, pts, r, material, verts=8):
    """A tube through game points: a rod between each pair, a sphere at each joint."""
    bpts = [F.at(*p) for p in pts]
    for i in range(len(bpts) - 1):
        G.rod(f'{name}_{i}', bpts[i], bpts[i + 1], r, material=material, verts=verts)
    for i, p in enumerate(pts[1:-1], 1):
        F.sphere(f'{name}_j{i}', r, p, material, segments=8, rings=5)


def _gooseneck(F, name, x, base_y, z, spout_z, top, material, r=0.022, tip_y=None):
    """A tap: a riser from (x, base_y, z) up to `top`, arcing forward to a downward spout. Returns the
    spout tip's game point (where the water starts)."""
    tip_y = tip_y if tip_y is not None else base_y + 0.16
    pts = [(x, base_y, z),
           (x, top - 0.05, z),
           (x, top, z + (spout_z - z) * 0.35),
           (x, top, z + (spout_z - z) * 0.8),
           (x, top - 0.03, spout_z),
           (x, tip_y + 0.04, spout_z),
           (x, tip_y, spout_z)]
    _tube(F, name, pts, r, material)
    return (x, tip_y, spout_z)


def _bar(F, name, x, y0, y1, z, material, t=0.016, stand=0.018):
    """A vertical bar handle on two stand-offs, proud of the front at game depth `z`."""
    F.box(f'{name}_bar', t, y1 - y0, t, (x, (y0 + y1) / 2, z + stand + t / 2), material, bevel=t * 0.45)
    for i, y in enumerate((y0 + 0.02, y1 - 0.02)):
        F.box(f'{name}_stand{i}', t * 0.8, t * 0.8, stand, (x, y, z + stand / 2), material, bevel=0.003)


def _water(F, spout, floor_y):
    """The stream: a thin cylinder from the spout tip down to the basin, joined with its origin at the
    tip and scaled to nothing along its length, so it is hidden until `grow` runs the clip."""
    x, top_y, z = spout
    length = max(0.06, top_y - floor_y)
    with G.new_parts() as parts:
        F.cyl('water', 0.012, length, (x, (top_y + floor_y) / 2, z), M.water, verts=10)
    obj = G.join(parts, 'water', origin=F.at(x, top_y, z))
    obj.scale[2] = 0.001
    return obj


def _basin(F, name, x, z, floor_y, rim_y, w, d, material=M.porcelain, wall=0.02, segments=28):
    """A solid oval basin: a revolve with thickness, so its inner surface faces up into the bowl."""
    r = w / 2
    profile = [(0, floor_y), (r - wall, floor_y), (r, floor_y + (rim_y - floor_y) * 0.35),
               (r, rim_y), (r - wall, rim_y), (r - wall, floor_y + (rim_y - floor_y) * 0.3),
               (0.02, floor_y + wall)]
    return G.revolve(name, profile, at=F.at(x, 0, z), material=material, segments=segments,
                     scale=(1, d / w, 1))


def _rect_basin(F, name, x, z, w, d, floor_y, rim_y, material, wall=0.02):
    """A rectangular basin shell (floor and four thin walls), filling a rectangular worktop hole."""
    F.box(f'{name}_floor', w, wall, d, (x, floor_y + wall / 2, z), material, bevel=0.008)
    for s in (-1, 1):
        F.box(f'{name}_side{s}', wall, rim_y - floor_y, d, (x + s * (w - wall) / 2, (floor_y + rim_y) / 2, z),
              material, bevel=0.008)
        F.box(f'{name}_end{s}', w - 2 * wall, rim_y - floor_y, wall,
              (x, (floor_y + rim_y) / 2, z + s * (d - wall) / 2), material, bevel=0.008)


# --- the base sink: a shaker cabinet, a worktop with an inset basin and a gooseneck tap ---------------
def sink():
    F = G.GameFrame(shift=0.145)
    w, h, d, t = 0.86, 0.92, 0.61, 0.04
    z0, zt = -0.455, -0.455 + d          # worktop back and front
    zc = zt - 0.045                      # carcass front
    hole = 0.48
    hz0, hz1 = z0 + 0.17, zt - 0.09      # the basin opening
    by = h - 0.17                        # the basin floor
    with G.new_parts() as still:
        F.box('plinth', w - 0.04, 0.09, d - 0.10, (0, 0.045, z0 + (d - 0.10) / 2), M.trim, bevel=0.006)
        F.box('carcass', w - 0.01, by - 0.09 - 0.004, zc - z0, (0, (0.09 + by - 0.004) / 2, (z0 + zc) / 2),
              M.body, bevel=0.01)
        # the cabinet's shoulders beside the basin, from the basin floor up to the worktop
        sw = (w - hole) / 2
        sx = (hole / 2 + w / 2) / 2
        F.box('shoulder_back', w, h - t - by, hz0 - z0, (0, (by + h - t) / 2, (z0 + hz0) / 2), M.body, bevel=0.008)
        F.box('shoulder_front', w, h - t - by, zc - hz1, (0, (by + h - t) / 2, (hz1 + zc) / 2), M.body, bevel=0.008)
        for s in (-1, 1):
            F.box(f'shoulder{s}', sw, h - t - by, hz1 - hz0, (s * sx, (by + h - t) / 2, (hz0 + hz1) / 2),
                  M.body, bevel=0.008)
        yt = h - t / 2
        F.box('top_back', w, t, hz0 - z0, (0, yt, (z0 + hz0) / 2), M.stone, bevel=0.008)
        F.box('top_front', w, t, zt - hz1, (0, yt, (hz1 + zt) / 2), M.stone, bevel=0.008)
        F.box('top_left', sw, t, hz1 - hz0, (-sx, yt, (hz0 + hz1) / 2), M.stone, bevel=0.008)
        F.box('top_right', sw, t, hz1 - hz0, (sx, yt, (hz0 + hz1) / 2), M.stone, bevel=0.008)
        _rect_basin(F, 'basin', 0, (hz0 + hz1) / 2, hole + 0.02, (hz1 - hz0) + 0.02, by, h - t, M.metal)
        F.cyl('drain', 0.035, 0.012, (0, by + 0.006, (hz0 + hz1) / 2 + 0.03), M.dark, verts=12)
        dw = (w - 0.03) / 2
        dy0, dy1 = 0.11, h - t - 0.02
        for s in (-1, 1):
            cx = s * (dw / 2 + 0.005)
            F.box(f'door{s}', dw, dy1 - dy0, 0.02, (cx, (dy0 + dy1) / 2, zc + 0.01), M.body, bevel=0.006)
            F.box(f'door_panel{s}', dw - 0.1, dy1 - dy0 - 0.1, 0.012, (cx, (dy0 + dy1) / 2, zc + 0.022),
                  M.body, bevel=0.004)
            _bar(F, f'handle{s}', cx - s * (dw / 2 - 0.05), dy1 - 0.2, dy1 - 0.06, zc + 0.02, M.chrome)
        F.cyl('tap_base', 0.045, 0.03, (0, h + 0.015, -0.37), M.chrome, verts=14)
        spout = _gooseneck(F, 'tap', 0, h, -0.37, -0.23, h + 0.32, M.chrome)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, by)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- pedestal sink: a tapered column and a round basin on a tap deck -----------------------------------
def sinkPedestal():
    F = G.GameFrame(shift=0.222)
    z = -0.22
    with G.new_parts() as still:
        G.revolve('column', [(0, 0), (0.12, 0), (0.11, 0.05), (0.09, 0.30), (0.08, 0.55),
                             (0.11, 0.66), (0.13, 0.70), (0, 0.70)], at=F.at(0, 0, z), material=M.body,
                  segments=24, scale=(1, 0.83, 1))
        _basin(F, 'basin', 0, z, 0.70, 0.88, 0.56, 0.46, material=M.porcelain)
        G.cylinder('basin_floor', 0.20, 0.012, at=F.at(0, 0.735, z), material=M.metal, verts=20,
                   scale=(1, 0.82, 1))
        F.cyl('drain', 0.03, 0.012, (0, 0.745, z), M.dark, verts=12)
        F.box('deck', 0.36, 0.05, 0.16, (0, 0.87, -0.38), M.porcelain, bevel=0.012)
        spout = _gooseneck(F, 'tap', 0, 0.88, -0.36, -0.28, 1.04, M.chrome, r=0.018, tip_y=1.0)
        F.cyl('soap', 0.035, 0.02, (0.13, 0.885, -0.37), M.teal, verts=12)
        F.cyl('cup', 0.04, 0.09, (-0.13, 0.92, -0.37), M.porcelain, verts=14)
        F.cyl('brush', 0.008, 0.11, (-0.13, 1.0, -0.37), M.brass, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, 0.745)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- double vanity: a two-basin cabinet ----------------------------------------------------------------
# No mirrors: the player hangs the ones they like above it (the wall mirror, the mirror cabinet).


def vanityDouble():
    F = G.GameFrame(shift=0.152)
    with G.new_parts() as still:
        F.box('plinth', 1.86, 0.06, 0.5, (0, 0.03, -0.17), M.trim, bevel=0.006)
        F.box('cabinet', 1.86, 0.78, 0.52, (0, 0.45, -0.17), M.body, bevel=0.012)
        F.box('top', 1.9, 0.04, 0.56, (0, 0.86, -0.15), M.stone, bevel=0.008)
        for i, x in enumerate((-0.68, -0.23, 0.23, 0.68)):
            F.box(f'door{i}', 0.43, 0.62, 0.02, (x, 0.44, 0.1), M.body, bevel=0.008)
            F.box(f'door_panel{i}', 0.33, 0.52, 0.012, (x, 0.44, 0.112), M.body, bevel=0.004)
            _bar(F, f'handle{i}', x + (0.16 if i < 2 else -0.16), 0.34, 0.56, 0.1, M.chrome, stand=0.016)
        for s in (-1, 1):
            x = s * 0.48
            _basin(F, f'basin{s}', x, -0.12, 0.88, 1.0, 0.5, 0.38, material=M.porcelain, wall=0.018)
            F.cyl(f'drain{s}', 0.03, 0.012, (x, 0.895, -0.1), M.dark, verts=12)
            F.cyl(f'tap_base{s}', 0.04, 0.03, (x, 0.9, -0.35), M.chrome, verts=14)
            _gooseneck(F, f'tap{s}', x, 0.9, -0.35, -0.22, 1.1, M.chrome, tip_y=1.04)
    G.join(still, 'body', origin=(0, 0, 0))
    waters = [_water(F, (x, 1.04, -0.22), 0.895) for x in (-0.48, 0.48)]
    G.settle()
    for water in waters:
        G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- vessel sink: a bowl on a painted vanity, towels on the shelf below --------------------------------
def sinkVessel():
    F = G.GameFrame(shift=0.2)
    with G.new_parts() as still:
        F.box('vanity', 0.8, 0.52, 0.5, (0, 0.56, -0.21), M.body, bevel=0.012)
        F.box('top', 0.84, 0.04, 0.54, (0, 0.84, -0.2), M.stone, bevel=0.008)
        F.box('shelf', 0.7, 0.025, 0.42, (0, 0.18, -0.21), M.wood, bevel=0.004)
        for x in (-0.38, 0.38):
            for z in (-0.43, 0.01):
                F.box(f'leg{x}{z}', 0.025, 0.18, 0.025, (x, 0.09, z), M.wood, bevel=0.003)
        _basin(F, 'bowl', 0, -0.18, 0.86, 1.02, 0.44, 0.36, material=M.porcelain, wall=0.018)
        F.cyl('drain', 0.028, 0.012, (0, 0.875, -0.18), M.dark, verts=12)
        F.cyl('tap_base', 0.035, 0.03, (0, 0.86, -0.42), M.blackmetal, verts=12)
        spout = _gooseneck(F, 'tap', 0, 0.86, -0.42, -0.26, 1.14, M.blackmetal, r=0.02, tip_y=1.08)
        F.box('towel0', 0.36, 0.06, 0.24, (-0.15, 0.225, -0.06), M.towel, bevel=0.02)
        F.box('towel1', 0.36, 0.06, 0.24, (0.2, 0.225, -0.06), M.towel, bevel=0.02)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, 0.875)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- console sink: a basin on brass legs, a towel bar between them --------------------------------------
def sinkConsole():
    F = G.GameFrame(shift=0.2)
    z = -0.21
    with G.new_parts() as still:
        F.box('basin', 0.6, 0.12, 0.46, (0, 0.82, z), M.body, bevel=0.04)
        F.box('basin_inner', 0.48, 0.04, 0.32, (0, 0.865, -0.18), M.metal, bevel=0.04)
        F.cyl('drain', 0.028, 0.012, (0, 0.875, -0.18), M.dark, verts=12)
        for s in (-1, 1):
            F.cyl(f'leg{s}', 0.03, 0.76, (s * 0.27, 0.38, 0), M.brass, verts=10)
            G.rod(f'leg_arm{s}', F.at(s * 0.27, 0.76, 0), F.at(s * 0.27, 0.8, -0.2), 0.028, material=M.brass,
                  verts=10)
        F.cyl('bar', 0.02, 0.54, (0, 0.6, 0), M.brass, rot=(0, 90, 0), verts=8)
        F.box('towel', 0.4, 0.3, 0.03, (0, 0.45, 0.03), M.fabric, bevel=0.01)
        F.cyl('tap_base', 0.035, 0.03, (0, 0.88, -0.4), M.brass, verts=12)
        spout = _gooseneck(F, 'tap', 0, 0.88, -0.4, -0.26, 1.04, M.brass, r=0.02, tip_y=0.98)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, 0.875)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- the toilets: a bowl, a seat and a cistern; the lid lifts while a resident uses it ----------------------
# Built where the placeholder stands, the cistern against the wall face (game depth -0.43), not centred:
# the game sits the resident at the footprint's middle (characters.ts SEATS, the content's `seat`), so the
# bowl must stay where the seat is. The seat's top is the seat height. The lid is the moving part, hinged
# at its back edge; `lid_up` lifts it 100° to lean on the cistern (or over the smart seat's housing).
WALL = -0.43


def _bowl(F, zb, base_y, rim, r, ratio, water_y):
    """An oval bowl, `r` wide (half) and `ratio` times that deep: a revolve with thickness, so its inside
    faces up from the rim down to the water."""
    h = rim - base_y
    profile = [(0, base_y), (r * 0.5, base_y), (r * 0.78, base_y + h * 0.3), (r * 0.95, base_y + h * 0.7),
               (r, rim - 0.015), (r - 0.006, rim), (r - 0.03, rim), (r - 0.045, rim - 0.025),
               (r * 0.62, rim - h * 0.45), (r * 0.36, water_y + 0.01), (r * 0.3, water_y), (0, water_y)]
    G.revolve('bowl', profile, at=F.at(0, 0, zb), material=M.porcelain, segments=36, scale=(1, ratio, 1))
    G.cylinder('bowl_water', r * 0.36, 0.004, at=F.at(0, water_y + 0.006, zb), material=M.water, verts=24,
               scale=(1, ratio, 1))


def _foot(F, z, top, r, ratio, flare=1.0):
    """The bowl's foot on the floor, up into the bowl's underside (`flare` widens its plinth)."""
    G.revolve('foot', [(0, 0), (r * flare, 0), (r * flare, 0.015), (r * (0.9 + 0.06 * flare), 0.04),
                       (r * 0.82, top * 0.5), (r * 0.88, top * 0.85), (r * 1.1, top), (0, top)],
              at=F.at(0, 0, z), material=M.porcelain, segments=32, scale=(1, ratio, 1))


def _column(F, name, y0, y1, z0, z1, w=0.26):
    """The porcelain behind the bowl (the deck a cistern stands on, a wall-hung bowl's fixing): an oval
    column from game depth `z0` to `z1`, `w` wide, rounded on its edges."""
    d = z1 - z0
    G.cylinder(name, w / 2, y1 - y0, at=F.at(0, (y0 + y1) / 2, (z0 + z1) / 2), material=M.porcelain,
               scale=(1, d / w, 1), verts=28, bevel=0.03, bevel_segments=3)


def _seat(F, zb, y, r, ratio, material, t=0.024):
    """The seat ring on the rim, from `y` up `t`, its outside `r` (half) wide."""
    m = 0.034
    ring = G.torus('seat', r - m, m, at=F.at(0, y + t / 2, zb), material=material, major_segments=36,
                   minor_segments=10)
    ring.scale = (1, ratio, t / (2 * m))


def _lid(F, zb, y, r, ratio, material, t=0.02, pivot='bottom'):
    """The lid over the seat, from `y` up `t`, hinged across its back edge: on its underside, or on its top
    (`pivot='top'`, where it lifts over a housing behind it). Returns it, ready to swing about X."""
    with G.new_parts() as parts:
        G.cylinder('lid', r, t, at=F.at(0, y + t / 2, zb), material=material, scale=(1, ratio, 1), verts=36,
                   bevel=0.008)
    lid = G.join(parts, 'lid')
    return G.hinged(lid, F.at(0, y + (t if pivot == 'top' else 0), zb - r * ratio))


def _hinges(F, z, y, material, x=0.09):
    """The seat's two hinge posts on the bowl's back deck."""
    for s in (-1, 1):
        F.cyl(f'hinge{s}', 0.014, 0.03, (s * x, y + 0.008, z), material, rot=(0, 90, 0), verts=10)


def _roll(F, x, y, z, side=1):
    """A toilet roll on a chrome arm out from the wall (or the panel) at game depth `z`; `side` is the
    end the arm comes from (the roll runs along x)."""
    F.box('roll_plate', 0.012, 0.05, 0.03, (x - side * 0.06, y, z + 0.015), M.chrome, bevel=0.004)
    G.rod('roll_arm', F.at(x - side * 0.06, y, z + 0.02), F.at(x - side * 0.06, y, z + 0.065), 0.006,
          material=M.chrome, verts=8)
    G.rod('roll_rod', F.at(x - side * 0.06, y, z + 0.065), F.at(x + side * 0.055, y, z + 0.065), 0.006,
          material=M.chrome, verts=8)
    F.cyl('roll', 0.055, 0.1, (x, y, z + 0.065), M.paper, rot=(0, 90, 0), verts=16)
    F.cyl('roll_core', 0.02, 0.102, (x, y, z + 0.065), M.dark, rot=(0, 90, 0), verts=10)


def _close_coupled(F, zb, rim, r, ratio, cz, cw, ch, seat_material, foot_flare=1.0):
    """The bowl on its foot, the back deck from the bowl to the wall, the seat, and a cistern `cw` wide,
    `ch` tall, from the wall to game depth `cz`, standing on the deck."""
    _foot(F, zb - 0.06, 0.2, 0.115, 1.3, flare=foot_flare)
    _bowl(F, zb, 0.17, rim, r, ratio, water_y=0.27)
    back = zb - r * ratio                       # the seat's back edge, where the lid hinges
    _column(F, 'deck', 0.16, rim, WALL, back + 0.04)
    _seat(F, zb, rim, r - 0.002, ratio, seat_material)
    _hinges(F, back + 0.012, rim, M.chrome)
    F.box('cistern', cw, ch, cz - WALL, (0, rim + ch / 2, (WALL + cz) / 2), M.cistern, bevel=0.03)


def toilet():
    F = G.GameFrame()
    r, ratio, rim = 0.18, 1.2, 0.405           # the seat's top at 0.43 m (SEATS)
    zb = -0.186 + r * ratio                    # the lid hinges 9.4 cm in front of the cistern
    with G.new_parts() as still:
        _close_coupled(F, zb, rim, r, ratio, -0.28, 0.42, 0.38, M.seat)
        F.box('cistern_lid', 0.44, 0.03, 0.155, (0, rim + 0.395, WALL + 0.0775), M.cistern, bevel=0.012)
        F.cyl('lever_boss', 0.018, 0.012, (-0.13, 0.72, -0.274), M.chrome, rot=(90, 0, 0), verts=12)
        F.box('lever', 0.075, 0.014, 0.012, (-0.1, 0.72, -0.266), M.chrome, bevel=0.005)
    G.join(still, 'body', origin=(0, 0, 0))
    lid = _lid(F, zb, rim + 0.024, r, ratio, M.seat)
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.6, axis='X')
    return G.meshes()


def toiletCompact():
    F = G.GameFrame()
    r, ratio, rim = 0.17, 1.22, 0.405
    zb = -0.214 + r * ratio                    # a slim cistern, the lid hinged 8.6 cm in front of it
    with G.new_parts() as still:
        _close_coupled(F, zb, rim, r, ratio, -0.30, 0.4, 0.37, M.seat)
        F.box('cistern_lid', 0.41, 0.025, 0.13, (0, rim + 0.382, WALL + 0.065), M.cistern, bevel=0.01)
        F.cyl('button', 0.032, 0.012, (0, rim + 0.398, -0.365), M.chrome, verts=16)
        _roll(F, 0.33, 0.62, WALL)
    G.join(still, 'body', origin=(0, 0, 0))
    lid = _lid(F, zb, rim + 0.024, r, ratio, M.seat)
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.6, axis='X')
    return G.meshes()


def toiletClassic():
    F = G.GameFrame()
    r, ratio, rim = 0.18, 1.2, 0.425           # the wooden seat's top at 0.45 m (content `seat`)
    zb = -0.15 + r * ratio
    with G.new_parts() as still:
        _close_coupled(F, zb, rim, r, ratio, -0.25, 0.44, 0.4, M.seat_wood, foot_flare=1.25)
        # a moulded two-step lid and a lever with a ball end
        F.box('cistern_lid', 0.47, 0.03, 0.185, (0, rim + 0.415, WALL + 0.0925), M.cistern, bevel=0.01)
        F.box('cistern_cap', 0.43, 0.02, 0.15, (0, rim + 0.44, WALL + 0.075), M.cistern, bevel=0.008)
        F.box('cistern_band', 0.445, 0.03, 0.005, (0, rim + 0.06, -0.248), M.cistern, bevel=0.002)
        F.cyl('lever_boss', 0.022, 0.014, (-0.15, 0.76, -0.243), M.chrome, rot=(90, 0, 0), verts=12)
        G.rod('lever', F.at(-0.15, 0.76, -0.236), F.at(-0.07, 0.745, -0.236), 0.007, material=M.chrome, verts=8)
        F.sphere('lever_end', 0.014, (-0.065, 0.745, -0.236), M.chrome, segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    lid = _lid(F, zb, rim + 0.024, r, ratio, M.seat_wood)
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.6, axis='X')
    return G.meshes()


def _wall_hung(F, zb, rim, r, ratio, panel_z):
    """A bowl off the floor, fixed to the boxed-in cistern at `panel_z` by a block behind it."""
    _bowl(F, zb, 0.21, rim, r, ratio, water_y=0.3)
    _column(F, 'neck', 0.25, rim - 0.004, panel_z - 0.02, zb - r * ratio * 0.45)


def toiletSmart():
    F = G.GameFrame()
    r, ratio, rim = 0.19, 1.22, 0.41           # the seat's top at 0.44 m (SEATS)
    panel_z = -0.28
    housing = -0.18                            # the seat's housing (the bidet) from the panel to here
    zb = housing + r * ratio
    with G.new_parts() as still:
        F.box('panel', 0.9, 1.2, panel_z - WALL, (0, 0.6, (WALL + panel_z) / 2), M.panel, bevel=0.01)
        F.box('display', 0.2, 0.12, 0.012, (0, 1.0, panel_z + 0.006), M.screen, bevel=0.004)
        F.sphere('ready', 0.008, (0.06, 0.98, panel_z + 0.013), M.led, segments=8, rings=5)
        F.box('remote', 0.06, 0.15, 0.024, (0.34, 0.72, panel_z + 0.012), M.screen, bevel=0.008)
        _roll(F, -0.34, 0.8, panel_z, side=-1)
        _wall_hung(F, zb, rim, r, ratio, panel_z)
        _seat(F, zb, rim, r - 0.002, ratio, M.seat, t=0.03)
        F.box('housing', 0.4, 0.055, housing - panel_z + 0.01, (0, rim + 0.0275, (panel_z + housing) / 2 - 0.005),
              M.seat, bevel=0.02)
    G.join(still, 'body', origin=(0, 0, 0))
    # the lid is level with the housing's top and hinged on its front edge, so it lifts over it
    lid = _lid(F, zb, rim + 0.03, r, ratio, M.seat, t=0.025, pivot='top')
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.8, axis='X')
    return G.meshes()


def toiletWallHung():
    F = G.GameFrame()
    r, ratio, rim = 0.18, 1.25, 0.42           # the seat's top at 0.445 m (content `seat` 0.45)
    panel_z = -0.31
    zb = panel_z + 0.095 + r * ratio
    with G.new_parts() as still:
        F.box('panel', 0.6, 1.1, panel_z - WALL, (0, 0.55, (WALL + panel_z) / 2), M.boxing, bevel=0.01)
        F.box('flush_plate', 0.22, 0.15, 0.012, (0, 1.0, panel_z + 0.006), M.screen, bevel=0.005)
        for s in (-1, 1):
            F.box(f'flush{s}', 0.09, 0.12, 0.008, (s * 0.05, 1.0, panel_z + 0.014), M.chrome, bevel=0.004)
        _wall_hung(F, zb, rim, r, ratio, panel_z)
        back = zb - r * ratio
        F.box('deck', 0.24, 0.03, back - panel_z + 0.02, (0, rim - 0.015, (panel_z + back) / 2), M.porcelain,
              bevel=0.012)
        _seat(F, zb, rim, r - 0.002, ratio, M.seat)
        _hinges(F, back + 0.012, rim, M.chrome)
    G.join(still, 'body', origin=(0, 0, 0))
    lid = _lid(F, zb, rim + 0.024, r, ratio, M.seat)
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.6, axis='X')
    return G.meshes()


TOILET = {'uses': {'toilet': 'lid_up'}}

FAMILY = G.Family(
    'bathroom', M,
    [G.Model('sink', sink, size=(0.86, 1.262, 0.621), key='model.sink',
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('stone', 'Worktop'), ('metal', 'Basin'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')]),
     G.Model('sinkPedestal', sinkPedestal, size=(0.56, 1.060, 0.464), key='model.sinkPedestal',
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Pedestal'), ('porcelain', 'Basin & cup'), ('chrome', 'Tap')]),
     G.Model('vanityDouble', vanityDouble, size=(1.90, 1.122, 0.562), key='model.vanityDouble',
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('stone', 'Worktop'), ('porcelain', 'Basins'), ('chrome', 'Taps & handles'),
                    ('trim', 'Plinth')]),
     G.Model('sinkVessel', sinkVessel, size=(0.84, 1.165, 0.540), key='model.sinkVessel',
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Vanity'), ('stone', 'Worktop'), ('porcelain', 'Bowl'), ('blackmetal', 'Tap'),
                    ('wood', 'Shelf & legs'), ('towel', 'Towels')]),
     G.Model('sinkConsole', sinkConsole, size=(0.60, 1.062, 0.480), key='model.sinkConsole',
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Basin'), ('brass', 'Legs & tap'), ('fabric', 'Towel')]),
     G.Model('toilet', toilet, size=(0.44, 0.815, 0.676), key='model.toilet', centre=(0, -0.092),
             motions=TOILET, paint=[('porcelain', 'Bowl'), ('seat', 'Seat & lid'), ('cistern', 'Cistern')]),
     G.Model('toiletCompact', toiletCompact, size=(0.59, 0.809, 0.631), key='model.toiletCompact',
             centre=(0.09, -0.1145), motions=TOILET,
             paint=[('porcelain', 'Bowl'), ('seat', 'Seat & lid'), ('cistern', 'Cistern')]),
     G.Model('toiletSmart', toiletSmart, size=(0.9, 1.2, 0.714), key='model.toiletSmart', centre=(0, -0.073),
             motions=TOILET, paint=[('porcelain', 'Bowl'), ('seat', 'Seat & lid'), ('panel', 'Cistern')]),
     G.Model('toiletClassic', toiletClassic, size=(0.47, 0.875, 0.712), key='model.toiletClassic',
             centre=(0, -0.074), motions=TOILET,
             paint=[('porcelain', 'Bowl'), ('seat_wood', 'Seat & lid'), ('cistern', 'Cistern')]),
     G.Model('toiletWallHung', toiletWallHung, size=(0.6, 1.1, 0.665), key='model.toiletWallHung',
             centre=(0, -0.0975), motions=TOILET,
             paint=[('porcelain', 'Bowl'), ('seat', 'Seat & lid'), ('boxing', 'Cistern')])],
    comment='Blender-built bathroom models (tools/art/blender/families/bathroom.py). Replaces the '
            'primitive-part placeholders by key; listed in the base manifest\'s packs. The placeholder is '
            'kept as a fallback if a file fails to load, and the materials map onto the game\'s finishes so '
            'a model costs one draw call per finish group. Each sink has a `water_on` clip (the stream from '
            'the tap) played while a resident washes.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
