# The bathroom sinks: the base sink (`model.sink`), the pedestal (`model.sinkPedestal`), the double
# vanity (`model.vanityDouble`), the vessel sink (`model.sinkVessel`) and the console sink
# (`model.sinkConsole`); and the five toilets (`model.toilet`, `model.toiletCompact`, `model.toiletSmart`,
# `model.toiletClassic`, `model.toiletWallHung`) and the four showers (`model.shower`, `model.showerStall`,
# `model.showerWalkIn`, `model.showerRain`) and the three baths (`model.bathtub`, `model.bathtubClawfoot`,
# `model.bathtubFreestanding`). Notes: bathroom.md.
#
# Each sink is one still body (cabinet / basin / pedestal / legs) plus one moving part: a stream of
# water (`water`) that grows from the tap's spout into the basin while a resident washes (`motions`).
# Each toilet is one still body plus its lid, which lifts while a resident uses it; each shower one still
# body plus its rain, which grows down from the head while a resident showers (and the base shower's sliding
# glass door, the stall's curtain front, each a part of its own); each bath one still body plus
# its tap's stream and its water, which fills while a resident bathes.
# Built to the old part's size, in its numbers: the parts are placed in game axes (`F`).
#
# Run:  cargo art bathroom [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402
import gamekit as G  # noqa: E402
from gamekit.motion import _clip, last_frame  # noqa: E402
from gamekit.shapes import _recalc_normals  # noqa: E402
from mathutils import Vector  # noqa: E402

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
    # the showers
    tray=('#F4F4F2', 'gloss'),       # a shower tray
    tile=('#E4ECEC', 'satin'),       # wall tiles
    grout=('#C8D3D4', 'satin'),      # the grout between them
    glass=('#DCE6E8', 'glass'),      # screens and doors (see-through: its own draw call)
    grip=('#7E9096', 'glass'),       # the shower door's grip, smoked glass (drawn with the door's glass)
    frame=('#D0D3D6', 'chrome'),     # a screen's frame, a curtain rail
    fittings=('#D0D3D6', 'chrome'),  # heads, risers and mixers
    frame_dark=('#2B2C2E', 'metal'),     # a black screen frame
    fittings_dark=('#2B2C2E', 'metal'),  # a black rain head and mixer
    curtain=('#CFE0E4', 'satin'),    # the stall's curtain: polyester, plain (the moving front costs no extra draw)
    slate=('#C9C6C0', 'matte'),      # the walk-in shower's stone floor
    slate_wall=('#C9C6C0', 'matte'),  # and its stone wall
    groove=('#AFACA6', 'matte'),     # the joints in the stone wall
    tray_stone=('#E8E6E1', 'matte'),  # the rain shower's stone tray
    wall_stone=('#D7D3CC', 'matte'),  # and its walls
    bottle=('#F2F1EE', 'gloss'),     # shampoo bottles
    sage=('#9DAF8C', 'gloss'),
    # the baths
    bath_panel=('#ECECE8', 'gloss'),  # the built-in bath's front and end panels
    feet=('#B8925A', 'metal'),       # the clawfoot tub's feet
    duck=('#F2C14E', 'gloss'),       # a rubber duck
    beak=('#E9853B', 'gloss'),
    # the bathroom storage and mirrors
    cab=('#F2F1EE', 'satin'),        # a bathroom cabinet's carcass
    cab_door=('#E4E3DF', 'satin'),   # its doors and the mirror's frame
    mirror=('#CFDCE2', 'gloss', 'front'),  # mirror glass: reflects the room live on High, plain gloss elsewhere
    bulb=('#FFF1CF', 'gloss'),       # the mirror cabinet's light strip
    frame_brass=('#B8925A', 'metal'),  # the round mirror's frame and hook
    leather=('#6B4A33', 'satin'),    # its strap
    towel_blue=('#9DC6CF', 'satin'),
    towel_cream=('#F2F0EA', 'satin'),
)

# The parts the player may colour, the biggest first (the cabinet, vanity, pedestal, console basin or
# vessel vanity: the one the quick colour row paints). Drains, water, soap and the basins' metal
# insides keep their own material.


def _tube(F, name, pts, r, material, verts=8, round=3, scale=1.0):
    """A tube in one piece through game points, its bends rounded (`G.kit.tube`)."""
    return G.kit.tube(F, name, pts, r, material, verts=verts, round=round, scale=scale)


def _loop(F, cx, cz, y, rx, rz, verts=24, square=2.0, egg=0.0, back=None):
    """A closed loop of Blender points round game (cx, cz) at height `y`: `rx` across, `rz` to the front (`back`
    behind, if given), squarer towards a rounded rectangle as `square` grows, narrower to the front by `egg`."""
    out = []
    e = 2.0 / square
    for i in range(verts):
        a = math.tau * i / verts
        c, s = math.cos(a), math.sin(a)
        u, v = math.copysign(abs(c) ** e, c), math.copysign(abs(s) ** e, s)
        dz = v * (rz if v >= 0 or back is None else back)
        out.append(F.at(cx + u * rx * (1 - egg * v), y, cz + dz))
    return out


def _lathe(F, name, cx, cz, rings, material, verts=24, square=2.0, egg=0.0, caps=True, angle=40):
    """A body lofted through game rings [(y, rx, rz[, square[, back[, dz]]]), ...] round (cx, cz) (each moved `dz` to
    the front), in profile order: a
    basin's shell (up the outside, over the rim, down the inside to its floor), a cistern, a bowl, a bottle."""
    sections = []
    for ring in rings:
        y, rx, rz = ring[:3]
        sq = ring[3] if len(ring) > 3 else square
        back = ring[4] if len(ring) > 4 else None
        dz = ring[5] if len(ring) > 5 else 0.0
        sections.append(_loop(F, cx, cz + dz, y, rx, rz, verts, sq, egg, back))
    return G.loft(name, sections, material=material, caps=caps, angle=angle)


def _rose(F, name, x, y, z, r, material, h=0.012, verts=16):
    """A tap's or a fitting's round base plate, domed: from `r` on the deck at height `y` up `h`."""
    return _lathe(F, name, x, z, [(y, r, r), (y + h * 0.45, r * 0.97, r * 0.97), (y + h, r * 0.62, r * 0.62)],
                  material, verts=verts, angle=60)


def _gooseneck(F, name, x, base_y, z, spout_z, top, material, r=0.018, tip_y=None, lever=True):
    """A deck tap in one piece: a riser from (x, base_y, z) up and over in a half circle whose top is `top`, down to
    a spout at depth `spout_z` with an aerator; a domed rose under it and a side lever. Returns the spout tip's
    game point (where the water starts)."""
    tip_y = tip_y if tip_y is not None else base_y + 0.16
    R = (spout_z - z) / 2
    cz, cy = z + R, top - R
    end = min(tip_y, cy)
    arc = [(x, cy + R * math.sin(math.pi * k / 8), cz - R * math.cos(math.pi * k / 8)) for k in range(9)]
    pts = [(x, base_y, z), (x, cy - 0.04, z)] + arc + ([(x, end, spout_z)] if end < cy - 0.01 else [])
    _tube(F, name, pts, r, material, verts=10, round=2)
    F.cyl(f'{name}_aerator', r * 1.12, 0.02, (x, end + 0.006, spout_z), material, verts=10)
    _rose(F, f'{name}_rose', x, base_y, z, r * 2.2, material)
    if lever:
        ly = base_y + 0.07
        _tube(F, f'{name}_lever', [(x - r * 0.5, ly, z), (x - r - 0.022, ly + 0.004, z), (x - r - 0.028, ly + 0.05, z - 0.012)],
              0.0065, material, verts=8, round=2, scale=[1.3, 1.0, 0.75])
    return (x, end - 0.014, spout_z)


def _pull(F, name, x, y, z, length, material, vertical=True, r=0.0065, stand=0.024):
    """A bar pull in one piece, its middle at game (x, y) on a face at depth `z`: out on rounded legs, along
    `length` (up, or across), and back in."""
    a, b = -length / 2, length / 2
    if vertical:
        def p(t, out):
            return (x, y + t, z + out)
    else:
        def p(t, out):
            return (x + t, y, z + out)
    pts = [p(a, -0.002), p(a, stand - 0.009), p(a + 0.009, stand), p(b - 0.009, stand), p(b, stand - 0.009), p(b, -0.002)]
    return _tube(F, name, pts, r, material, verts=8, round=2)


def _shaker(F, name, w, h, x, y, z, material, t=0.02, rail=0.06):
    """A shaker door or drawer front, its back on game depth `z`: a frame (an outline with a hole, its edges eased)
    round a panel set 6 mm back inside it."""
    F.extrude(name, G.rounded_rect(w, h, 0.004, corner=1), t, (x, y, z + t / 2), material,
              holes=[G.rounded_rect(w - 2 * rail, h - 2 * rail, 0.003, corner=1)], bevel=0.003, bevel_segments=1)
    pw, ph = w - 2 * rail + 0.004, h - 2 * rail + 0.004
    _panel(F, f'{name}_panel', pw, ph, x, y, z, t - 0.006, material, band=min(0.025, pw / 5, ph / 5))


def _panel(F, name, w, h, x, y, z, t, material, band=0.03):
    """A flat panel set in a frame, its back on game depth `z`: a band round its edge and its middle, so the shading
    baked into its vertices darkens only the band in the frame's crease, not the whole face."""
    inner = G.rounded_rect(w - 2 * band, h - 2 * band, 0.002, corner=1)
    F.extrude(f'{name}_band', G.rounded_rect(w, h, 0.002, corner=1), t, (x, y, z + t / 2), material, holes=[inner])
    F.extrude(name, inner, t, (x, y, z + t / 2), material)


def _bottle(F, name, foot, r, h, material, cap, verts=12, pump=False):
    """A bottle standing at game point `foot`: a round body, a shoulder, a neck and its cap (or a pump)."""
    x, y, z = foot
    _lathe(F, name, x, z, [(y, r * 0.9, r * 0.9), (y + h * 0.03, r, r), (y + h * 0.7, r, r), (y + h * 0.8, r * 0.6, r * 0.6),
                           (y + h * 0.86, r * 0.4, r * 0.4), (y + h * 0.9, r * 0.4, r * 0.4)], material, verts=verts)
    if pump:
        F.cyl(f'{name}_pump', r * 0.45, h * 0.1, (x, y + h * 0.95, z), cap, verts=8)
        F.box(f'{name}_nozzle', 0.008, 0.01, r * 1.3, (x, y + h, z + r * 0.5), cap, bevel=0.003, bevel_segments=1)
    else:
        F.cyl(f'{name}_cap', r * 0.48, h * 0.1, (x, y + h * 0.95, z), cap, verts=verts)


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


def _drain(F, x, y, z, r=0.03, material=None):
    """A plug's flange (chrome, or the tap's metal) and its dark hole, on a basin's floor at height `y`."""
    _rose(F, 'drain_flange', x, y - 0.002, z, r, material or M.chrome, h=0.006, verts=16)
    F.cyl('drain', r * 0.55, 0.004, (x, y + 0.003, z), M.dark, verts=12)


def _bowl_shell(F, name, x, z, floor_y, rim_y, w, d, material, wall=0.018, foot=0.5, belly=1.0, verts=32, square=2.0):
    """A vessel bowl in one shell (up the outside from its foot, over a rounded rim, down the inside to a floor),
    `w` × `d` across the rim, standing on `floor_y`. `foot` is the foot's share of the width, `belly` how full
    its sides are."""
    rx, rz, h = w / 2, d / 2, rim_y - floor_y
    k = rz / rx

    def ring(y, r, sq=square):
        return (y, r, r * k, sq)
    return _lathe(F, name, x, z, [
        ring(floor_y, rx * foot), ring(floor_y + h * 0.12, rx * (foot + 0.18 * belly)),
        ring(floor_y + h * 0.45, rx * (0.86 + 0.06 * belly)), ring(floor_y + h * 0.8, rx * 0.985),
        ring(rim_y - 0.004, rx), ring(rim_y, rx - wall * 0.35), ring(rim_y - 0.002, rx - wall),
        ring(floor_y + h * 0.6, rx * (0.92 + 0.03 * belly) - wall), ring(floor_y + h * 0.25, rx * 0.72 - wall),
        ring(floor_y + wall + 0.006, rx * 0.4), ring(floor_y + wall, rx * 0.25)], material, verts=verts, angle=55)


