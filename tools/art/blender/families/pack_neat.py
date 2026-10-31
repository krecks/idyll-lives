# The neat pack (P13): the steam mop dock, the stacked laundry centre, the rainfall steam shower, the
# label-and-sort organizer desk, the robot vacuum and its dock and the tidy entry console.
# Notes: pack_neat.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the mop leaves its dock for the hands (`mop_off`), the washer's door swings open
# (`door_open`), the rain falls (`water_on`) and the shower's glass door swings open (`door_open`), a bin slides out of the
# desk's hutch (`drawer_open`), the robot
# rolls out of its dock and turns (`vacuum_out`) and the console's key tray drawer slides out (`tray`).
#
# Appliances are lofted through rounded rectangles (`G.loft`, `G.ring`), their portholes turned (`G.revolve`); bins,
# bottles and baskets lofted too (the baskets' weave in ridges); towels, pads and cushions stuffed (`G.soft_box`);
# poles, hangers, hooks and rails swept in one piece; the mirror framed with `G.kit.moulding`, the drawer's knob
# `G.kit.knob`.
#
# Run:  cargo art pack_neat [--only steamMopDock] [--preview] [--lit]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import cupped, leaf  # noqa: E402  (leaf blades, tulip cups)

M = G.Palette(
    body=('#F4F3EF', 'satin'),       # crisp white: docks, cabinets, cubbies, frames; the main part
    appliance=('#F7F7F5', 'gloss'),  # the washer and dryer, the robot's dock
    trim=('#D5D7D8', 'satin'),
    birch=('#E3CFA8', 'wood'),       # textured birch: the desk, the console
    birch_plain=('#E3CFA8', 'satin'),
    oak=('#CDB08A', 'satin'),
    teak=('#B07A4A', 'satin'),       # the shower bench, plain
    chrome=('#D0D3D6', 'chrome'),
    steel=('#9EA3A8', 'metal'),
    brass=('#C9A24A', 'metal'),
    dark=('#3A4250', 'satin'),
    screen=('#2E3A46', 'gloss'),
    screen_glow=('#7FC8E8', 'gloss'),
    porthole=('#C9E0E8', 'gloss'),   # porthole and bin windows, plain (no glass draw)
    drum=('#8C949B', 'satin'),
    teal=('#4E7A7F', 'gloss'),
    led=('#6CC27A', 'gloss'),
    tile=('#F1F0EC', 'satin'),
    joint=('#D6D4CE', 'satin'),
    glass=('#DCE8EA', 'glass'),
    water=('#BFE3F2', 'gloss'),
    pad=('#F7F7F7', 'satin'),
    rug=('#D5D7D8', 'fabric'),
    rug_inner=('#E6E5E0', 'fabric'),
    paper=('#F6F3EA', 'matte'),
    ledger=('#3E6E73', 'satin'),
    coral=('#E8846B', 'gloss'),
    wicker=('#D8C29C', 'satin'),
    wicker_b=('#CDB48A', 'satin'),
    green=('#6FA35A', 'satin'),
    stem=('#5E8F4E', 'satin'),
    mirror=('#DCE6EA', 'chrome'),
    shoe_a=('#B33A32', 'satin'),
    shoe_b=('#3E6E73', 'satin'),
    shoe_c=('#E9D58A', 'satin'),
    # soft pastels: towels and shirts (fabric), bins and bottles (satin, gloss)
    towel_a=('#9FD3C7', 'fabric'), towel_b=('#F2C6A0', 'fabric'), towel_c=('#B8C9E8', 'fabric'),
    towel_d=('#E9D58A', 'fabric'), towel_e=('#D9B8D6', 'fabric'), towel_f=('#A8D5A2', 'fabric'),
    towel_w=('#F7F7F7', 'fabric'),
    pastel_a=('#9FD3C7', 'satin'), pastel_b=('#F2C6A0', 'satin'), pastel_c=('#B8C9E8', 'satin'),
    pastel_d=('#E9D58A', 'satin'), pastel_e=('#D9B8D6', 'satin'), pastel_f=('#A8D5A2', 'satin'),
    bottle_a=('#9FD3C7', 'gloss'), bottle_b=('#B8C9E8', 'gloss'), bottle_c=('#F2C6A0', 'gloss'),
)

TOWELS = ('towel_a', 'towel_b', 'towel_c', 'towel_d', 'towel_e', 'towel_f')   # by name: made anew after each reset
PASTELS = ('pastel_a', 'pastel_b', 'pastel_c', 'pastel_d', 'pastel_e', 'pastel_f')
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth


def _m(name):
    return getattr(M, name)


def _towels(F, name, x, z, y, n, w=0.26, d=0.2, names=TOWELS):
    """`n` folded towels stacked on `y`, each plump with its fold rounded; returns the stack's top."""
    for i in range(n):
        F.soft(f'{name}{i}', w, 0.044, d, (x + 0.004 * (i % 2), y + 0.022 + i * 0.046, z), _m(names[i % len(names)]),
               round=0.018, puff=(0, 0.004, 0), flat=1, round_segments=1)
    return y + n * 0.046


def _box(F, name, w, h, d, at, material, r=0.02, sq=8, verts=24):
    """A rounded-cornered box lofted upright (an appliance's cabinet, a dock): `r`-ish corners in plan (`sq`), its top
    and bottom edges eased."""
    x, y, z = at
    bx, by, bz = F.at(x, y - h / 2, z)
    rings = [(0.0, -0.006), (0.006, 0.0), (h - 0.006, 0.0), (h, -0.006)]
    return G.loft(name, [G.ring(w / 2 + e, d / 2 + e, bz + hh, verts, square=sq, at=(bx, by)) for hh, e in rings],
                  material=material)


def _bottle(F, name, x, z, y, material, h=0.22, d=0.08, cap=None):
    """A bottle: a rounded-square body, its shoulder, a neck and a cap."""
    r = d / 2
    bx, by, bz = F.at(x, y, z)
    G.loft(name, [G.ring(r * k, r * k, bz + h * t, 12, square=sq, at=(bx, by)) for t, k, sq in
                  ((0.0, 0.92, 3), (0.03, 1.0, 3), (0.7, 1.0, 3), (0.84, 0.6, 2), (0.88, 0.42, 2), (1.0, 0.42, 2))],
           material=material)
    F.cyl(f'{name}_cap', r * 0.48, 0.03, (x, y + h + 0.012, z), cap or M.body, verts=10, bevel=0.003, bevel_segments=1)


# --- the steam mop dock (1x1): a slim white tower against the wall, the mop docked before it ----------------
MOP = (0.0, 0.04, -0.07)    # the mop head's foot (x, height, depth): it pivots there


