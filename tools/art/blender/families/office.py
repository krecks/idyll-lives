# The office: the desks (O1: the simple, corner, standing and executive desks, the computer, writing and
# laptop desks and the gaming PC), the desk chairs (O2: the executive, desk and office chairs) and the storage
# and boards (O3: the filing cabinet, the whiteboard and the corkboard). Each replaces the primitive-part
# placeholder of `catalog.py` / `catalog2.py` / `furniture.py`, which stays the fallback if the file fails to
# load. Notes: office.md.
#
# The parts are placed in game axes (`F`, no shift), where each placeholder stands; a desk's top is the
# content's `surface` height (`DT`, 0.75 for the furniture.py desks, 1.05 standing). Backs against the wall
# stop at its face (game depth -0.43, -0.93 two tiles deep). The computer, writing, laptop and gaming desks
# keep their placeholder's chair in the back half (`characters.ts` SEATS: seat 0.44, 17 cm back), facing the
# desk, as residents sit there. Moving parts:
# - the standing and computer desks' keyboard shelf slides out to the resident while they type (`tray_out`);
# - the writing desk's drawer slides out as they sit down to write (`drawer_open`, shut after 2.5 s);
# - the laptop opens from shut to 110 degrees while it's used (`lid_open`);
# - the gaming PC's two case fans spin while a game is played (`fans`, a loop);
# - a desk chair's seat swivels 15 degrees and back as a resident sits down, carrying them (`swivel`, ride);
# - the whiteboard's marker lines grow across it while a resident plans (`scribble`).
# The executive desk's middle drawer and the filing cabinet's top drawer are parts of their own, still: no use
# opens them yet, so a clip can be added once content gives them one.
#
# Run:  python3 -B tools/art/blender/build.py office [--only deskSimple] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    oak=('#CDB08A', 'wood'),          # oak tops: the simple, computer, writing and laptop desks
    oak_panel=('#CDB08A', 'wood'),    # the corner desk's oak panels
    birch=('#E3CFA8', 'wood'),        # the standing desk's top
    walnut=('#5A3825', 'wood'),       # the executive desk's top
    walnut_case=('#5A3825', 'wood'),  # its pedestals and modesty panel
    walnut2=('#6B4630', 'wood'),      # its drawer fronts and plinths
    laminate=('#F1F0EC', 'satin'),    # the corner desk's white top
    drawers=('#F1F0EC', 'satin'),     # painted drawer units and drawer fronts
    frame=('#2B2C2E', 'metal'),       # black steel: desk frames, legs, chair bases and arms
    shelf=('#3A3D41', 'satin'),       # the standing desk's keyboard shelf
    tray=('#3A3D41', 'satin'),        # the computer desk's keyboard tray
    brass=('#B8925A', 'metal'),       # the executive desk's handles
    chrome=('#D0D3D6', 'chrome'),     # gas lifts, the filing cabinet's handles
    plastic=('#2F3236', 'satin'),     # dark plastic: chair shells and hubs, controls, monitor stands
    castor=('#1E1F22', 'satin'),
    inlay=('#2F4A3A', 'satin'),       # the executive desk's leather writing pad
    leather=('#3B2A20', 'satin'),     # the executive chair
    leather2=('#2E2019', 'satin'),    # its channels
    fabric=('#4E6E7A', 'fabric'),     # the desk chair's seat
    mesh=('#3A3D41', 'fabric'),       # its mesh back
    upholstery=('#5F6E7A', 'fabric'), # the office chair
    chair=('#5F6E7A', 'satin'),       # the chairs at the computer and laptop desks
    chair_sage=('#8FA58A', 'satin'),  # the writing desk's chair
    chair_gamer=('#3A3D41', 'satin'), # the gaming chair
    monitor=('#D9DADB', 'satin'),     # the computer desk's monitor
    screen=('#16202B', 'gloss'),
    wallpaper=('#2E5470', 'gloss'),
    keys=('#E8E8E6', 'satin'),        # keyboards and mice
    pad=('#3A3D41', 'matte'),         # mouse pads, the laptop's touchpad and keys
    paper=('#FFFFFF', 'matte'),
    notebook=('#D4A843', 'matte'),
    pen=('#2A2C30', 'satin'),
    mug=('#3F8C8C', 'gloss'),
    pot=('#F5F3EE', 'gloss'),
    soil=('#4A3A2C', 'matte'),
    cactus=('#5E8F4E', 'satin'),
    leaf=('#5E8F4E', 'foliage'),
    lamp=('#2B2C2E', 'metal'),        # desk lamps
    bulb=('#FFF1CF', 'gloss'),
    aluminium=('#C9CBCC', 'metal'),   # the laptop; the whiteboard's frame and tray
    # the gaming PC
    desk=('#2F3236', 'satin'),
    mat=('#3A3D41', 'matte'),
    case=('#22252A', 'satin'),
    accent=('#6E5CE6', 'gloss'),      # its lights: fan rings, the LED strip, backlit keys
    side=('#3B2E6E', 'gloss'),        # the tower's tinted side panel
    screen2=('#3B2E6E', 'gloss'),
    blade=('#1E1F22', 'satin'),       # fan blades
    # the filing cabinet
    steel=('#8E959B', 'metal'),
    label=('#F2F0EA', 'gloss'),
    folder=('#C9B79C', 'matte'),
    # the whiteboard and the corkboard
    board=('#FBFBF9', 'gloss'),
    ink_blue=('#3E8EDE', 'gloss'),
    ink_red=('#D9483B', 'gloss'),
    ink_black=('#2B2C2E', 'gloss'),
    note=('#F2C14E', 'matte'),
    note2=('#F2F0EA', 'matte'),
    note3=('#E4B8AE', 'matte'),
    note4=('#9DC6CF', 'matte'),
    cork=('#C49A6C', 'matte'),
    cork_frame=('#8A5A36', 'wood'),
    pin=('#D9483B', 'gloss'),
)

F = G.GameFrame()
WALL = -0.43          # the wall's face, game depth (one tile deep)
DT = 0.74             # the catalog2 desks' top (catalog2.py DT)
FT = 0.7525           # the furniture.py desks' top (content surface 0.75)
CHAIR_Z = -0.17       # where a desk's own chair stands (characters.ts SEATS fwd)


def _box(name, w, h, d, at, m, **kw):
    return F.box(name, w, h, d, at, m, **kw)


