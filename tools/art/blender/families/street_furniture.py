# The street furniture (CM2): the street lamp (`model.streetLamp`), the long park bench (`model.parkBench`), the
# drinking fountain (`model.drinkingFountain`), the litter bin (`model.litterBin`), the bus shelter
# (`model.busStop`), the post box (`model.postBox`), the notice board (`model.noticeBoard`), the phone box
# (`model.phoneBooth`), the town fountain (`model.fountainTown`) and the founder's statue (`model.townStatue`). Notes:
# street_furniture.md.
#
# Each is built in game axes (`F`) where the game places it, at the content's heights (the bench's and the
# shelter's seats, the fountain's rim, the lamp's light), and each has a far version (`far=0.25`): they stand on
# the town's lots and are seen from afar. The lamp's lantern glows at night; the drinking fountain's arc of water
# grows while someone drinks (`water_on`); the post box's flap swings in for a card (`flap`); the phone box's
# door opens and shuts behind the caller (`door_open`); the town fountain's water runs all the time near the
# camera (`water_run`, ambient). The rest are still.
#
# Run:  cargo art street_furniture [--only postBox] [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
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


def _tube(F, name, pts, r, material, verts=10):
    """A bent tube through game points `pts`: rods joined by balls (the drinking fountain's arc)."""
    b = [F.at(*p) for p in pts]
    for i in range(len(b) - 1):
        G.rod(f'{name}_{i}', b[i], b[i + 1], r, material=material, verts=verts)
    for i, p in enumerate(pts[1:-1], 1):
        F.sphere(f'{name}_j{i}', r, p, material, segments=verts, rings=5)


# --- the street lamp: a cast-iron post with a square lantern ------------------------------------------------
LAMP_LIGHT = 3.5     # the content's `light.height`: the lantern's panes run from 3.46 to 3.88 m


