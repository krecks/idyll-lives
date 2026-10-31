# The street furniture (CM2): the street lamp (`model.streetLamp`), the long park bench (`model.parkBench`), the
# drinking fountain (`model.drinkingFountain`), the litter bin (`model.litterBin`), the bus shelter
# (`model.busStop`), the post box (`model.postBox`), the notice board (`model.noticeBoard`), the phone box
# (`model.phoneBooth`), the town fountain (`model.fountainTown`) and the founder's statue (`model.townStatue`). Notes:
# street_furniture.md.
#
# Each is built in game axes (`F`) where the game places it, at the content's heights (the bench's and the
# shelter's seats, the fountain's rim, the lamp's light), and each has a far version (a quarter): they stand on
# the town's lots and are seen from afar. The lamp's lantern glows at night; the drinking fountain's arc of water
# grows while someone drinks (`water_on`); the post box's flap swings in for a card (`flap`); the phone box's
# door opens and shuts behind the caller (`door_open`); the town fountain's water runs all the time near the
# camera (`water_run`, ambient). The rest are still.
#
# Run:  cargo art street_furniture [--only postBox] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
import bpy  # noqa: E402

import gamekit as G  # noqa: E402

# Colours every model may use. Each model adds its own `body` (the part the player colours), whose colour and
# finish differ from model to model (a red gloss post box, a cast-iron lamp, a stone fountain), so each model
# has a palette of its own: `_palette(body=...)`.
SHARED = dict(
    iron=('#2B302E', 'metal'),        # cast iron: posts, frames, the lamp's lantern frame
    stone=('#C9C2B4', 'matte'),       # pale stone
    granite=('#8E8C88', 'matte'),     # plinths and slabs
    water=('#6FA8C0', 'gloss'),       # standing water: opaque, as the old parts'
    spray=('#BFDDE8', 'gloss'),       # running water: jets, streams and arcs (gloss, never emissive)
    chrome=('#C8CCD0', 'chrome'),     # taps, handles, the pedal
    dark=('#1E2120', 'matte'),        # openings, slots, the phone
    paper=('#F2F0EA', 'matte'),       # labels, timetables, plates
    brass=('#B08D4A', 'metal'),       # crests and plaques
    wood=('#9A6B45', 'wood'),         # slats
    glass=('#C9DDE3', 'glass'),       # the shelter's and the phone box's glazing (see-through)
    blue=('#2E6FA8', 'gloss'),        # the bus stop's sign
    map=('#DCE6D2', 'matte'),         # the shelter's map
    # notices pinned on the board: plain, so they cost no draw call
    note_white=('#F4F1E6', 'satin'),
    note_yellow=('#F2D16B', 'satin'),
    note_pink=('#E9A6A6', 'satin'),
    note_blue=('#9FC5E8', 'satin'),
    note_green=('#B6D7A8', 'satin'),
    pin=('#B8322B', 'satin'),
    cork=('#B88A5A', 'matte'),
    # at night: the lamp's lantern and the phone box's sign glow (E8), nothing by day
    lantern=('#F6EBCF', 'satin', {'glow': '#FFD596'}),
    sign=('#F2F0EA', 'matte', {'glow': '#FFF1D2'}),
)


class _Palettes:
    """The family's palette: the one of the model being built (`Family.run` asks it for the pack's `materials`
    right after each build)."""
    current = None

    def finishes(self, names):
        return self.current.finishes(names)


PALETTES = _Palettes()


def _palette(**own):
    """The palette of the model about to be built: the shared colours and its own (its `body`)."""
    PALETTES.current = G.Palette(**{**SHARED, **own})
    return PALETTES.current


def _sweep_tube(F, name, pts, r, material, verts=8, scale=1.0):
    """A bent tube in one piece through game points `pts`, curved through them (a stream of water, the bubbler's
    arc): replaces the rods joined by balls."""
    return F.sweep(name, pts, r, material, verts=verts, resolution=3, scale=scale)


def _fluted(r, y, flutes, depth=0.08, per=3, phase=0.0):
    """A ring of `flutes` shallow flutes round a column of radius `r` at height `y` (Blender points, about the
    origin): `per` points a flute, sunk `depth` of the radius at their middles."""
    n = flutes * per
    out = []
    for i in range(n):
        a = math.tau * i / n + phase
        k = 1 - depth * (0.5 + 0.5 * math.cos(flutes * a))
        out.append((r * k * math.cos(a), r * k * math.sin(a), y))
    return out


def _square(h, y):
    """A square loop `h` either side of the middle at height `y`, its sides square to the axes (Blender)."""
    return [(h, h, y), (-h, h, y), (-h, -h, y), (h, -h, y)]


# --- the street lamp: a cast-iron post with a square lantern ------------------------------------------------
LAMP_LIGHT = 3.5     # the content's `light.height`: the lantern's panes run from 3.46 to 3.88 m


def streetLamp():
    M = _palette(body=('#2B302E', 'metal'))
    F = G.GameFrame()
    iron = M.body
    with G.new_parts():
        # the base: a square plinth with a chamfer, a fluted bell rising to a moulded collar
        G.loft('plinth', [G.ring(0.19, 0.19, 0.0, 24, square=10), G.ring(0.19, 0.19, 0.035, 24, square=10),
                          G.ring(0.172, 0.172, 0.06, 24, square=10), G.ring(0.16, 0.16, 0.075, 24, square=4)],
               material=iron)
        bell = [(0.158, 0.075), (0.152, 0.12), (0.135, 0.2), (0.112, 0.3), (0.095, 0.4), (0.086, 0.5)]
        G.loft('bell', [_fluted(r, y, 12, 0.07) for r, y in bell], material=iron)
        G.revolve('collar', [(0, 0.495), (0.098, 0.5), (0.104, 0.515), (0.092, 0.53), (0.082, 0.55), (0.09, 0.565),
                             (0.07, 0.58), (0, 0.58)], material=iron, segments=16)
        # the post: fluted, a collar part way, a moulded capital under the lantern's bracket
        G.loft('post', [_fluted(0.047, 0.575, 12, 0.1), _fluted(0.044, 2.93, 12, 0.1)], material=iron)
        G.revolve('collar2', [(0, 2.92), (0.06, 2.925), (0.066, 2.945), (0.056, 2.965), (0, 2.965)], material=iron,
                  segments=14)
        F.cyl('neck', 0.042, 0.42, (0, 3.17, 0), iron, verts=12)
        # the ladder bar a lamplighter leant on, ball ends
        F.sweep('ladder_bar', [(-0.29, 3.12, 0), (0.29, 3.12, 0)], 0.016, iron, verts=8)
        for s in (-1, 1):
            F.sphere(f'ladder_ball{s}', 0.0285, (s * 0.3, 3.12, 0), iron, segments=10, rings=6)
        # four scrolled brackets from the capital out under the lantern's floor
        for k in range(4):
            arm = G.kit.scroll(F, f'bracket{k}', (0, 3.43, 0.045), (3.36, 0.1), iron, turns=-0.9, end=0.35,
                               t=0.012, w=0.022, steps=10)
            arm.rotation_euler[2] = math.radians(90 * k)
        G.revolve('capital', [(0, 3.28), (0.05, 3.3), (0.062, 3.33), (0.075, 3.38), (0.11, 3.43), (0.11, 3.445),
                              (0, 3.445)], material=iron, segments=16)
        # the lantern: four frosted panes, wider at the top, in an iron frame
        sq = 2 ** 0.5
        F.cone('panes', 0.13 * sq, 0.19 * sq, 0.42, (0, 3.67, 0), M.lantern, verts=4, rot=(0, 0, 45), smooth=False)
        G.loft('lantern_floor', [_square(0.12, 3.43), _square(0.155, 3.455), _square(0.155, 3.475)], material=iron,
               smooth=False)
        G.loft('lantern_top', [_square(0.2, 3.875), _square(0.212, 3.885), _square(0.212, 3.91)], material=iron,
               smooth=False)
        for k in range(4):
            a = 45 + 90 * k
            lo, hi = 0.137 * sq, 0.197 * sq
            F.sweep(f'frame{k}', [(lo * G.cosd(a), 3.46, lo * G.sind(a)), (hi * G.cosd(a), 3.885, hi * G.sind(a))], 0.013,
                    iron, verts=6)
        # the roof: a hipped canopy, its sides curving in, a vented crown and a finial
        G.loft('roof', [_square(0.233, 3.905), _square(0.233, 3.925), _square(0.2, 3.95), _square(0.14, 3.99),
                        _square(0.085, 4.04), _square(0.06, 4.08)], material=iron, smooth=False)
        G.revolve('vent', [(0, 4.07), (0.065, 4.075), (0.065, 4.09), (0.05, 4.1), (0.05, 4.13), (0.07, 4.14),
                           (0.07, 4.152), (0.03, 4.165), (0, 4.165)], material=iron, segments=12)
        G.revolve('finial', [(0, 4.16), (0.014, 4.165), (0.012, 4.18), (0.032, 4.2), (0.026, 4.225), (0.008, 4.236),
                             (0, 4.24)], material=iron, segments=10)
    G.join_all('streetLamp')
    G.settle()
    return G.meshes()