def steamMopDock():
    F = G.GameFrame()
    with G.new_parts() as still:
        _box(F, 'tower', 0.42, 1.25, 0.2, (0, 0.625, WALL + 0.1), M.body, sq=5)
        F.soft('panel', 0.34, 0.26, 0.012, (0, 1.0, WALL + 0.204), M.porthole, round=0.006, round_segments=1)
        for x, w, y in ((-0.08, 0.1, 1.06), (0.04, 0.16, 1.02), (-0.02, 0.2, 0.98)):
            F.box(f'panel_line{y}', w, 0.012, 0.004, (x, y, WALL + 0.212), M.teal, bevel=0.002, bevel_segments=1)
        F.sphere('led', 0.012, (0.12, 0.92, WALL + 0.212), M.led, segments=8, rings=4)
        _box(F, 'base_plate', 0.44, 0.04, 0.36, (0, 0.02, WALL + 0.18), M.trim, sq=4, verts=20)
        # a C-clip holding the pole, on a stub from the tower
        F.box('clip_stub', 0.04, 0.03, 0.03, (MOP[0], 1.1, WALL + 0.215), M.trim, bevel=0.006, bevel_segments=1)
        F.sweep('clip', [(MOP[0] + 0.022 * math.cos(math.radians(a)), 1.1, WALL + 0.26 + 0.022 * math.sin(math.radians(a)))
                         for a in range(-230, 51, 35)], None, M.trim, profile=G.rounded_rect(0.006, 0.03, 0.002, corner=1),
                up=(0, 0, 1))
        # the refill shelf on two brackets, three bottles on it
        F.box('shelf', 0.38, 0.02, 0.08, (0, 0.7, WALL + 0.24), M.birch_plain, bevel=0.005, bevel_segments=1)
        for x in (-0.16, 0.16):
            F.extrude(f'shelf_bracket{x}', [(WALL + 0.2, 0.69), (WALL + 0.27, 0.69), (WALL + 0.2, 0.62)], 0.012, (x, 0, 0),
                      M.birch_plain, plane='side')
        for x, m in ((-0.13, M.bottle_a), (-0.07, M.bottle_b), (0.13, M.bottle_a)):
            _bottle(F, f'refill{x}', x, WALL + 0.24, 0.71, m, h=0.14, d=0.05)
        # a bucket with its bail handle, spare pads stacked on the floor
        G.revolve('bucket', [(0, 0), (0.105, 0), (0.134, 0.25), (0.14, 0.255), (0.14, 0.262), (0.128, 0.262),
                             (0.122, 0.24), (0.095, 0.015), (0, 0.015)], at=F.at(-0.26, 0, 0.14), material=M.pastel_a,
                  segments=20)
        F.sweep('bucket_handle', [(-0.26 + 0.134 * math.cos(math.radians(a)), 0.25 + 0.06 * math.sin(math.radians(a)) +
                                   0.02 * math.sin(math.radians(a)) ** 8, 0.14 + 0.03 * math.sin(math.radians(a)))
                                  for a in range(0, 181, 20)], 0.004, M.trim, verts=5)
        F.soft('spare_pad', 0.22, 0.02, 0.14, (0.27, 0.014, 0.16), M.towel_w, round=0.008, puff=0.004, flat=2,
               round_segments=1)
        F.soft('spare_pad_b', 0.22, 0.02, 0.14, (0.27, 0.036, 0.16), M.towel_c, rot=(0, 0, 8), round=0.008, puff=0.004,
               flat=2, round_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The mop: pole, grip, water tank, head and pad, its origin at its head's foot: hidden while it's used
    # (`mop_off`), the game drawing it in the resident's hands (content `held`), pushed over the floor.
    x, y, z = MOP
    top = (x, 1.12, WALL + 0.26)
    with G.new_parts() as mop:
        F.soft('mop_head', 0.34, 0.04, 0.13, (x, y, z), M.trim, round=0.018, round_segments=1)
        F.soft('mop_pad', 0.33, 0.022, 0.125, (x, 0.016, z), M.pad, round=0.01, puff=0.004, flat=2, round_segments=1)
        G.revolve('mop_swivel', [(0, 0), (0.026, 0), (0.024, 0.02), (0.016, 0.03), (0.016, 0.06), (0, 0.06)],
                  at=F.at(x, y + 0.02, z), material=M.dark, segments=10)
        F.sweep('mop_pole', [(x, y + 0.07, z), top], 0.013, M.trim, verts=10)
        F.sweep('mop_grip', G.kit.fillet([(x, 1.1, WALL + 0.262), (x, 1.17, WALL + 0.255), (x - 0.05, 1.2, WALL + 0.25),
                                          (x, 1.23, WALL + 0.245), (x + 0.05, 1.2, WALL + 0.25), (x, 1.17, WALL + 0.255)], 0.02),
                None, M.teal, profile=G.rounded_rect(0.024, 0.03, 0.01, corner=1), up=(0, 0, 1))
        bx, by, bz = F.at(x, 0.2, z - 0.032)
        G.loft('mop_tank', [G.ring(0.05 * k, 0.04 * k, bz + h, 14, square=3, at=(bx, by)) for h, k in
                            ((0.0, 0.0), (0.012, 0.8), (0.04, 1.0), (0.24, 1.0), (0.27, 0.8), (0.28, 0.0))],
               material=M.teal, rot=(0, 0, 0))
    mop = G.hinged(G.join(mop, 'mop'), F.at(x, 0.0, z))
    objs = G.settle(centre=False)
    G.squash(mop, 'mop_off', 0.001, seconds=0.2)
    return objs


# --- the stacked laundry centre (2x1): washer and dryer at the left, a folding counter with cubbies, a rail ---
WX, WW, WH, WD = -0.56, 0.68, 1.72, 0.58     # the stack's middle, width, height and depth
CX = 0.45                                     # the counter's middle


def _porthole(F, name, cy, zf):
    """A porthole door: a chrome ring turned with a lip, a domed window, the handle recess on its right."""
    G.revolve(name, [(0.17, 0.0), (0.235, 0.0), (0.24, 0.012), (0.232, 0.026), (0.2, 0.03), (0.18, 0.024), (0.17, 0.012)],
              at=F.at(WX, cy, zf), material=M.chrome, segments=28, rot=(90, 0, 0))
    G.revolve(f'{name}_window', [(0, 0.022), (0.175, 0.008), (0.175, 0.014), (0.1, 0.026), (0, 0.03)],
              at=F.at(WX, cy, zf), material=M.porthole, segments=24, rot=(90, 0, 0))
    F.soft(f'{name}_handle', 0.04, 0.11, 0.04, (WX + 0.25, cy, zf + 0.025), M.chrome, round=0.014, round_segments=1)


def _appliance(F, name, y0, hh, zf, door):
    """One of the stack: a rounded cabinet, a control strip with its detergent drawer, display and dial, the drum
    behind the porthole, and the porthole door unless `door` (the washer's: its moving part)."""
    _box(F, f'{name}_cabinet', WW, hh - 0.006, WD - 0.02, (WX, y0 + hh / 2, WALL + (WD - 0.02) / 2), M.appliance, sq=10)
    F.soft(f'{name}_strip', WW - 0.03, 0.1, 0.012, (WX, y0 + hh - 0.075, zf - 0.004), M.trim, round=0.005, round_segments=1)
    F.soft(f'{name}_drawer', 0.2, 0.07, 0.012, (WX - 0.2, y0 + hh - 0.075, zf + 0.004), M.appliance, round=0.005,
           round_segments=1)
    F.box(f'{name}_drawer_grip', 0.08, 0.012, 0.006, (WX - 0.2, y0 + hh - 0.1, zf + 0.01), M.dark)
    F.soft(f'{name}_display', 0.16, 0.05, 0.006, (WX + 0.1, y0 + hh - 0.075, zf + 0.004), M.screen, round=0.003,
           round_segments=1)
    F.box(f'{name}_display_glow', 0.07, 0.02, 0.002, (WX + 0.08, y0 + hh - 0.075, zf + 0.008), M.screen_glow)
    G.revolve(f'{name}_dial', [(0, 0), (0.034, 0), (0.034, 0.012), (0.03, 0.02), (0, 0.022)],
              at=F.at(WX + 0.25, y0 + hh - 0.075, zf), material=M.chrome, segments=16, rot=(90, 0, 0))
    cy = y0 + (hh - 0.13) / 2 + 0.03
    G.revolve(f'{name}_drum', [(0, -0.06), (0.17, -0.06), (0.19, -0.02), (0.2, 0.0), (0.21, 0.004), (0.21, 0.0)],
              at=F.at(WX, cy, zf - 0.004), material=M.drum, segments=24, rot=(90, 0, 0))
    if not door:
        _porthole(F, f'{name}_porthole', cy, zf)
    return cy


def _basket(F, name, x, y, z, w, h, d, material):
    """A woven basket: a rounded body in ridges, a dark rim and two hand holes' grips."""
    bx, by, bz = F.at(x, y, z)
    rings = []
    for k in range(7):
        t = k / 6
        e = 0.006 if k % 2 else 0.0
        rings.append(G.ring(w / 2 * (0.94 + 0.06 * t) + e, d / 2 * (0.94 + 0.06 * t) + e, bz + h * t, 20, square=5, at=(bx, by)))
    G.loft(name, rings, material=material)
    G.loft(f'{name}_rim', [G.ring(w / 2 + 0.008, d / 2 + 0.008, bz + h - 0.004, 20, square=5, at=(bx, by)),
                           G.ring(w / 2 + 0.008, d / 2 + 0.008, bz + h + 0.012, 20, square=5, at=(bx, by))], material=M.dark)


def _shirt(F, name, x, y, z, material):
    """A shirt on a wooden hanger, seen edge on from the room's side: a hanger hooked over the rail, the shirt's
    outline (shoulders, sleeves, hem) a thin slab across the depth."""
    F.sweep(f'{name}_hook', [(x, y - 0.03, z), (x, y + 0.012, z), (x, y + 0.018, z + 0.012), (x, y + 0.008, z + 0.02)],
            0.0025, M.chrome, verts=4)
    F.sweep(f'{name}_hanger', [(x, y - 0.075, z - 0.125), (x, y - 0.03, z), (x, y - 0.075, z + 0.125)], None, M.oak,
            profile=G.rounded_rect(0.012, 0.02, 0.004, corner=1), up=(1, 0, 0), resolution=2)
    outline = G.smooth_outline([(-0.04, -0.03), (-0.11, -0.05), (-0.14, -0.18), (-0.11, -0.19), (-0.09, -0.12),
                                (-0.095, -0.44), (0.095, -0.44), (0.09, -0.12), (0.11, -0.19), (0.14, -0.18), (0.11, -0.05),
                                (0.04, -0.03), (0.0, -0.06)], passes=1)
    F.extrude(name, [(z + u, y - 0.07 + v) for u, v in outline], 0.022, (x, 0, 0), material, plane='side')


def laundryCenter():
    F = G.GameFrame()
    zf = WALL + WD                                    # the stack's front
    hh = WH / 2
    with G.new_parts() as still:
        cy = _appliance(F, 'washer', 0.0, hh, zf, True)   # the washer, its door the moving part
        F.box('kick', WW - 0.03, 0.05, 0.012, (WX, 0.03, zf + 0.002), M.trim, bevel=0.004, bevel_segments=1)
        _appliance(F, 'dryer', hh, hh, zf, False)         # the dryer
        # a slim filler between the stack and the counter, a towel over its bar
        F.box('filler', 0.2, 0.9, 0.56, (-0.12, 0.45, WALL + 0.28), M.body, bevel=0.006, bevel_segments=1)
        F.sweep('filler_bar', [(-0.19, 0.7, WALL + 0.56), (-0.19, 0.7, WALL + 0.585), (-0.05, 0.7, WALL + 0.585),
                               (-0.05, 0.7, WALL + 0.56)], 0.007, M.chrome, verts=6)
        F.sweep('filler_towel', [(-0.12, 0.45, WALL + 0.575), (-0.12, 0.68, WALL + 0.576), (-0.12, 0.712, WALL + 0.585),
                                 (-0.12, 0.7, WALL + 0.598), (-0.12, 0.5, WALL + 0.6)], None, M.towel_a,
                profile=G.rounded_rect(0.008, 0.13, 0.004, corner=1), up=(0, 1, 0), resolution=2)
        # the counter: open cubbies under a top, towels above and baskets below
        F.box('counter_top', 0.94, 0.04, 0.58, (CX, 0.92, WALL + 0.29), M.birch_plain, bevel=0.008, bevel_segments=2)
        for y in (0.02, 0.42):
            F.box(f'cubby_shelf{y}', 0.9, 0.04, 0.56, (CX, y, WALL + 0.28), M.body, bevel=0.005, bevel_segments=1)
        for x in (CX - 0.44, CX, CX + 0.44):
            F.box(f'cubby_side{x}', 0.03, 0.88, 0.56, (x, 0.45, WALL + 0.28), M.body, bevel=0.005, bevel_segments=1)
        F.box('cubby_back', 0.9, 0.88, 0.02, (CX, 0.45, WALL + 0.01), M.tile)
        _towels(F, 'cubby_towel_l', CX - 0.22, -0.14, 0.44, 4)
        _towels(F, 'cubby_towel_r', CX + 0.22, -0.14, 0.44, 3, names=('towel_w', 'towel_c', 'towel_w'))
        for x, m in ((CX - 0.22, M.wicker), (CX + 0.22, M.wicker_b)):
            _basket(F, f'basket{x}', x, 0.04, -0.16, 0.34, 0.29, 0.42, m)
        # on the counter: a folded stack, detergents, a mini ironing board and its iron
        _towels(F, 'counter_towel', CX - 0.18, -0.2, 0.94, 3, w=0.3, d=0.22, names=('towel_w', 'towel_a', 'towel_w'))
        _bottle(F, 'detergent', CX + 0.32, -0.3, 0.94, M.bottle_a, h=0.24, cap=M.teal)
        F.sweep('detergent_handle', [(CX + 0.32, 1.1, -0.26), (CX + 0.32, 1.12, -0.24), (CX + 0.32, 1.05, -0.24),
                                     (CX + 0.32, 1.03, -0.26)], 0.008, M.bottle_a, verts=5, resolution=2)
        _bottle(F, 'softener', CX + 0.2, -0.34, 0.94, M.bottle_b, h=0.2, cap=M.coral)
        board = G.smooth_outline([(-0.21, -0.1), (0.12, -0.1), (0.21, -0.04), (0.21, 0.04), (0.12, 0.1), (-0.21, 0.1)],
                                 passes=2)
        F.extrude('ironing_board', [(CX + 0.12 + u, 0.03 + v) for u, v in board], 0.02, (0, 0.95, 0), M.towel_c,
                  plane='top', bevel=0.006, bevel_segments=1)
        bx, by, bz = F.at(CX + 0.12, 0.96, 0.03)
        G.loft('iron', [[(bx + px * k, by + py * k, bz + h) for px, py in
                         ((0.07, 0.0), (0.02, 0.045), (-0.06, 0.045), (-0.065, 0.0), (-0.06, -0.045), (0.02, -0.045))]
                        for h, k in ((0.0, 1.0), (0.025, 0.98), (0.04, 0.85))], material=M.appliance)
        F.sweep('iron_handle', [(CX + 0.18, 1.0, 0.03), (CX + 0.14, 1.04, 0.03), (CX + 0.06, 1.04, 0.03), (CX + 0.05, 1.0, 0.03)],
                None, M.teal, profile=G.rounded_rect(0.014, 0.02, 0.006, corner=1), up=(0, 0, 1), resolution=2)
        # the rail over the counter on two posts, shirts on hangers
        G.kit.tube(F, 'rail', G.kit.fillet([(CX - 0.42, 0.94, -0.29), (CX - 0.42, 1.72, -0.29), (CX + 0.42, 1.72, -0.29),
                                            (CX + 0.42, 0.94, -0.29)], 0.04, steps=3), 0.01, M.chrome, verts=8, round=1)
        for x in (CX - 0.42, CX + 0.42):
            F.cyl(f'rail_foot{x}', 0.03, 0.01, (x, 0.945, -0.29), M.chrome, verts=12)
        for i, m in enumerate(('towel_c', 'towel_w', 'towel_b', 'towel_f')):
            _shirt(F, f'shirt{i}', CX - 0.27 + i * 0.18, 1.72, -0.29, _m(m))
    G.join(still, 'body', origin=(0, 0, 0))
    # The washer's porthole door, hinged on its left: `door_open` swings it open.
    with G.new_parts() as door:
        _porthole(F, 'door', cy, zf)
    door = G.hinged(G.join(door, 'door'), F.at(WX - 0.235, cy, zf + 0.005))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=0.8)
    return objs