def streetLamp():
    M = _palette(body=('#2B302E', 'metal'))
    F = G.GameFrame()
    iron = M.body
    with G.new_parts():
        G.revolve('base', [(0, 0), (0.19, 0), (0.19, 0.04), (0.165, 0.07), (0.155, 0.17), (0.115, 0.21),
                           (0.085, 0.52), (0.07, 0.58), (0, 0.58)], material=iron, segments=12)
        F.cyl('collar', 0.07, 0.05, (0, 0.6, 0), iron, verts=12)
        F.cyl('pole', 0.045, 2.75, (0, 0.55 + 2.75 / 2, 0), iron, verts=10)
        F.cyl('collar2', 0.062, 0.05, (0, 2.95, 0), iron, verts=12)
        # the ladder bar a lamplighter leant on
        G.rod('ladder_bar', F.at(-0.29, 3.12, 0), F.at(0.29, 3.12, 0), 0.016, material=iron, verts=8)
        for s in (-1, 1):
            F.sphere(f'ladder_ball{s}', 0.03, (s * 0.3, 3.12, 0), iron, segments=8, rings=5)
        F.cone('cup', 0.05, 0.14, 0.16, (0, 3.38, 0), iron, verts=12)
        # the lantern: four frosted panes, wider at the top, in an iron frame under a pyramid roof
        sq = 2 ** 0.5
        F.cone('panes', 0.13 * sq, 0.19 * sq, 0.42, (0, 3.67, 0), M.lantern, verts=4, rot=(0, 0, 45), smooth=False)
        F.cone('lantern_floor', 0.15 * sq, 0.15 * sq, 0.03, (0, 3.47, 0), iron, verts=4, rot=(0, 0, 45), smooth=False)
        F.cone('lantern_top', 0.205 * sq, 0.21 * sq, 0.035, (0, 3.895, 0), iron, verts=4, rot=(0, 0, 45), smooth=False)
        for k in range(4):
            a = 45 + 90 * k
            lo, hi = 0.135 * sq, 0.195 * sq
            G.rod(f'frame{k}', F.at(lo * G.cosd(a), 3.46, lo * G.sind(a)), F.at(hi * G.cosd(a), 3.88, hi * G.sind(a)),
                  0.014, material=iron, verts=6)
        F.cone('roof', 0.33, 0.04, 0.22, (0, 4.02, 0), iron, verts=4, rot=(0, 0, 45), smooth=False)
        F.cyl('vent', 0.05, 0.05, (0, 4.15, 0), iron, verts=10)
        F.sphere('finial', 0.04, (0, 4.2, 0), iron, segments=10, rings=6)
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
    with G.new_parts():
        for x in BENCH_LEGS:
            G.rod(f'leg_front{x}', F.at(x, 0.0, 0.2), F.at(x, 0.42, 0.18), 0.024, material=frame, verts=8)
            G.rod(f'leg_back{x}', F.at(x, 0.0, -0.25), F.at(x, 0.42, -0.2), 0.024, material=frame, verts=8)
            G.rod(f'upright{x}', F.at(x, 0.42, -0.2), F.at(x, 0.9, -0.3), 0.022, material=frame, verts=8)
            G.rod(f'rail{x}', F.at(x, 0.4, -0.22), F.at(x, 0.4, 0.2), 0.02, material=frame, verts=8)
            for z in (0.2, -0.25):
                F.box(f'foot{x}{z}', 0.06, 0.03, 0.1, (x, 0.015, z), frame, bevel=0.008)
        for s in (-1, 1):
            x = s * BENCH_LEGS[2]
            G.rod(f'arm{s}', F.at(x, 0.66, -0.25), F.at(x, 0.66, 0.2), 0.022, material=frame, verts=8)
            G.rod(f'arm_post{s}', F.at(x, 0.4, 0.16), F.at(x, 0.66, 0.19), 0.02, material=frame, verts=8)
            F.sphere(f'arm_scroll{s}', 0.032, (x, 0.66, 0.21), frame, segments=10, rings=6)
        for k, z in enumerate((-0.17, -0.085, 0.0, 0.085, 0.17)):
            F.box(f'slat{k}', BENCH_LONG, 0.035, 0.072, (0, SEAT - 0.0175, z), M.wood, bevel=0.008)
        for k, y in enumerate((0.6, 0.72, 0.84)):
            z = -0.2 - (y - 0.42) * 0.1 / 0.48 + 0.03
            F.box(f'back{k}', BENCH_LONG, 0.085, 0.03, (0, y, z), M.wood, bevel=0.008, rot=(-12, 0, 0))
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
        G.revolve('pillar', [(0, 0.07), (0.2, 0.07), (0.2, 0.12), (0.165, 0.15), (0.15, 0.2), (0.135, 0.78),
                             (0.15, 0.83), (0.19, 0.88), (0, 0.88)], material=M.body, segments=16)
        G.revolve('bowl', [(0, 0.86), (0.16, 0.86), (0.215, 0.93), (0.225, 0.985), (0.2, 1.0), (0.185, 0.97),
                           (0.1, 0.94), (0, 0.935)], material=M.body, segments=24)
        F.cyl('drain', 0.03, 0.006, (0, 0.937, 0.02), M.chrome, verts=10)
        # the bubbler: a short chrome post at the back of the bowl, its mouth turned to the front
        F.cyl('bubbler', 0.028, 0.11, (0, 0.995, -0.12), M.chrome, verts=12)
        G.rod('bubbler_neck', F.at(0, 1.05, -0.12), F.at(0, 1.075, -0.065), 0.02, material=M.chrome, verts=10)
        F.sphere('bubbler_cap', 0.028, (0, 1.05, -0.12), M.chrome, segments=10, rings=6)
        # the foot pedal at the front of the pillar
        F.box('pedal', 0.18, 0.04, 0.08, (0, 0.12, 0.2), M.chrome, bevel=0.01)
    G.join(still, 'drinkingFountain', origin=(0, 0, 0))
    # the arc of water from the mouth into the bowl, its origin at the mouth, hidden (scaled to nothing) at rest
    with G.new_parts() as arc:
        _tube(F, 'arc', [TIP, (0, 1.115, -0.03), (0, 1.13, 0.005), (0, 1.115, 0.045), (0, 1.065, 0.08), (0, 0.985, 0.1)],
              0.013, M.spray)
    water = G.hinged(G.join(arc, 'water'), F.at(*TIP))
    water.scale = (0.001, 0.001, 0.001)
    G.settle()
    for axis in 'XYZ':
        G.grow(water, 'water_on', seconds=0.35, axis=axis)
    return G.meshes()