def _rod(name, a, b, r, m, verts=8):
    """A round bar between two game points."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=m, verts=verts)


def _turned(parts, name, pivot, x=0.0, z=0.0):
    """Joins `parts` and turns them about game point `pivot`: `x` degrees about Blender X (negative leans a
    back backwards), `z` about the vertical. The turn is baked in when the part is joined again."""
    obj = G.join(parts, name)
    G.set_origin(obj, F.at(*pivot))
    obj.rotation_euler = (math.radians(x), 0, math.radians(z))
    return obj


def _slab(name, pts, y0, t, material, bevel=0.006):
    """A flat slab `t` thick, its bottom at height `y0`, outlined by game (x, depth) points."""
    ring = [(-x, -z) for x, z in pts]
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(ring, ring[1:] + ring[:1]))
    if area < 0:
        ring.reverse()
    n = len(ring)
    verts = [(x, y, y0) for x, y in ring] + [(x, y, y0 + t) for x, y in ring]
    faces = [tuple(range(n - 1, -1, -1)), tuple(range(n, 2 * n))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    mod = obj.modifiers.new('bevel', 'BEVEL')
    mod.width, mod.segments, mod.limit_method, mod.angle_limit = bevel, 2, 'ANGLE', math.radians(40)
    return obj


def _pull(name, x, y, z, w, m, d=0.014):
    """A bar handle on a drawer front at depth `z`, standing out towards +z (towards -z if `d` < 0)."""
    _box(name, w, 0.012, abs(d), (x, y, z + d / 2), m, bevel=0.004)


# --- desk chairs: a padded seat and back on a gas lift and five castors ------------------------------------
def _chair_base(z, seat_y, frame):
    """The still base: five legs and castors, the hub and the gas lift, up to under the seat."""
    for i in range(5):
        a = math.radians(i * 72 + 18)
        tip = (math.cos(a) * 0.27, 0.05, z + math.sin(a) * 0.27)
        _rod(f'leg{i}', (0, 0.1, z), tip, 0.017, frame)
        _box(f'leg_cap{i}', 0.03, 0.02, 0.03, (tip[0], 0.05, tip[2]), frame, bevel=0.006)
        F.cyl(f'castor{i}', 0.026, 0.022, (tip[0], 0.026, tip[2]), M.castor, rot=(0, 90, 0), verts=12)
    F.cyl('hub', 0.045, 0.06, (0, 0.1, z), frame, verts=16, bevel=0.01)
    F.cyl('lift_sleeve', 0.03, 0.12, (0, 0.18, z), M.plastic, verts=14)
    lift = seat_y - 0.15 - 0.24
    F.cyl('lift', 0.02, lift, (0, 0.24 + lift / 2, z), M.chrome, verts=14)


def _chair_seat(z, seat_y, back_h, seat, back=None, frame=None, arm=None, head=False, channels=None,
                mesh_frame=None, back_z=0.25, lean=9):
    """The swivelling part: the mechanism, the seat, the back on its stem, the arms (and a headrest)."""
    back = back or seat
    frame = frame or M.frame
    arm = arm or frame
    _box('mech', 0.18, 0.05, 0.2, (0, seat_y - 0.125, z), M.plastic, bevel=0.01)
    _box('shell', 0.46, 0.025, 0.44, (0, seat_y - 0.0925, z + 0.01), M.plastic, bevel=0.008)
    _box('seat', 0.5, 0.08, 0.48, (0, seat_y - 0.04, z + 0.01), seat, bevel=0.03, bevel_segments=3)
    _box('stem_h', 0.07, 0.025, back_z - 0.08, (0, seat_y - 0.115, z - (back_z + 0.08) / 2), frame, bevel=0.006)
    _box('stem_v', 0.07, 0.28, 0.025, (0, seat_y + 0.02, z - back_z - 0.01), frame, bevel=0.006)
    # the back, built upright and leant back `lean` degrees about its middle
    yb = seat_y + 0.07 + back_h / 2
    with G.new_parts() as parts:
        if mesh_frame:
            _box('back', 0.44, back_h - 0.04, 0.012, (0, yb, z - back_z + 0.005), back)
            for s in (-1, 1):
                _box(f'back_rail{s}', 0.03, back_h, 0.04, (s * 0.235, yb, z - back_z), mesh_frame, bevel=0.012)
            for k, y in enumerate((yb - back_h / 2 + 0.015, yb + back_h / 2 - 0.015)):
                _box(f'back_bar{k}', 0.47, 0.03, 0.04, (0, y, z - back_z), mesh_frame, bevel=0.012)
        else:
            _box('back', 0.48, back_h, 0.07, (0, yb, z - back_z), back, bevel=0.03, bevel_segments=3)
        if channels:
            for x in (-0.1, 0.1):
                _box(f'channel{x}', 0.014, back_h - 0.16, 0.006, (x, yb, z - back_z + 0.036), channels, bevel=0.002)
        if head:
            _box('headrest', 0.3, 0.14, 0.07, (0, yb + back_h / 2 + 0.1, z - back_z), back, bevel=0.03)
            _box('head_stem', 0.06, 0.08, 0.02, (0, yb + back_h / 2 + 0.03, z - back_z - 0.02), frame, bevel=0.006)
    _turned(parts, 'back', (0, yb, z - back_z), x=-lean)
    for s in (-1, 1):
        _box(f'arm_bracket{s}', 0.06, 0.02, 0.06, (s * 0.255, seat_y - 0.075, z - 0.03), frame, bevel=0.005)
        _box(f'arm_post{s}', 0.028, 0.2, 0.03, (s * 0.275, seat_y + 0.04, z - 0.03), frame, bevel=0.006)
        _box(f'arm_pad{s}', 0.065, 0.03, 0.27, (s * 0.275, seat_y + 0.15, z + 0.0), arm, bevel=0.012)


def _desk_chair(seat, z=CHAIR_Z, seat_y=0.44, back_h=0.5, head=False, channels=None):
    """A desk's own chair (part of the desk, still): in its back half, facing the desk, its back short of
    the wall."""
    _chair_base(z, seat_y, M.frame)
    _chair_seat(z, seat_y, back_h, seat, head=head, channels=channels, back_z=0.19, lean=5 if head else 7)


def _chair(seat_y, back_h, z, seat, back=None, frame=None, arm=None, head=False, channels=None, mesh_frame=None):
    frame = frame or M.frame
    with G.new_parts() as still:
        _chair_base(z, seat_y, frame)
    G.join(still, 'base', origin=(0, 0, 0))
    with G.new_parts() as parts:
        _chair_seat(z, seat_y, back_h, seat, back, frame, arm, head, channels, mesh_frame)
    swivel = G.hinged(G.join(parts, 'seat'), F.at(0, seat_y - 0.15, z))
    G.settle(centre=False)
    G.swing(swivel, 'swivel', 15, seconds=0.5)
    return G.meshes()


def chairExecutive():
    return _chair(0.49, 0.78, 0.05, M.leather, arm=M.leather, channels=M.leather2)


def chairTask():
    return _chair(0.48, 0.5, 0.05, M.fabric, back=M.mesh, mesh_frame=M.plastic)


def officeChair():
    return _chair(0.47, 0.62, 0.0, M.upholstery)


# --- the furniture.py desks: a top at the front of the tile, a frame, the chair behind -----------------------
def _desk_frame(w, legs=None, gap=None, apron=None):
    """Four square legs, side and back aprons, and a front (resident's side) apron with a drawer gap
    (x0, x1) left open."""
    legs, apron = legs or M.frame, apron or M.frame
    for s in (-1, 1):
        for z in (0.06, 0.42):
            _box(f'leg{s}{z}', 0.04, 0.72, 0.04, (s * (w / 2 - 0.04), 0.36, z), legs, bevel=0.005)
        _box(f'apron_side{s}', 0.02, 0.06, 0.32, (s * (w / 2 - 0.04), 0.69, 0.24), apron, bevel=0.003)
    _box('apron_back', w - 0.1, 0.06, 0.02, (0, 0.69, 0.43), apron, bevel=0.003)
    x0, x1 = gap or (0, 0)
    for k, (a, b) in enumerate(((-w / 2 + 0.06, x0), (x1, w / 2 - 0.06))):
        if b - a > 0.01:
            _box(f'apron_front{k}', b - a, 0.06, 0.02, ((a + b) / 2, 0.69, 0.06), apron, bevel=0.003)


def _top(w, d=0.42, z=0.24, m=None, y=FT, t=0.035):
    m = m or M.oak
    _box('top', w, t, d, (0, y - t / 2, z), m, bevel=0.006)


def _desk_lamp(x, z, m=None):
    """An angled desk lamp on the top, its shade turned to the resident's side."""
    m = m or M.lamp
    F.cyl('lamp_base', 0.065, 0.02, (x, FT + 0.01, z), m, verts=20, bevel=0.005)
    _rod('lamp_arm', (x, FT + 0.02, z), (x + 0.04, 1.1, z - 0.05), 0.009, m)
    _rod('lamp_arm2', (x + 0.04, 1.1, z - 0.05), (x + 0.15, 1.06, z - 0.14), 0.008, m)
    F.sphere('lamp_joint', 0.014, (x + 0.04, 1.1, z - 0.05), m, segments=10, rings=6)
    F.cone('lamp_shade', 0.06, 0.022, 0.1, (x + 0.17, 1.03, z - 0.16), m, rot=(35, 0, 0), verts=20)
    F.sphere('lamp_bulb', 0.024, (x + 0.17, 1.005, z - 0.18), M.bulb, segments=10, rings=6)


def _mug(x, z):
    F.cyl('mug', 0.04, 0.095, (x, FT + 0.0475, z), M.mug, verts=18)
    G.torus('mug_handle', 0.025, 0.006, at=F.at(x + 0.045, FT + 0.05, z), material=M.mug, rot=(90, 0, 0),
            major_segments=12, minor_segments=5)


def _drawer_box(name, w, h, d, x, y0, z_front, m):
    """An open-topped drawer box behind a front at depth `z_front`, running back (towards +z)."""
    t = 0.01
    zc = z_front + d / 2
    _box(f'{name}_bottom', w, t, d, (x, y0 + t / 2, zc), m)
    for s in (-1, 1):
        _box(f'{name}_side{s}', t, h, d, (x + s * (w - t) / 2, y0 + h / 2, zc), m)
    _box(f'{name}_back', w, h, t, (x, y0 + h / 2, z_front + d - t / 2), m)


def computerDesk():
    with G.new_parts() as still:
        _top(1.54, d=0.38, z=0.26)
        for s in (-1, 1):
            for z in (0.12, 0.43):
                _box(f'leg{s}{z}', 0.04, 0.72, 0.04, (s * 0.72, 0.36, z), M.frame, bevel=0.005)
            _box(f'foot{s}', 0.04, 0.04, 0.35, (s * 0.72, 0.02, 0.275), M.frame, bevel=0.005)
            _box(f'rail{s}', 0.04, 0.04, 0.35, (s * 0.72, 0.7, 0.275), M.frame, bevel=0.005)
        _box('rail_back', 1.4, 0.04, 0.02, (0, 0.7, 0.435), M.frame, bevel=0.004)
        # the drawer pedestal hanging under the left, its drawers to the resident
        _box('pedestal', 0.36, 0.42, 0.34, (-0.52, 0.48, 0.27), M.drawers, bevel=0.008)
        for k, (y, h) in enumerate(((0.6, 0.12), (0.42, 0.2))):
            _box(f'pedestal_front{k}', 0.34, h, 0.012, (-0.52, y, 0.095), M.drawers, bevel=0.004)
            _pull(f'pedestal_pull{k}', -0.52, y + h / 2 - 0.035, 0.089, 0.1, M.frame, d=-0.012)
        # the runners the keyboard tray hangs from
        for x in (-0.18, 0.5):
            _box(f'runner{x}', 0.015, 0.055, 0.32, (x, 0.69, 0.25), M.frame, bevel=0.003)
        # the monitor, facing the resident; its back to the room
        _box('monitor', 0.62, 0.37, 0.03, (0.05, 1.03, 0.4), M.monitor, bevel=0.01)
        _box('screen', 0.6, 0.345, 0.004, (0.05, 1.035, 0.3835), M.screen, bevel=0.002)
        _box('wallpaper', 0.5, 0.26, 0.002, (0.05, 1.04, 0.381), M.wallpaper)
        _box('monitor_neck', 0.06, 0.22, 0.03, (0.05, 0.86, 0.42), M.monitor, bevel=0.008)
        _box('monitor_foot', 0.22, 0.012, 0.14, (0.05, FT + 0.006, 0.375), M.monitor, bevel=0.005)
        _mug(0.58, 0.3)
        _desk_lamp(-0.6, 0.38)
        F.cyl('pot', 0.045, 0.08, (0.62, FT + 0.04, 0.38), M.pot, verts=16)
        F.cyl('pot_soil', 0.04, 0.005, (0.62, FT + 0.078, 0.38), M.soil, verts=16)
        F.sphere('cactus', 0.035, (0.62, FT + 0.12, 0.38), M.cactus, scale=(1, 1, 1.6), segments=12, rings=8)
        _box('notebook', 0.2, 0.015, 0.26, (-0.3, FT + 0.0075, 0.26), M.notebook, rot=(0, 0, 12), bevel=0.003)
        _desk_chair(M.chair)
    G.join(still, 'computerDesk', origin=(0, 0, 0))
    # the keyboard tray: under the top, sliding 15 cm out to the resident
    with G.new_parts() as parts:
        _box('tray', 0.66, 0.012, 0.3, (0.16, 0.655, 0.25), M.tray, bevel=0.004)
        _box('tray_lip', 0.66, 0.03, 0.012, (0.16, 0.668, 0.106), M.tray, bevel=0.004)
        _box('keyboard', 0.42, 0.018, 0.13, (0.06, 0.67, 0.22), M.keys, bevel=0.005)
        _box('keyboard_keys', 0.39, 0.004, 0.1, (0.06, 0.68, 0.225), M.pad, bevel=0.001)
        _box('mouse_pad', 0.18, 0.003, 0.18, (0.38, 0.6625, 0.22), M.pad, bevel=0.002)
        _box('mouse', 0.06, 0.022, 0.1, (0.38, 0.675, 0.22), M.keys, bevel=0.01)
    tray = G.join(parts, 'tray')
    G.settle(centre=False)
    G.slide(tray, 'tray_out', (0, 0.15, 0), seconds=0.6)
    return G.meshes()


def deskWriting():
    with G.new_parts() as still:
        _top(1.54)
        _desk_frame(1.54, gap=(-0.1, 0.4))
        _box('drawer_rails', 0.52, 0.01, 0.38, (0.15, 0.662, 0.24), M.frame)
        _box('paper', 0.3, 0.003, 0.21, (-0.12, FT + 0.0015, 0.2), M.paper, rot=(0, 0, 8))
        _box('paper2', 0.3, 0.003, 0.21, (-0.1, FT + 0.0045, 0.22), M.paper, rot=(0, 0, -4))
        F.cyl('pen', 0.005, 0.14, (0.08, FT + 0.006, 0.2), M.pen, rot=(90, 0, 85), verts=8)
        for k, m in enumerate((M.notebook, M.mug)):
            _box(f'notebook{k}', 0.2, 0.025, 0.27, (0.45, FT + 0.0125 + 0.025 * k, 0.26), m, rot=(0, 0, 6 * k - 3),
                 bevel=0.004)
        _desk_lamp(-0.57, 0.36)
        _desk_chair(M.chair_sage)
    G.join(still, 'deskWriting', origin=(0, 0, 0))
    # the drawer: a front with a knob and a box of letters and pencils, sliding out to the resident
    with G.new_parts() as parts:
        _box('drawer', 0.5, 0.055, 0.018, (0.15, 0.69, 0.051), M.drawers, bevel=0.004)
        F.cyl('drawer_knob', 0.013, 0.02, (0.15, 0.69, 0.032), M.brass, rot=(90, 0, 0), verts=12, bevel=0.004)
        _drawer_box('drawer_box', 0.48, 0.045, 0.36, 0.15, 0.665, 0.06, M.drawers)
        _box('letters', 0.22, 0.012, 0.15, (0.05, 0.681, 0.2), M.paper, rot=(0, 0, 5))
        for k in range(3):
            F.cyl(f'pencil{k}', 0.004, 0.15, (0.3, 0.68, 0.16 + 0.02 * k), (M.notebook, M.pen, M.mug)[k],
                  rot=(0, 90, 0), verts=6)
    drawer = G.join(parts, 'drawer')
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, 0.22, 0), seconds=0.6)
    return G.meshes()