# --- the base sink: a shaker cabinet, a worktop with an undermount basin and a gooseneck tap -------------
def sink():
    F = G.GameFrame(shift=0.145)
    w, h, d, t = 0.86, 0.92, 0.61, 0.04
    z0, zt = -0.455, -0.455 + d          # worktop back and front
    zc = zt - 0.045                      # carcass front
    hole, hd = 0.48, 0.35
    hz0, hz1 = z0 + 0.17, zt - 0.09      # the basin opening
    bz = (hz0 + hz1) / 2
    by = h - 0.17                        # the basin floor
    with G.new_parts() as still:
        # the carcass on a toe kick set back under it
        F.box('plinth', w - 0.04, 0.09, d - 0.10, (0, 0.045, z0 + (d - 0.10) / 2), M.trim, bevel=0.006)
        # (its hidden top 3 cm under the basin's floor: `cargo review` tells a worktop from what stands on it by the
        # faces under it, and a face within 1 cm of the basin's floor made the worktop look like something on it)
        cy = by - 0.03
        F.box('carcass', w - 0.01, cy - 0.09, zc - z0, (0, (0.09 + cy) / 2, (z0 + zc) / 2), M.body, bevel=0.01)
        # the cabinet's sides in one piece up to the worktop, and its shoulders round the basin
        for s in (-1, 1):
            F.box(f'side{s}', 0.018, h - t - 0.09, zc - z0, (s * (w / 2 - 0.013), (0.09 + h - t) / 2, (z0 + zc) / 2), M.body,
                  bevel=0.003, bevel_segments=1)
        sw = (w - hole) / 2
        sx = (hole / 2 + w / 2) / 2
        F.box('shoulder_back', w - 0.01, h - t - cy, hz0 - z0, (0, (cy + h - t) / 2, (z0 + hz0) / 2), M.body)
        F.box('shoulder_front', w - 0.01, h - t - cy, zc - hz1, (0, (cy + h - t) / 2, (hz1 + zc) / 2), M.body)
        for s in (-1, 1):
            F.box(f'shoulder{s}', sw - 0.005, h - t - cy, hz1 - hz0, (s * (sx - 0.0025), (cy + h - t) / 2, (hz0 + hz1) / 2),
                  M.body)
        # the worktop in one slab, its opening's corners rounded, its edges eased
        F.extrude('top', G.rounded_rect(w, d, 0.006, corner=1), t, (0, h - t / 2, (z0 + zt) / 2), M.stone, plane='top',
                  holes=[[(u, v + bz - (z0 + zt) / 2) for u, v in G.rounded_rect(hole, hd, 0.035, corner=3)]],
                  bevel=0.005, bevel_segments=2)
        # the undermount basin: one steel shell, its rim under the worktop, its inside rounding into the floor
        _lathe(F, 'basin', 0, bz, [(by - 0.012, 0.236, 0.166, 6), (h - t - 0.001, 0.262, 0.192, 9),
                                   (h - t - 0.001, 0.25, 0.18, 9), (by + 0.04, 0.246, 0.176, 8),
                                   (by + 0.01, 0.232, 0.162, 6), (by, 0.214, 0.144, 5)], M.metal, verts=40, angle=50)
        _drain(F, 0, by, bz + 0.03, r=0.036)
        # a false drawer front under the worktop and two shaker doors under it, bar pulls on each
        dw = (w - 0.03) / 2
        dy0, dy1 = 0.11, 0.71
        F.soft('false_front', w - 0.02, 0.15, 0.02, (0, 0.795, zc + 0.01), M.body, round=0.004, round_segments=1)
        _pull(F, 'pull_top', 0, 0.795, zc + 0.02, 0.16, M.chrome, vertical=False)
        for s in (-1, 1):
            cx = s * (dw / 2 + 0.005)
            _shaker(F, f'door{s}', dw, dy1 - dy0, cx, (dy0 + dy1) / 2, zc, M.body)
            _pull(F, f'handle{s}', cx - s * (dw / 2 - 0.045), dy1 - 0.11, zc + 0.02, 0.14, M.chrome)
        # the tap, and a soap pump beside it
        spout = _gooseneck(F, 'tap', 0, h, -0.37, -0.23, h + 0.32, M.chrome, r=0.02)
        _bottle(F, 'soap', (0.25, h, -0.36), 0.03, 0.17, M.teal, M.chrome, pump=True)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, by)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- pedestal sink: a tapered column under a basin with a tap deck at its back -----------------------------
def sinkPedestal():
    F = G.GameFrame(shift=0.222)
    z = -0.22
    rim, floor_y = 0.88, 0.735
    bz = z + 0.03                        # the bowl sits forward of the deck
    with G.new_parts() as still:
        # the column: an oval that waists in and flares into its foot and under the basin
        _lathe(F, 'column', 0, z - 0.02, [(0.0, 0.122, 0.1), (0.012, 0.122, 0.1), (0.03, 0.112, 0.092), (0.08, 0.098, 0.08),
                                          (0.3, 0.084, 0.069), (0.52, 0.08, 0.066), (0.62, 0.095, 0.078),
                                          (0.69, 0.14, 0.115)], M.body, verts=28, angle=60)
        # the basin in one shell: up the outside, over a rolled rim, across the flat deck at the back and down
        # the bowl (set forward) to its floor
        _lathe(F, 'basin', 0, z, [(0.66, 0.2, 0.15, 2.4), (0.7, 0.25, 0.195, 2.6), (0.8, 0.274, 0.222, 2.8),
                                  (0.865, 0.28, 0.229, 3.0, 0.233), (rim, 0.276, 0.225, 3.0, 0.229),
                                  (rim, 0.255, 0.2, 3.0, 0.21), (rim, 0.236, 0.16, 2.4, 0.16, bz - z),
                                  (rim - 0.02, 0.228, 0.152, 2.2, 0.15, bz - z), (0.8, 0.19, 0.125, 2.1, 0.12, bz - z),
                                  (0.755, 0.12, 0.08, 2.0, 0.08, bz - z), (floor_y, 0.06, 0.042, 2.0, 0.042, bz - z)],
               M.porcelain, verts=36, angle=50)
        _drain(F, 0, floor_y, bz, r=0.028)
        # an overflow slot in the bowl's front wall
        F.box('overflow', 0.05, 0.012, 0.01, (0, 0.85, bz + 0.147), M.dark, bevel=0.004, bevel_segments=1, rot=(-25, 0, 0))
        spout = _gooseneck(F, 'tap', 0, rim, -0.405, -0.28, 1.04, M.chrome, r=0.018, tip_y=1.0)
        # a soap on its dish, a tooth mug with a brush
        _lathe(F, 'dish', 0.15, -0.408, [(rim, 0.035, 0.026), (rim + 0.008, 0.045, 0.034), (rim + 0.012, 0.043, 0.032),
                                         (rim + 0.006, 0.036, 0.026)], M.porcelain, verts=16, square=3)
        F.soft('soap', 0.06, 0.02, 0.04, (0.15, rim + 0.017, -0.408), M.teal, round=0.008, round_segments=2)
        _lathe(F, 'cup', -0.15, -0.41, [(rim, 0.028, 0.028), (rim + 0.004, 0.032, 0.032), (rim + 0.09, 0.038, 0.038),
                                        (rim + 0.09, 0.034, 0.034), (rim + 0.02, 0.029, 0.029)], M.porcelain, verts=16)
        _tube(F, 'brush', [(-0.15, rim + 0.03, -0.41), (-0.143, rim + 0.12, -0.405), (-0.137, rim + 0.16, -0.4)], 0.006,
              M.teal, verts=6, round=1)
        F.box('brush_head', 0.01, 0.03, 0.014, (-0.136, rim + 0.17, -0.396), M.paper, bevel=0.003, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, floor_y + 0.01)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- double vanity: a two-basin cabinet ----------------------------------------------------------------
# No mirrors: the player hangs the ones they like above it (the wall mirror, the mirror cabinet).


def vanityDouble():
    F = G.GameFrame(shift=0.152)
    with G.new_parts() as still:
        # the worktop at 0.76 m, so the bowls standing on it have their rims at 0.88 (a basin's 0.76-0.91)
        F.box('plinth', 1.82, 0.06, 0.46, (0, 0.03, -0.19), M.trim, bevel=0.006)
        F.box('cabinet', 1.86, 0.66, 0.52, (0, 0.39, -0.17), M.body, bevel=0.006, bevel_segments=1)
        F.soft('top', 1.9, 0.04, 0.56, (0, 0.74, -0.15), M.stone, round=0.008, round_segments=2)
        F.box('upstand', 1.9, 0.08, 0.02, (0, 0.8, -0.42), M.stone, bevel=0.004, bevel_segments=1)
        # two drawers across the top, four shaker doors under them, bar pulls on each
        for s in (-1, 1):
            F.soft(f'drawer{s}', 0.915, 0.085, 0.02, (s * 0.4625, 0.672, 0.1), M.body, round=0.004, round_segments=1)
            _pull(F, f'drawer_pull{s}', s * 0.4625, 0.672, 0.11, 0.2, M.chrome, vertical=False, stand=0.018)
        for i, x in enumerate((-0.6975, -0.2325, 0.2325, 0.6975)):
            _shaker(F, f'door{i}', 0.455, 0.53, x, 0.36, 0.09, M.body)
            _pull(F, f'handle{i}', x + (0.18 if i % 2 == 0 else -0.18), 0.535, 0.11, 0.13, M.chrome, stand=0.018)
        tips = []
        for s in (-1, 1):
            x = s * 0.48
            _bowl_shell(F, f'basin{s}', x, -0.12, 0.76, 0.88, 0.5, 0.38, M.porcelain, wall=0.016, foot=0.46, belly=0.8)
            _drain(F, x, 0.776, -0.12, r=0.026)
            tips.append(_gooseneck(F, f'tap{s}', x, 0.76, -0.35, -0.22, 0.98, M.chrome, r=0.02, tip_y=0.92))
    G.join(still, 'body', origin=(0, 0, 0))
    waters = [_water(F, tip, 0.785) for tip in tips]
    G.settle()
    for water in waters:
        G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- vessel sink: a bowl on a painted vanity on a wooden stand, towels on its shelf ------------------------
def sinkVessel():
    F = G.GameFrame(shift=0.2)
    with G.new_parts() as still:
        # the worktop at 0.76 m, so the bowl on it has its rim at 0.88 (a basin's 0.76-0.91)
        F.box('vanity', 0.8, 0.42, 0.48, (0, 0.51, -0.22), M.body, bevel=0.006, bevel_segments=1)
        F.soft('drawer', 0.78, 0.38, 0.022, (0, 0.51, 0.03), M.body, round=0.006, round_segments=2)
        _pull(F, 'pull', 0, 0.64, 0.041, 0.32, M.blackmetal, vertical=False, r=0.006)
        F.soft('top', 0.84, 0.04, 0.54, (0, 0.74, -0.2), M.stone, round=0.008, round_segments=2)
        # the stand: four tapered legs from the floor up to the vanity, a slatted shelf between them
        for x in (-0.38, 0.38):
            for z in (-0.43, 0.01):
                _tube(F, f'leg{x}{z}', [(x, 0.0, z), (x, 0.305, z)], 0.016, M.wood, verts=8, round=1, scale=[0.7, 1.0])
            F.box(f'rail{x}', 0.03, 0.03, 0.44, (x, 0.18, -0.21), M.wood, bevel=0.004, bevel_segments=1)
        for k in range(5):
            F.box(f'slat{k}', 0.73, 0.02, 0.072, (0, 0.205, -0.385 + 0.088 * k), M.wood, bevel=0.004, bevel_segments=1)
        _bowl_shell(F, 'bowl', 0, -0.18, 0.76, 0.88, 0.44, 0.36, M.porcelain, wall=0.014, foot=0.45, belly=1.0)
        _drain(F, 0, 0.774, -0.18, r=0.026, material=M.blackmetal)
        spout = _gooseneck(F, 'tap', 0, 0.76, -0.42, -0.26, 1.04, M.blackmetal, r=0.02, tip_y=0.98)
        # two folded towels on the shelf, each two soft layers
        for i, x in enumerate((-0.17, 0.17)):
            for k in range(2):
                F.soft(f'towel{i}_{k}', 0.3, 0.036, 0.24, (x, 0.233 + 0.036 * k, -0.13 + 0.008 * k), M.towel, round=0.017,
                       puff=(0, 0.006, 0), round_segments=2, flat=2)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, 0.785)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- console sink: a basin on a brass frame, a towel over the bar between its legs -----------------------
def sinkConsole():
    F = G.GameFrame(shift=0.2)
    z = -0.21
    with G.new_parts() as still:
        # the basin in one shell: a rounded slab, a flat deck round the bowl, the bowl set forward
        bz = z + 0.03
        _lathe(F, 'basin', 0, z, [(0.765, 0.29, 0.22, 7), (0.77, 0.297, 0.227, 7), (0.87, 0.3, 0.23, 7),
                                  (0.88, 0.296, 0.226, 7), (0.88, 0.25, 0.168, 5, 0.168, bz - z),
                                  (0.875, 0.245, 0.163, 5, 0.163, bz - z), (0.83, 0.225, 0.145, 4, 0.145, bz - z),
                                  (0.805, 0.17, 0.1, 3, 0.1, bz - z), (0.8, 0.1, 0.06, 2.4, 0.06, bz - z)],
               M.body, verts=36, angle=50)
        _drain(F, 0, 0.8, bz, r=0.026, material=M.brass)
        # the frame: each side a leg from a foot up and back under the basin to the wall, and a towel bar
        for s in (-1, 1):
            x = s * 0.27
            _tube(F, f'leg{s}', _bends([(x, 0.012, 0.0), (x, 0.752, 0.0), (x, 0.752, -0.41)], 0.05, 5), 0.016, M.brass, verts=10,
                  round=1)
            _lathe(F, f'foot{s}', x, 0.0, [(0.0, 0.024, 0.024), (0.012, 0.022, 0.022), (0.03, 0.017, 0.017)], M.brass,
                   verts=12)
            F.cyl(f'wall_plate{s}', 0.03, 0.012, (x, 0.752, -0.42), M.brass, rot=(90, 0, 0), verts=14)
            F.cyl(f'bar_collar{s}', 0.02, 0.02, (s * 0.255, 0.6, 0.0), M.brass, rot=(0, 90, 0), verts=12)
        F.cyl('bar', 0.011, 0.5, (0, 0.6, 0), M.brass, rot=(0, 90, 0), verts=10)
        # the towel hangs over the bar, longer at the front
        F.sweep('towel', [(0, 0.36, 0.036), (0, 0.5, 0.03), (0, 0.585, 0.021), (0, 0.617, 0.0), (0, 0.585, -0.022),
                          (0, 0.5, -0.03), (0, 0.42, -0.036)], None, M.fabric,
                profile=G.rounded_rect(0.4, 0.012, 0.005, corner=2), up=(1, 0, 0), resolution=2)
        spout = _gooseneck(F, 'tap', 0, 0.88, -0.4, -0.26, 1.04, M.brass, r=0.02, tip_y=0.98)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(F, spout, 0.81)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- the toilets: a bowl, a seat and a cistern; the lid lifts while a resident uses it ----------------------