# --- the litter bin: a slatted green bin on a post ----------------------------------------------------------
def litterBin():
    M = _palette(body=('#3C6B3A', 'metal'))
    F = G.GameFrame()
    z = 0.02
    with G.new_parts():
        F.box('post', 0.07, 0.95, 0.07, (0, 0.475, -0.28), M.iron, bevel=0.008)
        F.cyl('post_cap', 0.045, 0.02, (0, 0.96, -0.28), M.iron, verts=8)
        for y in (0.3, 0.68):
            F.box(f'strap{y}', 0.06, 0.04, 0.1, (0, y, -0.225), M.iron, bevel=0.006)
        G.revolve('bin', [(0, 0.18), (0.18, 0.18), (0.2, 0.2), (0.21, 0.78), (0, 0.78)], at=F.at(0, 0, z),
                  material=M.body, segments=16)
        for k in range(8):
            a = 22.5 + 45 * k
            F.box(f'rib{k}', 0.035, 0.56, 0.02, (0.212 * G.sind(a), 0.48, z + 0.212 * G.cosd(a)), M.body,
                  rot=(0, 0, -a), bevel=0.005)
        G.torus('rim', 0.215, 0.022, at=F.at(0, 0.79, z), material=M.iron, major_segments=16, minor_segments=6)
        F.cyl('opening', 0.195, 0.01, (0, 0.785, z), M.dark, verts=16)
        F.box('label', 0.18, 0.08, 0.02, (0, 0.62, z + 0.22), M.paper, bevel=0.006)
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
        for x in (-1.42, 1.42):
            for z in (back, front):
                F.box(f'post{x}{z}', 0.07, 2.36, 0.07, (x, 1.18, z), frame, bevel=0.01)
        F.box('roof', 2.98, 0.06, deep + 0.1, (0, 2.39, mid), frame, bevel=0.015)
        F.box('fascia', 2.98, 0.12, 0.03, (0, 2.38, front + 0.035), frame, bevel=0.008)
        F.box('roof_top', 2.9, 0.04, deep + 0.02, (0, 2.44, mid - 0.01), frame, bevel=0.015)
        # the glazing: the back and the right-hand end, in rails
        F.box('glass_back', 2.78, 1.9, 0.015, (0, 1.27, back), M.glass)
        F.box('glass_side', 0.015, 1.9, deep - 0.08, (1.42, 1.27, mid), M.glass)
        for y in (0.3, 2.24):
            F.box(f'rail_back{y}', 2.8, 0.05, 0.05, (0, y, back), frame, bevel=0.008)
            F.box(f'rail_side{y}', 0.05, 0.05, deep - 0.06, (1.42, y, mid), frame, bevel=0.008)
        # the bench: two slats on brackets from the back rail
        for k, z in enumerate((SHELTER_BENCH + 0.08, SHELTER_BENCH - 0.08)):
            F.box(f'slat{k}', 2.5, 0.035, 0.15, (0, SHELTER_SEAT - 0.0175, z), M.wood, bevel=0.008)
        for x in (-1.0, 0.0, 1.0):
            F.box(f'bracket{x}', 0.05, 0.04, 0.34, (x, 0.42, SHELTER_BENCH - 0.05), frame, bevel=0.006)
            F.box(f'bracket_leg{x}', 0.05, 0.42, 0.05, (x, 0.21, SHELTER_BENCH - 0.18), frame, bevel=0.006)
        # the timetable on the back glass
        F.box('timetable_frame', 0.56, 0.78, 0.03, (-0.95, 1.45, back + 0.02), frame, bevel=0.008)
        F.box('timetable', 0.5, 0.68, 0.01, (-0.95, 1.43, back + 0.038), M.paper)
        F.box('timetable_head', 0.5, 0.08, 0.012, (-0.95, 1.73, back + 0.04), M.blue)
        for k in range(6):
            F.box(f'times{k}', 0.4, 0.014, 0.004, (-0.95, 1.6 - 0.08 * k, back + 0.044), M.dark)
        # the stop's flag on the roof, at the front right
        F.box('flag_pole', 0.05, 0.2, 0.05, (1.18, 2.53, front - 0.02), frame)
        F.box('flag', 0.6, 0.34, 0.04, (1.18, 2.72, front - 0.02), M.blue, bevel=0.015)
        F.box('flag_face', 0.44, 0.2, 0.046, (1.18, 2.72, front - 0.02), M.paper, bevel=0.01)
        F.box('flag_bus', 0.22, 0.09, 0.05, (1.18, 2.73, front - 0.02), M.blue, bevel=0.012)
    G.join_all('busStop')
    G.settle(centre=False)
    return G.meshes()