def deskLaptop():
    with G.new_parts() as still:
        _top(0.9)
        _desk_frame(0.9, gap=(-0.38, -0.02))
        _box('drawer', 0.34, 0.055, 0.016, (-0.2, 0.69, 0.052), M.drawers, bevel=0.004)
        F.cyl('drawer_knob', 0.012, 0.02, (-0.2, 0.69, 0.034), M.frame, rot=(90, 0, 0), verts=12, bevel=0.004)
        _box('drawer_back', 0.36, 0.055, 0.3, (-0.2, 0.69, 0.21), M.drawers)
        # the laptop's base: keys towards the hinge, the touchpad to the resident
        _box('laptop', 0.32, 0.012, 0.22, (0.02, FT + 0.006, 0.2), M.aluminium, bevel=0.003)
        _box('laptop_keys', 0.28, 0.0006, 0.1, (0.02, FT + 0.0123, 0.245), M.pad)
        _box('laptop_pad', 0.1, 0.0006, 0.06, (0.02, FT + 0.0123, 0.14), M.pad)
        _mug(0.32, 0.36)
        _box('notebook', 0.18, 0.012, 0.24, (-0.28, FT + 0.006, 0.28), M.notebook, rot=(0, 0, -10), bevel=0.003)
        F.cyl('pen', 0.005, 0.14, (-0.27, FT + 0.017, 0.27), M.pen, rot=(90, 0, 70), verts=8)
        _desk_chair(M.chair)
    G.join(still, 'deskLaptop', origin=(0, 0, 0))
    # the lid, shut on the base, hinged along the base's far edge
    with G.new_parts() as parts:
        _box('lid', 0.32, 0.007, 0.22, (0.02, FT + 0.0165, 0.2), M.aluminium, bevel=0.003)
        _box('lid_screen', 0.29, 0.0008, 0.19, (0.02, FT + 0.0129, 0.205), M.screen)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0.02, FT + 0.013, 0.31))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', 110, seconds=0.8, axis='X')
    return G.meshes()