# --- the rainfall steam shower (1x1): a glass cabin, tiled back and side, a ceiling rain head, a teak bench ---
S, T, HC = 0.45, 0.07, 2.18      # half the cabin's width, the tray's top, the cabin's height
HEAD_Y = 2.075                   # the rain head's face
DOOR_HALF = 0.28                 # half the door's width, across the middle of the front


def rainShowerTower():
    F = G.GameFrame()
    w = 2 * S
    with G.new_parts() as still:
        _box(F, 'tray', w, T - 0.012, w, (0, (T - 0.012) / 2, 0), M.body, sq=12)
        F.soft('tray_floor', w - 0.06, 0.012, w - 0.06, (0, T - 0.018, 0), M.body, round=0.006, round_segments=1)
        F.box('drain', 0.5, 0.004, 0.05, (0.0, T - 0.01, -0.3), M.chrome, bevel=0.002, bevel_segments=1)
        for k in range(9):                                          # the linear drain's slots
            F.box(f'drain_slot{k}', 0.035, 0.002, 0.02, (-0.2 + k * 0.05, T - 0.008, -0.3), M.dark)
        F.box('back_wall', w, HC - T, 0.03, (0, T + (HC - T) / 2, -S + 0.015), M.tile)
        F.box('side_wall', 0.03, HC - T, w - 0.03, (-S + 0.015, T + (HC - T) / 2, 0.015), M.tile)
        for k in range(1, 10):                                      # tile joints: courses, and a joint between each tile
            y = T + k * (HC - T) / 10
            F.box(f'joint_b{k}', w - 0.04, 0.006, 0.004, (0, y, -S + 0.031), M.joint)
            F.box(f'joint_s{k}', 0.004, 0.006, w - 0.07, (-S + 0.031, y, 0.015), M.joint)
        for k in (1, 2, 3):
            F.box(f'joint_bv{k}', 0.006, HC - T, 0.004, (-S + k * w / 4, T + (HC - T) / 2, -S + 0.031), M.joint)
            F.box(f'joint_sv{k}', 0.004, HC - T, 0.006, (-S + 0.031, T + (HC - T) / 2, -S + k * w / 4 + 0.03), M.joint)
        # a niche in the side wall with bottles in it
        F.box('niche_back', 0.006, 0.3, 0.36, (-S + 0.032, 1.25, 0.1), M.joint)
        for y in (1.1, 1.4):
            F.box(f'niche_ledge{y}', 0.06, 0.02, 0.38, (-S + 0.06, y, 0.1), M.tile, bevel=0.004, bevel_segments=1)
        for k, (z, m) in enumerate(((0.0, M.bottle_c), (0.08, M.bottle_b), (0.2, M.bottle_a))):
            _bottle(F, f'niche_bottle{k}', -S + 0.07, z, 1.11, m, h=0.16 - 0.02 * k, d=0.055)
        # the glass: the right side and the front, a narrow fixed pane either side of the door (its own part)
        F.box('glass_side', 0.012, HC - T - 0.06, w - 0.06, (S - 0.012, T + (HC - T - 0.06) / 2, 0.0), M.glass)
        pw = S - 0.03 - DOOR_HALF - 0.004
        for x in (-S + 0.03 + pw / 2, S - 0.03 - pw / 2):
            F.box(f'glass_front{x}', pw, HC - T - 0.08, 0.012, (x, T + (HC - T - 0.08) / 2 + 0.01, S - 0.012), M.glass)
        prof = G.rounded_rect(0.03, 0.03, 0.006, corner=1)
        F.sweep('frame_bar', [(S - 0.015, T, -S + 0.035), (S - 0.015, HC - 0.015, -S + 0.035), (S - 0.015, HC - 0.015, S - 0.015),
                              (-S + 0.03, HC - 0.015, S - 0.015)], None, M.chrome, profile=prof, up=(1, 0, 0))
        F.box('frame_post', 0.03, HC - T, 0.03, (S - 0.015, T + (HC - T) / 2, S - 0.015), M.chrome, bevel=0.004,
              bevel_segments=1)
        F.box('frame_sill', w - 0.03, 0.02, 0.03, (0.015, T + 0.01, S - 0.015), M.chrome, bevel=0.004, bevel_segments=1)
        for y in (T + 0.2, HC - 0.25):                                  # the door's hinges on the right pane
            F.soft(f'hinge{y}', 0.05, 0.05, 0.03, (DOOR_HALF + 0.01, y, S - 0.012), M.chrome, round=0.008, round_segments=1)
        # the rain head on its arm from the back wall
        F.soft('rain_head', 0.44, 0.02, 0.44, (0, HEAD_Y + 0.012, 0.0), M.chrome, round=0.008, round_segments=1)
        F.box('rain_face', 0.4, 0.004, 0.4, (0, HEAD_Y + 0.001, 0.0), M.steel)
        G.kit.tube(F, 'rain_arm', G.kit.fillet([(0, HEAD_Y + 0.02, 0.0), (0, HEAD_Y + 0.085, 0.0), (0, HEAD_Y + 0.085, -S + 0.03)],
                                               0.04), 0.014, M.chrome, verts=8, round=1)
        G.revolve('rain_rose', [(0, 0), (0.035, 0), (0.035, 0.008), (0, 0.012)], at=F.at(0, HEAD_Y + 0.085, -S + 0.03),
                  material=M.chrome, segments=12, rot=(90, 0, 0))
        # the steam column with its jets, the control panel
        F.soft('steam_column', 0.16, 1.2, 0.04, (-0.24, 1.15, -S + 0.05), M.appliance, round=0.012, round_segments=1)
        for k in range(5):
            G.revolve(f'jet{k}', [(0, 0), (0.02, 0), (0.016, 0.01), (0.008, 0.012), (0, 0.008)],
                      at=F.at(-0.24, 0.75 + k * 0.18, -S + 0.07), material=M.chrome, segments=10, rot=(90, 0, 0))
        F.soft('controls', 0.2, 0.12, 0.012, (0.22, 1.25, -S + 0.036), M.screen, round=0.006, round_segments=1)
        F.box('controls_line', 0.12, 0.04, 0.004, (0.22, 1.27, -S + 0.043), M.teal)
        for k in range(3):
            F.cyl(f'control_button{k}', 0.01, 0.006, (0.16 + k * 0.06, 1.215, -S + 0.043), M.chrome, rot=(90, 0, 0), verts=8)
        # the teak bench along the back: slats on two shaped legs, towels and bottles on it
        for k in range(4):
            F.box(f'bench_slat{k}', 0.72, 0.03, 0.06, (0.0, 0.455, -0.345 + k * 0.07), M.teak, bevel=0.006, bevel_segments=1)
        for x in (-0.3, 0.3):
            F.extrude(f'bench_leg{x}', G.smooth_outline([(-0.36, 0.0), (-0.12, 0.0), (-0.16, 0.08), (-0.32, 0.08), (-0.32, 0.37),
                                                         (-0.12, 0.37), (-0.12, 0.44), (-0.36, 0.44)], passes=0),
                      0.035, (x, 0, 0), M.teak, plane='side', bevel=0.004, bevel_segments=1)
        _towels(F, 'bench_towel', 0.2, -0.26, 0.47, 2, w=0.24, d=0.2, names=('pad', 'pastel_a'))   # satin: no fabric draw
        _bottle(F, 'bench_bottle_a', -0.22, -0.3, 0.47, M.bottle_c, h=0.16, d=0.06)
        _bottle(F, 'bench_bottle_b', -0.12, -0.3, 0.47, M.bottle_b, h=0.18, d=0.06)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: streaks from the head's face to the tray, origin at the head: `water_on` grows them down.
    with G.new_parts() as rain:
        rnd = random.Random(3)
        for i in range(5):
            for j in range(5):
                x, z = -0.16 + i * 0.08 + rnd.uniform(-0.01, 0.01), -0.16 + j * 0.08 + rnd.uniform(-0.01, 0.01)
                G.rod(f'stream{i}{j}', F.at(x, HEAD_Y, z), F.at(x, T + rnd.uniform(0.0, 0.03), z), 0.004,
                      material=M.water, verts=4)
    rain = G.join(rain, 'water', origin=F.at(0, HEAD_Y, 0))
    rain.scale[2] = 0.001
    # The glass door across the middle of the front, hinged on its right: `door_open` swings it out to let the
    # bather in and out (a fourth draw: Q-05). All glass, no pull, to keep it one draw.
    with G.new_parts() as door:
        F.box('glass_door', 2 * DOOR_HALF, HC - T - 0.1, 0.012, (0, T + (HC - T - 0.1) / 2 + 0.02, S - 0.012), M.glass)
    door = G.hinged(G.join(door, 'door'), F.at(DOOR_HALF, T, S - 0.012))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.grow(rain, 'water_on', seconds=0.6)
    G.swing(door, 'door_open', -95, seconds=0.8)
    return objs