# --- the post box: a red pillar box; its slot's flap swings in for a card ------------------------------------
SLOT_Y = 1.16        # the flap's hinge (its top edge), game height
SLOT_Z = 0.262       # the flap's face, game depth


def postBox():
    M = _palette(body=('#B8322B', 'gloss'))
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('plinth', 0.24, 0.08, (0, 0.04, 0), M.dark, verts=24)
        F.cyl('pillar', 0.21, 1.24, (0, 0.7, 0), M.body, verts=28)
        F.cyl('band', 0.222, 0.04, (0, 0.24, 0), M.body, verts=28)
        F.cyl('cornice', 0.25, 0.08, (0, 1.36, 0), M.body, verts=28, bevel=0.01)
        F.sphere('dome', 0.235, (0, 1.4, 0), M.body, segments=28, rings=12, scale=(1, 1, 0.55))
        # the slot's housing, proud of the pillar: a hood, a sill and cheeks round a dark opening
        F.box('hood', 0.29, 0.03, 0.09, (0, SLOT_Y + 0.025, 0.225), M.body, bevel=0.008)
        F.box('sill', 0.27, 0.025, 0.07, (0, SLOT_Y - 0.07, 0.23), M.body, bevel=0.006)
        for s in (-1, 1):
            F.box(f'cheek{s}', 0.025, 0.1, 0.07, (s * 0.1225, SLOT_Y - 0.02, 0.23), M.body, bevel=0.006)
        F.box('opening', 0.22, 0.08, 0.02, (0, SLOT_Y - 0.025, 0.21), M.dark)
        F.box('times', 0.15, 0.12, 0.012, (0, 0.86, 0.208), M.paper, bevel=0.004)
        F.box('crest', 0.09, 0.06, 0.012, (0, 1.26, 0.208), M.brass, bevel=0.004)
    G.join(still, 'postBox', origin=(0, 0, 0))
    with G.new_parts() as flap:
        F.box('flap', 0.21, 0.05, 0.008, (0, SLOT_Y - 0.025, SLOT_Z), M.body, bevel=0.003)
        F.box('flap_lip', 0.12, 0.008, 0.012, (0, SLOT_Y - 0.045, SLOT_Z + 0.006), M.body, bevel=0.002)
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
        F.box('board', 0.84, 0.72, 0.05, (0, 1.36, z), M.cork, bevel=0.005)
        F.box('frame_top', 0.9, 0.05, 0.08, (0, 1.74, z + 0.01), M.body, bevel=0.01)
        F.box('frame_bottom', 0.9, 0.05, 0.08, (0, 0.98, z + 0.01), M.body, bevel=0.01)
        for s in (-1, 1):
            F.box(f'roof{s}', 1.0, 0.03, 0.2, (0, 2.0, z + s * 0.08), M.body, bevel=0.005, rot=(s * 28, 0, 0))
        F.box('ridge', 1.02, 0.03, 0.04, (0, 2.05, z), M.body, bevel=0.008)
        notes = [(-0.27, 1.55, 0.18, 0.22, M.note_white), (-0.05, 1.5, 0.16, 0.2, M.note_yellow),
                 (0.2, 1.56, 0.2, 0.16, M.note_pink), (-0.25, 1.2, 0.16, 0.22, M.note_blue),
                 (0.0, 1.22, 0.2, 0.18, M.note_white), (0.25, 1.24, 0.16, 0.2, M.note_green)]
        for k, (x, y, w, h, m) in enumerate(notes):
            F.box(f'note{k}', w, h, 0.006, (x, y, z + 0.028), m, rot=(0, (k % 3 - 1) * 3, 0))
            F.sphere(f'pin{k}', 0.014, (x, y + h / 2 - 0.025, z + 0.034), M.pin, segments=8, rings=5)
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