# Built where the game places it, the cistern against the wall face (game depth -0.43), not centred:
# the game sits the resident at the footprint's middle (characters.ts SEATS, the content's `seat`), so the
# bowl must stay where the seat is. The seat's top is the seat height. The lid is the moving part, hinged
# at its back edge; `lid_up` lifts it 100° to lean on the cistern (or over the smart seat's housing).
# The bowl, its seat and lid are egg-shaped from above (`EGG`: narrower to the front), as an elongated bowl is.
WALL = -0.43
EGG = 0.1


def _egg(r, ratio, y, rx_k=1.0):
    """A ring of the bowl's shape (a `_lathe` ring): `r` across (half), `ratio` times that deep."""
    return (y, r * rx_k, r * ratio * rx_k)


def _bowl(F, zb, base_y, rim, r, ratio, water_y):
    """The bowl in one shell, egg-shaped: up its outside to a rounded rim and down the inside to the water."""
    h = rim - base_y
    rings = [(base_y, r * 0.5), (base_y + h * 0.3, r * 0.78), (base_y + h * 0.7, r * 0.95), (rim - 0.018, r),
             (rim - 0.004, r - 0.001), (rim, r - 0.006), (rim, r - 0.03), (rim - 0.012, r - 0.04), (rim - 0.03, r - 0.05),
             (rim - h * 0.45, r * 0.62), (water_y + 0.02, r * 0.4), (water_y, r * 0.3)]
    _lathe(F, 'bowl', 0, zb, [(y, k, k * ratio) for y, k in rings], M.porcelain, verts=32, egg=EGG, angle=50)
    _lathe(F, 'bowl_water', 0, zb, [_egg(r, ratio, water_y + 0.004, 0.36), _egg(r, ratio, water_y + 0.008, 0.36)], M.water,
           verts=20, egg=EGG)


def _foot(F, z, top, r, ratio, flare=1.0):
    """The bowl's foot on the floor, waisted, flaring into the bowl's underside (`flare` widens its plinth), with
    a bolt cap each side."""
    rings = [(0.0, flare), (0.012, flare), (0.03, 0.9 + 0.06 * flare), (top * 0.45, 0.8), (top * 0.8, 0.88), (top, 0.95)]
    _lathe(F, 'foot', 0, z, [(y, r * k, r * ratio * k) for y, k in rings], M.porcelain, verts=32, egg=EGG * 0.5, angle=50)
    for s in (-1, 1):
        _lathe(F, f'bolt_cap{s}', s * (r * flare + 0.006), z, [(0.0, 0.016, 0.016), (0.012, 0.013, 0.013), (0.02, 0.006, 0.006)],
               M.porcelain, verts=8)


def _column(F, name, y0, y1, z0, z1, w=0.26):
    """The porcelain behind the bowl (the deck a cistern stands on, a wall-hung bowl's fixing): a rounded block
    from game depth `z0` to `z1`, `w` wide, its edges rolled."""
    d = (z1 - z0) / 2
    _lathe(F, name, 0, (z0 + z1) / 2, [(y0, w / 2 - 0.03, d - 0.03, 3), (y0 + 0.012, w / 2 - 0.008, d - 0.008, 3.5),
                                       (y0 + 0.03, w / 2, d, 4), (y1 - 0.03, w / 2, d, 4), (y1 - 0.008, w / 2 - 0.008, d - 0.008, 3.5),
                                       (y1, w / 2 - 0.03, d - 0.03, 3)], M.porcelain, verts=28, angle=50)


def _ring_path(F, zb, y, r, ratio, verts=32):
    """Game points round the bowl's egg at height `y`, `r` across (half)."""
    out = []
    for i in range(verts):
        a = math.tau * i / verts
        v = math.sin(a)
        out.append((r * math.cos(a) * (1 - EGG * v), y, zb + r * ratio * v))
    return out


def _seat(F, zb, y, r, ratio, material, t=0.024):
    """The seat ring on the rim, from `y` up `t`, its outside `r` (half) wide: a rounded section swept round the
    egg, with two rubber buffers under it."""
    m = 0.034
    F.sweep('seat', _ring_path(F, zb, y + t / 2, r - m, ratio), None, material, closed=True,
            profile=G.rounded_rect(t, 2 * m, t * 0.48, corner=2), angle=70)
    for s in (-1, 1):
        F.cyl(f'buffer{s}', 0.008, 0.006, (s * (r - m) * 0.92, y - 0.002, zb + 0.06), M.trim, verts=8)


def _lid(F, zb, y, r, ratio, material, t=0.02, pivot='bottom'):
    """The lid over the seat, from `y` up `t`, domed, hinged across its back edge: on its underside, or on its
    top (`pivot='top'`, where it lifts over a housing behind it). Returns it, ready to swing about X."""
    with G.new_parts() as parts:
        _lathe(F, 'lid', 0, zb, [_egg(r, ratio, y, 0.985), _egg(r, ratio, y + t * 0.3, 1.0), _egg(r, ratio, y + t * 0.75, 0.97),
                                 _egg(r, ratio, y + t, 0.88)], material, verts=32, egg=EGG, angle=60)
    lid = G.join(parts, 'lid')
    return G.hinged(lid, F.at(0, y + (t if pivot == 'top' else 0), zb - r * ratio))


def _hinges(F, z, y, material, x=0.09):
    """The seat's two hinges on the bowl's back deck: a round knuckle on a domed post each."""
    for s in (-1, 1):
        _lathe(F, f'hinge_post{s}', s * x, z - 0.004, [(y - 0.002, 0.016, 0.016), (y + 0.006, 0.014, 0.014), (y + 0.012, 0.009, 0.009)],
               material, verts=10)
        _tube(F, f'hinge{s}', [(s * x - 0.02, y + 0.014, z), (s * x + 0.02, y + 0.014, z)], 0.011, material, verts=10, round=1)


def _wall_rose(F, name, x, y, z, r, material, h=0.01, verts=14):
    """A round plate on a wall (or a cistern's front) at game depth `z`, its edge eased."""
    return F.cyl(name, r, h, (x, y, z + h / 2), material, rot=(90, 0, 0), verts=verts, bevel=h * 0.4, bevel_segments=1)


def _roll(F, x, y, z, side=1):
    """A toilet roll on a chrome holder out from the wall (or the panel) at game depth `z`; `side` is the end the
    arm comes from (the roll runs along x). Its loose end hangs down the front."""
    ax = x - side * 0.06
    _wall_rose(F, 'roll_rose', ax, y, z, 0.024, M.chrome)
    _tube(F, 'roll_arm', [(ax, y, z + 0.004), (ax, y, z + 0.065), (x + side * 0.055, y, z + 0.065)], 0.0055, M.chrome,
          verts=8, round=2)
    F.cyl('roll', 0.055, 0.1, (x, y, z + 0.065), M.paper, rot=(0, 90, 0), verts=18)
    F.cyl('roll_core', 0.02, 0.102, (x, y, z + 0.065), M.dark, rot=(0, 90, 0), verts=10)
    F.box('roll_sheet', 0.098, 0.07, 0.002, (x, y - 0.035, z + 0.065 + 0.0555), M.paper)


def _cistern(F, cw, ch, y0, cz, material):
    """A cistern from the wall to game depth `cz`, `cw` wide and `ch` tall from `y0`: a rounded block whose
    corners soften towards its foot."""
    d = (cz - WALL) / 2
    zc = (WALL + cz) / 2
    _lathe(F, 'cistern', 0, zc, [(y0, cw / 2 - 0.02, d - 0.02, 4), (y0 + 0.02, cw / 2 - 0.004, d - 0.004, 5),
                                 (y0 + 0.05, cw / 2, d, 6), (y0 + ch - 0.012, cw / 2, d, 6), (y0 + ch, cw / 2 - 0.008, d - 0.008, 6)],
           material, verts=32, angle=50)


def _cistern_lid(F, w, y, d, material, t=0.03, step=False):
    """The cistern's lid, a little proud of it all round: a soft slab, and a raised step on it (`step`)."""
    F.soft('cistern_lid', w, t, d, (0, y + t / 2, WALL + d / 2), material, round=0.01, round_segments=1)
    if step:
        F.soft('cistern_cap', w - 0.04, 0.02, d - 0.035, (0, y + t + 0.01, WALL + d / 2 - 0.004), material, round=0.008,
               round_segments=1)


def _lever(F, x, y, z, material, ball=False):
    """A flush lever on the cistern's front at game depth `z`: a round boss and an arm towards the middle."""
    _wall_rose(F, 'lever_boss', x, y, z, 0.02, material, h=0.012, verts=12)
    _tube(F, 'lever', [(x, y, z + 0.008), (x, y, z + 0.016), (x + 0.04, y - 0.004, z + 0.016), (x + 0.075, y - 0.012, z + 0.014)],
          0.006, material, verts=8, round=2, scale=[1.0, 1.0, 0.9, 0.8])
    if ball:
        F.sphere('lever_end', 0.013, (x + 0.08, y - 0.013, z + 0.014), material, segments=8, rings=5)


def _close_coupled(F, zb, rim, r, ratio, cz, cw, ch, seat_material, foot_flare=1.0):
    """The bowl on its foot, the back deck from the bowl to the wall, the seat, and a cistern `cw` wide,
    `ch` tall, from the wall to game depth `cz`, standing on the deck."""
    _foot(F, zb - 0.04, 0.26, 0.115, 1.3, flare=foot_flare)
    _bowl(F, zb, 0.17, rim, r, ratio, water_y=0.27)
    back = zb - r * ratio                       # the seat's back edge, where the lid hinges
    _column(F, 'deck', 0.16, rim, WALL, back + 0.04)
    _seat(F, zb, rim, r - 0.002, ratio, seat_material)
    _hinges(F, back + 0.012, rim, M.chrome)
    _cistern(F, cw, ch, rim, cz, M.cistern)


def toilet():
    F = G.GameFrame()
    r, ratio, rim = 0.18, 1.2, 0.405           # the seat's top at 0.43 m (SEATS)
    zb = -0.186 + r * ratio                    # the lid hinges 9.4 cm in front of the cistern
    with G.new_parts() as still:
        _close_coupled(F, zb, rim, r, ratio, -0.28, 0.42, 0.38, M.seat)
        _cistern_lid(F, 0.44, rim + 0.38, 0.155, M.cistern)
        _lever(F, -0.15, 0.72, -0.28, M.chrome)
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
        _cistern_lid(F, 0.41, rim + 0.37, 0.13, M.cistern, t=0.025)
        # a dual flush button on the lid
        _rose(F, 'button_ring', 0, rim + 0.395, -0.365, 0.034, M.chrome, h=0.006, verts=20)
        for s in (-1, 1):
            F.box(f'button{s}', 0.024, 0.006, 0.04, (s * 0.0135, rim + 0.401, -0.365), M.chrome, bevel=0.003, bevel_segments=1)
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
        # a moulded two-step lid, a band round the cistern and a lever with a ball end
        _cistern_lid(F, 0.47, rim + 0.4, 0.185, M.cistern, step=True)
        _tube(F, 'cistern_band', [(-0.195, rim + 0.06, -0.253), (0.195, rim + 0.06, -0.253)], 0.007, M.cistern, verts=8, round=1)
        _lever(F, -0.15, 0.76, -0.25, M.chrome, ball=True)
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
        # a shelf along the panel's top, a display with a ready light, a remote in its holder, a roll
        F.soft('shelf', 0.9, 0.025, panel_z - WALL + 0.02, (0, 1.1875, (WALL + panel_z) / 2 + 0.01), M.panel, round=0.008,
               round_segments=2)
        F.soft('display', 0.2, 0.12, 0.012, (0, 1.0, panel_z + 0.006), M.screen, round=0.004, round_segments=1)
        for k in range(3):
            F.box(f'key{k}', 0.03, 0.012, 0.004, (-0.05 + 0.05 * k, 0.96, panel_z + 0.013), M.panel, bevel=0.002, bevel_segments=1)
        F.sphere('ready', 0.008, (0.06, 1.035, panel_z + 0.013), M.led, segments=8, rings=5)
        F.soft('remote_holder', 0.075, 0.07, 0.03, (0.34, 0.68, panel_z + 0.015), M.panel, round=0.008, round_segments=1)
        F.soft('remote', 0.055, 0.15, 0.022, (0.34, 0.73, panel_z + 0.02), M.screen, round=0.009, round_segments=2)
        _roll(F, -0.34, 0.8, panel_z, side=-1)
        _wall_hung(F, zb, rim, r, ratio, panel_z)
        _seat(F, zb, rim, r - 0.002, ratio, M.seat, t=0.03)
        F.soft('housing', 0.4, 0.055, housing - panel_z + 0.01, (0, rim + 0.0275, (panel_z + housing) / 2 - 0.005), M.seat,
               round=0.02, round_segments=2)
        F.box('housing_seam', 0.36, 0.004, 0.004, (0, rim + 0.04, housing + 0.004), M.trim)
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
        F.soft('panel_top', 0.6, 0.02, panel_z - WALL + 0.015, (0, 1.09, (WALL + panel_z) / 2 + 0.0075), M.stone, round=0.006,
               round_segments=1)
        F.soft('flush_plate', 0.22, 0.15, 0.012, (0, 1.0, panel_z + 0.006), M.screen, round=0.005, round_segments=1)
        for s in (-1, 1):
            F.soft(f'flush{s}', 0.09, 0.12, 0.008, (s * 0.05, 1.0, panel_z + 0.014), M.chrome, round=0.004, round_segments=1)
        _wall_hung(F, zb, rim, r, ratio, panel_z)
        back = zb - r * ratio
        F.soft('deck', 0.24, 0.03, back - panel_z + 0.02, (0, rim - 0.015, (panel_z + back) / 2), M.porcelain, round=0.012,
               round_segments=2)
        _seat(F, zb, rim, r - 0.002, ratio, M.seat)
        _hinges(F, back + 0.012, rim, M.chrome)
    G.join(still, 'body', origin=(0, 0, 0))
    lid = _lid(F, zb, rim + 0.024, r, ratio, M.seat)
    G.settle(centre=False)
    G.swing(lid, 'lid_up', -100, seconds=0.6, axis='X')
    return G.meshes()