# --- the label-and-sort organizer desk (2x1): its back on the wall, the hutch of labelled bins on the desk's back,
# open towards the sitter and the room; the chair in front, the sitter facing the wall (2026-10-21: it stood the
# other way round, its backs to the room) ------------------------------------------------------------------------
HD = 0.16                    # the hutch's own depth
HZ = WALL + 0.012 + HD / 2   # the hutch's middle depth, its back panel on the wall face
DZ, DD = WALL + 0.24, 0.48   # the desk top's middle depth and its depth: from the wall face to 0.05
CZ = 0.25                    # the chair seat's middle depth (the hips 4 cm nearer the desk: `furniture.json`)
BIN = (0.0, 0.87)            # the bin `drawer_open` pulls out (x, middle height): the lower row's middle


def _bin(F, name, x, y, material):
    """A storage bin: lofted, a little wider at its mouth, a label holder and a grip cut in its front."""
    bx, by, bz = F.at(x, y - 0.095, HZ - 0.005)
    G.loft(name, [G.ring(0.13 * k, (HD - 0.02) / 2 * k, bz + h, 16, square=6, at=(bx, by)) for h, k in
                  ((0.0, 0.94), (0.005, 0.97), (0.18, 1.0), (0.19, 1.02))], material=material)
    face = HZ - 0.005 + (HD - 0.02) / 2
    F.soft(f'{name}_label', 0.1, 0.04, 0.006, (x, y + 0.035, face + 0.002), M.chrome, round=0.003, round_segments=1)
    F.box(f'{name}_card', 0.085, 0.028, 0.002, (x, y + 0.035, face + 0.005), M.paper)
    F.soft(f'{name}_grip', 0.08, 0.022, 0.006, (x, y - 0.04, face + 0.001), M.dark, round=0.008, round_segments=1)