# --- the long park bench: wooden slats on three cast-iron supports --------------------------------------------
SEAT = 0.45          # the content's `seat.height`; the seat runs centred on depth 0 (`seat.forward` 0)
# 2.5 m on its three tiles, its three sitters a metre apart (it was 2.86 m, past a long bench's 1.8-2.5)
BENCH_LONG = 2.5
BENCH_LEGS = (-1.21, 0.0, 1.21)


def parkBench():
    M = _palette(body=('#2F3A33', 'metal'))
    F = G.GameFrame()
    frame = M.body
    rail = SEAT - 0.035      # the supports' seat rails, under the slats
    with G.new_parts():
        # three cast-iron supports, each in one piece (gamekit's bench end), arms and scrolls on the end two
        for x in BENCH_LEGS:
            G.kit.bench_end(F, f'end{x}', x, rail, 0.2, -0.25, (0.885, -0.3), frame, arm=0.682 if x else None,
                            t=0.028, w=0.044, foot=0.1)
        for k, z in enumerate((-0.17, -0.085, 0.0, 0.085, 0.17)):
            F.box(f'slat{k}', BENCH_LONG, 0.035, 0.072, (0, SEAT - 0.0175, z), M.wood, bevel=0.009)
            for x in BENCH_LEGS[::2]:                  # domed bolts over the end supports
                F.sphere(f'bolt{k}{x}', 0.009, (x, SEAT, z), frame, segments=6, rings=3, scale=(1, 1, 0.5))
        for k, y in enumerate((0.6, 0.72, 0.84)):
            z = -0.2 - (y - 0.42) * 0.1 / 0.48 + 0.03
            F.box(f'back{k}', BENCH_LONG, 0.085, 0.03, (0, y, z), M.wood, bevel=0.009, rot=(-12, 0, 0))
            for x in BENCH_LEGS[::2]:
                F.sphere(f'back_bolt{k}{x}', 0.009, (x, y, z + 0.017), frame, segments=6, rings=3, scale=(1, 0.5, 1))
    G.join_all('parkBench')
    G.settle(centre=False)
    return G.meshes()


# --- the drinking fountain: a stone pillar and bowl, a chrome bubbler -------------------------------------------
TIP = (0, 1.075, -0.06)    # the bubbler's mouth (game), where the arc of water starts


def drinkingFountain():
    M = _palette(body=('#C9C2B4', 'matte'))
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('slab', 0.56, 0.08, 0.56, (0, 0.04, 0), M.granite, bevel=0.015)
        # the pillar: a moulded base, a fluted shaft narrowing up, a moulded capital under the bowl
        G.revolve('pillar_base', [(0, 0.07), (0.2, 0.07), (0.2, 0.1), (0.188, 0.112), (0.175, 0.12), (0.18, 0.135),
                                  (0.162, 0.15), (0.152, 0.17), (0, 0.17)], material=M.body, segments=24)
        G.loft('pillar', [_fluted(r, y, 16, 0.06) for r, y in ((0.152, 0.165), (0.143, 0.48), (0.136, 0.78))],
               material=M.body)
        G.revolve('capital', [(0, 0.775), (0.14, 0.775), (0.15, 0.79), (0.142, 0.8), (0.15, 0.82), (0.19, 0.86),
                              (0.19, 0.88), (0, 0.88)], material=M.body, segments=24)
        # the bowl: a dish with a rolled lip, falling to the drain
        G.revolve('bowl', [(0, 0.86), (0.16, 0.86), (0.205, 0.9), (0.222, 0.94), (0.228, 0.97), (0.225, 0.99),
                           (0.214, 0.998), (0.2, 0.995), (0.19, 0.975), (0.15, 0.955), (0.08, 0.942), (0, 0.938)],
                  material=M.body, segments=32)
        F.cyl('drain', 0.03, 0.006, (0, 0.94, 0.02), M.chrome, verts=12)
        for k in range(3):                             # the drain's grating
            F.box(f'grate{k}', 0.044 - 0.012 * abs(k - 1), 0.004, 0.005, (0, 0.944, 0.008 + 0.012 * k), M.dark)
        # the bubbler: a chrome riser at the back of the bowl, bent forward to its mouth, a guard arched over it
        G.revolve('bubbler_foot', [(0, 0.95), (0.034, 0.95), (0.032, 0.96), (0.022, 0.97), (0, 0.97)],
                  at=F.at(0, 0, -0.12), material=M.chrome, segments=12)
        F.sweep('bubbler', G.kit.fillet([(0, 0.955, -0.12), (0, 1.073, -0.12), (0, 1.073, -0.062)], 0.025), 0.02,
                M.chrome, verts=10)
        F.cyl('mouth', 0.011, 0.006, (0, 1.091, -0.074), M.dark, verts=8)
        F.sweep('guard', [(-0.03, 1.07, -0.075), (-0.03, 1.08, -0.045), (0, 1.088, -0.03), (0.03, 1.08, -0.045),
                          (0.03, 1.07, -0.075)], 0.005, M.chrome, verts=6, resolution=2)
        # the foot pedal: a bent lever out of the pillar's foot, a ribbed tread on its end
        F.sweep('pedal_arm', G.kit.fillet([(0, 0.11, 0.12), (0, 0.11, 0.17), (0, 0.13, 0.2)], 0.02), 0.012, M.chrome,
                verts=8)
        F.box('pedal', 0.18, 0.025, 0.08, (0, 0.135, 0.22), M.chrome, bevel=0.008)
        for k in range(3):
            F.box(f'tread{k}', 0.16, 0.006, 0.01, (0, 0.149, 0.195 + 0.025 * k), M.dark)
    G.join(still, 'drinkingFountain', origin=(0, 0, 0))
    # the arc of water from the mouth into the bowl, its origin at the mouth, hidden (scaled to nothing) at rest
    with G.new_parts() as arc:
        _sweep_tube(F, 'arc', [TIP, (0, 1.115, -0.03), (0, 1.13, 0.005), (0, 1.115, 0.045), (0, 1.065, 0.08), (0, 0.985, 0.1)],
                    0.013, M.spray, scale=[1.0, 1.0, 0.95, 0.9, 0.85, 0.75])
    water = G.hinged(G.join(arc, 'water'), F.at(*TIP))
    water.scale = (0.001, 0.001, 0.001)
    G.settle()
    for axis in 'XYZ':
        G.grow(water, 'water_on', seconds=0.35, axis=axis)
    return G.meshes()


# --- the litter bin: a slatted green bin on a post ----------------------------------------------------------
def _ribbed(r, y, ribs, out=0.009, at=(0.0, 0.0)):
    """A ring round a bin of radius `r` at height `y` (Blender): `ribs` slats standing `out` proud, steep-sided, narrow
    grooves between them."""
    pts = []
    span = math.tau / ribs
    for k in range(ribs):
        for f, rr in ((0.06, r), (0.1, r + out), (0.9, r + out), (0.94, r)):
            a = span * (k + f)
            pts.append((at[0] + rr * math.cos(a), at[1] + rr * math.sin(a), y))
    return pts