# --- the showers: a tray, walls, a screen or curtain, and a head that rains while a resident showers ------
# Each fills its tile, as its old part does (the resident stands inside, at the middle). The rain is
# the moving part: thin streams built full length below the head, joined with the origin on the head and
# scaled to nothing along their length, so they are hidden at rest; `water_on` grows them down for the
# whole use (the sinks' `water_on` way). Glass is its own draw call: tray, walls and metal (1), glass (1),
# the rain (1).


def _tiled_walls(F, x0, x1, z0, z1, y0, y1, t=0.03, size=0.3, gap=0.005):
    """The back wall (from game x0 to x1, at the back z0) and the left wall (at x0, from z0 to z1): grout
    backing with square tiles a little proud of it."""
    F.box('grout_back', x1 - x0, y1 - y0, t, ((x0 + x1) / 2, (y0 + y1) / 2, z0 + t / 2), M.grout)
    F.box('grout_left', t, y1 - y0, z1 - z0 - t, (x0 + t / 2, (y0 + y1) / 2, (z0 + t + z1) / 2), M.grout)
    rows = max(1, round((y1 - y0) / size))
    th = (y1 - y0) / rows
    cols = max(1, round((x1 - x0 - t) / size))
    tw = (x1 - x0 - t) / cols
    deep = max(1, round((z1 - z0 - t) / size))
    td = (z1 - z0 - t) / deep
    for j in range(rows):
        y = y0 + th * (j + 0.5)
        for i in range(cols):
            F.box(f'tile_b{i}_{j}', tw - gap, th - gap, 0.006, (x0 + t + tw * (i + 0.5), y, z0 + t + 0.003), M.tile)
        for i in range(deep):
            F.box(f'tile_l{i}_{j}', 0.006, th - gap, td - gap, (x0 + t + 0.003, y, z0 + t + td * (i + 0.5)), M.tile)


def _rain(F, head, r, bottom, rings=2, grid=None, towards=None):
    """The rain from a head at game `head` (the head's underside) down to the floor at height `bottom`:
    thin parallel streams across a disc of radius `r` (`rings` rings round a middle one) or a `grid`
    (n, spacing), joined with the origin on the head and scaled to nothing along their length. `towards`
    (a game point on the floor) slants the rain from a wall head onto the resident."""
    x, y, z = head
    if grid:
        n, step = grid
        spots = [((i - (n - 1) / 2) * step, (k - (n - 1) / 2) * step) for i in range(n) for k in range(n)]
    else:
        spots = [(0.0, 0.0)] + [(r * ring / rings * G.cosd(360 * i / (6 * ring)), r * ring / rings * G.sind(360 * i / (6 * ring)))
                                for ring in range(1, rings + 1) for i in range(6 * ring)]
    if towards:
        aim = Vector(F.at(*towards)) - Vector(F.at(x, y, z))
        length = aim.length
    else:
        length = y - bottom
    with G.new_parts() as parts:
        for i, (dx, dz) in enumerate(spots):
            F.cyl(f'rain{i}', 0.005, length, (x + dx, y - length / 2, z + dz), M.water, verts=6, smooth=False)
    rain = G.join(parts, 'water', origin=F.at(x, y, z))
    if towards:
        # turn the straight-down rain about its origin, onto the resident
        rain.rotation_mode = 'QUATERNION'
        rain.rotation_quaternion = Vector((0, 0, -1)).rotation_difference(aim)
    rain.scale[2] = 0.001
    return rain


def _bends(pts, r=0.012, steps=4):
    """The points of a path with each inner corner turned in a round bend of about `r` (`steps` points on it), as
    bent tube or a welded frame turns: sweep them as they are (resolution 1), so long runs don't overshoot."""
    out = [pts[0]]
    for a, b, c in zip(pts, pts[1:], pts[2:]):
        a_, b_, c_ = Vector(a), Vector(b), Vector(c)
        p0 = b_ + (a_ - b_).normalized() * min(r, (a_ - b_).length / 2)
        p2 = b_ + (c_ - b_).normalized() * min(r, (c_ - b_).length / 2)
        for k in range(steps + 1):
            t = k / steps
            out.append(tuple(p0 * (1 - t) ** 2 + b_ * 2 * (1 - t) * t + p2 * t * t))
    out.append(pts[-1])
    return out


def _frame(F, name, pts, material, t=0.025, bend=0.012, up=(1, 0, 0)):
    """A square-section frame (a screen's posts and rails) in one piece through game points, its corners bent."""
    return F.sweep(name, _bends(pts, bend, 3), None, material, profile=G.rounded_rect(t, t, 0.004, corner=1), up=up,
                   angle=50)


def _post(F, name, x, z, y0, y1, material, t=0.025):
    F.box(name, t, y1 - y0, t, (x, (y0 + y1) / 2, z), material, bevel=0.004)


def _tray(F, name, w, d, h, material, lip=0.035, fall=0.012, square=10):
    """A shower tray in one piece: a rounded slab `w` × `d`, `h` high, its top a raised lip `lip` wide round a
    floor that falls `fall` towards the middle."""
    _lathe(F, name, 0, 0, [(0.0, w / 2 - 0.004, d / 2 - 0.004, square), (h - 0.008, w / 2, d / 2, square),
                           (h, w / 2 - 0.006, d / 2 - 0.006, square), (h, w / 2 - lip, d / 2 - lip, square * 0.8),
                           (h - fall * 0.6, w / 2 - lip - 0.012, d / 2 - lip - 0.012, square * 0.7),
                           (h - fall, w / 4, d / 4, 3)], material, verts=40, angle=35)


def _grate(F, name, x, y, z, w, d, material, slots=5, round_=False):
    """A drain's grate on a dark well at height `y`: a plate with slots cut across it, or a round one."""
    F.box(f'{name}_well', w - 0.004, 0.004, d - 0.004, (x, y - 0.001, z), M.dark)
    if round_:
        holes = [[(u + (k - (slots - 1) / 2) * w / (slots + 1), v) for u, v in G.rounded_rect(0.006, d * 0.6, 0.003, corner=1)]
                 for k in range(slots)]
        F.extrude(name, [(w / 2 * math.cos(math.tau * i / 20), w / 2 * math.sin(math.tau * i / 20)) for i in range(20)], 0.004,
                  (x, y + 0.002, z), material, plane='top', holes=holes)
    else:
        holes = [[(u + (k - (slots - 1) / 2) * w / (slots + 0.6), v) for u, v in G.rounded_rect(w / (slots * 2.2), d - 0.024, 0.003, corner=1)]
                 for k in range(slots)]
        F.extrude(name, G.rounded_rect(w, d, 0.004, corner=1), 0.004, (x, y + 0.002, z), material, plane='top', holes=holes)


def _head(F, name, x, y, z, r, material, t=0.022):
    """A round shower head, its face at height `y` (where the rain starts): a disc rounding over into a shallow
    dome, its nozzle plate dark."""
    _lathe(F, name, x, z, [(y + 0.001, r * 0.94, r * 0.94), (y + t * 0.2, r, r), (y + t * 0.75, r, r), (y + t, r * 0.82, r * 0.82),
                           (y + t * 1.25, r * 0.25, r * 0.25)], material, verts=28, angle=50)
    F.cyl(f'{name}_nozzles', r * 0.78, 0.002, (x, y, z), M.dark, verts=20)


def _hand_shower(F, x, y0, y1, z, material, hose_from):
    """A hand shower on a slide rail on the wall (from `y0` to `y1` at depth `z`), its hose looping down from
    `hose_from` (a game point) up to its handle."""
    _tube(F, 'riser', [(x, y0, z), (x, y1, z)], 0.011, material, verts=10, round=1)
    for y in (y0, y1):
        _wall_rose(F, f'riser_bracket{y}', x, y, z - 0.03, 0.02, material, h=0.03, verts=12)
    F.box('slider', 0.03, 0.04, 0.03, (x, y0 + (y1 - y0) * 0.6, z + 0.012), material, bevel=0.008, bevel_segments=1)
    hy = y0 + (y1 - y0) * 0.6
    _tube(F, 'hand_shower', [(x, hy - 0.06, z + 0.03), (x, hy + 0.08, z + 0.06), (x, hy + 0.12, z + 0.075)], 0.013, material,
          verts=10, round=2, scale=[0.8, 1.0, 1.1])
    hx, hy0, hz = hose_from
    F.cyl('hand_head', 0.04, 0.02, (x, hy + 0.135, z + 0.08), material, rot=(-30, 0, 0), verts=16, bevel=0.006,
          bevel_segments=1)
    _tube(F, 'hose', [(hx, hy0, hz), (hx - 0.01, hy0 - 0.2, hz + 0.02), ((hx + x) / 2, hy0 - 0.38, hz + 0.05),
                      (x + 0.01, hy0 - 0.25, z + 0.045), (x, hy - 0.06, z + 0.03)], 0.006, material, verts=6, round=3)


def _bar_mixer(F, x, y, z, w, material):
    """A thermostatic bar mixer on the wall at depth `z`: a bar on two wall plates, a knob at each end, an outlet
    under it."""
    for s in (-1, 1):
        _wall_rose(F, f'mixer_plate{s}', x + s * (w / 2 - 0.03), y, z, 0.026, material, h=0.03, verts=14)
    _tube(F, 'mixer', [(x - w / 2, y, z + 0.045), (x + w / 2, y, z + 0.045)], 0.022, material, verts=14, round=1)
    for s in (-1, 1):
        F.cyl(f'mixer_knob{s}', 0.026, 0.03, (x + s * (w / 2 + 0.015), y, z + 0.045), material, rot=(0, 90, 0), verts=16,
              bevel=0.006, bevel_segments=1)
    F.cyl('mixer_outlet', 0.01, 0.03, (x + w / 4, y - 0.035, z + 0.045), material, verts=10)


def _caddy(F, name, x, y, z, w, d, material):
    """A wire shelf on the wall: a tray, a rail round its front on two posts."""
    F.box(f'{name}_tray', w, 0.008, d, (x, y, z), material, bevel=0.003, bevel_segments=1)
    _tube(F, f'{name}_rail', [(x - w / 2, y + 0.04, z - d / 2 + 0.005), (x - w / 2, y + 0.04, z + d / 2), (x + w / 2, y + 0.04, z + d / 2),
                              (x + w / 2, y + 0.04, z - d / 2 + 0.005)], 0.004, material, verts=6, round=2)
    for s in (-1, 1):
        _tube(F, f'{name}_post{s}', [(x + s * w / 2, y, z + d / 2), (x + s * w / 2, y + 0.04, z + d / 2)], 0.003, material, verts=6,
              round=1)


# The base shower's sliding door goes this far left (Blender +x is the game's -x), behind the fixed panel;
# the stall's front curtain is gathered to this part of its length at rest.
SHOWER_SLIDE = 0.41
STALL_GATHER = 0.32