def _chair(F, x, z, mat, legs):
    """A neat little chair, its back to the room: a padded seat (top 0.45) and back on splayed birch legs."""
    F.soft('chair_seat', 0.46, 0.06, 0.44, (x, 0.415, z), mat, round=0.025, puff=0.006, flat=2)
    F.soft('chair_back', 0.44, 0.3, 0.05, (x, 0.74, z + 0.185), mat, rot=(-6, 0, 0), round=0.022, puff=0.006, flat=2)
    for dx in (-0.19, 0.19):
        for dz in (-0.17, 0.17):
            F.sweep(f'chair_leg{dx}{dz}', [(x + dx, 0.385, z + dz), (x + dx * 1.12, 0.0, z + dz * 1.15)], 0.016, legs,
                    verts=8, scale=[1.0, 0.7])
        F.sweep(f'chair_post{dx}', [(x + dx, 0.42, z + 0.18), (x + dx, 0.62, z + 0.2)], 0.012, legs, verts=8)


def organizerDesk():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('desk_top', 1.6, 0.035, DD, (0, 0.745, DZ), M.birch, bevel=0.01, bevel_segments=2)
        for x in (-0.78, 0.78):
            F.box(f'desk_side{x}', 0.04, 0.73, DD - 0.02, (x, 0.365, DZ - 0.01), M.body, bevel=0.006, bevel_segments=1)
        F.box('modesty', 1.52, 0.6, 0.015, (0, 0.42, WALL + 0.02), M.birch, bevel=0.004, bevel_segments=1)
        # the hutch: two rows of five cubbies, open towards the sitter
        F.box('hutch_shelf', 1.6, 0.02, HD, (0, 0.99, HZ), M.body, bevel=0.004, bevel_segments=1)
        F.box('hutch_top', 1.6, 0.02, HD, (0, 1.22, HZ), M.body, bevel=0.004, bevel_segments=1)
        F.box('hutch_back', 1.6, 0.5, 0.012, (0, 1.0, HZ - HD / 2 + 0.004), M.birch)
        for i in range(6):
            F.box(f'divider{i}', 0.02, 0.48, HD, (-0.79 + i * 0.316, 1.0, HZ), M.body, bevel=0.003, bevel_segments=1)
        for r, y in enumerate((0.87, 1.105)):
            for i in range(5):
                x = -0.632 + i * 0.316
                if (x, y) != BIN:
                    _bin(F, f'bin{r}{i}', x, y, _m(PASTELS[(i + r * 2) % len(PASTELS)]))
        # on the desk in front of the hutch: the ledger, the label maker, a calculator, a pen cup
        z = -0.1
        F.soft('ledger', 0.36, 0.02, 0.24, (0.32, 0.773, z), M.ledger, round=0.006, round_segments=1)
        F.box('ledger_pages', 0.33, 0.012, 0.21, (0.32, 0.789, z), M.paper, bevel=0.002, bevel_segments=1)
        F.sweep('ledger_ribbon', [(0.3, 0.796, z - 0.1), (0.31, 0.796, z + 0.05), (0.32, 0.79, z + 0.13)], None, M.coral,
                profile=_flat(0.012, 0.002), up=(0, 0, 1), resolution=2)
        bx, by, bz = F.at(-0.4, 0.763, z)
        G.loft('label_maker', [G.ring(0.05 * k, 0.08 * k, bz + h, 14, square=3, at=(bx, by)) for h, k in
                               ((0.0, 0.92), (0.006, 1.0), (0.04, 1.0), (0.052, 0.86))], material=M.coral)
        F.soft('label_screen', 0.06, 0.004, 0.05, (-0.4, 0.816, z - 0.035), M.screen, round=0.002, round_segments=1)
        for i in range(3):
            for j in range(2):
                F.box(f'label_key{i}{j}', 0.014, 0.004, 0.012, (-0.42 + i * 0.02, 0.816, z + 0.02 + j * 0.02), M.paper)
        F.box('label_tape', 0.03, 0.004, 0.06, (-0.4, 0.766, z - 0.11), M.paper)
        F.soft('calculator', 0.12, 0.016, 0.16, (-0.62, 0.77, z), M.trim, round=0.006, round_segments=1)
        F.box('calc_screen', 0.09, 0.003, 0.035, (-0.62, 0.779, z - 0.05), M.screen)
        for i in range(4):
            for j in range(4):
                F.box(f'calc_key{i}{j}', 0.018, 0.004, 0.016, (-0.65 + i * 0.022, 0.779, z - 0.01 + j * 0.022), M.paper)
        G.revolve('pen_cup', [(0, 0), (0.04, 0), (0.042, 0.11), (0.036, 0.11), (0.034, 0.01), (0, 0.01)],
                  at=F.at(0.62, 0.763, z - 0.06), material=M.pastel_f, segments=14)
        for k, m in enumerate(('pastel_d', 'ledger', 'coral')):
            F.sweep(f'pen{k}', [(0.61 + 0.012 * k, 0.775, z - 0.065 + 0.008 * k), (0.598 + 0.02 * k, 0.94 - 0.01 * k, z - 0.08 + 0.016 * k)],
                    0.006, _m(m), verts=6, scale=[1.0, 0.9])
        _chair(F, 0, CZ, M.pastel_c, M.birch)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bin in the lower row's middle: `drawer_open` slides it out towards the sitter.
    with G.new_parts() as drawer:
        _bin(F, 'pull_bin', BIN[0], BIN[1], M.pastel_b)
        for k in range(3):                                         # labelled cards inside
            F.box(f'card{k}', 0.2, 0.1, 0.004, (BIN[0] + 0.01 * k, BIN[1] + 0.05 + 0.01 * (k % 2), HZ + 0.025 - 0.03 * k),
                  (M.paper, M.pastel_d, M.paper)[k])
    drawer = G.join(drawer, 'bin')
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.13, 0), seconds=0.6)
    return objs