def litterBin():
    M = _palette(body=('#3C6B3A', 'metal'))
    F = G.GameFrame()
    z = 0.02
    bx, by, _ = F.at(0, 0, z)
    with G.new_parts():
        # the post: a square tube with a pyramid cap, the bin hung on it by two straps round its back
        F.box('post', 0.07, 0.95, 0.07, (0, 0.475, -0.28), M.iron, bevel=0.008, bevel_segments=1)
        F.cone('post_cap', 0.064, 0.012, 0.03, (0, 0.965, -0.28), M.iron, verts=4, rot=(0, 0, 45), smooth=False)
        for y in (0.33, 0.66):
            pts = [(0.216 * math.cos(math.radians(a)), y, z + 0.216 * math.sin(math.radians(a))) for a in range(200, 345, 12)]
            F.sweep(f'strap{y}', pts, None, M.iron, profile=G.rounded_rect(0.04, 0.008, 0.0))
            F.box(f'lug{y}', 0.05, 0.04, 0.06, (0, y, -0.225), M.iron, bevel=0.006, bevel_segments=1)
        # the bin: pressed ribs all round between two rolled beads, a rounded foot, a rolled rim round its dark mouth
        n = 48
        rows = [G.ring(0.17, 0.17, 0.18, n, at=(bx, by)), G.ring(0.19, 0.19, 0.185, n, at=(bx, by)),
                G.ring(0.2, 0.2, 0.2, n, at=(bx, by)), G.ring(0.208, 0.208, 0.215, n, at=(bx, by)),
                _ribbed(0.2, 0.235, 12, 0.012, at=(bx, by)), _ribbed(0.206, 0.71, 12, 0.012, at=(bx, by)),
                G.ring(0.214, 0.214, 0.735, n, at=(bx, by)), G.ring(0.208, 0.208, 0.775, n, at=(bx, by))]
        G.loft('bin', rows, material=M.body)
        F.sweep('rim', [(0.216 * math.cos(a), 0.785, z + 0.216 * math.sin(a)) for a in
                        [math.tau * i / 24 for i in range(24)]], 0.021, M.iron, verts=5, closed=True)
        F.cyl('opening', 0.2, 0.01, (0, 0.785, z), M.dark, verts=24)
        # a black bag's edge turned over the rim at the back
        F.sweep('bag', [(0.17 * math.cos(math.radians(a)), 0.79, z + 0.17 * math.sin(math.radians(a)))
                        for a in range(210, 335, 25)], None, M.dark, profile=G.rounded_rect(0.012, 0.012, 0.0),
                resolution=2)
        # the label on its front: a white plate with a bin sign and two lines on it
        F.extrude('label', G.rounded_rect(0.16, 0.08, 0.01, corner=2), 0.012, (0, 0.62, z + 0.216), M.paper)
        F.extrude('sign_bin', [(-0.012, -0.018), (0.012, -0.018), (0.016, 0.012), (-0.016, 0.012)], 0.004,
                  (-0.045, 0.615, z + 0.223), M.dark)
        F.box('sign_lid', 0.04, 0.005, 0.004, (-0.045, 0.633, z + 0.223), M.dark)
        for k, w in enumerate((0.07, 0.05)):
            F.box(f'sign_line{k}', w, 0.008, 0.004, (0.0 + w / 2 - 0.01, 0.628 - 0.02 * k, z + 0.223), M.dark)
    G.join_all('litterBin')
    G.settle(centre=False)
    return G.meshes()


# --- the bus shelter: a glazed frame with a bench for three -------------------------------------------------
SHELTER_SEAT = 0.46   # the content's `seat.height`
SHELTER_BENCH = -0.31  # the bench's middle, its two slats either side (the content's `seat.forward` -0.3: the front one)
# The posts' depths: 1.2 m deep under the roof on its one tile, a shelter's depth (it was 0.98 m, the bench filling it).
SHELTER_BACK, SHELTER_FRONT = -0.55, 0.55


def busStop():
    M = _palette(body=('#5C6168', 'metal'))
    F = G.GameFrame()
    frame = M.body
    back, front = SHELTER_BACK, SHELTER_FRONT
    mid, deep = (back + front) / 2, front - back
    with G.new_parts():
        # four posts on bolted foot plates, capped
        for x in (-1.42, 1.42):
            for z in (back, front):
                F.box(f'post{x}{z}', 0.07, 2.32, 0.07, (x, 1.16, z), frame, bevel=0.012)
                F.box(f'plate{x}{z}', 0.1, 0.012, 0.1, (x, 0.006, z), frame, bevel=0.003, bevel_segments=1)
                for k in range(4):
                    a = math.tau * (k + 0.5) / 4
                    F.cyl(f'bolt{x}{z}{k}', 0.007, 0.012, (x + 0.05 * math.cos(a), 0.016, z + 0.05 * math.sin(a)), frame,
                          verts=6)
        # the roof: a curved canopy on a beam along the front and the back, a gutter at the back
        arc = [(back - 0.0525 + (deep + 0.105) * i / 12, 0.0) for i in range(13)]
        lower = [(u, 2.352 + 0.05 * math.sin(math.pi * i / 12)) for i, (u, _) in enumerate(arc)]
        upper = [(u, y + 0.03) for u, y in reversed(lower)]
        F.extrude('roof', lower + upper, 2.98, (0, 0, 0), frame, plane='side', bevel=0.006, bevel_segments=1)
        for z in (back, front):
            F.box(f'beam{z}', 2.9, 0.09, 0.06, (0, 2.315, z), frame, bevel=0.01)
        F.box('fascia', 2.98, 0.11, 0.025, (0, 2.36, front + 0.037), frame, bevel=0.008)
        F.box('fascia_stripe', 2.7, 0.03, 0.004, (0, 2.36, front + 0.0505), M.blue)
        F.sweep('gutter', [(-1.49, 2.34, back - 0.035), (1.49, 2.34, back - 0.035)], None, frame,
                profile=G.rounded_rect(0.04, 0.035, 0.008, corner=1))
        # the glazing: the back in two panes and the right-hand end, in rails and a mullion, a white band across each
        F.box('glass_back', 2.78, 1.9, 0.015, (0, 1.27, back), M.glass)
        F.box('glass_side', 0.015, 1.9, deep - 0.08, (1.42, 1.27, mid), M.glass)
        F.box('mullion', 0.04, 1.96, 0.04, (0, 1.27, back), frame, bevel=0.006)
        for y in (0.3, 2.24):
            F.box(f'rail_back{y}', 2.8, 0.05, 0.05, (0, y, back), frame, bevel=0.008)
            F.box(f'rail_side{y}', 0.05, 0.05, deep - 0.06, (1.42, y, mid), frame, bevel=0.008)
        for s in (-1, 1):
            F.box(f'band_back{s}', 1.3, 0.05, 0.004, (s * 0.69, 1.45, back + 0.01), M.paper)
        F.box('band_side', 0.004, 0.05, deep - 0.14, (1.41, 1.45, mid), M.paper)
        # the bench: two slats on cantilevered brackets from the back rail, each on a slim leg
        for k, z in enumerate((SHELTER_BENCH + 0.08, SHELTER_BENCH - 0.08)):
            F.box(f'slat{k}', 2.5, 0.035, 0.15, (0, SHELTER_SEAT - 0.0175, z), M.wood, bevel=0.009)
        for x in (-1.0, 0.0, 1.0):
            F.sweep(f'bracket{x}', G.kit.fillet([(x, 0.44, SHELTER_BENCH + 0.15), (x, 0.42, back + 0.03),
                                                 (x, 0.3, back + 0.03)], 0.03, steps=2), None, frame,
                    profile=G.rounded_rect(0.04, 0.03, 0.008, corner=1), up=(1, 0, 0))
            F.box(f'bracket_leg{x}', 0.04, 0.42, 0.04, (x, 0.21, SHELTER_BENCH - 0.12), frame, bevel=0.006)
        # the timetable on the back glass, a map beside it
        F.box('timetable_frame', 0.56, 0.78, 0.03, (-0.95, 1.45, back + 0.02), frame, bevel=0.008)
        F.box('timetable', 0.5, 0.68, 0.01, (-0.95, 1.43, back + 0.038), M.paper)
        F.box('timetable_head', 0.5, 0.08, 0.012, (-0.95, 1.73, back + 0.04), M.blue)
        for k in range(8):
            F.box(f'times{k}', 0.26, 0.012, 0.004, (-1.02, 1.64 - 0.06 * k, back + 0.044), M.dark)
            F.box(f'clock{k}', 0.08, 0.012, 0.004, (-0.78, 1.64 - 0.06 * k, back + 0.044), M.dark)
        F.box('map_frame', 0.5, 0.6, 0.03, (-0.35, 1.4, back + 0.02), frame, bevel=0.008)
        F.box('map', 0.44, 0.54, 0.01, (-0.35, 1.4, back + 0.038), M.map)
        for k, (x, y, w, h) in enumerate(((-0.4, 1.45, 0.3, 0.012), (-0.3, 1.35, 0.012, 0.36), (-0.36, 1.3, 0.25, 0.012))):
            F.box(f'route{k}', w, h, 0.004, (x, y, back + 0.045), M.blue if k != 1 else M.dark)
        # the stop's flag on the roof, at the front right: a blue plate with the white face and a bus
        F.box('flag_pole', 0.05, 0.22, 0.05, (1.18, 2.52, front - 0.02), frame, bevel=0.006)
        F.box('flag', 0.6, 0.34, 0.04, (1.18, 2.72, front - 0.02), M.blue, bevel=0.015)
        F.box('flag_face', 0.44, 0.2, 0.046, (1.18, 2.72, front - 0.02), M.paper, bevel=0.01)
        for s in (-1, 1):
            zf = front - 0.02 + s * 0.024
            F.box(f'flag_bus{s}', 0.24, 0.08, 0.004, (1.18, 2.735, zf), M.blue, bevel=0.002, bevel_segments=1)
            for k in range(4):
                F.box(f'flag_window{s}{k}', 0.035, 0.03, 0.005, (1.11 + 0.045 * k, 2.75, zf), M.paper)
            for x in (1.1, 1.26):
                G.cylinder(f'flag_wheel{s}{x}', 0.018, 0.006, at=F.at(x, 2.69, zf), material=M.dark, rot=(90, 0, 0),
                           verts=8)
    G.join_all('busStop')
    G.settle(centre=False)
    return G.meshes()