def shower():
    F = G.GameFrame()
    top = 2.15
    with G.new_parts() as still:
        _tray(F, 'tray', 0.9, 0.9, 0.058, M.tray)
        _grate(F, 'drain', 0.1, 0.047, 0.1, 0.09, 0.09, M.fittings, round_=True)
        _tiled_walls(F, -0.45, 0.45, -0.45, 0.45, 0.058, top)
        # a glass side, and a fixed panel in front (the sliding door, inside it, is its own part)
        F.box('glass_side', 0.01, 2.0, 0.83, (0.435, 1.09, -0.005), M.glass)
        F.box('glass_fixed', 0.46, 2.0, 0.01, (-0.195, 1.09, 0.437), M.glass)
        # the frame: the side's posts and top rail bent in one piece, the front's top rail and its track
        _frame(F, 'frame', [(0.435, 0.058, -0.42), (0.435, top - 0.0125, -0.42), (0.435, top - 0.0125, 0.435),
                            (0.435, 0.058, 0.435)], M.frame)
        F.sweep('rail_front', [(0.42, top - 0.0125, 0.435), (-0.414, top - 0.0125, 0.435)], None, M.frame,
                profile=G.rounded_rect(0.025, 0.025, 0.004, corner=1))
        F.box('rail_bottom', 0.85, 0.02, 0.03, (0.003, 0.068, 0.435), M.frame, bevel=0.004)
        F.box('door_stop', 0.02, 0.05, 0.02, (-0.404, 0.11, 0.42), M.frame, bevel=0.005, bevel_segments=1)
        # a slide rail with the hand shower and its hose, a bar mixer, and the overhead head on an arm from the wall
        _bar_mixer(F, 0.12, 1.1, -0.414, 0.2, M.fittings)
        _hand_shower(F, -0.15, 0.9, 1.7, -0.384, M.fittings, (0.17, 1.065, -0.37))
        _wall_rose(F, 'arm_rose', 0, 2.03, -0.414, 0.025, M.fittings, h=0.012, verts=14)
        _tube(F, 'arm', [(0, 2.03, -0.41), (0, 2.03, -0.2), (0, 2.018, -0.16), (0, 2.012, -0.15)], 0.011, M.fittings, verts=10)
        _head(F, 'head', 0, 1.989, -0.15, 0.12, M.fittings)
        # a corner shelf with two bottles
        arc = [(0.2 * math.cos(math.radians(a)), 0.2 * math.sin(math.radians(a))) for a in range(0, 91, 15)]
        F.extrude('corner_shelf', [(0, 0)] + arc, 0.01, (-0.414, 1.25, -0.414), M.frame, plane='top', bevel=0.003,
                  bevel_segments=1)
        _bottle(F, 'bottle0', (-0.36, 1.255, -0.36), 0.026, 0.16, M.teal, M.bottle)
        _bottle(F, 'bottle1', (-0.29, 1.255, -0.375), 0.022, 0.12, M.bottle, M.sage, pump=True)
    G.join(still, 'body', origin=(0, 0, 0))
    # the sliding door, inside the fixed panel on the right: it slides left behind it to let the resident in
    # and out (`door_open`). All glass, its grip a bar of smoked glass on the inside by its leading edge,
    # so the door is one draw call (Q-05: the shower's fourth).
    with G.new_parts() as parts:
        F.box('glass_door', 0.44, 1.98, 0.01, (0.205, 1.08, 0.423), M.glass)
        F.sweep('door_grip', _bends([(0.37, 0.85, 0.418), (0.37, 0.85, 0.398), (0.37, 1.25, 0.398), (0.37, 1.25, 0.418)], 0.01),
                0.009, M.grip, verts=8)
    door = G.join(parts, 'door', origin=F.at(0.205, 1.08, 0.423))
    rain = _rain(F, (0, 1.988, -0.15), 0.095, 0.06)
    G.settle()
    G.slide(door, 'door_open', (SHOWER_SLIDE, 0, 0), seconds=0.7)
    G.grow(rain, 'water_on', seconds=0.5)
    return G.meshes()


def _curtain(F, name, a, b, y0, y1, waves, depth=0.035, t=0.004, steps=6):
    """A curtain hanging in `waves` soft folds from game point `a` (x, z) to `b`, from `y0` up to `y1`: one wavy
    sheet, thick `t`."""
    ax, az = a
    bx, bz = b
    L = math.hypot(bx - ax, bz - az)
    ux, uz = (bx - ax) / L, (bz - az) / L                # along the rail
    nx, nz = -uz, ux                                      # across it
    n = waves * steps
    mid = [(ax + ux * L * k / n + nx * depth / 2 * math.sin(math.tau * waves * k / n),
            az + uz * L * k / n + nz * depth / 2 * math.sin(math.tau * waves * k / n)) for k in range(n + 1)]
    cx, cz = (ax + bx) / 2, (az + bz) / 2
    front = [(x + nx * t / 2 - cx, z + nz * t / 2 - cz) for x, z in mid]
    back = [(x - nx * t / 2 - cx, z - nz * t / 2 - cz) for x, z in reversed(mid)]
    return F.extrude(name, front + back, y1 - y0, (cx, (y0 + y1) / 2, cz), M.curtain, plane='top', smooth=True, angle=60)


def _curtain_rings(F, name, a, b, y, n, material):
    """`n` rings along a rail from game point `a` (x, z) to `b` at height `y`, each round the rail."""
    ax, az = a
    bx, bz = b
    along_x = abs(bx - ax) > abs(bz - az)
    for k in range(n):
        f = (k + 0.5) / n
        G.torus(f'{name}{k}', 0.016, 0.0022, at=F.at(ax + (bx - ax) * f, y - 0.004, az + (bz - az) * f), material=material,
                rot=(0, 90, 0) if along_x else (90, 0, 0), major_segments=8, minor_segments=3)


def showerStall():
    F = G.GameFrame()
    rail = 1.95
    with G.new_parts() as still:
        _tray(F, 'tray', 0.9, 0.9, 0.08, M.tray, lip=0.045, square=14)
        _grate(F, 'drain', 0.05, 0.069, 0.05, 0.09, 0.09, M.fittings, round_=True)
        _tiled_walls(F, -0.45, 0.45, -0.45, 0.45, 0.08, 2.08, t=0.04, size=0.25)
        # an L-shaped rail at the open sides, bent round the corner; the curtain hangs along the whole side, and
        # its front is its own part
        _tube(F, 'rail', _bends([(0.42, rail, -0.398), (0.42, rail, 0.42), (-0.398, rail, 0.42)], 0.04, 5), 0.011, M.frame,
              verts=10, round=1)
        _wall_rose(F, 'rail_mount_side', 0.42, rail, -0.404, 0.025, M.frame, h=0.012)
        F.cyl('rail_mount_front', 0.025, 0.012, (-0.398, rail, 0.42), M.frame, rot=(0, 90, 0), verts=14, bevel=0.004,
              bevel_segments=1)
        _curtain(F, 'curtain_side', (0.42, -0.39), (0.42, 0.39), 0.22, rail - 0.03, 6)
        _curtain_rings(F, 'ring_side', (0.42, -0.39), (0.42, 0.39), rail, 12, M.frame)
        # a riser on the back wall up to a tilted head, a lever mixer, a wire caddy with bottles
        _tube(F, 'riser', [(-0.2, 1.08, -0.39), (-0.2, 1.93, -0.39), (-0.2, 1.955, -0.37), (-0.2, 1.96, -0.33)], 0.013,
              M.fittings, verts=10, round=2)
        F.cyl('head', 0.07, 0.03, (-0.2, 1.96, -0.31), M.fittings, rot=(-25, 0, 0), verts=20, bevel=0.008, bevel_segments=2)
        _wall_rose(F, 'mixer', -0.2, 1.05, -0.404, 0.05, M.fittings, h=0.018, verts=20)
        _tube(F, 'lever', [(-0.2, 1.05, -0.38), (-0.2, 1.06, -0.35), (-0.16, 1.07, -0.345), (-0.12, 1.075, -0.345)], 0.008,
              M.fittings, verts=8, round=2, scale=[1.3, 1.1, 1.0, 0.8])
        _caddy(F, 'caddy', 0.15, 1.15, -0.355, 0.24, 0.1, M.frame)
        _bottle(F, 'bottle0', (0.1, 1.154, -0.36), 0.025, 0.15, M.teal, M.bottle)
        _bottle(F, 'bottle1', (0.18, 1.154, -0.36), 0.03, 0.12, M.bottle, M.teal, pump=True)
    G.join(still, 'body', origin=(0, 0, 0))
    # the front of the curtain, built drawn across the front and gathered into the right corner at rest
    # (scaled along the rail about the corner): drawn across once the resident is in (`curtain_draw`, a
    # grow along the rail), back as the use ends
    with G.new_parts() as parts:
        _curtain(F, 'curtain', (0.41, 0.42), (-0.41, 0.42), 0.22, rail - 0.03, 7)
        _curtain_rings(F, 'ring', (0.41, 0.42), (-0.41, 0.42), rail, 14, M.curtain)
    curtain = G.join(parts, 'curtain', origin=F.at(0.41, rail, 0.42))
    curtain.scale[0] = STALL_GATHER
    rain = _rain(F, (-0.2, 1.935, -0.3), 0.045, 0.08, rings=1, towards=(-0.05, 0.085, -0.02))
    G.settle()
    G.grow(curtain, 'curtain_draw', seconds=0.9, axis='X')
    G.grow(rain, 'water_on', seconds=0.5)
    return G.meshes()


def showerWalkIn():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.soft('floor', 0.92, 0.04, 0.92, (0, 0.02, 0), M.slate, round=0.008, round_segments=1)
        _grate(F, 'drain', 0, 0.04, 0.2, 0.3, 0.06, M.fittings_dark, slots=12)
        F.box('wall', 0.92, 2.2, 0.04, (0, 1.14, -0.44), M.slate_wall, bevel=0.004)
        for y in (0.6, 1.15, 1.7):
            F.box(f'joint{y}', 0.92, 0.006, 0.004, (0, y, -0.419), M.groove)
        F.box('joint_v', 0.006, 2.2, 0.004, (0.15, 1.14, -0.419), M.groove)
        # one glass screen on the right in a wall channel and a floor channel, a post at its end, and a stay bar
        # from its top to the wall
        F.box('glass_screen', 0.01, 2.0, 0.6, (0.42, 1.04, -0.11), M.glass)
        F.box('channel', 0.025, 2.0, 0.02, (0.42, 1.04, -0.41), M.frame_dark, bevel=0.003)
        F.box('floor_channel', 0.025, 0.02, 0.6, (0.42, 0.05, -0.11), M.frame_dark, bevel=0.003)
        _post(F, 'post', 0.42, 0.2, 0.04, 2.04, M.frame_dark, t=0.022)
        _tube(F, 'bar', [(0.42, 2.05, 0.2), (0.42, 2.05, -0.38), (0.42, 2.05, -0.415)], 0.01, M.frame_dark, verts=8, round=1)
        _wall_rose(F, 'bar_rose', 0.42, 2.05, -0.42, 0.018, M.frame_dark, h=0.01, verts=12)
        # a riser up the wall, bent over into an arm out over the middle, and the wide rain head
        _tube(F, 'riser', _bends([(0, 1.74, -0.4), (0, 2.245, -0.4), (0, 2.245, -0.08)], 0.045, 5), 0.015, M.fittings_dark,
              verts=10, round=1)
        _wall_rose(F, 'riser_rose', 0, 1.74, -0.42, 0.03, M.fittings_dark, h=0.012, verts=14)
        _head(F, 'head', 0, 2.208, -0.08, 0.17, M.fittings_dark, t=0.02)
        F.soft('mixer', 0.08, 0.16, 0.012, (-0.25, 1.1, -0.414), M.fittings_dark, round=0.004, round_segments=1)
        for k, y in enumerate((1.14, 1.06)):
            F.cyl(f'mixer_knob{k}', 0.02, 0.03, (-0.25, y, -0.393), M.fittings_dark, rot=(90, 0, 0), verts=14, bevel=0.005,
                  bevel_segments=1)
        # a stone ledge with two bottles
        F.soft('ledge', 0.4, 0.04, 0.12, (-0.12, 1.3, -0.36), M.slate_wall, round=0.006, round_segments=1)
        _bottle(F, 'bottle0', (-0.22, 1.32, -0.36), 0.025, 0.16, M.bottle, M.fittings_dark, pump=True)
        _bottle(F, 'bottle1', (-0.12, 1.32, -0.36), 0.025, 0.12, M.sage, M.bottle)
    G.join(still, 'body', origin=(0, 0, 0))
    rain = _rain(F, (0, 2.207, -0.08), 0.14, 0.04, rings=3)
    G.settle()
    G.grow(rain, 'water_on', seconds=0.5)
    return G.meshes()