def computerGaming():
    accent = M.accent
    with G.new_parts() as still:
        _top(1.7, d=0.44, m=M.desk)
        _desk_frame(1.7, legs=M.desk, apron=M.desk)
        _box('desk_mat', 1.6, 0.004, 0.36, (0, FT + 0.002, 0.22), M.mat, bevel=0.002)
        _box('led_strip', 1.62, 0.01, 0.006, (0, FT - 0.012, 0.463), accent)
        # two monitors on stands, turned in to the resident
        for s, scr in ((-1, M.screen2), (1, M.wallpaper)):
            x = s * 0.36
            with G.new_parts() as parts:
                _box(f'monitor{s}', 0.62, 0.36, 0.03, (x, 1.03, 0.36), M.case, bevel=0.01)
                _box(f'monitor_screen{s}', 0.6, 0.34, 0.004, (x, 1.03, 0.3435), scr, bevel=0.002)
                _box(f'monitor_neck{s}', 0.05, 0.24, 0.03, (x, 0.86, 0.39), M.case, bevel=0.008)
                _box(f'monitor_foot{s}', 0.22, 0.01, 0.15, (x, FT + 0.005, 0.36), M.case, bevel=0.004)
            _turned(parts, f'monitor{s}', (x, 1.0, 0.36), z=-s * 15)
        _box('keyboard', 0.46, 0.02, 0.15, (0, FT + 0.01, 0.17), M.case, bevel=0.005)
        _box('keyboard_keys', 0.44, 0.003, 0.13, (0, FT + 0.0215, 0.17), accent, bevel=0.001)
        _box('mouse', 0.07, 0.025, 0.12, (0.36, FT + 0.0125, 0.17), M.case, bevel=0.012)
        _box('speaker', 0.16, 0.2, 0.14, (-0.66, FT + 0.1, 0.36), M.case, bevel=0.012)
        F.cyl('woofer', 0.045, 0.006, (-0.66, FT + 0.08, 0.288), accent, rot=(90, 0, 0), verts=18)
        # the tower under the desk: a tinted side to the resident, two fans in the front
        _box('tower', 0.24, 0.5, 0.48, (0.62, 0.27, 0.2), M.case, bevel=0.01)
        for s in (-1, 1):
            _box(f'tower_foot{s}', 0.2, 0.02, 0.04, (0.62, 0.01, 0.2 + s * 0.19), M.plastic, bevel=0.004)
        _box('tower_side', 0.004, 0.4, 0.38, (0.499, 0.28, 0.2), M.side)
        _box('tower_strip', 0.012, 0.42, 0.004, (0.53, 0.27, 0.441), accent)
        for k, y in enumerate((0.38, 0.17)):
            G.torus(f'fan_ring{k}', 0.074, 0.008, at=F.at(0.62, y, 0.442), material=accent, rot=(90, 0, 0),
                    major_segments=24, minor_segments=6)
        _desk_chair(M.chair_gamer, back_h=0.62, head=True, channels=accent)
    G.join(still, 'computerGaming', origin=(0, 0, 0))
    fans = []
    for k, y in enumerate((0.38, 0.17)):
        with G.new_parts() as parts:
            F.cyl(f'fan_hub{k}', 0.024, 0.01, (0.62, y, 0.446), M.blade, rot=(90, 0, 0), verts=16)
            F.cyl(f'fan_badge{k}', 0.012, 0.012, (0.62, y, 0.447), accent, rot=(90, 0, 0), verts=12)
            for b in range(7):
                a = math.radians(b * 360 / 7)
                at = F.at(0.62, y, 0.445)
                G.cube(f'blade{k}{b}', (0.05, 0.003, 0.026),
                       at=(at[0] + 0.044 * math.cos(a), at[1], at[2] - 0.044 * math.sin(a)),
                       material=M.blade, rot=(25, math.degrees(a), 0))
        fans.append(G.hinged(G.join(parts, f'fan{k}'), F.at(0.62, y, 0.446)))
    G.settle(centre=False)
    for fan in fans:
        G.spin(fan, 'fans', turns=1, seconds=1.0, axis='Y')
    return G.meshes()