# --- the post box: a red pillar box; its slot's flap swings in for a card ------------------------------------
SLOT_Y = 1.16        # the flap's hinge (its top edge), game height
SLOT_Z = 0.262       # the flap's face, game depth


def _on_pillar(F, name, path, r, radius, material, closed=True):
    """A bead (a round cord `radius` thick) along `path`, [(degrees round from the front, height), ...], on the
    pillar's face of radius `r`: the door's outline, cast on."""
    pts = [(r * math.sin(math.radians(a)), y, r * math.cos(math.radians(a))) for a, y in path]
    return F.sweep(name, pts, radius, material, verts=5, closed=closed)


def postBox():
    M = _palette(body=('#B8322B', 'gloss'))
    F = G.GameFrame()
    n = 32
    with G.new_parts() as still:
        G.revolve('plinth', [(0, 0), (0.245, 0), (0.245, 0.05), (0.236, 0.062), (0.226, 0.068), (0.222, 0.082),
                             (0, 0.082)], material=M.dark, segments=n)
        # the pillar in one turned piece: a foot moulding, the band, the shaft, a stepped cornice and the domed cap
        G.revolve('pillar', [(0, 0.08), (0.222, 0.08), (0.222, 0.1), (0.214, 0.108), (0.212, 0.22), (0.222, 0.226),
                             (0.224, 0.24), (0.222, 0.254), (0.212, 0.26), (0.21, 1.3), (0.222, 1.31), (0.236, 1.33),
                             (0.25, 1.345), (0.25, 1.375), (0.24, 1.392), (0.232, 1.4), (0.226, 1.43), (0.205, 1.465),
                             (0.165, 1.495), (0.11, 1.516), (0.05, 1.527), (0, 1.529)], material=M.body, segments=n)
        # the door's outline cast round the front, its keyhole and a little lifting handle
        door = [(-38, 0.32), (38, 0.32), (38, 0.94), (30, 0.99), (0, 1.0), (-30, 0.99), (-38, 0.94)]
        _on_pillar(F, 'door', G.kit.fillet(door + [door[0]], 0.0, steps=1)[:-1], 0.211, 0.006, M.body)
        F.cyl('keyhole_plate', 0.022, 0.006, (0.0, 0.42, 0.212), M.body, verts=12, rot=(90, 0, 0), bevel=0.002,
              bevel_segments=1)
        F.box('keyhole', 0.006, 0.018, 0.004, (0.0, 0.42, 0.216), M.dark)
        # the slot's housing, proud of the pillar: an arched hood, a sill and cheeks round a dark opening
        F.extrude('hood', [(0.145, 0.0), (0.145, 0.018), (0.08, 0.034), (0.0, 0.04), (-0.08, 0.034), (-0.145, 0.018),
                           (-0.145, 0.0)], 0.09, (0, SLOT_Y + 0.01, 0.225), M.body, bevel=0.004, bevel_segments=1)
        F.box('sill', 0.27, 0.025, 0.07, (0, SLOT_Y - 0.07, 0.23), M.body, bevel=0.006)
        for s in (-1, 1):
            F.box(f'cheek{s}', 0.025, 0.1, 0.07, (s * 0.1225, SLOT_Y - 0.02, 0.23), M.body, bevel=0.006)
        F.box('opening', 0.22, 0.08, 0.02, (0, SLOT_Y - 0.025, 0.21), M.dark)
        # the collection times: a framed white plate with its lines
        F.extrude('times_frame', G.rounded_rect(0.16, 0.13, 0.012, corner=2), 0.012, (0, 0.82, 0.208), M.body,
                  holes=[G.rounded_rect(0.136, 0.106, 0.006, corner=1)])
        F.box('times', 0.14, 0.11, 0.012, (0, 0.82, 0.205), M.paper)
        for k in range(4):
            F.box(f'times_line{k}', 0.1 - 0.02 * (k % 2), 0.008, 0.003, (-0.005, 0.85 - 0.02 * k, 0.212), M.dark)
        # the crest over the slot: a little crown in brass on a plate
        F.extrude('crest', [(-0.04, -0.025), (0.04, -0.025), (0.045, 0.012), (0.026, 0.0), (0.014, 0.022), (0.0, 0.004),
                            (-0.014, 0.022), (-0.026, 0.0), (-0.045, 0.012)], 0.01, (0, 1.265, 0.21), M.brass)
        F.box('crest_band', 0.08, 0.012, 0.012, (0, 1.236, 0.211), M.brass, bevel=0.003, bevel_segments=1)
    G.join(still, 'postBox', origin=(0, 0, 0))
    with G.new_parts() as flap:
        F.box('flap', 0.21, 0.05, 0.008, (0, SLOT_Y - 0.025, SLOT_Z), M.body, bevel=0.003)
        F.box('flap_lip', 0.12, 0.008, 0.012, (0, SLOT_Y - 0.045, SLOT_Z + 0.006), M.body, bevel=0.002)
        F.box('flap_word', 0.08, 0.008, 0.002, (0, SLOT_Y - 0.022, SLOT_Z + 0.004), M.paper)   # "LETTERS"
    flap = G.hinged(G.join(flap, 'flap'), F.at(0, SLOT_Y, SLOT_Z))
    G.settle()
    G.swing(flap, 'flap', 65, seconds=0.5, axis='X')     # its bottom edge swings in, to the back
    return G.meshes()