def phoneBooth():
    import math
    M = _palette(body=('#B8322B', 'gloss'))
    F = G.GameFrame()
    red = M.body
    with G.new_parts() as still:
        F.box('plinth', 0.9, 0.12, 0.9, (0, 0.06, 0), red, bevel=0.01)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(f'post{sx}{sz}', 0.12, 2.1, 0.12, (sx * KIOSK, 1.17, sz * KIOSK), red, bevel=0.012)
        # the band at the top, its TELEPHONE signs (lit at night), the stepped roof and its dome
        F.box('band', 0.92, 0.3, 0.92, (0, 2.36, 0), red, bevel=0.015)
        F.box('cornice', 0.95, 0.05, 0.95, (0, 2.535, 0), red, bevel=0.012)
        F.sphere('dome', 0.42, (0, 2.53, 0), red, segments=24, rings=12, scale=(1, 1, 0.33))
        F.sphere('crown', 0.04, (0, 2.66, 0), red, segments=10, rings=6)
        # the back, the left and the right: a kick panel below each window; the back's window frames the phone
        for k, (rot, ox, oz) in enumerate(((math.pi, 0, -1), (math.pi / 2, 1, 0), (-math.pi / 2, -1, 0))):
            def at(x, y, d, ox=ox, oz=oz):
                # a side's (across, height, out) point: on the face at (ox, oz) * FACE
                if oz:
                    return (x * -oz, y, oz * (FACE + d))
                return (0 + ox * (FACE + d), y, x * ox)
            _glazing(F, f'side{k}', M.glass, red, lambda x, y, d, at=at: at(x, y, d), rot)
            kx, _, kz = at(0, 0, 0)
            F.box(f'kick{k}', 0.68 if oz else 0.04, 0.46, 0.04 if oz else 0.68, (kx, 0.35, kz), red, bevel=0.006)
        for k, (x, z, w, d) in enumerate(((0, 0.47, 0.6, 0.012), (0, -0.47, 0.6, 0.012),
                                          (0.47, 0, 0.012, 0.6), (-0.47, 0, 0.012, 0.6))):
            F.box(f'sign{k}', w, 0.1, d, (x, 2.38, z), M.sign)
        # the phone on a panel at the back, a shelf under it
        F.box('phone_panel', 0.4, 0.5, 0.02, (0, 1.35, -0.39), red, bevel=0.006)
        F.box('phone', 0.16, 0.24, 0.1, (0, 1.38, -0.33), M.dark, bevel=0.012)
        F.box('shelf', 0.4, 0.03, 0.18, (0, 1.06, -0.3), M.dark, bevel=0.006)
    G.join(still, 'phoneBooth', origin=(0, 0, 0))
    # the door: its frame, kick panel, window and handle, hinged on its edge at the game's -x (on the right as
    # you face it in the game)
    with G.new_parts() as door:
        F.box('door_kick', DOOR_W, 0.46, 0.04, (0, 0.35, FACE), red, bevel=0.006)
        for s in (-1, 1):
            F.box(f'door_stile{s}', 0.04, 1.62, 0.04, (s * (DOOR_W / 2 - 0.02), 1.37, FACE), red, bevel=0.006)
        F.box('door_head', DOOR_W, 0.06, 0.04, (0, 2.17, FACE), red, bevel=0.006)
        _glazing(F, 'door', M.glass, red, lambda x, y, d: (x, y, FACE + d), 0)
        F.box('door_handle', 0.025, 0.14, 0.03, (DOOR_W / 2 - 0.07, 1.15, FACE + 0.04), M.chrome, bevel=0.008)
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