def _flat(w, t):
    """A flat strip's profile, `w` wide and `t` thick (across, through)."""
    return [(-t / 2, -w / 2), (t / 2, -w / 2), (t / 2, w / 2), (-t / 2, w / 2)]


# --- the robot vacuum and dock (1x1): a self-emptying dock against the wall, the robot docked on a round rug ---
ROBOT = (0.0, -0.02)    # the docked robot's middle (x, depth)


def robotVacuumDock():
    F = G.GameFrame()
    with G.new_parts() as still:
        G.revolve('rug', [(0, 0), (0.43, 0), (0.432, 0.006), (0.425, 0.012), (0.36, 0.012), (0.352, 0.008), (0.344, 0.014),
                          (0, 0.014)], at=F.at(0, 0, 0), material=M.rug, segments=36)
        G.revolve('rug_inner', [(0, 0.008), (0.34, 0.008), (0.34, 0.014), (0.33, 0.016), (0, 0.016)], at=F.at(0, 0, 0),
                  material=M.rug_inner, segments=32)
        # the dock: a rounded tower, its bin window and the bin behind it, a ramp out over the rug
        bx, by, bz = F.at(0, 0, WALL + 0.12)
        G.loft('dock', [G.ring(0.19 * k, 0.12 * k, bz + h, 24, square=4, at=(bx, by)) for h, k in
                        ((0.0, 0.98), (0.01, 1.0), (0.4, 1.0), (0.44, 0.9), (0.46, 0.7), (0.462, 0.0))], material=M.appliance)
        F.soft('bin_window', 0.28, 0.2, 0.014, (0, 0.3, WALL + 0.238), M.porthole, round=0.02, round_segments=2)
        F.soft('bin', 0.2, 0.12, 0.006, (0, 0.3, WALL + 0.244), M.steel, round=0.012, round_segments=1)
        F.box('bin_latch', 0.06, 0.015, 0.008, (0, 0.415, WALL + 0.24), M.trim, bevel=0.003, bevel_segments=1)
        F.extrude('ramp', [(-0.17, WALL + 0.24), (0.17, WALL + 0.24), (0.17, WALL + 0.34), (-0.17, WALL + 0.34)], 0.02,
                  (0, 0.024, 0), M.trim, plane='top', bevel=0.008, bevel_segments=2)
        F.sphere('led', 0.01, (0.13, 0.42, WALL + 0.24), M.led, segments=8, rings=4)
        # a little plant beside the dock
        G.revolve('plant_pot', [(0, 0), (0.07, 0), (0.09, 0.19), (0.094, 0.2), (0.08, 0.2), (0.07, 0.18), (0, 0.18)],
                  at=F.at(0.31, 0, WALL + 0.12), material=M.body, segments=16)
        for k in range(9):                                         # a snake plant: upright, banded strap leaves
            leaf(f'plant_leaf{k}', F.at(0.31 + 0.02 * G.cosd(k * 40), 0.17, WALL + 0.12 + 0.02 * G.sind(k * 40)), k * 40 + 15,
                 82 + 2 * (k % 3), 0.36 - 0.04 * (k % 3), 0.05, (M.green, M.stem)[k % 2], shape='lance', droop=0.04,
                 fold=0.35, rows=4, ring=3, twist=40)
    G.join(still, 'body', origin=(0, 0, 0))
    # The robot: a disc with a teal bumper, its lid's seam, the lidar turret, a front sensor and a side brush:
    # `vacuum_round` (a loop) drives it out of its dock, round a circle in front of it, turning as it goes, and back in.
    x, z = ROBOT
    y = 0.014
    with G.new_parts() as robot:
        G.revolve('robot_body', [(0, 0), (0.155, 0), (0.168, 0.012), (0.17, 0.07), (0.162, 0.082), (0.12, 0.085),
                                 (0.118, 0.083), (0.11, 0.085), (0, 0.085)], at=F.at(x, y, z), material=M.appliance,
                  segments=28)
        G.revolve('robot_bumper', [(0.168, 0.018), (0.176, 0.024), (0.178, 0.05), (0.172, 0.058), (0.168, 0.056)],
                  at=F.at(x, y, z), material=M.teal, segments=28)
        G.revolve('lidar', [(0, 0), (0.046, 0), (0.046, 0.03), (0.04, 0.036), (0, 0.036)], at=F.at(x, y + 0.083, z - 0.06),
                  material=M.dark, segments=16)
        F.soft('robot_front', 0.22, 0.03, 0.02, (x, y + 0.04, z + 0.165), M.dark, round=0.01, round_segments=1)
        star_pts = [(0.045 * math.cos(math.tau * k / 6), 0.045 * math.sin(math.tau * k / 6)) for k in range(6)]
        for k, (u, v) in enumerate(star_pts):
            F.sweep(f'brush{k}', [(x + 0.12, y + 0.004, z + 0.12), (x + 0.12 + u, y + 0.002, z + 0.12 + v)], 0.002, M.steel,
                    verts=3)
    robot = G.hinged(G.join(robot, 'robot'), F.at(x, y, z))
    objs = G.settle(centre=False)
    G.path(robot, 'vacuum_round', _round(), linear=True)
    return objs