def showerRain():
    F = G.GameFrame()
    with G.new_parts() as still:
        # 0.93 deep: its back on the wall's face, its front on the tile's edge (the Model's centre)
        F.soft('tray', 0.94, 0.08, 0.93, (0, 0.04, 0.005), M.tray_stone, round=0.01, round_segments=1)
        _grate(F, 'drain', 0.15, 0.08, 0.15, 0.1, 0.1, M.fittings_dark, slots=4)
        F.box('wall_back', 0.94, 2.1, 0.05, (0, 1.13, -0.435), M.wall_stone, bevel=0.008)
        F.box('wall_left', 0.06, 2.0, 0.88, (-0.44, 1.08, 0.03), M.wall_stone, bevel=0.008)
        # a niche in the left wall with two bottles
        F.box('niche_back', 0.004, 0.3, 0.36, (-0.408, 1.25, -0.05), M.groove)
        for k, y in enumerate((1.1, 1.4)):
            F.box(f'niche_sill{k}', 0.03, 0.012, 0.38, (-0.395, y, -0.05), M.wall_stone, bevel=0.003, bevel_segments=1)
        _bottle(F, 'bottle0', (-0.385, 1.106, -0.12), 0.024, 0.15, M.bottle, M.fittings_dark, pump=True)
        _bottle(F, 'bottle1', (-0.385, 1.106, -0.04), 0.022, 0.12, M.sage, M.bottle)
        # glass along the right and the left half of the front: the way in is the front's right half
        F.box('glass_side', 0.012, 1.9, 0.86, (0.45, 1.03, 0.0), M.glass)
        F.box('glass_front', 0.46, 1.9, 0.012, (-0.18, 1.03, 0.45), M.glass)
        # the frame: the corner post bent into the top rails in one piece, a foot channel
        _frame(F, 'frame', [(0.45, 0.08, 0.45), (0.45, 2.02, 0.45), (0.45, 2.02, -0.41)], M.frame_dark, t=0.02, up=(1, 0, 0))
        F.sweep('rail_front', [(0.44, 2.02, 0.45), (-0.41, 2.02, 0.45)], None, M.frame_dark,
                profile=G.rounded_rect(0.02, 0.02, 0.004, corner=1))
        F.box('foot', 0.03, 0.012, 0.88, (0.45, 0.086, 0.0), M.frame_dark, bevel=0.003)
        # the square rain head on an arm from the back wall, and a mixer plate with its lever
        _wall_rose(F, 'arm_rose', 0, 2.12, -0.41, 0.025, M.fittings_dark, h=0.012, verts=14)
        _tube(F, 'arm', _bends([(0, 2.12, -0.405), (0, 2.12, -0.03), (0, 2.17, -0.03)], 0.03, 4), 0.0125, M.fittings_dark,
              verts=10, round=1)
        F.soft('head', 0.3, 0.02, 0.3, (0, 2.17, 0), M.fittings_dark, round=0.008, round_segments=2)
        for i in range(4):
            F.box(f'nozzle_row{i}', 0.24, 0.002, 0.006, (0, 2.159, -0.09 + 0.06 * i), M.dark)
        F.soft('mixer', 0.06, 0.18, 0.012, (0.3, 1.1, -0.404), M.fittings_dark, round=0.004, round_segments=1)
        _tube(F, 'mixer_lever', [(0.3, 1.14, -0.4), (0.3, 1.14, -0.37), (0.3, 1.1, -0.362)], 0.008, M.fittings_dark, verts=8,
              round=2)
    G.join(still, 'body', origin=(0, 0, 0))
    rain = _rain(F, (0, 2.158, 0), 0, 0.08, grid=(5, 0.055))
    G.settle()
    G.grow(rain, 'water_on', seconds=0.5)
    return G.meshes()


# --- the baths: a tub, its taps and the bath water ----------------------------------------------------------
# 2 x 1 tiles, centred; the resident sits along the tub, back at the end without the taps and legs out towards
# them (characters.ts SEATS: 0.25-0.28 m, so the tub's floor is 10-13 cm below). Each tub is one closed shell lofted through rounded rings (`_loft`): up the
# outside, over the rim and down the inside to the floor, so its inside faces up into the tub.
# Two moving parts: the tap's stream (`tap_on`, 0.4 s, held 2.4 s so the tap runs while the tub fills) and the
# bath water (`fill`): a body of water in the tub's shape up to two thirds of its depth, its origin just under
# the tub's floor and scaled to nothing upright, which waits 0.4 s for the tap and then rises in 2 s; it stays
# for the whole bath and drains as the bath ends (the game plays `fill` back).
BATH_TAP, BATH_FILL = 0.4, 2.0


def _ring(a, b, n, segments):
    """A superellipse |x/a|^n + |y/b|^n = 1 (n = 2: an ellipse; larger: squarer), as `segments` points."""
    pts = []
    for k in range(segments):
        t = 2 * math.pi * k / segments
        c, s = math.cos(t), math.sin(t)
        pts.append((a * math.copysign(abs(c) ** (2 / n), c), b * math.copysign(abs(s) ** (2 / n), s)))
    return pts