# --- the catalog2 desks --------------------------------------------------------------------------------------
def deskSimple():
    with G.new_parts():
        _top(1.5, d=0.69, z=-0.085, y=DT)
        # a drawer box hanging under the right
        _box('drawer_box', 0.42, 0.12, 0.6, (0.45, DT - 0.095, -0.12), M.drawers, bevel=0.006)
        _box('drawer_front', 0.4, 0.1, 0.012, (0.45, DT - 0.095, 0.186), M.drawers, bevel=0.004)
        _pull('drawer_pull', 0.45, DT - 0.095, 0.192, 0.14, M.frame)
        # hairpin legs: two bars from a plate under the top to one foot
        for x in (-0.68, 0.68):
            for z in (-0.33, 0.17):
                _box(f'plate{x}{z}', 0.06, 0.006, 0.18, (x, DT - 0.038, z), M.frame, bevel=0.002)
                for dz in (-0.07, 0.07):
                    _rod(f'pin{x}{z}{dz}', (x, DT - 0.04, z + dz), (x, 0.012, z), 0.007, M.frame)
                F.sphere(f'pin_foot{x}{z}', 0.012, (x, 0.012, z), M.plastic, segments=10, rings=6)
    G.join_all('deskSimple')
    G.settle(centre=False)
    return G.meshes()


def deskCorner():
    e = 0.93
    with G.new_parts():
        # an L of white laminate, its inner corner cut on the diagonal
        _slab('top', [(-e, -e), (e, -e), (e, 0.33), (0.33, e), (-e, e)], DT - 0.035, 0.035, M.laminate)
        # oak panels: the left end, the right end, the left wing's front end and a modesty panel
        h = DT - 0.035
        _box('panel_left', 0.03, h, 1.8, (-0.89, h / 2, 0), M.oak_panel, bevel=0.004)
        _box('panel_right', 0.03, h, 0.6, (0.89, h / 2, -0.6), M.oak_panel, bevel=0.004)
        _box('panel_front', 0.56, h, 0.03, (-0.6, h / 2, 0.89), M.oak_panel, bevel=0.004)
        _box('panel_back', 1.74, 0.4, 0.02, (0, 0.45, -0.89), M.oak_panel, bevel=0.004)
        # a drawer pedestal in the back left
        _box('pedestal', 0.42, 0.6, 0.55, (-0.62, 0.3, -0.6), M.drawers, bevel=0.008)
        for i in range(3):
            y = 0.5 - i * 0.18
            _box(f'pedestal_front{i}', 0.4, 0.16, 0.012, (-0.62, y, -0.319), M.drawers, bevel=0.004)
            _pull(f'pedestal_pull{i}', -0.62, y + 0.045, -0.313, 0.14, M.frame)
    G.join_all('deskCorner')
    G.settle()
    return G.meshes()