# The robot's round (game metres from where it docks): OUT forward, a circle of RADIUS to its left (seen from
# the front, game -x: clear of the plant), its middle that far ahead, and back in, at SPEED m/s; it stays a hair
# inside its tile and off the dock and the plant.
OUT, RADIUS, SPEED = 0.18, 0.14, 0.25


def _round():
    """[(seconds, Blender offset, Blender degrees)] for `G.path`: the robot faces the way it goes (its front, game
    +z, turned about Blender Z by the angle round the circle)."""
    keys, t = [(0.0, (0, 0, 0), (0, 0, 0))], OUT / SPEED
    keys.append((t, (0, -OUT, 0), (0, 0, 0)))
    steps = 24
    for k in range(1, steps + 1):
        a = 2 * math.pi * k / steps
        x, z = -RADIUS + RADIUS * math.cos(a), OUT + RADIUS * math.sin(a)   # game (x, depth)
        keys.append((t + 2 * math.pi * RADIUS * k / steps / SPEED, (-x, -z, 0), (0, 0, math.degrees(a))))
    t = keys[-1][0]
    keys.append((t + OUT / SPEED, (0, 0, 0), (0, 0, 360)))
    return keys


# --- the tidy entry console (1x1): a slim birch console, a shoe shelf, a key tray drawer, a leaning mirror -----
DRAWER = (-0.2, 0.74)    # the key tray drawer's middle (x, height)
FZ = WALL + 0.3          # the console's front
PANEL_TOP = 1.78         # the back panel's top, under its cap


def _shoe(F, name, x, y, z, material):
    """A shoe: a lofted upper rising to the heel, its opening dark, on a white sole."""
    bx, by, bz = F.at(x, y + 0.012, z)
    G.loft(name, [[(bx + px * k, by + py, bz + h) for px, py in
                   ((0.0, -0.11), (0.03, -0.09), (0.038, 0.0), (0.034, 0.09), (0.0, 0.105), (-0.034, 0.09), (-0.038, 0.0),
                    (-0.03, -0.09))]
                  for h, k in ((0.0, 1.0), (0.035, 1.0), (0.05, 0.85))], material=material)
    F.soft(f'{name}_opening', 0.05, 0.004, 0.09, (x, y + 0.064, z - 0.045), M.dark, round=0.002, round_segments=1)
    F.soft(f'{name}_sole', 0.08, 0.014, 0.225, (x, y + 0.007, z), M.body, round=0.006, round_segments=1)