def _loft(name, sections, material, at=(0, 0, 0), segments=48, sharp=40):
    """A closed body through superellipse rings `sections` [(a, b, height, n)]: half its length (game x) and
    half its depth at each height, in profile order; both end rings are capped. Edges sharper than `sharp`
    degrees are shaded flat."""
    verts, faces = [], []
    for a, b, y, n in sections:
        verts += [(x, z, y) for x, z in _ring(a, b, n, segments)]
    m = len(sections)
    for i in range(m - 1):
        for k in range(segments):
            k2 = (k + 1) % segments
            faces.append((i * segments + k, i * segments + k2, (i + 1) * segments + k2, (i + 1) * segments + k))
    for i in (0, m - 1):
        verts.append((0, 0, sections[i][2]))
        faces += [(i * segments + k, i * segments + (k + 1) % segments, len(verts) - 1) for k in range(segments)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    _recalc_normals(obj)
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.shade_smooth_by_angle(angle=math.radians(sharp))
    obj.location = at
    return obj


def _stream(F, tip, floor_y, name='stream'):
    """A tap's stream from its spout `tip` down to the tub's floor, hidden at rest (the sinks' `_water`)."""
    x, y, z = tip
    with G.new_parts() as parts:
        F.cyl(name, 0.011, y - floor_y, (x, (y + floor_y) / 2, z), M.water, verts=10)
    obj = G.join(parts, name, origin=F.at(x, y, z))
    obj.scale[2] = 0.001
    return obj


def _bath_water(F, floor, top, inset=0.006):
    """The bath water: the tub's inside from just under its floor ring `floor` (a, b, y, n) up to `top`,
    a little in from the walls, its origin under the floor; scaled to nothing upright and in to the floor's
    size across, so it is hidden under the floor at rest and widens with the walls as it rises."""
    a0, b0, y0, n0 = floor
    a1, b1, y1, n1 = top
    water = _loft('bathwater', [(a0 - inset, b0 - inset, y0 - 0.004, n0), (a1 - inset, b1 - inset, y1, n1)], M.water,
                  segments=40)
    G.set_origin(water, F.at(0, y0 - 0.006, 0))
    water.scale = ((a0 - inset) / (a1 - inset), (b0 - inset) / (b1 - inset), 0.001)
    return water


def _inside_at(sections, y):
    """The inside ring (a, b, y, n) at height `y`, between the two inside sections around it (the ones
    after the rim, going down)."""
    inner = sections[[s[2] for s in sections].index(max(s[2] for s in sections)):]
    for (a0, b0, y0, n0), (a1, b1, y1, n1) in zip(inner, inner[1:]):
        if y1 <= y <= y0 and y0 > y1:
            t = (y0 - y) / (y0 - y1)
            return (a0 + (a1 - a0) * t, b0 + (b1 - b0) * t, y, n0 + (n1 - n0) * t)
    raise ValueError(y)


def _fill(stream, water):
    """The clips: the stream runs out (`tap_on`), the water waits for it and rises (`fill`)."""
    G.grow(stream, 'tap_on', seconds=BATH_TAP)
    start, end = last_frame(BATH_TAP), last_frame(BATH_TAP + BATH_FILL)
    for axis in range(3):
        rest = water.scale[axis]
        _clip(water, 'fill', 'scale', axis, [(1, rest), (start, rest), (end, 1.0)])


def _tub(sections, rim, floor_y):
    """The tub's shell; returns its floor ring and the water's top ring, two thirds of the way up."""
    _loft('tub', sections, M.porcelain)
    fill = floor_y + (rim - floor_y) * 2 / 3
    return sections[-1], _inside_at(sections, fill)


def _duck(F, x, y, z):
    """A rubber duck on the deck at game (x, y, z), looking along -x: a plump body rising to a tail, a head, a beak."""
    G.sphere('duck', 0.035, at=F.at(x, y + 0.028, z), material=M.duck, scale=(1.25, 1, 0.85), segments=12, rings=8)
    F.cone('duck_tail', 0.016, 0.002, 0.03, (x + 0.04, y + 0.045, z), M.duck, rot=(0, 60, 0), verts=8)
    F.sphere('duck_head', 0.022, (x - 0.022, y + 0.07, z), M.duck, segments=10, rings=7)
    F.cone('duck_beak', 0.009, 0.002, 0.02, (x - 0.046, y + 0.066, z), M.beak, rot=(0, -90, 0), verts=8)


def _folded_towel(F, name, w, d, x, y, z, material, layers=2, t=0.024):
    """A towel folded on a surface at height `y`: soft layers, each a little back from the one under it."""
    for k in range(layers):
        F.soft(f'{name}{k}', w, t, d - 0.01 * k, (x, y + t * (k + 0.5), z - 0.005 * k), material, round=t * 0.45, puff=(0, 0.004, 0),
               round_segments=2, flat=2)


def _draped_towel(F, name, x, w, pts, material, t=0.012):
    """A towel hanging over an edge through game points (height, depth) at `x`, `w` wide, `t` thick."""
    F.sweep(name, [(x, y, z) for y, z in pts], None, material, profile=G.rounded_rect(w, t, t * 0.45, corner=2), up=(1, 0, 0),
            resolution=2)


def _waste(F, end_x, rim, floor_y, inner_x, material):
    """A bath's plug on its floor and the overflow's round plate high on its end wall at game x `inner_x`."""
    _rose(F, 'plug', end_x, floor_y, 0, 0.03, material, h=0.006, verts=16)
    side = 1 if inner_x > 0 else -1
    F.cyl('overflow', 0.028, 0.008, (inner_x - side * 0.004, rim - 0.1, 0), material, rot=(0, 90, 0), verts=16, bevel=0.003,
          bevel_segments=1)


def bathtub():
    F = G.GameFrame()
    rim, floor_y = 0.58, 0.15
    sections = [(0.70, 0.24, 0.12, 3), (0.79, 0.33, 0.48, 4),        # the tub's outside, behind the panels
                (0.91, 0.43, 0.50, 10), (0.91, 0.43, rim, 10),         # the deck's edge
                (0.77, 0.29, rim, 4), (0.765, 0.285, rim - 0.012, 4),   # the deck, and the inside's lip
                (0.71, 0.25, 0.25, 3.5), (0.66, 0.2, floor_y, 3)]      # down the inside to the floor
    with G.new_parts() as still:
        floor_ring, top = _tub(sections, rim, floor_y)
        # the panels down to a plinth set back under them, each with a moulded frame on it
        F.soft('panel_front', 1.82, 0.45, 0.03, (0, 0.275, 0.415), M.bath_panel, round=0.006, round_segments=1)
        F.extrude('panel_moulding', G.rounded_rect(1.7, 0.33, 0.01, corner=2), 0.006, (0, 0.285, 0.432), M.bath_panel,
                  holes=[G.rounded_rect(1.66, 0.29, 0.008, corner=2)], bevel=0.002, bevel_segments=1)
        for s in (-1, 1):
            F.soft(f'panel_end{s}', 0.03, 0.45, 0.83, (s * 0.895, 0.275, -0.015), M.bath_panel, round=0.006, round_segments=1)
        F.box('plinth', 1.76, 0.05, 0.8, (0, 0.025, -0.025), M.dark)
        _waste(F, -0.55, rim, floor_y, -0.705, M.chrome)
        # a deck mixer at the left end: a pillar, a spout curving over the water, two lever handles
        _rose(F, 'tap_rose', -0.85, rim, 0, 0.034, M.chrome, h=0.01)
        _lathe(F, 'tap_pillar', -0.85, 0, [(rim, 0.024, 0.024), (rim + 0.08, 0.022, 0.022), (rim + 0.095, 0.026, 0.026),
                                           (rim + 0.11, 0.02, 0.02)], M.chrome, verts=14)
        _tube(F, 'spout', _bends([(-0.85, rim + 0.09, 0), (-0.68, rim + 0.09, 0), (-0.68, rim + 0.06, 0)], 0.025, 4), 0.015,
              M.chrome, verts=10, round=1, scale=[1.0, 1.0, 0.9])
        for z in (-0.1, 0.1):
            _rose(F, f'handle_rose{z}', -0.85, rim, z, 0.026, M.chrome, h=0.008, verts=12)
            F.cyl(f'handle{z}', 0.018, 0.04, (-0.85, rim + 0.025, z), M.chrome, verts=12, bevel=0.004, bevel_segments=1)
            _tube(F, f'handle_lever{z}', [(-0.85, rim + 0.045, z), (-0.81, rim + 0.05, z), (-0.79, rim + 0.054, z)], 0.006, M.chrome,
                  verts=8, round=2, scale=[1.2, 1.0, 0.8])
        _folded_towel(F, 'towel', 0.4, 0.12, 0.55, rim, 0.36, M.towel)
        _duck(F, 0.12, rim, 0.36)
    G.join(still, 'body', origin=(0, 0, 0))
    stream = _stream(F, (-0.68, rim + 0.045, 0), floor_y)
    water = _bath_water(F, floor_ring, top)
    G.settle()
    _fill(stream, water)
    return G.meshes()


def _claw(F, x, z):
    """A brass ball-and-claw foot: a leg curving out from under the tub down onto a ball gripped by three claws."""
    sx, sz = (1 if x > 0 else -1), (1 if z > 0 else -1)
    _tube(F, f'leg{x}{z}', [(x - sx * 0.03, 0.2, z - sz * 0.02), (x + sx * 0.0, 0.14, z), (x + sx * 0.012, 0.08, z + sz * 0.008)],
          0.03, M.feet, verts=10, round=3, scale=[1.3, 0.85, 0.75])
    F.sphere(f'ball{x}{z}', 0.034, (x + sx * 0.012, 0.034, z + sz * 0.008), M.feet, segments=12, rings=8)
    for k in range(3):
        a = math.radians((k - 1) * 55)
        dx, dz = sx * math.cos(a) * 0.7 + 0.0, sz * math.sin(a) * 0.7 + sz * 0.3
        L = math.hypot(dx, dz)
        dx, dz = dx / L, dz / L
        cx, cz = x + sx * 0.012, z + sz * 0.008
        _tube(F, f'claw{x}{z}{k}', [(cx + dx * 0.012, 0.085, cz + dz * 0.012), (cx + dx * 0.03, 0.06, cz + dz * 0.03),
                                    (cx + dx * 0.036, 0.03, cz + dz * 0.036), (cx + dx * 0.03, 0.006, cz + dz * 0.03)],
              0.009, M.feet, verts=6, round=2, scale=[1.0, 1.0, 0.8, 0.5])


def bathtubClawfoot():
    F = G.GameFrame()
    rim, floor_y = 0.7, 0.17
    sections = [(0.6, 0.24, 0.14, 2.3), (0.75, 0.33, 0.3, 2.4), (0.84, 0.385, 0.6, 2.5),   # the outside
                (0.865, 0.398, 0.665, 2.5), (0.852, 0.392, rim, 2.5),                      # the rolled top
                (0.815, 0.358, rim, 2.5), (0.795, 0.342, 0.665, 2.5),
                (0.74, 0.3, 0.45, 2.4), (0.64, 0.22, 0.22, 2.3), (0.56, 0.17, floor_y, 2.3)]  # the inside
    with G.new_parts() as still:
        floor_ring, top = _tub(sections, rim, floor_y)
        # the rolled rim: a round bead swept all the way round the top
        F.sweep('rim_roll', [(x, rim - 0.008, z) for x, z in _ring(0.834, 0.374, 2.5, 48)], 0.021, M.porcelain, verts=8,
                closed=True, angle=60)
        for x in (-0.5, 0.5):
            for z in (-0.16, 0.16):
                _claw(F, x, z)
        _waste(F, -0.45, rim, floor_y, -0.775, M.brass)
        # a brass filler on the left end: a riser up over the rim and out over the water, two cross handles on a bar
        _tube(F, 'filler', _bends([(-0.785, rim - 0.01, 0), (-0.785, 1.0, 0), (-0.6, 1.0, 0), (-0.6, 0.93, 0)], 0.04, 5), 0.016,
              M.brass, verts=10, round=1)
        F.cyl('filler_aerator', 0.019, 0.022, (-0.6, 0.93, 0), M.brass, verts=10)
        _tube(F, 'filler_bar', [(-0.785, 0.8, -0.09), (-0.785, 0.8, 0.09)], 0.013, M.brass, verts=10, round=1)
        for z in (-0.08, 0.08):
            F.cyl(f'filler_stem{z}', 0.01, 0.04, (-0.785, 0.82, z), M.brass, verts=8)
            _rose(F, f'filler_hub{z}', -0.785, 0.835, z, 0.014, M.brass, h=0.012, verts=10)
            for a in (0, 90):
                ca, sa = math.cos(math.radians(a)), math.sin(math.radians(a))
                _tube(F, f'filler_cross{z}{a}', [(-0.785 - ca * 0.032, 0.845, z - sa * 0.032), (-0.785 + ca * 0.032, 0.845, z + sa * 0.032)],
                      0.006, M.brass, verts=6, round=1)
                for s in (-1, 1):
                    F.sphere(f'filler_tip{z}{a}{s}', 0.008, (-0.785 + s * ca * 0.034, 0.845, z + s * sa * 0.034), M.brass,
                             segments=6, rings=4)
        # a telephone hand shower on its cradle on the riser
        _tube(F, 'cradle', _bends([(-0.785, 0.9, 0), (-0.785, 0.9, 0.05), (-0.785, 0.93, 0.07)], 0.015, 3), 0.008, M.brass,
              verts=8, round=1)
        _tube(F, 'handset', [(-0.785, 0.93, 0.03), (-0.785, 0.945, 0.11)], 0.012, M.brass, verts=10, round=1, scale=[0.9, 1.1])
        F.cyl('handset_head', 0.022, 0.03, (-0.785, 0.955, 0.125), M.brass, rot=(-60, 0, 0), verts=12, bevel=0.005,
              bevel_segments=1)
        # a towel over the back rim at the right end
        _draped_towel(F, 'towel', 0.5, 0.32, [(rim + 0.008, -0.31), (rim + 0.012, -0.37), (rim, -0.415), (rim - 0.04, -0.425),
                                              (rim - 0.24, -0.424)], M.towel)
    G.join(still, 'body', origin=(0, 0, 0))
    stream = _stream(F, (-0.6, 0.919, 0), floor_y)
    water = _bath_water(F, floor_ring, top)
    G.settle()
    _fill(stream, water)
    return G.meshes()


def bathtubFreestanding():
    F = G.GameFrame()
    rim, floor_y = 0.58, 0.15
    sections = [(0.66, 0.27, 0.0, 2.2), (0.7, 0.3, 0.03, 2.2), (0.86, 0.395, 0.45, 2.3),   # the outside
                (0.895, 0.41, 0.56, 2.3), (0.885, 0.405, rim, 2.3),                       # the rim
                (0.85, 0.372, rim, 2.3), (0.832, 0.356, rim - 0.025, 2.3),
                (0.77, 0.315, 0.35, 2.25), (0.67, 0.245, 0.18, 2.2), (0.61, 0.205, floor_y, 2.2)]  # the inside
    with G.new_parts() as still:
        floor_ring, top = _tub(sections, rim, floor_y)
        _waste(F, 0.5, rim, floor_y, 0.84, M.chrome)
        # a floor-standing filler at the right end: a rose on the floor, a column with its mixer and a handset,
        # bent over into a spout above the water
        _rose(F, 'filler_base', 0.93, 0.0, 0.12, 0.07, M.chrome, h=0.02, verts=20)
        _tube(F, 'filler', _bends([(0.93, 0.02, 0.12), (0.93, 0.9, 0.12), (0.61, 0.9, 0.0), (0.61, 0.82, 0.0)], 0.06, 6), 0.019,
              M.chrome, verts=12, round=1)
        F.cyl('filler_aerator', 0.021, 0.02, (0.61, 0.822, 0.0), M.chrome, verts=12)
        _lathe(F, 'filler_mixer', 0.93, 0.12, [(0.7, 0.024, 0.024), (0.71, 0.03, 0.03), (0.77, 0.03, 0.03), (0.78, 0.024, 0.024)],
               M.chrome, verts=14)
        _tube(F, 'filler_lever', [(0.93, 0.775, 0.12), (0.93, 0.79, 0.15), (0.93, 0.8, 0.19)], 0.007, M.chrome, verts=8, round=2,
              scale=[1.2, 1.0, 0.8])
        _tube(F, 'handset', [(0.955, 0.5, 0.12), (0.955, 0.66, 0.12)], 0.012, M.chrome, verts=10, round=1, scale=[0.9, 1.1])
        F.cyl('handset_head', 0.022, 0.03, (0.955, 0.675, 0.12), M.chrome, verts=12, bevel=0.005, bevel_segments=1)
        _tube(F, 'hose', [(0.955, 0.5, 0.12), (0.97, 0.3, 0.13), (0.95, 0.1, 0.12), (0.93, 0.03, 0.12)], 0.006, M.chrome, verts=6,
              round=3)
        # a towel over the back rim at the left end
        _draped_towel(F, 'towel', -0.5, 0.32, [(rim + 0.008, -0.3), (rim + 0.012, -0.36), (rim, -0.41), (rim - 0.04, -0.415),
                                               (rim - 0.22, -0.41)], M.towel)
    G.join(still, 'body', origin=(0, 0, 0))
    stream = _stream(F, (0.61, 0.81, 0.0), floor_y)
    water = _bath_water(F, floor_ring, top)
    G.settle()
    _fill(stream, water)
    return G.meshes()


# --- bathroom storage and mirrors -------------------------------------------------------------------------
# Built where the game places them. The two wall things hang at their real height, their backs on the wall
# face (`G.settle(centre=False, floor=False)`, `floor=False` on the Model). Mirror glass is a flat piece of
# the `mirror` material (`gloss`, `'front'`): the game reflects the room in it where the preset has live mirrors
# (High) and draws it pale gloss elsewhere, as the old parts' mirrors. The cabinets' doors are parts of
# their own, hinged on their left edge. The mirror cabinet's swings open (`door_open`, the mirror with it)
# while a resident brushes their teeth; the tall cabinet's upper door onto its towels while one takes a
# fresh towel.


def _carcass(F, name, w, h, d, x, y0, z0, material, t=0.016):
    """An open-fronted box: back, sides, top and bottom, `w` x `h` x `d`, its back at game depth `z0`."""
    zc = z0 + d / 2
    F.box(f'{name}_back', w, h, t, (x, y0 + h / 2, z0 + t / 2), material)
    for s_ in (-1, 1):
        F.box(f'{name}_side{s_}', t, h, d, (x + s_ * (w - t) / 2, y0 + h / 2, zc), material, bevel=0.003)
    for k, y in enumerate((y0 + t / 2, y0 + h - t / 2)):
        F.box(f'{name}_end{k}', w, t, d, (x, y, zc), material, bevel=0.003)


def _hinge_cups(F, name, x, ys, z, material):
    """A cabinet's hinges inside its side at game x, at heights `ys`: a plate on the side and an arm towards the
    door at depth `z`."""
    for k, y in enumerate(ys):
        F.box(f'{name}{k}', 0.012, 0.045, 0.05, (x, y, z - 0.03), material, bevel=0.003, bevel_segments=1)


def _cup(F, name, x, y, z, r, h, material):
    """A tumbler standing at height `y`: a slightly flared cup with a rim, open at the top."""
    _lathe(F, name, x, z, [(y, r * 0.86, r * 0.86), (y + 0.004, r * 0.92, r * 0.92), (y + h, r, r), (y + h, r - 0.004, r - 0.004),
                           (y + 0.012, r * 0.86 - 0.004, r * 0.86 - 0.004)], material, verts=14)


def _brushes(F, name, x, y, z, colours):
    """Toothbrushes leaning out of a cup whose floor is at height `y`."""
    for k, m in enumerate(colours):
        s = 1 if k % 2 else -1
        _tube(F, f'{name}{k}', [(x + s * 0.004, y + 0.01, z), (x + s * 0.02, y + 0.14, z + 0.008 * s)], 0.0045, m, verts=6, round=1)
        F.box(f'{name}_head{k}', 0.008, 0.024, 0.012, (x + s * 0.0215, y + 0.155, z + 0.008 * s), M.paper, bevel=0.002,
              bevel_segments=1)


def mirrorCabinet():
    F = G.GameFrame()
    w, h, d, y0, z0 = 0.56, 0.72, 0.16, 1.2, WALL
    front = z0 + d
    with G.new_parts() as still:
        _carcass(F, 'cabinet', w, h, d, 0, y0, z0, M.cab)
        _hinge_cups(F, 'hinge', -(w / 2 - 0.022), (y0 + 0.1, y0 + h - 0.1), front, M.chrome)
        # two shelves with a lip along the front, a cup of toothbrushes, bottles, a jar and a tube
        for k, y in enumerate((1.43, 1.66)):
            F.box(f'shelf{k}', w - 0.04, 0.008, d - 0.03, (0, y, z0 + d / 2), M.porcelain, bevel=0.002)
            F.box(f'shelf_lip{k}', w - 0.04, 0.02, 0.006, (0, y + 0.006, z0 + d - 0.018), M.porcelain, bevel=0.002,
                  bevel_segments=1)
        _cup(F, 'cup', -0.15, y0 + 0.016, -0.36, 0.03, 0.09, M.teal)
        _brushes(F, 'brush', -0.15, y0 + 0.028, -0.36, (M.porcelain, M.sage))
        _bottle(F, 'bottle0', (0.12, 1.434, -0.36), 0.022, 0.12, M.bottle, M.teal, pump=True)
        _bottle(F, 'bottle1', (0.18, 1.434, -0.35), 0.02, 0.09, M.sage, M.bottle)
        _lathe(F, 'jar', -0.1, -0.36, [(1.664, 0.028, 0.028), (1.668, 0.03, 0.03), (1.71, 0.03, 0.03), (1.712, 0.026, 0.026)],
               M.porcelain, verts=14)
        F.cyl('jar_lid', 0.031, 0.014, (-0.1, 1.718, -0.36), M.chrome, verts=14, bevel=0.003, bevel_segments=1)
        F.sweep('paste', [(0.05, y0 + 0.03, -0.36), (0.14, y0 + 0.03, -0.355)], None, M.paper,
                profile=G.rounded_rect(0.03, 0.012, 0.005, corner=1), up=(0, 0, 1), scale=[1.0, 0.5])
        F.cyl('paste_cap', 0.008, 0.016, (0.148, y0 + 0.03, -0.355), M.teal, rot=(0, 90, 0), verts=8)
        # a light bar over the top on an arm from the wall
        _tube(F, 'lamp_arm', _bends([(0, 1.935, z0 + 0.004), (0, 1.935, -0.36), (0, 1.95, -0.345)], 0.015, 3), 0.007, M.chrome,
              verts=8, round=1)
        _wall_rose(F, 'lamp_rose', 0, 1.935, z0, 0.02, M.chrome, h=0.008, verts=12)
        F.soft('lamp', 0.46, 0.04, 0.06, (0, 1.95, -0.33), M.cab, round=0.015, round_segments=2)
        F.soft('lamp_strip', 0.42, 0.01, 0.03, (0, 1.927, -0.32), M.bulb, round=0.004, round_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.soft('door', w - 0.02, h - 0.02, 0.02, (0, y0 + h / 2, front + 0.01), M.cab_door, round=0.008, round_segments=2)
        F.box('door_mirror', 0.48, 0.64, 0.004, (0, y0 + h / 2, front + 0.0235), M.mirror)
        _pull(F, 'door_pull', 0.25, y0 + h / 2, front + 0.02, 0.1, M.chrome, r=0.005, stand=0.016)
        for k, y in enumerate((y0 + 0.1, y0 + h - 0.1)):
            F.box(f'door_hinge{k}', 0.02, 0.045, 0.012, (-(w / 2 - 0.04), y, front - 0.006), M.chrome, bevel=0.003,
                  bevel_segments=1)
    door = G.hinged(G.join(parts, 'door'), F.at(-(w - 0.02) / 2, y0 + h / 2, front))
    G.settle(centre=False, floor=False)
    G.swing(door, 'door_open', 100, seconds=0.8)
    return G.meshes()


def mirrorRound():
    F = G.GameFrame()
    y, z, r = 1.55, -0.405, 0.3
    with G.new_parts() as still:
        # the frame: a moulded section, flat at the back and rounded to the front, swept round the circle
        moulding = [(-0.02, -0.02), (-0.02, 0.02), (0.0, 0.02), (0.012, 0.016), (0.019, 0.007), (0.02, 0.0), (0.019, -0.007),
                    (0.012, -0.016), (0.0, -0.02)]
        F.sweep('frame', [(r * math.cos(math.tau * i / 48), y + r * math.sin(math.tau * i / 48), z) for i in range(48)], None,
                M.frame_brass, profile=moulding, closed=True, up=(0, -1, 0), angle=60)
        F.cyl('backing', r, 0.01, (0, y, z - 0.012), M.frame_brass, rot=(90, 0, 0), verts=48)
        # a leather strap riveted to the frame's top, up over a brass hook on the wall
        hook = (0, 1.98, WALL + 0.02)
        F.sweep('strap', [(-0.075, 1.83, z + 0.012), (-0.06, 1.87, z + 0.01), (-0.025, 1.94, WALL + 0.032), (0, 1.99, WALL + 0.036),
                          (0.025, 1.94, WALL + 0.032), (0.06, 1.87, z + 0.01), (0.075, 1.83, z + 0.012)], None, M.leather,
                profile=G.rounded_rect(0.004, 0.026, 0.0015, corner=1), up=(0, -1, 0), resolution=2)
        for s_ in (-1, 1):
            F.sphere(f'rivet{s_}', 0.005, (s_ * 0.072, 1.838, z + 0.014), M.frame_brass, segments=8, rings=4)
        _wall_rose(F, 'hook_plate', hook[0], hook[1], WALL, 0.022, M.frame_brass, h=0.01, verts=14)
        _tube(F, 'hook', _bends([(0, hook[1], WALL + 0.008), (0, hook[1], WALL + 0.04), (0, hook[1] + 0.025, WALL + 0.04)], 0.012, 3),
              0.006, M.frame_brass, verts=8, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as glass:
        F.cyl('mirror', r - 0.012, 0.004, (0, y, z - 0.004), M.mirror, rot=(90, 0, 0), verts=48)
    G.join(glass, 'mirror')
    G.join_all('mirrorRound')
    G.settle(centre=False, floor=False)
    return G.meshes()


def bathCabinetTall():
    F = G.GameFrame()
    w, d, z0 = 0.42, 0.34, WALL
    front = z0 + d
    niche = (0.61, 0.86)        # the open shelf between the doors
    with G.new_parts() as still:
        F.box('plinth', 0.4, 0.06, 0.3, (0, 0.03, -0.27), M.dark, bevel=0.004)
        _carcass(F, 'cabinet', w, 1.76, d, 0, 0.06, z0, M.cab)
        F.box('niche_floor', w - 0.03, 0.022, d - 0.02, (0, niche[0] + 0.011, z0 + d / 2), M.cab, bevel=0.003)
        F.box('niche_top', w - 0.03, 0.022, d - 0.02, (0, niche[1] - 0.011, z0 + d / 2), M.cab, bevel=0.003)
        # two towels rolled in the niche
        for k, (x, m) in enumerate(((-0.09, M.towel_blue), (0.09, M.towel_cream))):
            _tube(F, f'towel{k}', [(x, niche[0] + 0.07, -0.4), (x, niche[0] + 0.07, -0.13)], 0.048, m, verts=12, round=1)
            F.cyl(f'towel_end{k}', 0.03, 0.004, (x, niche[0] + 0.07, -0.128), m, rot=(90, 0, 0), verts=10)
        # behind the upper door: a stack of folded towels, a shelf with two more and bottles on those
        for k, m in enumerate((M.towel_cream, M.towel_blue, M.towel_cream)):
            F.soft(f'stack{k}', 0.3, 0.072, 0.24, (0, niche[1] + 0.047 + 0.075 * k, -0.26), m, round=0.03, puff=(0, 0.006, 0),
                   round_segments=2, flat=2)
        F.box('shelf', w - 0.03, 0.018, d - 0.03, (0, 1.36, z0 + d / 2 - 0.005), M.cab, bevel=0.003)
        for k, m in enumerate((M.towel_blue, M.towel_cream)):
            F.soft(f'stack_top{k}', 0.28, 0.068, 0.22, (0, 1.404 + 0.07 * k, -0.27), m, round=0.028, puff=(0, 0.006, 0),
                   round_segments=2, flat=2)
        _bottle(F, 'bottle0', (-0.08, 1.509, -0.3), 0.025, 0.14, M.bottle, M.teal, pump=True)
        _bottle(F, 'bottle1', (0.04, 1.509, -0.31), 0.022, 0.11, M.sage, M.bottle)
        _hinge_cups(F, 'hinge', -(w / 2 - 0.024), (niche[1] + 0.12, 1.69), front, M.chrome)
        # the lower door (still), shaker-panelled as the upper one
        _shaker(F, 'door_low', 0.4, niche[0] - 0.08, 0, (0.07 + niche[0]) / 2, front, M.cab_door, rail=0.055)
        _pull(F, 'pull_low', 0.15, niche[0] - 0.1, front + 0.02, 0.1, M.chrome, r=0.005, stand=0.015)
    G.join(still, 'body', origin=(0, 0, 0))
    top = 1.81
    with G.new_parts() as parts:
        _shaker(F, 'door', 0.4, top - niche[1] - 0.01, 0, (niche[1] + top) / 2, front, M.cab_door, rail=0.055)
        _pull(F, 'pull', 0.15, niche[1] + 0.1, front + 0.02, 0.12, M.chrome, r=0.005, stand=0.015)
    door = G.hinged(G.join(parts, 'door'), F.at(-0.2, (niche[1] + top) / 2, front))
    G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=0.8)
    return G.meshes()


TOILET = {'uses': {'toilet': 'lid_up'}}
BATH = {'uses': {'bath': [{'clip': 'tap_on', 'hold': BATH_TAP + BATH_FILL}, 'fill']}}
SHOWER = {'uses': {'shower': 'water_on'}}
# The base shower's door slides open as the resident steps in, shuts behind them, and opens again to let them
# out; the water comes on once it's shut. The stall's curtain is drawn across once they're in, back at the end.
SHOWER_DOOR = {'uses': {'shower': [{'clip': 'door_open', 'hold': 1.5, 'out': 1.5},
                                   {'clip': 'water_on', 'delay': 2.0}]}}
SHOWER_CURTAIN = {'uses': {'shower': [{'clip': 'curtain_draw', 'delay': 1.0}, {'clip': 'water_on', 'delay': 1.8}]}}

FAMILY = G.Family(
    'bathroom', M,
    [G.Model('sink', sink, size=(0.86, 1.262, 0.621), key='model.sink', centre=(0, -0.121),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('stone', 'Worktop'), ('metal', 'Basin'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')]),
     G.Model('sinkPedestal', sinkPedestal, size=(0.56, 1.060, 0.464), key='model.sinkPedestal', centre=(0, -0.195),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Pedestal'), ('porcelain', 'Basin & cup'), ('chrome', 'Tap')]),
     G.Model('vanityDouble', vanityDouble, size=(1.90, 1.002, 0.562), key='model.vanityDouble', centre=(0, -0.149),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('stone', 'Worktop'), ('porcelain', 'Basins'), ('chrome', 'Taps & handles'),
                    ('trim', 'Plinth')]),
     G.Model('sinkVessel', sinkVessel, size=(0.84, 1.060, 0.540), key='model.sinkVessel', centre=(0, -0.16),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Vanity'), ('stone', 'Worktop'), ('porcelain', 'Bowl'), ('blackmetal', 'Tap'),
                    ('wood', 'Shelf & legs'), ('towel', 'Towels')]),
     G.Model('sinkConsole', sinkConsole, size=(0.60, 1.062, 0.480), key='model.sinkConsole', centre=(0, -0.188),
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
             paint=[('porcelain', 'Bowl'), ('seat', 'Seat & lid'), ('boxing', 'Cistern')]),
     G.Model('shower', shower, size=(0.9, 2.15, 0.9), key='model.shower', centre=(0, 0.02), motions=SHOWER_DOOR,
             paint=[('tray', 'Tray'), ('frame', 'Frame'), ('fittings', 'Head & tap'), ('tile', 'Tiles')]),
     G.Model('showerStall', showerStall, size=(0.9, 2.08, 0.9), key='model.showerStall', centre=(0, 0.02), motions=SHOWER_CURTAIN,
             paint=[('tray', 'Tray'), ('frame', 'Rail'), ('fittings', 'Head & tap'), ('curtain', 'Curtain'),
                    ('tile', 'Tiles')]),
     G.Model('showerWalkIn', showerWalkIn, size=(0.92, 2.26, 0.92), key='model.showerWalkIn', centre=(0, 0.03), motions=SHOWER,
             paint=[('slate', 'Floor'), ('frame_dark', 'Frame'), ('fittings_dark', 'Head & tap'),
                    ('slate_wall', 'Wall')]),
     G.Model('showerRain', showerRain, size=(0.94, 2.192, 0.93), key='model.showerRain', centre=(0, 0.035), motions=SHOWER,
             paint=[('tray_stone', 'Tray'), ('frame_dark', 'Frame'), ('fittings_dark', 'Head & tap'),
                    ('wall_stone', 'Walls')]),
     G.Model('bathtub', bathtub, size=(1.82, 0.69, 0.86), key='model.bathtub', motions=BATH,
             paint=[('porcelain', 'Tub'), ('bath_panel', 'Panel'), ('chrome', 'Taps'), ('towel', 'Towel')]),
     G.Model('bathtubClawfoot', bathtubClawfoot, size=(1.73, 1.02, 0.829), key='model.bathtubClawfoot', motions=BATH,
             paint=[('porcelain', 'Tub'), ('feet', 'Feet'), ('brass', 'Taps'), ('towel', 'Towel')]),
     G.Model('bathtubFreestanding', bathtubFreestanding, size=(1.895, 0.919, 0.837), key='model.bathtubFreestanding',
             motions=BATH, paint=[('porcelain', 'Tub'), ('chrome', 'Taps'), ('towel', 'Towel')]),
     G.Model('mirrorCabinet', mirrorCabinet, size=(0.56, 0.77, 0.2), key='model.mirrorCabinet',
             centre=(0, -0.33), floor=False, motions={'uses': {'brushTeeth': 'door_open'}, 'hold': 1.5},
             paint=[('cab', 'Cabinet'), ('cab_door', 'Frame')]),
     G.Model('mirrorRound', mirrorRound, size=(0.64, 0.772, 0.046), key='model.mirrorRound',
             centre=(0, -0.407), floor=False, paint=[('frame_brass', 'Frame'), ('leather', 'Strap')]),
     G.Model('bathCabinetTall', bathCabinetTall, size=(0.42, 1.82, 0.38), key='model.bathCabinetTall',
             centre=(0, -0.24), motions={'uses': {'takeTowel': 'door_open'}, 'hold': 1.5},
             paint=[('cab', 'Cabinet'), ('cab_door', 'Doors'), ('towel_blue', 'Towels')])],
    comment='Blender-built bathroom models (tools/art/blender/families/bathroom.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each sink has a `water_on` clip (the stream from the tap) played while a '
            'resident washes.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