def deskStanding():
    with G.new_parts() as still:
        _box('top', 0.96, 0.03, 0.63, (0, 1.035, -0.115), M.birch, bevel=0.006)
        _box('crossbar', 0.84, 0.04, 0.06, (0, 0.995, -0.13), M.frame, bevel=0.006)
        for s in (-1, 1):
            x = s * 0.38
            _box(f'bracket{s}', 0.06, 0.03, 0.56, (x, 1.005, -0.12), M.frame, bevel=0.006)
            _box(f'column{s}', 0.08, 0.55, 0.06, (x, 0.315, -0.13), M.frame, bevel=0.008)
            _box(f'column_top{s}', 0.065, 0.44, 0.048, (x, 0.77, -0.13), M.frame, bevel=0.006)
            _box(f'foot{s}', 0.07, 0.04, 0.6, (x, 0.02, -0.12), M.frame, bevel=0.01)
            _box(f'runner{s}', 0.015, 0.035, 0.26, (s * 0.31, 1.0, 0.06), M.frame, bevel=0.003)
        _box('controls', 0.08, 0.02, 0.04, (0.38, 1.008, 0.17), M.plastic, bevel=0.005)
        for k in range(2):
            _box(f'button{k}', 0.018, 0.006, 0.012, (0.36 + 0.035 * k, 0.996, 0.185), M.chrome, bevel=0.002)
        # the monitor on a stand at the back, facing the resident
        _box('monitor_foot', 0.2, 0.012, 0.14, (0, 1.056, -0.33), M.plastic, bevel=0.005)
        _box('monitor_neck', 0.04, 0.22, 0.03, (0, 1.16, -0.37), M.plastic, bevel=0.008)
        _box('monitor', 0.56, 0.34, 0.03, (0, 1.37, -0.38), M.plastic, bevel=0.01)
        _box('screen', 0.53, 0.31, 0.004, (0, 1.37, -0.364), M.screen, bevel=0.002)
        _box('wallpaper', 0.44, 0.23, 0.002, (0, 1.375, -0.3615), M.wallpaper)
    G.join(still, 'deskStanding', origin=(0, 0, 0))
    # the keyboard shelf under the front, sliding 20 cm out to the resident
    with G.new_parts() as parts:
        _box('shelf', 0.6, 0.012, 0.24, (0, 0.976, 0.06), M.shelf, bevel=0.004)
        _box('shelf_lip', 0.6, 0.026, 0.012, (0, 0.983, 0.184), M.shelf, bevel=0.004)
        _box('keyboard', 0.42, 0.016, 0.13, (-0.04, 0.99, 0.08), M.keys, bevel=0.005)
        _box('keyboard_keys', 0.39, 0.003, 0.1, (-0.04, 0.9985, 0.085), M.pad, bevel=0.001)
        _box('mouse', 0.06, 0.02, 0.1, (0.22, 0.992, 0.08), M.keys, bevel=0.01)
    shelf = G.join(parts, 'shelf')
    G.settle(centre=False)
    G.slide(shelf, 'tray_out', (0, -0.2, 0), seconds=0.6)
    return G.meshes()


def deskExecutive():
    with G.new_parts() as still:
        _box('top', 1.8, 0.05, 0.77, (0, DT - 0.025, -0.045), M.walnut, bevel=0.012)
        _box('inlay', 1.3, 0.004, 0.46, (0, DT + 0.001, -0.08), M.inlay, bevel=0.002)
        for s in (-1, 1):
            x = s * 0.66
            _box(f'plinth{s}', 0.42, 0.06, 0.68, (x, 0.03, -0.07), M.walnut2, bevel=0.006)
            _box(f'pedestal{s}', 0.44, DT - 0.11, 0.72, (x, 0.06 + (DT - 0.11) / 2, -0.06), M.walnut_case,
                 bevel=0.01)
            for i, (y, h) in enumerate(((0.585, 0.18), (0.385, 0.18), (0.175, 0.2))):
                _box(f'front{s}{i}', 0.4, h, 0.02, (x, y, 0.31), M.walnut2, bevel=0.006)
                _box(f'handle{s}{i}', 0.12, 0.014, 0.014, (x, y + h / 2 - 0.045, 0.334), M.brass, bevel=0.005)
                for p in (-1, 1):
                    _box(f'handle_post{s}{i}{p}', 0.012, 0.012, 0.016, (x + p * 0.05, y + h / 2 - 0.045, 0.324),
                         M.brass)
        _box('modesty', 0.88, 0.45, 0.025, (0, 0.44, -0.38), M.walnut_case, bevel=0.006)
        _box('apron', 0.88, 0.07, 0.025, (0, DT - 0.085, -0.3), M.walnut_case, bevel=0.004)
        F.cyl('pen', 0.005, 0.14, (0.42, DT + 0.008, -0.05), M.pen, rot=(90, 0, 75), verts=8)
        F.cyl('pen_tip', 0.0055, 0.03, (0.4805, DT + 0.008, -0.034), M.brass, rot=(90, 0, 75), verts=8)
    G.join(still, 'deskExecutive', origin=(0, 0, 0))
    # the middle drawer, still: no use opens it yet
    with G.new_parts() as parts:
        _box('drawer', 0.84, 0.075, 0.02, (0, DT - 0.09, 0.31), M.walnut2, bevel=0.006)
        _box('drawer_pull', 0.18, 0.016, 0.016, (0, DT - 0.09, 0.327), M.walnut, bevel=0.006)
        _drawer_box('drawer_box', 0.8, 0.06, 0.5, 0, DT - 0.125, -0.2, M.walnut2)
    G.join(parts, 'drawer')
    G.settle(centre=False)
    return G.meshes()