# --- the notice board: a cork board between two posts, under a little roof -----------------------------------
def noticeBoard():
    M = _palette(body=('#5A3B26', 'wood'))
    F = G.GameFrame()
    z = -0.05
    with G.new_parts():
        for x in (-0.42, 0.42):
            F.box(f'post{x}', 0.08, 1.95, 0.08, (x, 0.975, z), M.body, bevel=0.01)
            F.box(f'foot{x}', 0.1, 0.03, 0.1, (x, 0.015, z), M.iron, bevel=0.004, bevel_segments=1)
        # the board: cork in a moulded frame, a ledge under it
        F.box('board', 0.8, 0.7, 0.04, (0, 1.36, z - 0.005), M.cork)
        F.extrude('frame', G.rounded_rect(0.9, 0.8, 0.01, corner=1), 0.07, (0, 1.36, z + 0.005), M.body,
                  holes=[G.rounded_rect(0.78, 0.68, 0.004, corner=1)], bevel=0.008, bevel_segments=1)
        F.box('ledge', 0.86, 0.025, 0.06, (0, 0.955, z + 0.02), M.body, bevel=0.006)
        # the roof: two boarded slopes, each three overlapping boards, a ridge cap and shaped barge boards
        for s in (-1, 1):
            for k in range(3):
                d = 0.025 + 0.058 * k
                F.box(f'board{s}{k}', 1.0, 0.02, 0.075, (0, 2.03 - d * 0.47 - 0.006 * k, z + s * (d * 0.88 + 0.01)), M.body,
                      bevel=0.004, bevel_segments=1, rot=(s * 28, 0, 0))
        F.box('ridge', 1.02, 0.03, 0.04, (0, 2.05, z), M.body, bevel=0.008)
        for x in (-0.5, 0.5):
            F.extrude(f"barge{x}", [(-0.1735, -0.075), (0.0, 0.03), (0.1735, -0.075), (0.1735, -0.105), (0.0, -0.005),
                                    (-0.1735, -0.105)], 0.02, (x, 2.0, z), M.body, plane='side', bevel=0.003, bevel_segments=1)
        # the notices, pinned: one a leaflet with tear-off strips along its foot
        notes = [(-0.27, 1.55, 0.18, 0.22, M.note_white), (-0.05, 1.5, 0.16, 0.2, M.note_yellow),
                 (0.2, 1.56, 0.2, 0.16, M.note_pink), (-0.25, 1.2, 0.16, 0.22, M.note_blue),
                 (0.0, 1.22, 0.2, 0.18, M.note_white), (0.25, 1.24, 0.16, 0.2, M.note_green)]
        for k, (x, y, w, h, m) in enumerate(notes):
            F.box(f'note{k}', w, h, 0.004, (x, y, z + 0.018), m, rot=(0, (k % 3 - 1) * 3, 0))
            F.sphere(f'pin{k}', 0.012, (x, y + h / 2 - 0.025, z + 0.024), M.pin, segments=8, rings=4)
            if k in (1, 4):
                F.sphere(f'pin{k}b', 0.012, (x + w / 3, y - h / 2 + 0.03, z + 0.024), M.pin, segments=8, rings=4)
            for j in range(3):                         # lines of writing
                F.box(f'note{k}_line{j}', w * (0.7 - 0.15 * (j % 2)), 0.008, 0.002, (x, y + h * 0.15 - j * 0.03, z + 0.021),
                      M.iron, rot=(0, (k % 3 - 1) * 3, 0))
        for j in range(6):                             # the leaflet's tear-off strips
            F.box(f'strip{j}', 0.022, 0.05, 0.004, (0.25 - 0.06 + j * 0.025, 1.24 - 0.12, z + 0.019), M.note_green)
    G.join_all('noticeBoard')
    G.settle(centre=False)
    return G.meshes()


# --- the phone box: a red kiosk, glazed on every side, its door hinged on one side ------------------------
KIOSK = 0.39         # the corner posts' middles, either side of the middle
FACE = 0.41          # the glazing's plane, out from the middle
DOOR_W = 0.66        # the door, between the front posts


def _glazing(F, name, material, bars, at, rotate):
    """One side's window: a pane and its glazing bars (three columns, eight rows), `at` its middle. `rotate`
    turns the side's points: (x, z) -> (x', z') about the kiosk's middle."""
    w, y0, y1 = 0.62, 0.58, 2.16
    pane = F.box(f'{name}_pane', w, y1 - y0, 0.012, at(0, (y0 + y1) / 2, 0), material)
    pane.rotation_euler[2] = rotate
    for k, x in enumerate((-0.11, 0.11)):
        b = F.box(f'{name}_bar{k}', 0.025, y1 - y0, 0.02, at(x, (y0 + y1) / 2, 0.012), bars)
        b.rotation_euler[2] = rotate
    for k in range(7):
        y = y0 + (y1 - y0) * (k + 1) / 8
        b = F.box(f'{name}_rail{k}', w, 0.022, 0.02, at(0, y, 0.012), bars)
        b.rotation_euler[2] = rotate


def _pediment(F, name, face, axis, material, crown):
    """One side's arched top over the sign band: a slab rising in a shallow arch from the cornice, a crown in
    relief on its face. `face` is +1/-1, `axis` 'z' for the front and back, 'x' for the sides."""
    pts = [(-0.46, 0.0), (0.46, 0.0), (0.46, 0.03)]
    pts += [(0.44 * math.cos(math.pi * i / 10), 0.03 + 0.075 * math.sin(math.pi * i / 10)) for i in range(1, 10)]
    pts += [(-0.46, 0.03)]
    cpts = [(-0.035, -0.02), (0.035, -0.02), (0.04, 0.012), (0.022, 0.0), (0.012, 0.02), (0.0, 0.004), (-0.012, 0.02),
            (-0.022, 0.0), (-0.04, 0.012)]
    if axis == 'z':
        F.extrude(name, pts, 0.03, (0, 2.56, face * 0.455), material)
        F.extrude(f'{name}_crown', cpts, 0.008, (0, 2.6, face * 0.472), crown)
    else:
        F.extrude(name, pts, 0.03, (face * 0.455, 2.56, 0), material, plane='side')
        F.extrude(f'{name}_crown', cpts, 0.008, (face * 0.472, 2.6, 0), crown, plane='side')