def entryConsole():
    F = G.GameFrame()
    dx, dy = DRAWER
    with G.new_parts() as still:
        F.box('top', 0.86, 0.035, 0.3, (0, 0.8, WALL + 0.15), M.birch, bevel=0.01, bevel_segments=2)
        for k in range(3):                                          # the shoe shelf: three slats
            F.box(f'shoe_slat{k}', 0.82, 0.022, 0.075, (0, 0.22, WALL + 0.05 + k * 0.1), M.birch, bevel=0.005,
                  bevel_segments=1)
        for x in (-0.41, 0.41):
            for z in (WALL + 0.02, FZ - 0.02):
                F.sweep(f'leg{x}{z}', [(x, 0.0, z), (x, 0.79, z)], None, M.body, profile=G.rounded_rect(0.03, 0.03, 0.008, corner=1),
                        up=(0, 1, 0), scale=[0.75, 1.0])
            F.box(f'shelf_rail{x}', 0.03, 0.03, 0.26, (x, 0.22, WALL + 0.15), M.body, bevel=0.004, bevel_segments=1)
        # the apron round the drawer's opening, and its housing
        F.box('apron_r', 0.82 - (dx + 0.17 + 0.41), 0.08, 0.02, ((dx + 0.17 + 0.41) / 2, 0.74, FZ - 0.02), M.body,
              bevel=0.003, bevel_segments=1)
        F.box('apron_l', 0.07, 0.08, 0.02, (-0.375, 0.74, FZ - 0.02), M.body, bevel=0.003, bevel_segments=1)
        F.box('housing', 0.36, 0.01, 0.26, (dx, 0.695, WALL + 0.15), M.body)
        F.box('apron_back', 0.82, 0.08, 0.02, (0, 0.74, WALL + 0.02), M.body)
        # shoes, neatly paired
        for i, m in enumerate(('shoe_a', 'shoe_b', 'shoe_c')):
            x = -0.27 + i * 0.27
            for ddx in (-0.045, 0.045):
                _shoe(F, f'shoe{i}{ddx}', x + ddx, 0.231, -0.27, _m(m))
        # on top: a bowl, a vase of tulips
        G.revolve('bowl', [(0, 0), (0.05, 0), (0.07, 0.03), (0.08, 0.058), (0.074, 0.06), (0.064, 0.032), (0.044, 0.012),
                           (0, 0.012)], at=F.at(0.02, 0.8175, -0.27), material=M.appliance, segments=18)
        G.revolve('vase', [(0, 0), (0.05, 0), (0.06, 0.09), (0.03, 0.19), (0.034, 0.22), (0.03, 0.22), (0, 0.2)],
                  at=F.at(0.28, 0.8175, -0.3), material=M.bottle_b, segments=14)
        for k in range(5):
            a = k * 72
            x, z = 0.28 + 0.035 * G.cosd(a), -0.3 + 0.035 * G.sind(a)
            F.sweep(f'tulip_stem{k}', [(0.28, 0.98, -0.3), (0.28 + 0.02 * G.cosd(a), 1.1, -0.3 + 0.02 * G.sind(a)), (x, 1.2, z)],
                    0.004, M.stem, verts=4, resolution=2)
            cupped(f'tulip{k}', [(0, 0), (0.016, 0.004), (0.024, 0.03), (0.02, 0.06), (0.012, 0.064), (0.014, 0.05), (0, 0.04)],
                   F.at(x, 1.198, z), _m(('coral', 'pastel_b', 'pad', 'coral', 'pastel_e')[k]), yaw=-a + 90, tilt=10,
                   segments=6)
            leaf(f'tulip_leaf{k}', F.at(0.28, 1.0, -0.3), k * 72 + 36, 70, 0.12, 0.03, M.stem, shape='strap', droop=0.3,
                 rows=3, ring=3)
        # the back panel rising from the top (a hall stand's), standing free or on the wall: the mirror on it, the
        # coat hooks with a tote and a scarf (2026-10-21: they hung on the air over a free-standing console)
        pb, pt = 0.8175, PANEL_TOP
        F.box('back_panel', 0.82, pt - pb, 0.018, (0, (pb + pt) / 2, WALL + 0.009), M.birch, bevel=0.004, bevel_segments=1)
        for x in (-0.41, 0.41):
            F.box(f'panel_post{x}', 0.03, pt - 0.79, 0.03, (x, (0.79 + pt) / 2, WALL + 0.02), M.body, bevel=0.006,
                  bevel_segments=1)
        F.box('panel_cap', 0.87, 0.025, 0.045, (0, pt + 0.0125, WALL + 0.0225), M.body, bevel=0.008, bevel_segments=2)
        pz = WALL + 0.018
        frame = [(-0.2, 0.9), (0.3, 0.9), (0.3, 1.76), (-0.2, 1.76)]
        G.kit.moulding(F, 'mirror_frame', frame, [(0, 0), (0.035, 0), (0.035, 0.012), (0.024, 0.026), (0.008, 0.026),
                                                  (0, 0.016)], M.body, at=pz)
        F.box('mirror', 0.44, 0.8, 0.006, (0.05, 1.33, pz + 0.004), M.mirror)
        F.box('hook_board', 0.05, 0.4, 0.02, (-0.33, 1.42, pz + 0.01), M.oak, bevel=0.006, bevel_segments=1)
        for y in (1.5, 1.32):                                       # J-hooks
            F.sweep(f'hook{y}', [(-0.33, y, pz + 0.02), (-0.33, y - 0.01, pz + 0.05), (-0.33, y - 0.02, pz + 0.06),
                                 (-0.33, y + 0.005, pz + 0.07)], 0.006, M.brass, verts=6, resolution=2)
        F.soft('tote', 0.2, 0.24, 0.06, (-0.31, 1.2, pz + 0.06), M.wicker, round=0.02, puff=0.01, flat=2, round_segments=1)
        F.sweep('tote_handle', [(-0.38, 1.32, pz + 0.06), (-0.34, 1.33, pz + 0.068), (-0.33, 1.315, pz + 0.07),
                                (-0.28, 1.33, pz + 0.068), (-0.24, 1.32, pz + 0.06)], 0.006, M.oak, verts=5, resolution=2)
        scarf = F.sweep('scarf', [(-0.3, 1.04, pz + 0.075), (-0.305, 1.25, pz + 0.07), (-0.33, 1.49, pz + 0.075),
                                  (-0.335, 1.5, pz + 0.06), (-0.33, 1.49, pz + 0.045), (-0.31, 1.28, pz + 0.04)], None, M.coral,
                        profile=_flat(0.06, 0.01), up=(0, 0, 1), resolution=2)
        G.jitter(scarf, 0.003, size=0.08, seed=4.0)
    G.join(still, 'body', origin=(0, 0, 0))
    # The key tray drawer: front, knob, the tray and keys inside: `tray` slides it out.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.32, 0.07, 0.018, (dx, dy, FZ - 0.009), M.body, bevel=0.005, bevel_segments=1)
        G.kit.knob(F, 'drawer_knob', dx, dy, FZ, 0.012, M.brass, segments=10)
        F.box('drawer_box', 0.3, 0.05, 0.24, (dx, dy - 0.005, FZ - 0.14), M.trim)
        F.soft('key_tray', 0.24, 0.012, 0.16, (dx, dy + 0.026, FZ - 0.12), M.oak, round=0.005, round_segments=1)
        G.torus('key_ring', 0.018, 0.0025, at=F.at(dx - 0.03, dy + 0.034, FZ - 0.11), material=M.brass,
                major_segments=10, minor_segments=4)
        for k in range(2):
            F.extrude(f'key{k}', [(0.0, -0.006), (0.04, -0.004), (0.045, 0.0), (0.04, 0.004), (0.0, 0.006)],
                      0.003, (dx - 0.01 + 0.005 * k, dy + 0.034, FZ - 0.1 + 0.012 * k), M.brass, plane='top')
    drawer = G.join(drawer, 'drawer')
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.slide(drawer, 'tray', (0, -0.2, 0), seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_neat', M,
    [G.Model('steamMopDock', steamMopDock, size=(0.788, 1.25, 0.708), key='model.neat.steamMopDock',
             centre=(-0.006, -0.076),
             motions={'uses': {'mop': 'mop_off'}},
             paint=[('body', 'Body'), ('teal', 'Mop')]),
     G.Model('laundryCenter', laundryCenter, size=(1.82, 1.738, 0.625), key='model.neat.laundryCenter',
             centre=(0.01, -0.1175),   # (-0.127 before 2026-10-22: its shirts stood 1 cm into the wall)
             # the porthole opens to load the washing and to take it out, and shuts for the wash and the dry
             motions={'uses': {'laundry': 'door_open', 'unload': 'door_open'}, 'hold': 2.5},
             paint=[('body', 'Body'), ('appliance', 'Washer & dryer')]),
     G.Model('rainShowerTower', rainShowerTower, size=(0.9, 2.18, 0.9), key='model.neat.rainShowerTower',
             centre=(0, 0.02),
             # the door lets the bather in, shuts behind them and opens again to let them out (as the glasshouse's)
             motions={'uses': {'steamShower': [{'clip': 'door_open', 'hold': 1.5, 'out': 1.5}, 'water_on']}},
             paint=[('tile', 'Body'), ('teak', 'Bench')]),
     G.Model('organizerDesk', organizerDesk, size=(1.6, 1.25, 0.905), key='model.neat.organizerDesk',
             centre=(0, 0.0225),
             motions={'uses': {'sort': 'drawer_open', 'ledger': 'drawer_open'}},
             paint=[('body', 'Body'), ('birch', 'Wood'), ('pastel_c', 'Chair')]),
     G.Model('robotVacuumDock', robotVacuumDock, size=(0.86, 0.519, 0.86), key='model.neat.robotVacuumDock',
             # once programmed the robot drives out of its dock, round its rug and back, again and again
             motions={'loops': {'program': 'vacuum_round'}},
             paint=[('appliance', 'Body'), ('teal', 'Trim'), ('rug', 'Rug')]),
     G.Model('entryConsole', entryConsole, size=(0.87, 1.805, 0.318), key='model.neat.entryConsole',
             centre=(-0.005, -0.262),
             motions={'uses': {'tidy': 'tray'}},
             paint=[('body', 'Body'), ('birch', 'Wood')])],
    comment='Blender-built neat pack models (tools/art/blender/families/pack_neat.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves one part while it is used: the mop leaves its dock for the '
            'resident\'s hands, the washer\'s door opens, the rain falls, a bin slides out, the robot rolls '
            'out and the key tray drawer opens.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