# --- storage and boards ----------------------------------------------------------------------------------------
def filingCabinet():
    z0, d = WALL, 0.6
    front = z0 + d
    with G.new_parts() as still:
        _box('cabinet', 0.46, 1.0, d, (0, 0.5, z0 + d / 2), M.steel, bevel=0.01)
        for i, y in enumerate((0.175, 0.5)):
            _box(f'front{i}', 0.42, 0.3, 0.02, (0, y, front + 0.01), M.steel, bevel=0.006)
            _box(f'handle{i}', 0.14, 0.02, 0.02, (0, y + 0.06, front + 0.03), M.chrome, bevel=0.006)
            _box(f'label_holder{i}', 0.1, 0.04, 0.004, (0, y + 0.11, front + 0.022), M.chrome, bevel=0.001)
            _box(f'label{i}', 0.085, 0.03, 0.002, (0, y + 0.11, front + 0.0245), M.label)
        F.cyl('lock', 0.012, 0.01, (0.17, 0.97, front + 0.005), M.chrome, rot=(90, 0, 0), verts=12)
        # a paper tray and a potted plant on top
        _box('paper_tray', 0.24, 0.03, 0.3, (-0.1, 1.015, -0.2), M.plastic, bevel=0.004)
        _box('papers', 0.21, 0.02, 0.28, (-0.1, 1.022, -0.2), M.paper)
        F.cyl('pot', 0.065, 0.11, (0.12, 1.055, -0.2), M.pot, verts=18, bevel=0.006)
        F.cyl('pot_soil', 0.058, 0.005, (0.12, 1.108, -0.2), M.soil, verts=18)
        for k in range(7):
            a = k * 360 / 7
            lean = 25 + 12 * (k % 2)
            G.sphere(f'leaf{k}', 0.05, at=F.at(0.12 + 0.04 * G.cosd(a), 1.16, -0.2 + 0.04 * G.sind(a)),
                     material=M.leaf, scale=(0.4, 0.12, 1.1), rot=(lean, 0, -a + 90), segments=10, rings=6)
    G.join(still, 'filingCabinet', origin=(0, 0, 0))
    # the top drawer with its box of files behind it, sliding out while a resident files papers
    with G.new_parts() as parts:
        _box('drawer', 0.42, 0.3, 0.02, (0, 0.825, front + 0.01), M.steel, bevel=0.006)
        _box('drawer_handle', 0.14, 0.02, 0.02, (0, 0.885, front + 0.03), M.chrome, bevel=0.006)
        _box('drawer_label_holder', 0.1, 0.04, 0.004, (0, 0.935, front + 0.022), M.chrome, bevel=0.001)
        _box('drawer_label', 0.085, 0.03, 0.002, (0, 0.935, front + 0.0245), M.label)
        _box('drawer_bottom', 0.38, 0.015, 0.5, (0, 0.69, front - 0.25), M.steel, bevel=0.003)
        for s in (-1, 1):
            _box(f'drawer_side{s}', 0.015, 0.22, 0.5, (s * 0.19, 0.79, front - 0.25), M.steel, bevel=0.003)
        _box('drawer_back', 0.38, 0.22, 0.015, (0, 0.79, front - 0.5), M.steel, bevel=0.003)
        for k in range(6):
            _box(f'file{k}', 0.34, 0.2, 0.012, (0, 0.8, front - 0.06 - k * 0.07), M.paper, bevel=0.002)
    drawer = G.join(parts, 'drawer', origin=F.at(0, 0.825, front))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=1.2)
    return G.meshes()


def whiteboard():
    y, z = 1.4, WALL
    face = z + 0.0145          # the board's writing face
    with G.new_parts() as still:
        _box('board', 0.88, 0.58, 0.014, (0, y, z + 0.0075), M.board)
        for k, s in enumerate((-1, 1)):
            _box(f'frame_h{k}', 0.92, 0.025, 0.025, (0, y + s * 0.2975, z + 0.0125), M.aluminium, bevel=0.005)
            _box(f'frame_v{k}', 0.025, 0.62, 0.025, (s * 0.4475, y, z + 0.0125), M.aluminium, bevel=0.005)
            for t in (-1, 1):
                _box(f'corner{k}{t}', 0.035, 0.035, 0.03, (s * 0.4475, y + t * 0.2975, z + 0.015), M.plastic,
                     bevel=0.008)
        # the marker tray, three markers and an eraser
        _box('tray', 0.6, 0.012, 0.07, (0, 1.074, z + 0.035), M.aluminium, bevel=0.003)
        _box('tray_lip', 0.6, 0.026, 0.006, (0, 1.083, z + 0.067), M.aluminium, bevel=0.002)
        for k, (x, m) in enumerate(((-0.2, M.ink_red), (-0.12, M.ink_blue), (-0.04, M.ink_black))):
            F.cyl(f'marker{k}', 0.009, 0.1, (x, 1.089, z + 0.04), M.label, rot=(0, 90, 0), verts=10)
            F.cyl(f'marker_cap{k}', 0.0095, 0.03, (x + 0.06, 1.089, z + 0.04), m, rot=(0, 90, 0), verts=10)
        _box('eraser', 0.12, 0.03, 0.05, (0.18, 1.095, z + 0.037), M.plastic, bevel=0.008)
        _box('eraser_felt', 0.116, 0.008, 0.046, (0.18, 1.084, z + 0.037), M.note2)
        # what's on it at rest: a heading and a note
        _box('heading', 0.26, 0.008, 0.002, (0.22, 1.6, face), M.ink_black)
        _box('heading_rule', 0.3, 0.004, 0.002, (0.2, 1.585, face), M.ink_black)
        _box('sticky', 0.1, 0.1, 0.003, (-0.33, 1.6, face + 0.0005), M.note, rot=(0, 4, 0))
    G.join(still, 'whiteboard', origin=(0, 0, 0))
    # the plan, drawn while a resident plans: lines and a chart grow across the board from its left (game +x,
    # as seen from the front), out of nothing: scaled to a point on the left edge at rest
    left = 0.36
    with G.new_parts() as parts:
        for k, (x, yy, w, m, r) in enumerate((
                (0.2, 1.52, 0.3, M.ink_blue, 0), (0.22, 1.48, 0.26, M.ink_blue, 0),
                (0.24, 1.44, 0.22, M.ink_blue, 0), (0.12, 1.32, 0.46, M.ink_black, 0),
                (-0.16, 1.48, 0.22, M.ink_red, -25), (-0.16, 1.43, 0.16, M.ink_red, 12))):
            _box(f'line{k}', w, 0.008, 0.002, (x, yy, face + 0.001), m, rot=(0, r, 0))
        for k, h in enumerate((0.08, 0.14, 0.11, 0.19)):
            _box(f'bar{k}', 0.035, h, 0.002, (-0.08 - 0.06 * k, 1.17 + h / 2, face + 0.001), M.ink_red)
        _box('axis', 0.28, 0.006, 0.002, (-0.17, 1.168, face + 0.0012), M.ink_black)
        _box('ring', 0.07, 0.07, 0.002, (0.3, 1.2, face + 0.001), M.ink_blue, rot=(0, 45, 0))
    scribble = G.hinged(G.join(parts, 'scribble'), F.at(left, y, face))
    scribble.scale[0] = scribble.scale[2] = 0.001
    G.settle(centre=False, floor=False)
    G.grow(scribble, 'scribble', seconds=1.2, axis='X')
    G.grow(scribble, 'scribble', seconds=1.2, axis='Z')
    return G.meshes()