def fountainTown():
    M = _palette(body=('#C9C2B4', 'matte'))
    F = G.GameFrame()
    stone = M.body
    with G.new_parts() as still:
        F.box('slab', 2.9, 0.06, 2.9, (0, 0.03, 0), M.granite, bevel=0.01)
        for s in (-1, 1):
            F.box(f'wall_fb{s}', 2.9, 0.4, 0.24, (0, 0.26, s * 1.33), stone, bevel=0.015)
            F.box(f'wall_lr{s}', 0.24, 0.4, 2.42, (s * 1.33, 0.26, 0), stone, bevel=0.015)
            F.box(f'cap_fb{s}', 2.96, 0.05, 0.3, (0, RIM - 0.025, s * 1.32), stone, bevel=0.015)
            F.box(f'cap_lr{s}', 0.3, 0.05, 2.36, (s * 1.32, RIM - 0.025, 0), stone, bevel=0.015)
        F.box('basin_water', 2.42, 0.02, 2.42, (0, WATER_Y - 0.01, 0), M.water)
        # the fountain: a foot, a column, the wide lower bowl, a stem, the small top bowl and a knob
        G.revolve('foot', [(0, 0.3), (0.36, 0.3), (0.34, 0.4), (0.24, 0.52), (0.2, 0.66), (0, 0.66)],
                  material=stone, segments=20)
        G.revolve('column', [(0, 0.64), (0.14, 0.64), (0.12, 0.72), (0.12, 1.14), (0.16, 1.22), (0, 1.22)],
                  material=stone, segments=16)
        G.revolve('bowl', [(0, 1.18), (0.2, 1.2), (0.5, 1.36), (0.6, 1.44), (0.6, 1.48), (0.55, 1.49),
                           (0.5, 1.44), (0, 1.42)], material=stone, segments=24)
        F.cyl('bowl_water', 0.53, 0.02, (0, BOWL_Y, 0), M.water, verts=24)
        G.revolve('stem', [(0, 1.44), (0.1, 1.44), (0.07, 1.52), (0.06, 1.82), (0.09, 1.88), (0, 1.88)],
                  material=stone, segments=14)
        G.revolve('top_bowl', [(0, 1.84), (0.08, 1.85), (0.22, 1.94), (0.24, 1.99), (0.21, 2.0), (0, 1.97)],
                  material=stone, segments=18)
        F.sphere('knob', 0.07, (0, 2.03, 0), stone, segments=12, rings=8, scale=(1, 1, 1.2))
        # the water spilling over the lips: streams curving out of the top bowl into the lower one, and out of
        # the lower one into the basin
        for k in range(6):
            a = 30 + 60 * k
            _tube(F, f'trickle{k}', [(r * G.cosd(a), y, r * G.sind(a)) for r, y in TRICKLE], 0.014, M.spray, verts=6)
        for k in range(8):
            a = 22.5 + 45 * k
            _tube(F, f'stream{k}', [(r * G.cosd(a), y, r * G.sind(a)) for r, y in STREAM], 0.024, M.spray, verts=6)
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
            F.sphere(f'splash{k}', 0.09, (x, WATER_Y, z), M.spray, segments=10, rings=5, scale=(1, 1, 0.35))
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
        F.box('step', 1.8, 0.2, 1.8, (0, 0.1, 0), M.granite, bevel=0.02)
        F.box('die', 1.2, 1.1, 1.2, (0, 0.75, 0), M.granite, bevel=0.02)
        F.box('cornice', 1.36, 0.12, 1.36, (0, 1.36, 0), M.granite, bevel=0.025)
        F.box('plaque', 0.5, 0.28, 0.02, (0, 0.85, 0.605), M.brass, bevel=0.008)
        F.box('plaque_text', 0.36, 0.12, 0.006, (0, 0.85, 0.615), M.iron)
        # the figure: a little bronze base, shoes, a long coat, the body, the head with its bun
        F.box('base', 0.7, 0.05, 0.6, (0, y + 0.025, 0), bronze, bevel=0.015)
        F.box('shoe_left', 0.12, 0.08, 0.26, (-0.1, y + 0.09, 0.06), bronze, bevel=0.03)
        F.box('shoe_right', 0.12, 0.08, 0.26, (0.12, y + 0.09, 0.02), bronze, bevel=0.03)
        G.revolve('coat', [(0, 0.08), (0.32, 0.08), (0.3, 0.3), (0.24, 0.75), (0.2, 1.05), (0.19, 1.15), (0, 1.15)],
                  at=F.at(0, y, 0), material=bronze, segments=18, scale=(1, 0.8, 1))
        G.revolve('body', [(0, 1.1), (0.2, 1.1), (0.21, 1.3), (0.24, 1.48), (0.2, 1.58), (0.07, 1.62), (0, 1.62)],
                  at=F.at(0, y, 0), material=bronze, segments=16, scale=(1, 0.72, 1))
        F.box('lapels', 0.16, 0.3, 0.04, (0, y + 1.36, 0.14), bronze, bevel=0.015)
        F.cyl('neck', 0.055, 0.12, (0, y + 1.66, 0), bronze, verts=10)
        F.sphere('head', 0.12, (0, y + 1.82, 0.01), bronze, segments=14, rings=10, scale=(0.95, 1.05, 1.15))
        F.sphere('hair', 0.125, (0, y + 1.85, -0.02), bronze, segments=14, rings=8, scale=(1.0, 1.0, 0.95))
        F.sphere('bun', 0.07, (0, y + 1.9, -0.13), bronze, segments=10, rings=6)
        F.sphere('nose', 0.02, (0, y + 1.81, 0.13), bronze, segments=6, rings=4)
        # the right arm points up and away; the left holds a book to her side
        G.rod('arm_right', F.at(0.2, y + 1.52, 0), F.at(0.4, y + 1.62, 0.16), 0.055, material=bronze, verts=10)
        G.rod('forearm_right', F.at(0.4, y + 1.62, 0.16), F.at(0.6, y + 1.8, 0.3), 0.045, material=bronze, verts=10)
        F.sphere('elbow_right', 0.05, (0.4, y + 1.62, 0.16), bronze, segments=10, rings=6)
        F.sphere('hand_right', 0.045, (0.63, y + 1.83, 0.32), bronze, segments=10, rings=6)
        G.rod('finger', F.at(0.63, y + 1.83, 0.32), F.at(0.69, y + 1.87, 0.37), 0.014, material=bronze, verts=6)
        G.rod('arm_left', F.at(-0.21, y + 1.52, 0), F.at(-0.27, y + 1.2, 0.04), 0.055, material=bronze, verts=10)
        F.sphere('elbow_left', 0.05, (-0.27, y + 1.2, 0.04), bronze, segments=10, rings=6)
        G.rod('forearm_left', F.at(-0.27, y + 1.2, 0.04), F.at(-0.2, y + 1.14, 0.18), 0.045, material=bronze, verts=10)
        F.box('book', 0.06, 0.26, 0.2, (-0.25, y + 1.12, 0.16), bronze, bevel=0.01)
    G.join_all('townStatue')
    G.settle()
    return G.meshes()


FAR = 0.25           # E7: they stand on the town's lots, seen from afar


def _far(build, ratio=FAR):
    """A far version that keeps its feet: as gamekit's `far=ratio` (the model at rest, still, without the water
    hidden at rest, joined and decimated to `ratio`), but the faces on the floor are left out of the decimation,
    which otherwise lifted the drinking fountain's slab 4 cm off the ground and pushed it out 7 cm a side."""
    def far():
        build()
        bpy.context.scene.frame_set(1)
        for o in G.meshes():
            o.animation_data_clear()
            if min(abs(k) for k in o.scale) < 0.01:
                bpy.data.objects.remove(o, do_unlink=True)
        obj = G.join(G.meshes(), f'{build.__name__}_far', origin=(0, 0, 0))
        keep = obj.vertex_groups.new(name='decimate')
        keep.add([v.index for v in obj.data.vertices if v.co.z > FAR_FEET], 1.0, 'REPLACE')
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
     G.Model('fountainTown', fountainTown, size=(2.96, 2.473, 2.94), key='model.fountainTown', far=FAR,
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