def phoneBooth():
    M = _palette(body=('#B8322B', 'gloss'))
    F = G.GameFrame()
    red = M.body
    with G.new_parts() as still:
        F.box('step', 0.94, 0.04, 0.94, (0, 0.02, 0), red, bevel=0.008, bevel_segments=1)
        F.box('plinth', 0.9, 0.1, 0.9, (0, 0.07, 0), red, bevel=0.01)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(f'post{sx}{sz}', 0.12, 2.1, 0.12, (sx * KIOSK, 1.17, sz * KIOSK), red, bevel=0.012)
        # the band at the top, its TELEPHONE signs (lit at night), the cornice, an arched pediment on each side with
        # a crown on it, and the shallow dome rising between them to a crown knob
        F.box('band', 0.92, 0.3, 0.92, (0, 2.36, 0), red, bevel=0.015)
        F.box('cornice', 0.95, 0.05, 0.95, (0, 2.535, 0), red, bevel=0.012)
        for s in (-1, 1):
            _pediment(F, f'pediment_z{s}', s, 'z', red, red)
            _pediment(F, f'pediment_x{s}', s, 'x', red, red)
        G.loft('dome', [G.ring(0.44, 0.44, 2.555, 32, square=5), G.ring(0.4, 0.4, 2.6, 32, square=4),
                        G.ring(0.3, 0.3, 2.632, 32, square=3), G.ring(0.16, 0.16, 2.652, 32, square=2.5),
                        G.ring(0.001, 0.001, 2.66, 32)], material=red)
        F.sphere('crown', 0.04, (0, 2.66, 0), red, segments=10, rings=6)
        # the back, the left and the right: a panelled kick panel below each window; the back's window frames the phone
        for k, (rot, ox, oz) in enumerate(((math.pi, 0, -1), (math.pi / 2, 1, 0), (-math.pi / 2, -1, 0))):
            def at(x, y, d, ox=ox, oz=oz):
                # a side's (across, height, out) point: on the face at (ox, oz) * FACE
                if oz:
                    return (x * -oz, y, oz * (FACE + d))
                return (0 + ox * (FACE + d), y, x * ox)
            _glazing(F, f'side{k}', M.glass, red, lambda x, y, d, at=at: at(x, y, d), rot)
            kx, _, kz = at(0, 0, 0)
            F.box(f'kick{k}', 0.68 if oz else 0.04, 0.46, 0.04 if oz else 0.68, (kx, 0.35, kz), red, bevel=0.006)
            px, _, pz = at(0, 0, 0.024)
            F.box(f'kick_panel{k}', 0.5 if oz else 0.012, 0.3, 0.012 if oz else 0.5, (px, 0.35, pz), red, bevel=0.004,
                  bevel_segments=1)
        for k, (x, z, w, d) in enumerate(((0, 0.47, 0.6, 0.012), (0, -0.47, 0.6, 0.012),
                                          (0.47, 0, 0.012, 0.6), (-0.47, 0, 0.012, 0.6))):
            F.box(f'sign{k}', w, 0.1, d, (x, 2.38, z), M.sign)
        # the phone on a panel at the back: its case, a dial, the hook for the handset, the coin box under it, a
        # coiled cord; a shelf of directories under it
        F.box('phone_panel', 0.4, 0.5, 0.02, (0, 1.35, -0.39), red, bevel=0.006)
        F.box('phone', 0.16, 0.2, 0.09, (0, 1.42, -0.335), M.dark, bevel=0.012)
        F.cyl('dial', 0.04, 0.01, (0, 1.44, -0.287), M.chrome, verts=12, rot=(90, 0, 0))
        F.box('coin_box', 0.14, 0.12, 0.08, (0, 1.26, -0.34), M.dark, bevel=0.008)
        F.box('coin_slot', 0.04, 0.006, 0.01, (0, 1.29, -0.299), M.chrome)
        # (The handset on its hook is drawn by code, in the caller's hand while they call: held.py `receiver`,
        # actions.ts `REST_PROPS`. A part of its own here would be a fifth draw.)
        F.sweep('cord', [(-0.1, 1.33, -0.32), (-0.08, 1.25, -0.33), (-0.06, 1.2, -0.34), (-0.07, 1.26, -0.35)], 0.006,
                M.dark, verts=5, resolution=3)
        F.box('shelf', 0.4, 0.03, 0.18, (0, 1.06, -0.3), M.dark, bevel=0.006)
        for k, (x, w, m) in enumerate(((-0.1, 0.05, M.paper), (-0.045, 0.04, M.blue))):
            F.box(f'directory{k}', w, 0.24, 0.16, (x, 1.195, -0.3), m, bevel=0.004, bevel_segments=1)
    G.join(still, 'phoneBooth', origin=(0, 0, 0))
    # the door: its frame, panelled kick panel, window and handle, hinged on its edge at the game's -x (on the right
    # as you face it in the game)
    with G.new_parts() as door:
        F.box('door_kick', DOOR_W, 0.46, 0.04, (0, 0.35, FACE), red, bevel=0.006)
        F.box('door_kick_panel', 0.5, 0.3, 0.012, (0, 0.35, FACE + 0.024), red, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            F.box(f'door_stile{s}', 0.04, 1.62, 0.04, (s * (DOOR_W / 2 - 0.02), 1.37, FACE), red, bevel=0.006)
        F.box('door_head', DOOR_W, 0.06, 0.04, (0, 2.17, FACE), red, bevel=0.006)
        _glazing(F, 'door', M.glass, red, lambda x, y, d: (x, y, FACE + d), 0)
        G.kit.pull(F, 'door_handle', DOOR_W / 2 - 0.06, 1.15, FACE + 0.02, 0.16, M.chrome, r=0.008, out=0.035, upright=True)
    door = G.hinged(G.join(door, 'door'), F.at(-DOOR_W / 2, 1.0, FACE))
    G.settle()
    # Out, about its hinge, to stand square to the front, as a real kiosk's: the caller stands back from the
    # tile in front while it opens, steps inside and it shuts behind them (and it opens again to let them
    # out: the motions' `out`; `STEP_IN` in characters.ts times their steps to it). Opening in, it swept
    # through them inside.
    G.swing(door, 'door_open', 90, seconds=0.8)
    return G.meshes()


# --- the town fountain: a square stone basin, a two-tier fountain in the middle ------------------------------
RIM = 0.5            # the rim's top: the content's `seat.height`, its front at `seat.forward` 1.3
WATER_Y = 0.37       # the basin's water
BOWL_Y = 1.46        # the lower bowl's water
# (radius, height) along a stream falling from a bowl's lip: out over the lip, then down into the water below
TRICKLE = [(0.235, 1.995), (0.265, 1.96), (0.29, 1.86), (0.3, BOWL_Y)]
STREAM = [(0.6, 1.49), (0.65, 1.45), (0.69, 1.33), (0.715, 1.05), (0.73, WATER_Y)]


def _lobed(r, y, lobes, depth, per=2):
    """A ring of `lobes` rounded lobes (gadroons) round a bowl's underside, radius `r` at height `y` (Blender)."""
    return _fluted(r, y, lobes, depth, per=per, phase=math.pi / lobes)


def fountainTown():
    M = _palette(body=('#C9C2B4', 'matte'))
    F = G.GameFrame()
    stone = M.body
    with G.new_parts() as still:
        F.box('slab', 2.9, 0.06, 2.9, (0, 0.03, 0), M.granite, bevel=0.01)
        # the basin: its walls on a plinth band, a raised panel on each face between pilasters, a moulded coping
        # with a rounded nose all round (its top the seat, `RIM`)
        for s in (-1, 1):
            F.box(f'wall_fb{s}', 2.9, 0.4, 0.24, (0, 0.26, s * 1.33), stone, bevel=0.012)
            F.box(f'wall_lr{s}', 0.24, 0.4, 2.42, (s * 1.33, 0.26, 0), stone, bevel=0.012)
            F.box(f'band_fb{s}', 2.93, 0.06, 0.25, (0, 0.09, s * 1.335), stone, bevel=0.01, bevel_segments=1)
            F.box(f'band_lr{s}', 0.25, 0.06, 2.43, (s * 1.335, 0.09, 0), stone, bevel=0.01, bevel_segments=1)
            for k in range(3):
                u = (k - 1) * 0.9
                F.box(f'panel_fb{s}{k}', 0.7, 0.2, 0.02, (u, 0.28, s * 1.455), stone, bevel=0.006, bevel_segments=1)
                F.box(f'panel_lr{s}{k}', 0.02, 0.2, 0.7, (s * 1.455, 0.28, u), stone, bevel=0.006, bevel_segments=1)
        nose = [(0.0, 0.0), (-0.02, 0.006), (-0.03, 0.025), (-0.02, 0.044), (0.0, 0.05), (0.31, 0.05), (0.32, 0.04),
                (0.32, 0.0)]
        G.kit.moulding(F, 'coping', [(1.45, 1.45), (-1.45, 1.45), (-1.45, -1.45), (1.45, -1.45)], nose, stone, at=RIM - 0.05,
                       plane='top')
        F.box('basin_water', 2.42, 0.02, 2.42, (0, WATER_Y - 0.01, 0), M.water)
        # the fountain: a moulded foot, a fluted column, the wide lower bowl gadrooned underneath with a rolled lip, a
        # stem, the small top bowl and a knob
        G.revolve('foot', [(0, 0.3), (0.38, 0.3), (0.38, 0.34), (0.35, 0.37), (0.3, 0.41), (0.24, 0.5), (0.21, 0.6),
                           (0.2, 0.66), (0, 0.66)], material=stone, segments=24)
        G.revolve('column_base', [(0, 0.64), (0.16, 0.64), (0.16, 0.67), (0.135, 0.7), (0, 0.7)], material=stone,
                  segments=16)
        G.loft('column', [_fluted(0.12, 0.69, 12, 0.1), _fluted(0.115, 1.13, 12, 0.1)], material=stone)
        G.revolve('capital', [(0, 1.12), (0.13, 1.12), (0.15, 1.15), (0.17, 1.2), (0, 1.2)], material=stone, segments=16)
        G.revolve('bowl', [(0, 1.18), (0.2, 1.2), (0.5, 1.36), (0.6, 1.44), (0.612, 1.455), (0.612, 1.475), (0.6, 1.488),
                           (0.55, 1.49), (0.5, 1.44), (0, 1.42)], material=stone, segments=24)
        G.loft('bowl_lobes', [_lobed(r, y, 16, 0.12) for r, y in ((0.16, 1.19), (0.32, 1.255), (0.47, 1.335),
                                                                    (0.565, 1.4))], material=stone)
        F.cyl('bowl_water', 0.53, 0.02, (0, BOWL_Y, 0), M.water, verts=24)
        G.revolve('stem', [(0, 1.44), (0.11, 1.44), (0.1, 1.47), (0.075, 1.52), (0.062, 1.62), (0.07, 1.72),
                           (0.062, 1.8), (0.09, 1.85), (0.1, 1.88), (0, 1.88)], material=stone, segments=14)
        G.revolve('top_bowl', [(0, 1.84), (0.08, 1.85), (0.22, 1.94), (0.244, 1.97), (0.244, 1.99), (0.21, 2.0),
                               (0, 1.97)], material=stone, segments=24)
        G.loft('top_lobes', [_lobed(r, y, 12, 0.14) for r, y in ((0.085, 1.852), (0.16, 1.895), (0.225, 1.945))],
               material=stone)
        G.revolve('knob', [(0, 1.97), (0.05, 1.97), (0.045, 1.99), (0.07, 2.03), (0.065, 2.07), (0.04, 2.1), (0.0, 2.114)],
                  material=stone, segments=12)
        # the water spilling over the lips: streams curving out of the top bowl into the lower one, and out of
        # the lower one into the basin, thinning as they fall
        for k in range(6):
            a = 30 + 60 * k
            F.sweep(f'trickle{k}', [(r * G.cosd(a), y, r * G.sind(a)) for r, y in TRICKLE], 0.014, M.spray, verts=5,
                    resolution=2, scale=[1.0, 0.95, 0.85, 0.75])
        for k in range(8):
            a = 22.5 + 45 * k
            F.sweep(f'stream{k}', [(r * G.cosd(a), y, r * G.sind(a)) for r, y in STREAM], 0.024, M.spray, verts=5,
                    resolution=2, scale=[1.0, 0.95, 0.85, 0.75, 0.7])
    G.join(still, 'fountainTown', origin=(0, 0, 0))
    # what moves: the jet on top, rising and falling, and the splashes where the streams land, bobbing with it
    with G.new_parts() as jet:
        F.cone('jet', 0.03, 0.022, 0.32, (0, 2.24, 0), M.spray, verts=10)
        F.sphere('jet_crown', 0.065, (0, 2.405, 0), M.spray, segments=12, rings=6, scale=(1, 1, 1.05))
    jet = G.hinged(G.join(jet, 'jet'), F.at(0, 2.08, 0))
    with G.new_parts() as splashes:
        for k in range(8):
            a = 22.5 + 45 * k
            x, z = 0.73 * G.cosd(a), 0.73 * G.sind(a)
            # a crown of spray: a low ring flaring up round where the stream lands
            G.revolve(f'splash{k}', [(0.0, -0.02), (0.08, -0.02), (0.1, 0.025), (0.05, 0.012), (0.0, 0.02)],
                      at=F.at(x, WATER_Y, z), material=M.spray, segments=8)
    splash = G.hinged(G.join(splashes, 'splash'), F.at(0, WATER_Y, 0))
    G.settle()
    G.squash(jet, 'water_run', (1, 1, 1.3), seconds=1.2, back=True)
    G.squash(splash, 'water_run', (1, 1, 1.8), seconds=1.2, back=True)
    return G.meshes()


# --- the founder's statue: a bronze woman in a long coat on a granite plinth, pointing --------------------------
def townStatue():
    M = _palette(body=('#5E7F6E', 'metal'))
    F = G.GameFrame()
    bronze = M.body
    y = 1.42         # the plinth's top
    with G.new_parts():
        # the plinth: two steps, a moulded base, the die with a sunk panel on each face, a moulded cornice
        F.box('step', 1.8, 0.1, 1.8, (0, 0.05, 0), M.granite, bevel=0.015)
        F.box('step2', 1.56, 0.1, 1.56, (0, 0.15, 0), M.granite, bevel=0.015)
        F.box('die', 1.2, 1.1, 1.2, (0, 0.75, 0), M.granite, bevel=0.02)
        sq = [(0.6, 0.6), (-0.6, 0.6), (-0.6, -0.6), (0.6, -0.6)]
        G.kit.moulding(F, 'base_moulding', sq, [(-0.07, 0.0), (-0.07, 0.04), (-0.04, 0.06), (-0.02, 0.1), (0.02, 0.1),
                                                (0.02, 0.0)], M.granite, at=0.2, plane='top')
        G.kit.moulding(F, 'cornice', sq, [(0.02, -0.04), (-0.02, -0.04), (-0.04, 0.0), (-0.08, 0.03), (-0.08, 0.11),
                                          (0.02, 0.11)], M.granite, at=1.31, plane='top')
        for s in (-1, 1):
            for axis in 'xz':
                if axis == 'z':
                    F.box(f'panel_z{s}', 0.9, 0.7, 0.02, (0, 0.78, s * 0.6), M.granite, bevel=0.01, bevel_segments=1)
                else:
                    F.box(f'panel_x{s}', 0.02, 0.7, 0.9, (s * 0.6, 0.78, 0), M.granite, bevel=0.01, bevel_segments=1)
        # the plaque: brass with a bevelled edge, three lines of raised lettering
        F.box('plaque', 0.5, 0.28, 0.02, (0, 0.85, 0.615), M.brass, bevel=0.008, bevel_segments=2)
        for k, w in enumerate((0.36, 0.28, 0.2)):
            F.box(f'plaque_text{k}', w, 0.03 if k == 0 else 0.018, 0.006, (0, 0.92 - 0.065 * k, 0.626), M.iron)
        # the figure on a rough bronze base: shoes, a long coat falling in folds, the body, a collar and lapels
        base = F.box('base', 0.7, 0.05, 0.6, (0, y + 0.025, 0), bronze, bevel=0.015)
        F.box('shoe_left', 0.12, 0.08, 0.26, (-0.1, y + 0.09, 0.06), bronze, bevel=0.03)
        F.box('shoe_right', 0.12, 0.08, 0.26, (0.12, y + 0.09, 0.02), bronze, bevel=0.03)
        bx, by, _ = F.at(0, 0, 0)
        coat = [(0.32, 0.26, 0.08), (0.31, 0.25, 0.2), (0.285, 0.23, 0.42), (0.255, 0.2, 0.68), (0.225, 0.175, 0.9),
                (0.205, 0.16, 1.05), (0.195, 0.15, 1.15)]
        rings = []
        for i, (rx, rz, h) in enumerate(coat):
            fold = 0.11 * (1 - h / 1.15)               # the folds deepest at the hem, gone at the waist
            ring = _lobed(1.0, y + h, 9, fold, per=3)
            rings.append([(bx + p[0] * rx, by + p[1] * rz, p[2]) for p in ring])
        G.loft('coat', rings, material=bronze)
        body = [(0.19, 0.145, 1.1), (0.205, 0.15, 1.3), (0.235, 0.16, 1.46), (0.22, 0.15, 1.54), (0.16, 0.12, 1.6),
                (0.07, 0.07, 1.63)]
        G.loft('body', [G.ring(rx, rz, y + h, 18, square=2.4, at=(bx, by)) for rx, rz, h in body], material=bronze)
        F.extrude('lapel_l', [(0.0, -0.18), (0.07, 0.0), (0.06, 0.12), (0.0, 0.02)], 0.03, (-0.01, y + 1.42, 0.13), bronze,
                  bevel=0.006, bevel_segments=1, rot=(-12, 0, 0))
        F.extrude('lapel_r', [(0.0, -0.18), (0.0, 0.02), (-0.06, 0.12), (-0.07, 0.0)], 0.03, (0.01, y + 1.42, 0.13), bronze,
                  bevel=0.006, bevel_segments=1, rot=(-12, 0, 0))
        G.revolve('collar', [(0, 1.6), (0.09, 1.6), (0.085, 1.66), (0.06, 1.68), (0, 1.68)], at=F.at(0, y, 0),
                  material=bronze, segments=12)
        for k, h in enumerate((1.18, 1.3, 1.42)):      # the coat's buttons
            F.sphere(f'button{k}', 0.016, (0, y + h, 0.152 + 0.012 * k), bronze, segments=6, rings=4)
        F.cyl('neck', 0.055, 0.12, (0, y + 1.66, 0), bronze, verts=10)
        F.sphere('head', 0.12, (0, y + 1.82, 0.01), bronze, segments=16, rings=10, scale=(0.95, 1.05, 1.15))
        F.sphere('hair', 0.125, (0, y + 1.85, -0.02), bronze, segments=16, rings=8, scale=(1.0, 1.0, 0.95))
        F.sphere('bun', 0.07, (0, y + 1.9, -0.13), bronze, segments=10, rings=6)
        F.sphere('nose', 0.02, (0, y + 1.81, 0.13), bronze, segments=6, rings=4)
        F.sphere('chin', 0.05, (0, y + 1.74, 0.07), bronze, segments=8, rings=5, scale=(1.1, 0.8, 0.9))
        # the right arm points up and away, one sleeve from the shoulder to a turned-back cuff; the left holds a book
        # to her side
        F.sweep('arm_right', [(0.2, y + 1.52, 0), (0.31, y + 1.57, 0.08), (0.4, y + 1.62, 0.16), (0.5, y + 1.71, 0.23),
                              (0.59, y + 1.79, 0.29)], 0.056, bronze, verts=10, resolution=2, scale=[1.0, 0.92, 0.85, 0.8, 0.85])
        F.sphere('hand_right', 0.042, (0.63, y + 1.83, 0.32), bronze, segments=10, rings=6, scale=(1, 0.8, 1.2))
        F.sweep('finger', [(0.63, y + 1.83, 0.32), (0.69, y + 1.87, 0.37)], 0.013, bronze, verts=6)
        F.sweep('arm_left', [(-0.21, y + 1.52, 0), (-0.25, y + 1.36, 0.02), (-0.27, y + 1.2, 0.04), (-0.24, y + 1.16, 0.12),
                             (-0.21, y + 1.14, 0.18)], 0.056, bronze, verts=10, resolution=2, scale=[1.0, 0.9, 0.85, 0.82, 0.85])
        F.sphere('hand_left', 0.04, (-0.2, y + 1.14, 0.2), bronze, segments=8, rings=5)
        F.box('book', 0.06, 0.26, 0.2, (-0.25, y + 1.12, 0.16), bronze, bevel=0.01)
        F.box('book_pages', 0.064, 0.24, 0.18, (-0.25, y + 1.12, 0.165), bronze)
        F.sweep('cuff_right', [(0.57, y + 1.775, 0.278), (0.605, y + 1.805, 0.302)], 0.05, bronze, verts=10)
        F.sweep('cuff_left', [(-0.215, y + 1.145, 0.16), (-0.205, y + 1.14, 0.19)], 0.05, bronze, verts=10)
    G.join_all('townStatue')
    G.settle()
    return G.meshes()


FAR = 0.25           # E7: they stand on the town's lots, seen from afar


def _far(build, ratio=FAR):
    """A far version that keeps its feet: as gamekit's `far=ratio` (the model at rest, still, without the water
    hidden at rest, joined and decimated to `ratio`), but the faces on the floor are left out of the decimation,
    which otherwise lifted the drinking fountain's slab 4 cm off the ground and pushed it out 7 cm a side; the
    vertices that sit on one another (turned parts' ends meeting on their axis, a loft's seams) are merged first,
    as decimating them unmerged tore the fountains' pillars and copings into spikes; and the outermost faces are
    kept, so its box stays the model's."""
    def far():
        build()
        bpy.context.scene.frame_set(1)
        for o in G.meshes():
            o.animation_data_clear()
            if min(abs(k) for k in o.scale) < 0.01:
                bpy.data.objects.remove(o, do_unlink=True)
        obj = G.join(G.meshes(), f'{build.__name__}_far', origin=(0, 0, 0))
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        bm.to_mesh(obj.data)
        bm.free()
        lo = [min(v.co[i] for v in obj.data.vertices) for i in range(3)]
        hi = [max(v.co[i] for v in obj.data.vertices) for i in range(3)]

        def free(v):
            return v.co.z > FAR_FEET and all(lo[i] + 0.003 < v.co[i] < hi[i] - 0.003 for i in range(3))
        keep = obj.vertex_groups.new(name='decimate')
        keep.add([v.index for v in obj.data.vertices if free(v)], 1.0, 'REPLACE')
        decimate = obj.modifiers.new('far', 'DECIMATE')
        decimate.ratio = ratio
        decimate.vertex_group = 'decimate'
        return [obj]
    return far


FAR_FEET = 0.1       # below this height (m) a far version keeps every vertex

FAMILY = G.Family(
    'street_furniture', PALETTES,
    [G.Model('streetLamp', streetLamp, size=(0.657, 4.24, 0.467), key='model.streetLamp', far=FAR,
             paint=[('body', 'Lamp post')]),
     G.Model('parkBench', parkBench, size=(2.5, 0.907, 0.572), key='model.parkBench', centre=(0, -0.036), far=FAR,
             paint=[('body', 'Frame')]),
     G.Model('drinkingFountain', drinkingFountain, size=(0.56, 1.093, 0.56), key='model.drinkingFountain', far=_far(drinkingFountain),
             motions={'uses': {'drink': 'water_on'}}, paint=[('body', 'Pillar')]),
     G.Model('litterBin', litterBin, size=(0.474, 0.97, 0.582), key='model.litterBin', centre=(0, -0.034), far=FAR,
             paint=[('body', 'Bin')]),
     G.Model('busStop', busStop, size=(2.98, 2.89, 1.205), key='model.busStop', far=FAR,
             paint=[('body', 'Frame')]),
     G.Model('postBox', postBox, size=(0.5, 1.529, 0.524), key='model.postBox', far=FAR,
             motions={'uses': {'post': 'flap'}, 'hold': 1}, paint=[('body', 'Box')]),
     G.Model('noticeBoard', noticeBoard, size=(1.02, 2.065, 0.347), key='model.noticeBoard', centre=(0, -0.05), far=FAR,
             paint=[('body', 'Frame')]),
     G.Model('phoneBooth', phoneBooth, size=(0.952, 2.7, 0.952), key='model.phoneBooth', far=FAR,
             motions={'uses': {'call': {'clip': 'door_open', 'delay': 0.3, 'hold': 2.4, 'out': 1.6}}},
             paint=[('body', 'Kiosk')]),
     G.Model('fountainTown', fountainTown, size=(2.96, 2.473, 2.94), key='model.fountainTown', far=_far(fountainTown),
             motions={'idle': 'water_run'}, paint=[('body', 'Stone')]),
     G.Model('townStatue', townStatue, size=(1.8, 3.39, 1.8), key='model.townStatue', far=_far(townStatue),
             paint=[('body', 'Statue')])],
    pack='street_furniture',
    comment='Blender-built street furniture (tools/art/blender/families/street_furniture.py): the street '
            'lamp, the park bench, the drinking fountain, the litter bin, the bus shelter, the post box, '
            'the notice board, the phone box, the town fountain and the founder\'s statue. Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw '
            'call per finish group. Each has a far version for the town overview. The lamp\'s lantern and '
            'the phone box\'s signs glow at night; the drinking fountain\'s water arcs for `drink` '
            '(`water_on`), the post box\'s flap swings in for `post` (`flap`), the phone box\'s door opens '
            'for `call` (`door_open`) and the town fountain\'s water runs near the camera (`water_run`, '
            'ambient).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