def corkboard():
    y, z = 1.45, WALL
    face = z + 0.016           # the cork's face
    with G.new_parts():
        _box('cork', 0.68, 0.46, 0.016, (0, y, z + 0.008), M.cork)
        for k, s in enumerate((-1, 1)):
            _box(f'frame_h{k}', 0.74, 0.035, 0.03, (0, y + s * 0.2425, z + 0.015), M.cork_frame, bevel=0.005)
            _box(f'frame_v{k}', 0.035, 0.45, 0.03, (s * 0.3525, y, z + 0.015), M.cork_frame, bevel=0.005)
        for k, (x, yy, m, r, pin) in enumerate((
                (-0.2, 1.55, M.note, 6, M.pin), (0.0, 1.5, M.note2, -4, M.ink_blue), (0.2, 1.56, M.note3, 3, M.pin),
                (-0.15, 1.35, M.note4, -6, M.note), (0.18, 1.34, M.note2, 4, M.pin))):
            _box(f'note{k}', 0.14, 0.12, 0.002, (x, yy, face + 0.001), m, rot=(0, r, 0))
            F.sphere(f'pin{k}', 0.009, (x, yy + 0.045, face + 0.008), pin, segments=10, rings=6)
            F.cyl(f'pin_stem{k}', 0.002, 0.008, (x, yy + 0.045, face + 0.003), M.chrome, rot=(90, 0, 0), verts=6)
        # a few lines on the notes
        for k, (x, yy, r) in enumerate(((-0.2, 1.55, 6), (0.0, 1.5, -4), (0.18, 1.34, 4))):
            for j in range(3):
                dy = 0.02 - 0.025 * j
                _box(f'scrawl{k}{j}', 0.09, 0.004, 0.001,
                     (x - G.sind(r) * dy, yy + G.cosd(r) * dy, face + 0.0025), M.ink_black, rot=(0, r, 0))
    G.join_all('corkboard')
    G.settle(centre=False, floor=False)
    return G.meshes()


TYPE = {'uses': {u: 'tray_out' for u in ('code', 'write', 'markets', 'browse', 'homework')}}
SWIVEL = {'uses': {u: 'swivel' for u in ('sit', 'write', 'homework')}, 'hold': 0.6, 'ride': 'swivel'}
DESK = [('frame', 'Frame'), ('drawers', 'Drawers')]

FAMILY = G.Family(
    'office', M,
    [G.Model('deskSimple', deskSimple, size=(1.5, 0.74, 0.69), key='model.deskSimple', centre=(0, -0.085),
             paint=[('oak', 'Top'), *DESK]),
     G.Model('deskCorner', deskCorner, size=(1.86, 0.74, 1.86), key='model.deskCorner',
             paint=[('laminate', 'Top'), ('oak_panel', 'Frame'), ('drawers', 'Drawers')]),
     G.Model('deskStanding', deskStanding, size=(0.96, 1.54, 0.63), key='model.deskStanding', centre=(0, -0.115),
             motions={'uses': {'browse': 'tray_out', 'code': 'tray_out'}},
             paint=[('birch', 'Top'), ('frame', 'Frame'), ('shelf', 'Shelf')]),
     G.Model('deskExecutive', deskExecutive, size=(1.8, 0.754, 0.771), key='model.deskExecutive', centre=(0, -0.0445),
             paint=[('walnut', 'Top'), ('walnut_case', 'Frame'), ('walnut2', 'Drawers'), ('brass', 'Handles'),
                    ('inlay', 'Desk pad')]),
     G.Model('computerDesk', computerDesk, size=(1.54, 1.215, 0.872), key='model.computerDesk', centre=(0, 0.0142),
             motions=TYPE,
             paint=[('oak', 'Top'), *DESK, ('tray', 'Keyboard tray'), ('chair', 'Chair')]),
     G.Model('deskWriting', deskWriting, size=(1.54, 1.114, 0.872), key='model.deskWriting', centre=(0, 0.0142),
             motions={'uses': {'write': 'drawer_open', 'homework': 'drawer_open'}, 'hold': 2.5},
             paint=[('oak', 'Top'), *DESK, ('chair_sage', 'Chair')]),
     G.Model('deskLaptop', deskLaptop, size=(0.9, 1.009, 0.872), key='model.deskLaptop', centre=(0, 0.0142),
             motions={'uses': {u: 'lid_open' for u in ('browse', 'code', 'write', 'homework')}},
             paint=[('oak', 'Top'), ('frame', 'Frame'), ('drawers', 'Drawer'), ('aluminium', 'Laptop'),
                    ('chair', 'Chair')]),
     G.Model('computerGaming', computerGaming, size=(1.7, 1.299, 0.9), key='model.computerGaming', centre=(0, 0.016),
             motions={'loops': {'game': 'fans'}},
             paint=[('desk', 'Desk'), ('case', 'PC & monitors'), ('accent', 'Lights'), ('chair_gamer', 'Chair')]),
     G.Model('chairExecutive', chairExecutive, size=(0.615, 1.336, 0.637), key='model.chairExecutive', centre=(0, 0.0276),
             motions=SWIVEL, paint=[('frame', 'Frame'), ('leather', 'Seat')]),
     G.Model('chairTask', chairTask, size=(0.615, 1.048, 0.603), key='model.chairTask', centre=(0, 0.0445),
             motions=SWIVEL, paint=[('frame', 'Frame'), ('fabric', 'Seat'), ('mesh', 'Back')]),
     G.Model('officeChair', officeChair, size=(0.615, 1.157, 0.624), key='model.officeChair', centre=(0, -0.0162),
             motions={'uses': {'sit': 'swivel'}, 'hold': 0.6, 'ride': 'swivel'},
             paint=[('frame', 'Frame'), ('upholstery', 'Seat')]),
     G.Model('filingCabinet', filingCabinet, size=(0.46, 1.21, 0.64), key='model.filingCabinet', centre=(0, -0.11),
             motions={'uses': {'file': 'drawer_open'}, 'hold': 2.0},
             paint=[('steel', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('whiteboard', whiteboard, size=(0.93, 0.647, 0.07), key='model.whiteboard', centre=(0, -0.395),
             floor=False, motions={'uses': {'plan': 'scribble'}},
             paint=[('board', 'Board'), ('aluminium', 'Frame')]),
     G.Model('corkboard', corkboard, size=(0.74, 0.52, 0.033), key='model.corkboard', centre=(0, -0.4135),
             floor=False, paint=[('cork', 'Board'), ('cork_frame', 'Frame')])],
    comment='Blender-built office models (tools/art/blender/families/office.py): the desks, the desk chairs, the '
            'filing cabinet and the boards. Replaces the primitive-part placeholders by key; listed in the base '
            'manifest\'s packs. The placeholder is kept as a fallback if a file fails to load, and the materials map '
            'onto the game\'s finishes so a model costs one draw call per finish group. The keyboard shelves slide '
            'out (`tray_out`), the writing desk\'s drawer opens (`drawer_open`), the laptop opens (`lid_open`), the '
            'gaming PC\'s fans spin (`fans`), the chairs swivel (`swivel`) and the whiteboard fills with a plan '
            '(`scribble`) while residents use them.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
