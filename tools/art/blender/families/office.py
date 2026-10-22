# The office: the desks (O1: the simple, corner, standing and executive desks, the computer, writing and
# laptop desks and the gaming PC), the desk chairs (O2: the executive, desk and office chairs) and the storage
# and boards (O3: the filing cabinet, the whiteboard and the corkboard). Notes: office.md.
#
# The parts are placed in game axes (`F`, no shift), where the game places it; a desk's top is the
# content's `surface` height (`DT`, 0.75 for the plain desks, 1.05 standing). Backs against the wall
# stop at its face (game depth -0.43, -0.93 two tiles deep). The computer, writing, laptop and gaming desks
# keep the old part's chair in the back half (`characters.ts` SEATS: seat 0.44, 17 cm back), facing the
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
# Run:  cargo art office [--only deskSimple] [--preview]
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
    walnut_case=('#5A3825', 'wood'),  # its pedestals, their plinths and the modesty panel
    walnut2=('#6B4630', 'wood'),      # its drawer fronts
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
    coffee=('#4A3121', 'gloss'),
    reveal=('#1E1F22', 'matte'),      # the dark gaps round drawer fronts
    keycap=('#2A2C30', 'satin'),      # the laptop's keys
    keycap_light=('#D4D5D3', 'satin'),  # the standing desk's keyboard's keys
    led=('#5EC28B', 'gloss'),
    leafy=('#7FA86A', 'matte'),       # the corkboard photo's hill
)

F = G.GameFrame()
WALL = -0.43          # the wall's face, game depth (one tile deep)
DT = 0.74             # the second catalogue's desk top
FT = 0.7525           # the plain desks' top (content surface 0.75)
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
    obj.rotation_mode = 'XYZ'   # (a joined part keeps its first part's mode: a rod's is quaternion, which ignores an Euler turn)
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
LEG_R = 0.27          # how far a chair's legs reach out from the lift


def _contour_rings(w, h, t, at, verts=20, n=10, dish=0.02, lumbar=0.012, square=4.0, taper=0.0):
    """Rings for a contoured chair back `w` wide, `h` high and `t` thick, its middle at game `at`: its outline
    rounding over the top and the foot, a lumbar swell low down, its sides wrapping forward (`dish`), narrower
    at the foot by `taper`. Bottom first, closed to a point at each end (Blender points, for `G.loft`)."""
    x0, y0, z0 = at
    secs = []
    for i in range(n + 1):
        q = math.pi * (i / n - 0.5)             # rings spaced evenly round the outline's ends, not up its height
        v = math.copysign(abs(math.sin(q)) ** 0.5, q)
        e = abs(math.cos(q)) ** 0.5 if 0 < i < n else 0.0
        rx = w / 2 * e * (1 - taper * (1 - v) / 2)
        rz = t / 2 * max(0.0, 1 - abs(v) ** 8) ** 0.125
        fwd = lumbar * math.exp(-((v + 0.35) / 0.35) ** 2)
        ring = []
        for k in range(verts):
            a = math.tau * k / verts
            c, s = math.cos(a), math.sin(a)
            px = rx * math.copysign(abs(c) ** (2 / square), c)
            pz = rz * math.copysign(abs(s) ** (2 / square), s)
            ring.append(F.at(x0 + px, y0 + v * h / 2, z0 + pz + fwd + dish * (px / (w / 2)) ** 2))
        secs.append(ring)
    return secs


def _contour_front(w, h, t, at, v, u, dish=0.02, lumbar=0.012, square=4.0, taper=0.0):
    """The game point on a contoured back's front at `v` up it (-1 foot .. 1 top) and `u` across (-1 .. 1)."""
    x0, y0, z0 = at
    e = max(0.0, 1 - abs(v) ** 4) ** 0.25
    rx = w / 2 * e * (1 - taper * (1 - v) / 2)
    rz = t / 2 * max(0.0, 1 - abs(v) ** 8) ** 0.125
    px = u * rx
    pz = rz * max(0.0, 1 - abs(u) ** square) ** (1 / square)
    fwd = lumbar * math.exp(-((v + 0.35) / 0.35) ** 2)
    return (x0 + px, y0 + v * h / 2, z0 + pz + fwd + dish * (px / (w / 2)) ** 2)


def _outline(w, h, t, at, taper, dish=0.02, n=32, k=1.0):
    """A closed loop round a contoured back's front (a frame's or a welt's path), `k` of the way out to its edge."""
    out = []
    for i in range(n):
        q = math.tau * i / n
        c, s = math.cos(q), math.sin(q)
        u, v = math.copysign(abs(c) ** 0.5, c), math.copysign(abs(s) ** 0.5, s)
        out.append(_contour_front(w, h, t, at, v * k, u * k, dish=dish, taper=taper))
    return out


def _castor(name, x, z, a, lite=False):
    """A twin-wheel castor under a leg's tip at (x, z), its wheels rolling along the leg (`a`, radians): a
    socket, a rounded hood and two wheels either side of it (`lite`: one wide wheel, for a chair half under a desk)."""
    if not lite:
        F.cyl(f'{name}_socket', 0.009, 0.03, (x, 0.058, z), M.castor, verts=8)
    G.sphere(f'{name}_hood', 0.024, at=F.at(x, 0.034, z), material=M.castor, scale=(1.0, 1.0, 0.75),
             segments=8 if lite else 10, rings=4 if lite else 5)
    w = 0.014 if lite else 0.0065
    tyre = [(0, -w), (0.02, -w), (0.025, 0.0), (0.02, w), (0, w)]
    for s in ((0,) if lite else (-1, 1)):
        # the wheels' axle across the leg
        ox, oz = -math.sin(a) * 0.015 * s, math.cos(a) * 0.015 * s
        G.revolve(f'{name}_wheel{s}', tyre, at=F.at(x + ox, 0.025, z + oz), material=M.castor, segments=8 if lite else 12,
                  rot=(0, 90, math.degrees(a) - 90))


def _chair_base(z, seat_y, frame, lite=False):
    """The still base: five swept legs with castors, the hub, and the gas lift up to under the seat."""
    prof = G.rounded_rect(0.032, 0.042, 0.012, corner=1)
    for i in range(5):
        a = math.radians(i * 72 + 18)
        c, s = math.cos(a), math.sin(a)
        pts = [(c * 0.03, 0.118, z + s * 0.03), (c * 0.17, 0.105, z + s * 0.17), (c * 0.25, 0.082, z + s * 0.25),
               (c * LEG_R, 0.078, z + s * LEG_R)]
        F.sweep(f'leg{i}', pts if lite else G.kit.fillet(pts, 0.04, 2), None, frame, profile=prof, up=(0, 0, 1),
                scale=[1.0, 0.85, 0.75, 0.72], angle=50)
        _castor(f'castor{i}', c * LEG_R, z + s * LEG_R, a, lite)
    G.revolve('hub', [(0, 0.085), (0.05, 0.085), (0.056, 0.1), (0.05, 0.128), (0.034, 0.138), (0, 0.138)],
              at=F.at(0, 0, z), material=frame, segments=10 if lite else 16)
    # the lift: a tapered plastic sleeve over the cylinder, its chrome piston up into the mechanism
    G.revolve('lift_sleeve', [(0, 0.13), (0.031, 0.13), (0.029, 0.15), (0.025, 0.25), (0.021, 0.262), (0, 0.262)],
              at=F.at(0, 0, z), material=M.plastic, segments=10 if lite else 14)
    lift = seat_y - 0.13 - 0.25
    F.cyl('lift', 0.016, lift, (0, 0.25 + lift / 2, z), M.chrome, verts=10 if lite else 12)


def _t_arm(name, s, z, seat_y, frame, pad, lite=False):
    """A T arm: a bar bent from under the seat out and up, a padded rest on its top."""
    x0, x1 = s * 0.2, s * 0.268
    pts = [(x0, seat_y - 0.105, z - 0.03), (x1, seat_y - 0.105, z - 0.03), (x1, seat_y + 0.125, z - 0.02)]
    F.sweep(f'{name}_post', G.kit.fillet(pts, 0.035, 2 if lite else 3), None, frame,
            profile=G.rounded_rect(0.03, 0.05, 0.01, corner=1), up=(0, 1, 0), angle=50)
    F.soft(f'{name}_pad', 0.07, 0.032, 0.27, (s * 0.272, seat_y + 0.15, z), pad, round=0.014,
           round_segments=1 if lite else 2)


def _loop_arm(name, s, z, seat_y, frame, pad, lite=False):
    """A loop arm: a flat bar bent from under the seat's front up, back along the top and down under its back,
    a padded rest along the top."""
    x = s * 0.275
    pts = [(s * 0.2, seat_y - 0.095, z + 0.12), (x, seat_y - 0.095, z + 0.12), (x, seat_y + 0.13, z + 0.1),
           (x, seat_y + 0.13, z - 0.14), (x, seat_y - 0.095, z - 0.12), (s * 0.2, seat_y - 0.095, z - 0.12)]
    F.sweep(f'{name}_loop', G.kit.fillet(pts, 0.045, 3), None, frame, profile=G.rounded_rect(0.012, 0.04, 0.005, corner=1),
            up=(1, 0, 0), angle=50)
    F.soft(f'{name}_pad', 0.068, 0.034, 0.26, (x, seat_y + 0.152, z - 0.015), pad, round=0.015, round_segments=2)


def _chair_seat(z, seat_y, back_h, seat, back=None, frame=None, arm=None, head=False, channels=None,
                mesh_frame=None, back_z=0.25, lean=9, arms='t', lite=False):
    """The swivelling part: the mechanism and its levers, a shell under a padded seat, the back on its spine (a
    contoured pad; a mesh in a frame with `mesh_frame`; stitched across with `channels`, or striped up for a
    `head`ed racing back), and the arms."""
    back = back or seat
    frame = frame or M.frame
    arm = arm or frame
    verts = 16 if lite else 20
    F.box('mech', 0.17, 0.05, 0.2, (0, seat_y - 0.125, z), M.plastic, bevel=0.012, bevel_segments=1)
    F.sweep('lever', G.kit.fillet([(0.06, seat_y - 0.12, z + 0.04), (0.22, seat_y - 0.12, z + 0.05),
                                   (0.25, seat_y - 0.13, z + 0.09)], 0.03, 2),
            0.006, M.plastic, verts=6)
    F.box('lever_tip', 0.03, 0.016, 0.05, (0.25, seat_y - 0.13, z + 0.1), M.plastic, bevel=0.007, bevel_segments=1)
    F.cyl('tension', 0.03, 0.04, (0, seat_y - 0.135, z + 0.11), M.plastic, rot=(90, 0, 0), verts=12, bevel=0.006,
          bevel_segments=1)
    # the seat: a moulded shell, the cushion on it rounded over a waterfall front, piped round its top
    F.box('shell', 0.47, 0.03, 0.45, (0, seat_y - 0.088, z + 0.01), M.plastic, bevel=0.012, bevel_segments=1)
    F.soft('seat', 0.5, 0.068, 0.48, (0, seat_y - 0.044, z + 0.01), seat, round=0.03, puff=0.01, flat=2,
           round_segments=1 if lite else 2, piping=None if lite else 0.005)
    # the back, built upright and leant back `lean` degrees about its middle
    yb = seat_y + 0.07 + back_h / 2
    bw, bt = 0.48, (0.03 if mesh_frame else 0.07)
    z0 = z - back_z + (0.0 if mesh_frame else 0.003)
    at = (0, yb, z0)
    rear = z0 - (0.0125 if mesh_frame else 0.034)          # the back's rear face, upright
    # the spine: a bar from the mechanism back and up behind the back, to a plate on the back (which leans)
    plate_y = yb - back_h / 2 + 0.08
    plate_z = rear - 0.01 + (back_h / 2 - 0.08) * math.sin(math.radians(lean))
    sz = plate_z - 0.022
    F.sweep('spine', G.kit.fillet([(0, seat_y - 0.115, z - 0.06), (0, seat_y - 0.115, sz), (0, plate_y, sz)], 0.06, 3),
            None, frame, profile=G.rounded_rect(0.024, 0.07, 0.01, corner=1), up=(0, 1, 0), angle=50)
    welt = []
    with G.new_parts() as parts:
        F.box('back_plate', 0.12, 0.14, 0.02, (0, plate_y, rear - 0.01), frame, bevel=0.008, bevel_segments=1)
        if mesh_frame:
            # a moulded frame round a taut mesh, the mesh bowed to the back
            G.loft('back', _contour_rings(bw - 0.03, back_h - 0.03, 0.008, at, verts=verts, n=8, dish=0.025), material=back)
            F.sweep('back_frame', _outline(bw - 0.012, back_h - 0.012, 0.0, at, 0.0, dish=0.025), None, mesh_frame,
                    profile=G.rounded_rect(0.025, 0.035, 0.008, corner=1), closed=True, up=(0, 1, 0), angle=60)
            F.box('lumbar', bw - 0.08, 0.07, 0.02, (0, yb - back_h * 0.18, z0 - 0.016), mesh_frame, bevel=0.009, bevel_segments=1)
        elif channels and not head:
            # a leather shell, padded in front with four stitched pillows, each as wide as the back where it is
            G.loft('back', _contour_rings(bw, back_h, 0.044, (0, yb, z0 - 0.012), verts=verts, n=12, taper=0.08,
                                          lumbar=0.0), material=back)
            n = 4
            span = back_h * 0.86
            for k in range(n):
                v = -0.86 + (k + 0.5) * 1.72 / n
                e = max(0.0, 1 - abs(v) ** 4) ** 0.25 * (1 - 0.08 * (1 - v) / 2)
                fwd = 0.012 * math.exp(-((v + 0.35) / 0.35) ** 2)
                F.soft(f'pillow{k}', bw * e - 0.03, span / n + 0.006, 0.05, (0, yb + v * back_h / 2, z0 + 0.012 + fwd), back,
                       round=0.022, puff=0.01, flat=2, round_segments=2)
        else:
            G.loft('back', _contour_rings(bw, back_h, bt, at, verts=verts, n=8 if lite else 12, taper=0.08), material=back)
            G.loft('back_shell', _contour_rings(bw - 0.02, back_h - 0.02, 0.044, (0, yb, z0 - 0.012), verts=verts,
                                                n=6, lumbar=0.0, taper=0.08), material=M.plastic)
            if not lite:
                welt = _outline(bw, back_h, bt, at, 0.08, k=0.9)
                F.sweep('welt', welt, 0.005, back, verts=5, closed=True)
        if channels and head:
            # a racing back: stripes up either side of its front
            for s in (-1, 1):
                pts = [_contour_front(bw, back_h, bt, at, v, s * 0.55, taper=0.08) for v in [-0.8 + 1.6 * i / 8 for i in range(9)]]
                F.sweep(f'stripe{s}', [(px, py, pz - 0.001) for px, py, pz in pts], None, channels,
                        profile=G.rounded_rect(0.006, 0.03, 0.002, corner=1), up=(0, 0, 1), angle=60)
        if head:
            F.soft('headrest', 0.3, 0.14, 0.07, (0, yb + back_h / 2 + 0.1, z - back_z - 0.005), back, round=0.03, puff=0.008,
                   round_segments=1 if lite else 2)
            for s in (-1, 1):
                F.cyl(f'head_post{s}', 0.008, 0.1, (s * 0.08, yb + back_h / 2 + 0.01, z - back_z - 0.01), M.chrome, verts=8)
    _turned(parts, 'back', at, x=-lean)
    for s in (-1, 1):
        (_loop_arm if arms == 'loop' else _t_arm)(f'arm{s}', s, z, seat_y, frame, arm, lite)


def _desk_chair(seat, z=CHAIR_Z, seat_y=0.44, back_h=0.5, head=False, channels=None):
    """A desk's own chair (part of the desk, still): in its back half, facing the desk, its back short of
    the wall."""
    _chair_base(z, seat_y, M.frame, lite=True)
    _chair_seat(z, seat_y, back_h, seat, head=head, channels=channels, back_z=0.18 if head else 0.184, lean=5 if head else 7, lite=True)


def _chair(seat_y, back_h, z, seat, back=None, frame=None, arm=None, head=False, channels=None, mesh_frame=None,
           arms='t', back_z=0.25):
    frame = frame or M.frame
    with G.new_parts() as still:
        _chair_base(z, seat_y, frame)
    G.join(still, 'base', origin=(0, 0, 0))
    with G.new_parts() as parts:
        _chair_seat(z, seat_y, back_h, seat, back, frame, arm, head, channels, mesh_frame, arms=arms, back_z=back_z)
    swivel = G.hinged(G.join(parts, 'seat'), F.at(0, seat_y - 0.15, z))
    G.settle(centre=False)
    G.swing(swivel, 'swivel', 15, seconds=0.5)
    return G.meshes()


def chairExecutive():
    return _chair(0.49, 0.78, 0.05, M.leather, arm=M.leather, channels=M.leather2, arms='loop', back_z=0.254)


def chairTask():
    return _chair(0.48, 0.5, 0.05, M.fabric, back=M.mesh, mesh_frame=M.plastic)


def officeChair():
    return _chair(0.47, 0.62, 0.0, M.upholstery, back_z=0.254)


# --- desk pieces -------------------------------------------------------------------------------------------
def _axle(gx, gz):
    """The rot for a cylinder whose axis runs along the game direction (gx, 0, gz)."""
    return (0, 90, math.degrees(math.atan2(-gz, -gx)))


def _bar_pull(name, x, y, face, length, m, out=0.026, r=0.005, upright=False):
    """A bar pull bent from one rod, out on two stand-offs from a front at game depth `face` (towards -depth when
    `out` is negative)."""
    h, b = length / 2, min(0.01, abs(out) * 0.45)
    into = face - math.copysign(0.003, out)
    ends = [(-h, into), (-h, face + out), (h, face + out), (h, into)]
    pts = [(x, y + u, zz) if upright else (x + u, y, zz) for u, zz in ends]
    return F.sweep(name, G.kit.fillet(pts, b), r, m, verts=8)


def _top(w, d=0.42, z=0.24, m=None, y=FT, t=0.035, name='top', x=0.0):
    """A desk top `w` x `d`, `t` thick, its surface at `y`, its edges softly rounded."""
    return F.soft(name, w, t, d, (x, y - t / 2, z), m or M.oak, round=0.008, round_segments=2)


def _front(name, w, h, x, y, z, m, t=0.018):
    """A slab drawer front, its edges rounded, centred at game (x, y, z)."""
    return F.box(name, w, h, t, (x, y, z), m, bevel=0.005, bevel_segments=1)


def _desk_frame(w, legs=None, gap=None, apron=None):
    """A welded frame of square tube: each side one piece (a leg up, a rail along, a leg down) on glides, a back
    apron, and a front (resident's side) apron with a drawer gap (x0, x1) left open."""
    legs, apron = legs or M.frame, apron or M.frame
    sq = G.rounded_rect(0.04, 0.04, 0.007, corner=1)
    for s in (-1, 1):
        x = s * (w / 2 - 0.04)
        F.sweep(f'side{s}', G.kit.fillet([(x, 0.012, 0.06), (x, 0.70, 0.06), (x, 0.70, 0.42), (x, 0.012, 0.42)], 0.012, 2),
                None, legs, profile=sq, up=(1, 0, 0), angle=50)
        for z in (0.06, 0.42):
            F.cyl(f'glide{s}{z}', 0.017, 0.012, (x, 0.006, z), M.plastic, verts=10)
    F.box('apron_back', w - 0.12, 0.05, 0.02, (0, 0.695, 0.43), apron, bevel=0.004, bevel_segments=1)
    x0, x1 = gap or (0, 0)
    for k, (a, b) in enumerate(((-w / 2 + 0.06, x0), (x1, w / 2 - 0.06))):
        if b - a > 0.01:
            F.box(f'apron_front{k}', b - a, 0.05, 0.02, ((a + b) / 2, 0.695, 0.06), apron, bevel=0.004,
                  bevel_segments=1)


def _desk_lamp(x, z, m=None, y=FT):
    """An architect's lamp on the top: a weighted base, two arms bent at a knuckle with their springs, the shade
    turned down to the resident's side."""
    m = m or M.lamp
    G.revolve('lamp_base', [(0, 0), (0.064, 0), (0.066, 0.008), (0.06, 0.018), (0.03, 0.024), (0.016, 0.034),
                            (0, 0.034)], at=F.at(x, y, z), material=m, segments=20)
    root, elbow, head = (x, y + 0.03, z), (x + 0.04, 1.1, z - 0.05), (x + 0.15, 1.068, z - 0.14)
    F.sweep('lamp_arm', [root, elbow, head], 0.0075, m, verts=8)
    for k, (a, b) in enumerate(((root, elbow), (elbow, head))):
        ends = [tuple(p + (q - p) * t + o for p, q, o in zip(a, b, (0.011, 0.0, 0.011))) for t in (0.2, 0.8)]
        F.sweep(f'lamp_spring{k}', ends, 0.0028, M.chrome, verts=4)
    F.cyl('lamp_knuckle', 0.013, 0.03, elbow, m, rot=_axle(0.14, 0.15), verts=12)
    F.cyl('lamp_neck', 0.011, 0.024, head, m, rot=_axle(0.14, 0.15), verts=10)
    G.revolve('lamp_shade', [(0, 0.05), (0.016, 0.05), (0.024, 0.042), (0.06, -0.045), (0.063, -0.05), (0.057, -0.05),
                             (0.02, 0.035), (0, 0.036)], at=F.at(x + 0.17, 1.03, z - 0.16), material=m, segments=20,
              rot=(35, 0, 0))
    F.sphere('lamp_bulb', 0.022, (x + 0.17, 1.012, z - 0.175), M.bulb, segments=10, rings=6)


def _mug(x, z, y=FT, m=None):
    """A mug of coffee, hollow, its handle to the right."""
    m = m or M.mug
    G.revolve('mug', [(0, 0), (0.036, 0), (0.04, 0.006), (0.041, 0.095), (0.037, 0.095), (0.036, 0.012), (0, 0.012)],
              at=F.at(x, y, z), material=m, segments=16)
    F.cyl('mug_coffee', 0.0365, 0.004, (x, y + 0.078, z), M.coffee, verts=16)
    F.sweep('mug_handle', [(x + 0.038, y + 0.078, z), (x + 0.062, y + 0.072, z), (x + 0.066, y + 0.05, z),
                           (x + 0.06, y + 0.028, z), (x + 0.038, y + 0.024, z)], 0.0055, m, verts=6, resolution=2)


def _cactus(x, z, y=FT, r=0.045, h=0.08):
    """A ribbed cactus in a rimmed pot of soil."""
    G.revolve('pot', [(0, 0), (r * 0.78, 0), (r * 0.94, h * 0.8), (r * 1.06, h * 0.82), (r * 1.08, h), (r * 0.94, h),
                      (r * 0.92, h * 0.9), (0, h * 0.9)], at=F.at(x, y, z), material=M.pot, segments=16)
    F.cyl('pot_soil', r * 0.93, 0.004, (x, y + h * 0.9, z), M.soil, verts=16)
    secs = []
    for t, k in ((0.0, 0.8), (0.12, 0.98), (0.5, 1.0), (0.8, 0.88), (0.93, 0.6), (1.0, 0.18)):
        ring = []
        for j in range(20):
            a = math.tau * j / 20
            rr = r * 0.72 * k * (1 + 0.1 * math.cos(10 * a))
            ring.append(F.at(x + rr * math.cos(a), y + h * 0.86 + t * 0.12, z + rr * math.sin(a)))
        secs.append(ring)
    G.loft('cactus', secs, material=M.cactus)


def _notebook(name, x, y, z, w, d, t, m, turn=0.0):
    """A closed notebook lying on `y`: its pages between two covers, a band round it."""
    with G.new_parts() as parts:
        F.box(f'{name}_pages', w - 0.006, t - 0.003, d - 0.006, (x + 0.002, y + t / 2, z), M.paper)
        for k, yy in enumerate((y + 0.0008, y + t - 0.0008)):
            F.box(f'{name}_cover{k}', w, 0.0016, d, (x, yy, z), m, bevel=0.0006, bevel_segments=1)
        F.box(f'{name}_spine', 0.004, t, d, (x - w / 2 + 0.001, y + t / 2, z), m)
        F.box(f'{name}_band', 0.008, t + 0.002, d + 0.001, (x + w * 0.3, y + t / 2, z), M.pen)
    return _turned(parts, name, (x, y, z), z=turn)


def _pen(name, x, y, z, turn, m=None, length=0.14):
    """A pen lying on `y`, turned `turn` degrees: a barrel, a clip and its tip."""
    c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    a = (x - c * length / 2, y + 0.005, z + s * length / 2)
    b = (x + c * length / 2, y + 0.005, z - s * length / 2)
    F.sweep(name, [a, tuple(p + (q - p) * 0.85 for p, q in zip(a, b)), b], 0.005, m or M.pen, verts=8,
            scale=[1.0, 1.0, 0.3])
    F.sweep(f'{name}_clip', [tuple(p + (q - p) * t + (0, 0.005, 0)[i] for i, (p, q) in enumerate(zip(a, b)))
                             for t in (0.04, 0.35)], 0.0018, M.chrome, verts=4)


def _monitor(name, x, z, w, h, cy, body, screen, wall=None, face=-1, foot_y=FT, foot=(0.22, 0.15)):
    """A flat monitor on its stand, its screen facing `face` (-1: the back, where the desks' chairs sit; 1: the
    front): a thin rounded bezel with a deeper chin, the screen and its picture, a rounded hump behind, a curved
    neck down to a foot. Returns its parts (to join or turn)."""
    with G.new_parts() as parts:
        F.box(name, w, h, 0.022, (x, cy, z), body, bevel=0.008, bevel_segments=1)
        sz = z + face * 0.011
        F.box(f'{name}_screen', w - 0.022, h - 0.04, 0.002, (x, cy + 0.009, sz), screen)
        if wall:
            F.box(f'{name}_wallpaper', w - 0.1, h - 0.1, 0.001, (x, cy + 0.012, sz + face * 0.0012), wall)
        F.box(f'{name}_logo', 0.03, 0.004, 0.001, (x, cy - h / 2 + 0.01, sz), M.chrome)
        F.box(f'{name}_back', w * 0.5, h * 0.55, 0.03, (x, cy - h * 0.05, z - face * 0.02), body, bevel=0.012,
              bevel_segments=2)
        F.sweep(f'{name}_neck', [(x, cy - h * 0.05, z - face * 0.032), (x, (cy + foot_y) / 2 - 0.03, z - face * 0.04),
                                 (x, foot_y + 0.01, z - face * 0.01)],
                None, body, profile=G.rounded_rect(0.016, 0.05, 0.006, corner=1), up=(0, 1, 0), resolution=3, angle=60)
        F.box(f'{name}_foot', foot[0], 0.012, foot[1], (x, foot_y + 0.006, z + face * 0.025), body, bevel=0.005, bevel_segments=1)
    return parts


def _keyboard(name, x, y, z, w, d, body, keys, face=-1, cols=13):
    """A keyboard lying on `y`: a low rounded case, four rows of keys and a row with the space bar towards the
    typist (`face`: -1 the back, 1 the front), the caps in `keys`."""
    F.box(name, w, 0.014, d, (x, y + 0.007, z), body, bevel=0.005, bevel_segments=1)
    pw, pd = (w - 0.024) / cols, (d - 0.024) / 5
    top = y + 0.014
    for r in range(5):
        kz = z + face * (d / 2 - 0.012 - pd * (r + 0.5))
        if r == 0:   # the space bar between modifier keys
            for k, (a, b) in enumerate(((0, 3), (3, cols - 4), (cols - 4, cols - 2), (cols - 2, cols))):
                F.box(f'{name}_key{r}{k}', pw * (b - a) - 0.003, 0.006, pd - 0.003,
                      (x - w / 2 + 0.012 + pw * (a + b) / 2, top + 0.002, kz), keys)
            continue
        for c in range(cols):
            F.box(f'{name}_key{r}{c}', pw - 0.003, 0.006, pd - 0.003, (x - w / 2 + 0.012 + pw * (c + 0.5), top + 0.002, kz),
                  keys)


def _mouse(name, x, y, z, m, face=-1):
    """A mouse on `y`, its buttons away from the hand (`face` the hand's side)."""
    F.sphere(name, 0.03, (x, y + 0.002, z), m, scale=(1.0, 1.75, 0.66), segments=12, rings=8)
    F.box(f'{name}_split', 0.0015, 0.004, 0.03, (x, y + 0.021, z - face * 0.022), M.pad)


def _drawer_box(name, w, h, d, x, y0, z_front, m):
    """An open-topped drawer box behind a front at depth `z_front`, running back (towards +z)."""
    t = 0.01
    zc = z_front + d / 2
    F.box(f'{name}_bottom', w, t, d, (x, y0 + t / 2, zc), m)
    for s in (-1, 1):
        F.box(f'{name}_side{s}', t, h, d, (x + s * (w - t) / 2, y0 + h / 2, zc), m, bevel=0.002, bevel_segments=1)
    F.box(f'{name}_back', w, h, t, (x, y0 + h / 2, z_front + d - t / 2), m)


# --- the desks with their own chair: a top at the front of the tile, a frame, the chair behind -----------------
def computerDesk():
    with G.new_parts() as still:
        _top(1.54, d=0.38, z=0.26)
        # the sides: a sled frame each, bent from one square tube round its foot, legs and top rail
        sq = G.rounded_rect(0.04, 0.04, 0.007, corner=1)
        for s in (-1, 1):
            x = s * 0.72
            loop = G.kit.fillet([(x, 0.02, 0.275), (x, 0.02, 0.43), (x, 0.7, 0.43), (x, 0.7, 0.12), (x, 0.02, 0.12),
                                 (x, 0.02, 0.275)], 0.03, 3)
            F.sweep(f'sled{s}', loop[:-1], None, M.frame, profile=sq, up=(1, 0, 0), closed=True, angle=50)
            for z in (0.12, 0.43):
                F.box(f'glide{s}{z}', 0.04, 0.006, 0.04, (x, 0.003, z), M.plastic, bevel=0.002, bevel_segments=1)
        F.box('rail_back', 1.4, 0.04, 0.02, (0, 0.7, 0.435), M.frame, bevel=0.005, bevel_segments=1)
        # the drawer pedestal hanging under the left, its drawers to the resident, a reveal between them
        F.box('pedestal', 0.36, 0.42, 0.34, (-0.52, 0.48, 0.27), M.drawers, bevel=0.008, bevel_segments=1)
        F.box('pedestal_reveal', 0.33, 0.39, 0.002, (-0.52, 0.48, 0.0995), M.reveal)
        for k, (y, h) in enumerate(((0.6, 0.12), (0.42, 0.2))):
            _front(f'pedestal_front{k}', 0.34, h - 0.006, -0.52, y, 0.092, M.drawers, t=0.014)
            _bar_pull(f'pedestal_pull{k}', -0.52, y + h / 2 - 0.035, 0.085, 0.11, M.frame, out=-0.022, r=0.0045)
        # the runners the keyboard tray hangs from
        for x in (-0.18, 0.5):
            F.box(f'runner{x}', 0.015, 0.055, 0.32, (x, 0.69, 0.25), M.frame, bevel=0.003, bevel_segments=1)
        # the monitor, facing the resident; its back to the room
        _monitor('monitor', 0.05, 0.4, 0.62, 0.37, 1.03, M.monitor, M.screen, M.wallpaper, face=-1, foot=(0.22, 0.14))
        F.sweep('monitor_cable', [(0.05, 0.86, 0.44), (0.08, 0.78, 0.443), (0.2, FT + 0.004, 0.435), (0.3, FT + 0.004, 0.43)],
                0.003, M.pen, verts=5, resolution=3)
        _mug(0.58, 0.3)
        _desk_lamp(-0.6, 0.38)
        _cactus(0.62, 0.38)
        _notebook('notebook', -0.3, FT, 0.26, 0.2, 0.26, 0.015, M.notebook, turn=12)
        _pen('pen', -0.27, FT + 0.015, 0.24, 60)
        _desk_chair(M.chair)
    G.join(still, 'computerDesk', origin=(0, 0, 0))
    # the keyboard tray: under the top, sliding 15 cm out to the resident
    with G.new_parts() as parts:
        F.box('tray', 0.66, 0.012, 0.3, (0.16, 0.655, 0.25), M.tray, bevel=0.004, bevel_segments=1)
        F.box('tray_lip', 0.66, 0.03, 0.012, (0.16, 0.668, 0.106), M.tray, bevel=0.004, bevel_segments=1)
        _keyboard('keyboard', 0.06, 0.661, 0.22, 0.42, 0.13, M.keys, M.keys, face=-1)
        F.box('mouse_pad', 0.18, 0.003, 0.18, (0.38, 0.6625, 0.22), M.pad, bevel=0.0014, bevel_segments=1)
        _mouse('mouse', 0.38, 0.664, 0.22, M.keys)
        F.sweep('mouse_cable', [(0.38, 0.666, 0.272), (0.37, 0.664, 0.33), (0.3, 0.663, 0.37), (0.15, 0.663, 0.38)], 0.002,
                M.keys, verts=4, resolution=3)
    tray = G.join(parts, 'tray')
    G.settle(centre=False)
    G.slide(tray, 'tray_out', (0, 0.15, 0), seconds=0.6)
    return G.meshes()


def deskWriting():
    with G.new_parts() as still:
        _top(1.54)
        _desk_frame(1.54, gap=(-0.1, 0.4))
        F.box('drawer_rails', 0.52, 0.01, 0.38, (0.15, 0.662, 0.24), M.frame)
        for k, (x, z, turn, dy) in enumerate(((-0.12, 0.2, 8, 0.0015), (-0.1, 0.22, -4, 0.0045), (-0.13, 0.19, 15, 0.0075))):
            F.box(f'paper{k}', 0.3, 0.003, 0.21, (x, FT + dy, z), M.paper, rot=(0, 0, turn))
        for j in range(5):   # writing on the top sheet
            F.box(f'line{j}', 0.2 - 0.03 * (j % 2), 0.0004, 0.004, (-0.13, FT + 0.0092, 0.15 + 0.024 * j), M.pen, rot=(0, 0, 15))
        _pen('pen', 0.08, FT, 0.2, 85)
        for k, m in enumerate((M.notebook, M.mug)):
            _notebook(f'notebook{k}', 0.45, FT + 0.025 * k, 0.26, 0.2, 0.27, 0.025, m, turn=6 * k - 3)
        _desk_lamp(-0.57, 0.36)
        _desk_chair(M.chair_sage)
    G.join(still, 'deskWriting', origin=(0, 0, 0))
    # the drawer: a front with a turned knob and a box of letters and pencils, sliding out to the resident
    with G.new_parts() as parts:
        _front('drawer', 0.5, 0.05, 0.15, 0.69, 0.051, M.drawers)
        G.revolve('drawer_knob', [(0, 0), (0.008, 0), (0.007, 0.004), (0.004, 0.008), (0.005, 0.012), (0.011, 0.016),
                                  (0.012, 0.02), (0.009, 0.024), (0, 0.025)], at=F.at(0.15, 0.69, 0.042),
                  material=M.brass, segments=12, rot=(-90, 0, 0))
        _drawer_box('drawer_box', 0.48, 0.042, 0.36, 0.15, 0.668, 0.06, M.drawers)
        for k, (x, turn) in enumerate(((0.04, 5), (0.07, -3))):
            F.box(f'letter{k}', 0.22, 0.006, 0.12, (x, 0.681 + 0.006 * k, 0.2), M.paper, rot=(0, 0, turn))
            F.box(f'stamp{k}', 0.022, 0.001, 0.026, (x + 0.08, 0.6845 + 0.006 * k, 0.24), M.note3, rot=(0, 0, turn))
        for k in range(3):
            px, pz = 0.3, 0.16 + 0.02 * k
            F.cyl(f'pencil{k}', 0.0042, 0.13, (px, 0.68, pz), (M.notebook, M.pen, M.mug)[k], rot=(0, 90, 0), verts=6)
            F.cone(f'pencil_tip{k}', 0.0042, 0.0008, 0.018, (px - 0.074, 0.68, pz), M.folder, rot=(0, -90, 0), verts=6)
    drawer = G.join(parts, 'drawer')
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, 0.22, 0), seconds=0.6)
    return G.meshes()


def deskLaptop():
    with G.new_parts() as still:
        _top(0.9)
        _desk_frame(0.9, gap=(-0.38, -0.02))
        _front('drawer', 0.34, 0.05, -0.2, 0.69, 0.052, M.drawers, t=0.016)
        G.revolve('drawer_knob', [(0, 0), (0.008, 0), (0.007, 0.004), (0.004, 0.008), (0.005, 0.012), (0.01, 0.015),
                                  (0.011, 0.019), (0.008, 0.022), (0, 0.023)], at=F.at(-0.2, 0.69, 0.044),
                  material=M.frame, segments=12, rot=(-90, 0, 0))
        F.box('drawer_back', 0.36, 0.055, 0.3, (-0.2, 0.69, 0.21), M.drawers)
        # the laptop's base: a rounded slab, the keys sunk in a well towards the hinge, the touchpad to the resident
        F.box('laptop', 0.32, 0.012, 0.22, (0.02, FT + 0.006, 0.2), M.aluminium, bevel=0.004, bevel_segments=1)
        _laptop_keys(0.02, FT + 0.012, 0.25)
        F.box('laptop_pad', 0.1, 0.0006, 0.06, (0.02, FT + 0.0123, 0.14), M.pad)
        F.cyl('laptop_hinge', 0.004, 0.24, (0.02, FT + 0.012, 0.309), M.pad, rot=(0, 90, 0), verts=8)
        _mug(0.32, 0.36)
        _notebook('notebook', -0.28, FT, 0.28, 0.18, 0.24, 0.012, M.notebook, turn=-10)
        _pen('pen', -0.27, FT + 0.012, 0.27, 70)
        F.sweep('charger', [(0.18, FT + 0.006, 0.26), (0.24, FT + 0.004, 0.3), (0.3, FT + 0.004, 0.42), (0.33, FT + 0.004, 0.45)],
                0.0025, M.pen, verts=4, resolution=3)
        _desk_chair(M.chair)
    G.join(still, 'deskLaptop', origin=(0, 0, 0))
    # the lid, shut on the base, hinged along the base's far edge
    with G.new_parts() as parts:
        F.box('lid', 0.32, 0.007, 0.22, (0.02, FT + 0.0165, 0.2), M.aluminium, bevel=0.003, bevel_segments=1)
        F.box('lid_bezel', 0.305, 0.0006, 0.205, (0.02, FT + 0.013, 0.2), M.pad)
        F.box('lid_screen', 0.29, 0.0008, 0.18, (0.02, FT + 0.0127, 0.205), M.screen)
        F.cyl('lid_logo', 0.014, 0.0008, (0.02, FT + 0.0201, 0.2), M.chrome, verts=12)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0.02, FT + 0.013, 0.31))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', 110, seconds=0.8, axis='X')
    return G.meshes()


def _laptop_keys(x, y, z):
    """A laptop's keys in their well: six rows of flat caps."""
    F.box('laptop_well', 0.29, 0.0006, 0.105, (x, y + 0.0001, z), M.pad)
    for r in range(6):
        n = 13 if r else 9
        pw = 0.276 / n
        for c in range(n):
            if r == 0 and c == 4:
                continue
            F.box(f'laptop_key{r}{c}', pw - 0.0035, 0.0012, 0.0125, (x - 0.138 + pw * (c + 0.5), y + 0.0006,
                                                                     z - 0.045 + 0.0168 * r), M.keycap)
    F.box('laptop_space', 0.276 / 9 * 1 + 0.06, 0.0012, 0.0125, (x - 0.138 + 0.276 / 9 * 4.5, y + 0.0006, z - 0.045),
          M.keycap)


def computerGaming():
    accent = M.accent
    with G.new_parts() as still:
        _top(1.7, d=0.44, m=M.desk)
        _desk_frame(1.7, legs=M.desk, apron=M.desk)
        F.box('desk_mat', 1.6, 0.004, 0.36, (0, FT + 0.002, 0.22), M.mat, bevel=0.0018, bevel_segments=1)
        F.box('led_strip', 1.62, 0.01, 0.006, (0, FT - 0.012, 0.463), accent)
        # two monitors on stands, turned in to the resident
        for s, scr in ((-1, M.screen2), (1, M.wallpaper)):
            x = s * 0.36
            parts = _monitor(f'monitor{s}', x, 0.36, 0.62, 0.36, 1.03, M.case, scr, face=-1, foot=(0.22, 0.15))
            with G.new_parts() as glow:
                F.box(f'monitor_glow{s}', 0.4, 0.006, 0.006, (x, 0.95, 0.385), accent)
            _turned(parts + glow, f'monitor{s}', (x, 1.0, 0.36), z=-s * 15)
        _keyboard('keyboard', 0, FT + 0.004, 0.17, 0.46, 0.15, M.case, accent, face=-1, cols=14)
        F.box('wrist_rest', 0.44, 0.012, 0.05, (0, FT + 0.01, 0.075), M.mat, bevel=0.005, bevel_segments=1)
        _mouse('mouse', 0.36, FT + 0.004, 0.17, M.case)
        F.box('mouse_light', 0.012, 0.002, 0.03, (0.36, FT + 0.0235, 0.135), accent)
        # the speaker: a rounded box, its woofer and tweeter behind rings
        F.box('speaker', 0.16, 0.2, 0.14, (-0.66, FT + 0.1, 0.36), M.case, bevel=0.014, bevel_segments=2)
        for k, (y, r) in enumerate(((FT + 0.075, 0.048), (FT + 0.155, 0.02))):
            F.cyl(f'driver{k}', r, 0.006, (-0.66, y, 0.288), M.blade, rot=(90, 0, 0), verts=18)
            G.torus(f'driver_ring{k}', r + 0.003, 0.0035, at=F.at(-0.66, y, 0.2885), material=accent, rot=(90, 0, 0),
                    major_segments=18, minor_segments=4)
        F.sphere('woofer', 0.016, (-0.66, FT + 0.075, 0.29), M.case, scale=(1, 0.5, 1), segments=10, rings=5)
        # headphones hung on a hook under the desk's edge
        F.sweep('headset', [(0.8, FT - 0.05, 0.07), (0.8, FT - 0.01, 0.06), (0.8, FT - 0.01, 0.0), (0.8, FT - 0.05, -0.01)],
                0.006, M.case, verts=6, resolution=3)
        for z in (0.07, -0.01):
            F.box(f'earcup{z}', 0.03, 0.07, 0.06, (0.8, FT - 0.08, z), M.case, bevel=0.012, bevel_segments=2)
        # the tower under the desk: a tinted side to the resident, two fans in the front behind a mesh grille
        F.box('tower', 0.24, 0.5, 0.48, (0.62, 0.27, 0.2), M.case, bevel=0.01, bevel_segments=2)
        for s in (-1, 1):
            F.box(f'tower_foot{s}', 0.2, 0.02, 0.04, (0.62, 0.01, 0.2 + s * 0.19), M.plastic, bevel=0.006, bevel_segments=1)
        F.box('tower_side', 0.004, 0.4, 0.38, (0.499, 0.28, 0.2), M.side)
        for k in range(2):   # what shows through the tinted side: the graphics card and the board's glow
            F.box(f'tower_card{k}', 0.004, 0.035, 0.26, (0.503, 0.24 + 0.08 * k, 0.21), M.blade)
        F.box('tower_strip', 0.012, 0.42, 0.004, (0.53, 0.27, 0.441), accent)
        F.box('tower_top_vent', 0.18, 0.002, 0.3, (0.62, 0.5205, 0.2), M.blade)
        for k in range(2):
            F.cyl(f'tower_button{k}', 0.008, 0.004, (0.66 - 0.04 * k, 0.522, 0.4), accent if k == 0 else M.chrome, verts=10)
        for k, y in enumerate((0.38, 0.17)):
            G.torus(f'fan_ring{k}', 0.074, 0.008, at=F.at(0.62, y, 0.442), material=accent, rot=(90, 0, 0),
                    major_segments=20, minor_segments=4)
        _desk_chair(M.chair_gamer, back_h=0.62, head=True, channels=accent)
    G.join(still, 'computerGaming', origin=(0, 0, 0))
    fans = []
    blade = [(0.0, -0.006), (0.03, -0.013), (0.052, -0.002), (0.05, 0.011), (0.026, 0.012)]
    for k, y in enumerate((0.38, 0.17)):
        with G.new_parts() as parts:
            F.cyl(f'fan_hub{k}', 0.022, 0.012, (0.62, y, 0.446), M.blade, rot=(90, 0, 0), verts=16, bevel=0.003,
                  bevel_segments=1)
            F.cyl(f'fan_badge{k}', 0.012, 0.012, (0.62, y, 0.447), accent, rot=(90, 0, 0), verts=12)
            for b in range(7):
                a = b * 360 / 7
                F.extrude(f'blade{k}{b}', blade, 0.003, (0.62, y, 0.444), M.blade, rot=(0, a, 0))
        fans.append(G.hinged(G.join(parts, f'fan{k}'), F.at(0.62, y, 0.446)))
    G.settle(centre=False)
    for fan in fans:
        G.spin(fan, 'fans', turns=1, seconds=1.0, axis='Y')
    return G.meshes()


# --- the catalog2 desks --------------------------------------------------------------------------------------
def deskSimple():
    with G.new_parts():
        _top(1.5, d=0.69, z=-0.085, y=DT)
        # a drawer box hanging under the right, its slab front with a bar pull
        F.box('drawer_box', 0.42, 0.12, 0.6, (0.45, DT - 0.095, -0.12), M.drawers, bevel=0.006, bevel_segments=1)
        F.box('drawer_reveal', 0.39, 0.096, 0.002, (0.45, DT - 0.095, 0.1805), M.reveal)
        _front('drawer_front', 0.4, 0.1, 0.45, DT - 0.095, 0.188, M.drawers, t=0.012)
        _bar_pull('drawer_pull', 0.45, DT - 0.095, 0.194, 0.14, M.frame, out=0.022, r=0.005)
        # hairpin legs: each bent from one rod, from a plate under the top down to its foot and up again
        for x in (-0.68, 0.68):
            for z in (-0.33, 0.17):
                F.box(f'plate{x}{z}', 0.06, 0.006, 0.18, (x, DT - 0.038, z), M.frame, bevel=0.002, bevel_segments=1)
                for dz in (-0.07, 0.07):
                    F.cyl(f'screw{x}{z}{dz}', 0.005, 0.003, (x, DT - 0.0345, z + dz * 1.15), M.chrome, verts=6)
                pin = [(x, DT - 0.04, z - 0.07), (x, 0.012, z - 0.004), (x, 0.012, z + 0.004), (x, DT - 0.04, z + 0.07)]
                F.sweep(f'pin{x}{z}', G.kit.fillet(pin, 0.006, 3), 0.0065, M.frame, verts=8)
                F.sphere(f'pin_foot{x}{z}', 0.012, (x, 0.012, z), M.plastic, segments=10, rings=6)
    G.join_all('deskSimple')
    G.settle(centre=False)
    return G.meshes()


# The corner desk: an L of two 0.7 m wings along the back and the left (the walls of a room's corner), the inner
# corner cut on the diagonal, and its own swivel chair in the inner corner facing it, both wings in reach (content
# `places`: `CORNER_SEAT`, turned to face the corner). Review M-office-01: it was a 1.86 m square slab with one
# corner cut, read as an island, its middle a metre from any edge.
CORNER_WING = 0.7
CORNER_CUT = -0.06      # the diagonal edge: game x + depth = this
CORNER_SEAT = (0.11, 0.11)   # the chair's seat (game x, depth): 0.2 m back from the diagonal edge


def deskCorner():
    e = 0.93
    i = -e + CORNER_WING                     # the wings' inner edges
    with G.new_parts():
        # an L of white laminate, its edges rounded
        cut = [(CORNER_CUT - i, i), (i, CORNER_CUT - i)]
        outline = [(-e, -e), (e, -e), (e, i), *cut, (i, e), (-e, e)]
        F.extrude('top', outline, 0.035, (0, DT - 0.0175, 0), M.laminate, plane='top', bevel=0.006,
                  bevel_segments=2)
        F.cyl('grommet', 0.03, 0.002, (-0.75, DT + 0.0005, -0.75), M.frame, verts=16)
        F.cyl('grommet_hole', 0.024, 0.0025, (-0.75, DT + 0.0006, -0.75), M.reveal, verts=16)
        # oak panels at the outer ends and along the back and the left, and a drawer pedestal under each wing
        h = DT - 0.035
        F.box('panel_left', 0.03, h, 1.8, (-0.89, h / 2, 0), M.oak_panel, bevel=0.004, bevel_segments=1)
        F.box('panel_right', 0.03, h, CORNER_WING - 0.04, (0.89, h / 2, -e + CORNER_WING / 2), M.oak_panel, bevel=0.004,
              bevel_segments=1)
        F.box('panel_front', CORNER_WING - 0.04, h, 0.03, (-e + CORNER_WING / 2, h / 2, 0.89), M.oak_panel, bevel=0.004,
              bevel_segments=1)
        F.box('panel_back', 1.74, 0.4, 0.02, (0, 0.45, -0.89), M.oak_panel, bevel=0.004, bevel_segments=1)
        for k, (x, z, turn) in enumerate(((0.5, -0.6, 0), (-0.6, 0.5, 1))):
            w, d = (0.42, 0.55) if not turn else (0.55, 0.42)
            F.box(f'pedestal{k}', w, 0.6, d, (x, 0.3, z), M.drawers, bevel=0.008, bevel_segments=1)
            # the plinth, set back in shadow, and three slab fronts with reveals and bar pulls
            if turn:
                F.box(f'pedestal{k}_reveal', 0.002, 0.53, 0.39, (x + 0.276, 0.3, z), M.reveal)
            else:
                F.box(f'pedestal{k}_reveal', 0.39, 0.53, 0.002, (x, 0.3, z + 0.276), M.reveal)
            for j in range(3):
                y = 0.5 - j * 0.18
                if turn:   # its drawers to the inner side, towards the chair
                    F.box(f'pedestal{k}_front{j}', 0.014, 0.17, 0.4, (x + 0.282, y, z), M.drawers, bevel=0.004,
                          bevel_segments=1)
                    pts = [(x + 0.286, y + 0.045, z - 0.07), (x + 0.31, y + 0.045, z - 0.07), (x + 0.31, y + 0.045, z + 0.07),
                           (x + 0.286, y + 0.045, z + 0.07)]
                    F.sweep(f'pedestal{k}_pull{j}', G.kit.fillet(pts, 0.009), 0.005, M.frame, verts=8)
                else:
                    F.box(f'pedestal{k}_front{j}', 0.4, 0.17, 0.014, (x, y, z + 0.282), M.drawers, bevel=0.004,
                          bevel_segments=1)
                    _bar_pull(f'pedestal{k}_pull{j}', x, y + 0.045, z + 0.289, 0.14, M.frame, out=0.024)
        # on the top: an open laptop on the back wing turned to the chair, a mug, a tray of papers, a cactus and a notebook
        _laptop_open(0.05, -0.62, DT, turn=-20)
        _mug(0.42, -0.45, y=DT)
        F.box('paper_tray', 0.25, 0.018, 0.32, (-0.62, DT + 0.009, -0.15), M.plastic, bevel=0.004, bevel_segments=1)
        F.box('paper_tray_papers', 0.215, 0.012, 0.29, (-0.62, DT + 0.022, -0.15), M.paper, rot=(0, 0, 3))
        _cactus(-0.78, -0.78, y=DT)
        _notebook('notebook', -0.58, DT, 0.35, 0.18, 0.24, 0.014, M.notebook, turn=20)
        _pen('pen', -0.5, DT + 0.014, 0.32, 40)
    with G.new_parts() as chair:
        _desk_chair(M.chair, z=0.0)
    seat = _turned(chair, 'chair', (0, 0, 0), z=135)
    seat.location.x -= CORNER_SEAT[0]
    seat.location.y -= CORNER_SEAT[1]
    G.join_all('deskCorner')
    G.settle(centre=False)
    return G.meshes()


def _laptop_open(x, z, y, turn=0.0):
    """An open laptop on `y` at (x, z), its typist to the front (+depth) turned `turn` degrees: the base with its
    keys and touchpad, the lid up at 110 degrees on its hinge."""
    with G.new_parts() as parts:
        F.box('laptop', 0.32, 0.012, 0.22, (x, y + 0.006, z), M.aluminium, bevel=0.004, bevel_segments=1)
        F.box('laptop_well', 0.29, 0.0006, 0.105, (x, y + 0.0121, z - 0.04), M.pad)
        for r in range(5):
            F.box(f'laptop_keys{r}', 0.27, 0.0012, 0.013, (x, y + 0.0126, z - 0.08 + 0.0168 * r), M.keycap)
        F.box('laptop_pad', 0.1, 0.0006, 0.06, (x, y + 0.0123, z + 0.06), M.pad)
        with G.new_parts() as lid:
            F.box('lid', 0.32, 0.007, 0.22, (x, y + 0.0165, z), M.aluminium, bevel=0.003, bevel_segments=1)
            F.box('lid_screen', 0.29, 0.0008, 0.18, (x, y + 0.0127, z + 0.005), M.screen)
        _turned(lid, 'laptop_lid', (x, y + 0.013, z - 0.11), x=-110)
    _turned(parts, 'laptop', (x, y, z), z=turn)


def deskStanding():
    with G.new_parts() as still:
        F.soft('top', 0.96, 0.03, 0.63, (0, 1.035, -0.115), M.birch, round=0.008, round_segments=2)
        F.box('crossbar', 0.84, 0.04, 0.06, (0, 0.995, -0.13), M.frame, bevel=0.006, bevel_segments=1)
        for s in (-1, 1):
            x = s * 0.38
            F.box(f'bracket{s}', 0.06, 0.03, 0.56, (x, 1.005, -0.12), M.frame, bevel=0.006, bevel_segments=1)
            # the telescoping leg: a wide outer column, a slimmer one sliding out of it, a collar between
            F.box(f'column{s}', 0.08, 0.55, 0.06, (x, 0.315, -0.13), M.frame, bevel=0.008, bevel_segments=2)
            F.box(f'collar{s}', 0.084, 0.012, 0.064, (x, 0.594, -0.13), M.plastic, bevel=0.004, bevel_segments=1)
            F.box(f'column_top{s}', 0.065, 0.44, 0.048, (x, 0.77, -0.13), M.frame, bevel=0.006, bevel_segments=2)
            # the foot: tapering to both ends, on two levellers
            F.sweep(f'foot{s}', [(x, 0.032, -0.415), (x, 0.038, -0.13), (x, 0.032, 0.175)], None, M.frame,
                    profile=G.rounded_rect(0.042, 0.07, 0.012, corner=2), up=(0, 0, 1), resolution=3,
                    scale=[0.7, 1.0, 0.7], angle=50)
            for z in (-0.39, 0.15):
                F.cyl(f'leveller{s}{z}', 0.014, 0.018, (x, 0.009, z), M.plastic, verts=10)
            F.box(f'runner{s}', 0.015, 0.035, 0.26, (s * 0.31, 1.0, 0.06), M.frame, bevel=0.003, bevel_segments=1)
        # the handset under the front right: a display and four buttons
        F.box('controls', 0.12, 0.022, 0.05, (0.36, 1.007, 0.165), M.plastic, bevel=0.006, bevel_segments=1)
        F.box('controls_display', 0.036, 0.012, 0.002, (0.325, 1.006, 0.1905), M.screen)
        F.box('controls_height', 0.022, 0.005, 0.0004, (0.325, 1.006, 0.1916), M.led)
        for k in range(4):
            F.cyl(f'button{k}', 0.005, 0.003, (0.36 + 0.014 * k, 1.006, 0.191), M.chrome, rot=(90, 0, 0), verts=8)
        # the monitor on its stand at the back, facing the resident, its cable down the left leg
        _monitor('monitor', 0, -0.38, 0.56, 0.34, 1.37, M.plastic, M.screen, M.wallpaper, face=1, foot_y=1.05,
                 foot=(0.2, 0.14))
        F.sweep('cable', [(-0.05, 1.3, -0.415), (-0.2, 1.06, -0.42), (-0.32, 1.0, -0.4), (-0.36, 0.9, -0.165),
                          (-0.36, 0.3, -0.165), (-0.36, 0.05, -0.3), (-0.3, 0.004, -0.41)], 0.004, M.pen, verts=5,
                resolution=3)
        _mug(0.32, -0.3, y=1.05)
        _cactus(-0.36, -0.36, y=1.05, r=0.04, h=0.07)
    G.join(still, 'deskStanding', origin=(0, 0, 0))
    # the keyboard shelf under the front, sliding 20 cm out to the resident
    with G.new_parts() as parts:
        F.box('shelf', 0.6, 0.012, 0.24, (0, 0.976, 0.06), M.shelf, bevel=0.004, bevel_segments=1)
        F.box('shelf_lip', 0.6, 0.026, 0.012, (0, 0.983, 0.184), M.shelf, bevel=0.004, bevel_segments=1)
        _keyboard('keyboard', -0.04, 0.982, 0.08, 0.42, 0.13, M.keys, M.keycap_light, face=1)
        _mouse('mouse', 0.22, 0.983, 0.08, M.keys, face=1)
    shelf = G.join(parts, 'shelf')
    G.settle(centre=False)
    G.slide(shelf, 'tray_out', (0, -0.2, 0), seconds=0.6)
    return G.meshes()


def deskExecutive():
    def drawer_front(name, w, h, x, y, z, pull):
        G.kit.shaker(F, name, w, h, (x, y, z), 0.02, M.walnut2, rail=min(0.045, h * 0.25), raised=True)
        G.kit.pull(F, f'{name}_pull', x, y + pull, z + 0.01, min(0.12, w * 0.3), M.brass, r=0.005, out=0.016)
        for p in (-1, 1):
            F.cyl(f'{name}_rose{p}', 0.009, 0.003, (x + p * min(0.06, w * 0.15), y + pull, z + 0.0115), M.brass,
                  rot=(90, 0, 0), verts=10)

    with G.new_parts() as still:
        F.soft('top', 1.8, 0.05, 0.77, (0, DT - 0.025, -0.045), M.walnut, round=0.014, round_segments=2)
        # the leather pad in the top, a tooled border stitched round it
        F.box('inlay', 1.3, 0.004, 0.46, (0, DT + 0.001, -0.08), M.inlay, bevel=0.0015, bevel_segments=1)
        border = [(u, DT + 0.0032, -0.08 + v) for u, v in G.rounded_rect(1.25, 0.41, 0.02, corner=2)]
        F.sweep('inlay_stitch', border, 0.0012, M.brass, verts=3, closed=True)
        for s in (-1, 1):
            x = s * 0.66
            F.box(f'plinth{s}', 0.42, 0.06, 0.68, (x, 0.03, -0.07), M.walnut_case, bevel=0.006, bevel_segments=1)
            F.box(f'pedestal{s}', 0.44, DT - 0.11, 0.72, (x, 0.06 + (DT - 0.11) / 2, -0.06), M.walnut_case, bevel=0.01,
                  bevel_segments=2)
            # a moulding round the pedestal's foot
            G.kit.moulding(F, f'base_moulding{s}', [(x + 0.22, -0.418), (x + 0.22, 0.3), (x - 0.22, 0.3), (x - 0.22, -0.418)],
                           [(0.0, 0.0), (-0.012, 0.0), (-0.012, 0.02), (-0.004, 0.035), (0.0, 0.04)], M.walnut_case,
                           at=0.06, plane='top')
            for i, (y, h) in enumerate(((0.585, 0.18), (0.385, 0.18), (0.175, 0.2))):
                drawer_front(f'front{s}{i}', 0.4, h - 0.008, x, y, 0.31, h / 2 - 0.045)
        # the modesty panel at the back of the kneehole, framed and panelled
        G.kit.shaker(F, 'modesty', 0.88, 0.45, (0, 0.44, -0.38), 0.025, M.walnut_case, rail=0.07, raised=True)
        F.box('apron', 0.88, 0.07, 0.025, (0, DT - 0.085, -0.3), M.walnut_case, bevel=0.004, bevel_segments=1)
        # a fountain pen and a letter opener on the pad
        _pen('pen', 0.42, DT + 0.003, -0.05, 75)
        F.cyl('pen_tip', 0.0055, 0.03, (0.4805, DT + 0.008, -0.034), M.brass, rot=(90, 0, 75), verts=8)
        F.extrude('opener', G.smooth_outline([(0.0, -0.004), (0.15, -0.002), (0.17, 0.0), (0.15, 0.002), (0.0, 0.006),
                                              (-0.05, 0.006), (-0.06, 0.0), (-0.05, -0.006)], passes=1),
                  0.003, (0.25, DT + 0.0045, 0.0), M.brass, plane='top', rot=(0, 0, 20))
    G.join(still, 'deskExecutive', origin=(0, 0, 0))
    # the middle drawer, still: no use opens it yet
    with G.new_parts() as parts:
        G.kit.shaker(F, 'drawer', 0.84, 0.075, (0, DT - 0.09, 0.31), 0.02, M.walnut2, rail=0.025)
        G.kit.pull(F, 'drawer_pull', 0, DT - 0.09, 0.32, 0.16, M.brass, r=0.006, out=0.015)
        _drawer_box('drawer_box', 0.8, 0.06, 0.5, 0, DT - 0.125, -0.2, M.walnut2)
    G.join(parts, 'drawer')
    G.settle(centre=False)
    return G.meshes()


# --- storage and boards ----------------------------------------------------------------------------------------
def _cabinet_front(name, y, front, label=True):
    """A steel drawer front, pressed with a panel line, a bar handle and a label card in its chrome holder."""
    G.kit.shaker(F, name, 0.42, 0.3, (0, y, front + 0.01), 0.02, M.steel, rail=0.028, recess=0.003)
    G.kit.pull(F, f'{name}_handle', 0, y + 0.06, front + 0.02, 0.15, M.chrome, r=0.006, out=0.012)
    if label:
        holder = [(u, y + 0.11 + v, front + 0.021) for u, v in G.rounded_rect(0.1, 0.04, 0.004, corner=1)]
        F.sweep(f'{name}_holder', holder, 0.0022, M.chrome, verts=4, closed=True)
        F.box(f'{name}_label', 0.09, 0.032, 0.002, (0, y + 0.11, front + 0.0205), M.label)
        F.box(f'{name}_writing', 0.06, 0.004, 0.0004, (0, y + 0.112, front + 0.0217), M.pen)


def _leaves(name, x, y, z, n, length, material, seed=0.0):
    """`n` long leaves arching out of a pot at (x, y, z): each a flat blade swept up and over, narrowing to its tip."""
    for k in range(n):
        a = math.tau * k / n + 0.4 * math.sin(k * 2.3 + seed)
        c, s = math.cos(a), math.sin(a)
        L = length * (0.75 + 0.25 * math.sin(k * 1.7 + seed) ** 2)
        lift = 0.75 + 0.2 * math.cos(k * 1.3)
        pts = [(x + c * 0.01, y, z + s * 0.01), (x + c * L * 0.25, y + L * lift * 0.75, z + s * L * 0.25),
               (x + c * L * 0.55, y + L * lift, z + s * L * 0.55), (x + c * L * 0.85, y + L * lift * 0.82, z + s * L * 0.85)]
        F.sweep(f'{name}{k}', pts, None, material, profile=[(0.0, -0.016), (0.003, 0.0), (0.0, 0.016), (-0.0015, 0.0)],
                up=(0, 0, 1), resolution=2, scale=[0.4, 1.0, 0.8, 0.15])


def filingCabinet():
    z0, d = WALL, 0.6
    front = z0 + d
    with G.new_parts() as still:
        F.box('cabinet', 0.46, 0.975, d, (0, 0.5125, z0 + d / 2), M.steel, bevel=0.01, bevel_segments=2)
        F.box('kick', 0.42, 0.025, d - 0.04, (0, 0.0125, z0 + d / 2 - 0.02), M.reveal, bevel=0.004, bevel_segments=1)
        F.box('top_lip', 0.46, 0.008, d, (0, 0.996, z0 + d / 2), M.steel, bevel=0.003, bevel_segments=1)
        # the drawers' dark gaps, then the two still fronts (the top one moves)
        F.box('gaps', 0.43, 0.94, 0.002, (0, 0.5, front + 0.0005), M.reveal)
        for i, y in enumerate((0.175, 0.5)):
            _cabinet_front(f'front{i}', y, front)
        F.cyl('lock', 0.012, 0.01, (0.17, 0.975, front + 0.005), M.chrome, rot=(90, 0, 0), verts=12)
        F.box('lock_key', 0.003, 0.012, 0.002, (0.17, 0.975, front + 0.0102), M.reveal)
        # a letter tray of papers and a potted plant on top
        y0 = 1.0
        side = [(-0.15, 0.0), (0.15, 0.0), (0.15, 0.03), (0.06, 0.04), (-0.15, 0.06)]
        for s in (-1, 1):
            F.extrude(f'tray_side{s}', side, 0.006, (-0.1 + s * 0.117, y0, -0.2), M.plastic, plane='side')
        F.box('tray_bottom', 0.24, 0.006, 0.3, (-0.1, y0 + 0.003, -0.2), M.plastic)
        F.box('tray_back', 0.24, 0.06, 0.006, (-0.1, y0 + 0.03, -0.347), M.plastic)
        for k in range(3):
            F.box(f'papers{k}', 0.21, 0.004, 0.28, (-0.1 + 0.004 * k, y0 + 0.008 + 0.005 * k, -0.2 - 0.003 * k), M.paper,
                  rot=(0, 0, 2 * k - 2))
        G.revolve('pot', [(0, 0), (0.05, 0), (0.06, 0.09), (0.068, 0.095), (0.068, 0.11), (0.06, 0.11), (0.058, 0.1),
                          (0, 0.1)], at=F.at(0.12, y0, -0.2), material=M.pot, segments=18)
        F.cyl('pot_soil', 0.059, 0.004, (0.12, y0 + 0.1, -0.2), M.soil, verts=18)
        _leaves('leaf', 0.12, y0 + 0.1, -0.2, 9, 0.12, M.leaf, seed=1.0)
    G.join(still, 'filingCabinet', origin=(0, 0, 0))
    # the top drawer with its hanging files behind it, sliding out while a resident files papers
    with G.new_parts() as parts:
        _cabinet_front('drawer', 0.825, front)
        F.box('drawer_bottom', 0.38, 0.015, 0.5, (0, 0.7, front - 0.25), M.steel, bevel=0.003)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.015, 0.22, 0.5, (s * 0.19, 0.8, front - 0.25), M.steel, bevel=0.003)
            F.box(f'drawer_rail{s}', 0.006, 0.008, 0.5, (s * 0.175, 0.905, front - 0.25), M.chrome)
        F.box('drawer_back', 0.38, 0.22, 0.015, (0, 0.8, front - 0.5), M.steel, bevel=0.003)
        tabs = (M.note, M.note4, M.note3, M.note, M.note2, M.note4)
        for k in range(6):
            zf = front - 0.06 - k * 0.07
            F.box(f'file{k}', 0.34, 0.19, 0.008, (0, 0.81, zf), M.folder, bevel=0.002, bevel_segments=1)
            F.box(f'file_hook{k}', 0.36, 0.006, 0.006, (0, 0.905, zf), M.folder)
            F.box(f'file_tab{k}', 0.05, 0.022, 0.009, (-0.12 + 0.05 * (k % 5), 0.915, zf), tabs[k])
            F.box(f'file_papers{k}', 0.3, 0.17, 0.004, (0.005, 0.8, zf + 0.006), M.paper)
    drawer = G.join(parts, 'drawer', origin=F.at(0, 0.825, front))
    G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.3, 0), seconds=1.2)
    return G.meshes()


def _marker(name, x, y, z, ink, turn=0.0):
    """A whiteboard marker lying across at (x, y, z): a barrel, its coloured cap with a clip and an end plug."""
    G.revolve(name, [(0, -0.05), (0.008, -0.05), (0.009, -0.045), (0.009, 0.02), (0.006, 0.03), (0, 0.032)],
              at=F.at(x, y, z), material=M.label, segments=10, rot=(0, 90, turn))
    G.revolve(f'{name}_cap', [(0, 0.012), (0.0098, 0.012), (0.0098, 0.048), (0.008, 0.052), (0, 0.053)],
              at=F.at(x, y, z), material=ink, segments=10, rot=(0, 90, turn))
    F.cyl(f'{name}_plug', 0.0085, 0.004, (x + 0.05, y, z), ink, rot=(0, 90, 0), verts=10)


def whiteboard():
    y, z = 1.4, WALL
    face = z + 0.0145          # the board's writing face
    with G.new_parts() as still:
        F.box('board', 0.88, 0.58, 0.014, (0, y, z + 0.0075), M.board)
        # the frame: one aluminium profile round the board, mitred at the corners, grey caps on them
        frame = [(0.4625, y - 0.3125), (0.4625, y + 0.3125), (-0.4625, y + 0.3125), (-0.4625, y - 0.3125)]
        G.kit.moulding(F, 'frame', frame, [(0.0, 0.0), (0.0, 0.02), (0.006, 0.025), (0.024, 0.025), (0.026, 0.019),
                                           (0.026, 0.0)], M.aluminium, at=z, angle=50)
        for s in (-1, 1):
            for t in (-1, 1):
                F.soft(f'corner{s}{t}', 0.036, 0.036, 0.03, (s * 0.4475, y + t * 0.2975, z + 0.015), M.plastic,
                       round=0.01, round_segments=2)
        # the marker tray on two brackets: a J-shaped trough, three markers and an eraser
        trough = [(0.0, 0.0), (0.0, 0.07), (0.012, 0.07), (0.026, 0.066), (0.026, 0.06), (0.014, 0.062), (0.008, 0.058),
                  (0.006, 0.006)]
        F.sweep('tray', [(-0.3, 1.068, z), (0.3, 1.068, z)], None, M.aluminium,
                profile=[(u, -v) for u, v in trough], up=(0, 0, 1), angle=50)
        for s in (-1, 1):
            F.box(f'tray_end{s}', 0.006, 0.03, 0.07, (s * 0.303, 1.083, z + 0.035), M.plastic, bevel=0.002,
                  bevel_segments=1)
        for k, (x, m) in enumerate(((-0.2, M.ink_red), (-0.12, M.ink_blue), (-0.04, M.ink_black))):
            _marker(f'marker{k}', x + 0.03, 1.084, z + 0.035 + 0.006 * (k - 1), m)
        F.box('eraser', 0.12, 0.022, 0.05, (0.18, 1.093, z + 0.037), M.plastic, bevel=0.008, bevel_segments=2)
        F.box('eraser_felt', 0.116, 0.008, 0.046, (0.18, 1.079, z + 0.037), M.note2)
        # what's on it at rest: a heading, a sticky note and a sheet held up by two magnets
        _scrawl('heading', [(0.1, 1.6), (0.14, 1.606), (0.18, 1.598), (0.22, 1.607), (0.26, 1.6), (0.32, 1.604)], face,
                M.ink_black, 0.004)
        F.box('heading_rule', 0.3, 0.003, 0.002, (0.2, 1.585, face), M.ink_black)
        F.box('sticky', 0.1, 0.1, 0.003, (-0.33, 1.6, face + 0.0005), M.note, rot=(0, 4, 0))
        F.box('sheet', 0.16, 0.21, 0.002, (-0.34, 1.33, face + 0.001), M.paper, rot=(0, -3, 0))
        for k, (x, yy, m) in enumerate(((-0.38, 1.43, M.ink_red), (-0.3, 1.43, M.ink_blue))):
            F.cyl(f'magnet{k}', 0.011, 0.008, (x, yy, face + 0.006), m, rot=(90, 0, 0), verts=12, bevel=0.003,
                  bevel_segments=1)
        for j in range(4):
            F.box(f'sheet_line{j}', 0.11 - 0.02 * (j % 2), 0.003, 0.0004, (-0.345, 1.38 - 0.03 * j, face + 0.0022), M.pen,
                  rot=(0, -3, 0))
    G.join(still, 'whiteboard', origin=(0, 0, 0))
    # the plan, drawn while a resident plans: lines and a chart grow across the board from its left (game +x,
    # as seen from the front), out of nothing: scaled to a point on the left edge at rest
    left = 0.36
    with G.new_parts() as parts:
        for k, (x0, yy, w, m) in enumerate(((0.05, 1.52, 0.3, M.ink_blue), (0.09, 1.48, 0.26, M.ink_blue),
                                            (0.13, 1.44, 0.22, M.ink_blue))):
            # handwriting: a wavering line of loops
            pts = [(x0 + w * i / 16, yy + 0.008 * math.sin(i * 2.1) + 0.004 * math.sin(i * 5.3)) for i in range(17)]
            _scrawl(f'line{k}', pts, face + 0.001, m, 0.0035)
        _scrawl('underline', [(-0.11, 1.32), (0.1, 1.322), (0.35, 1.318)], face + 0.001, M.ink_black, 0.004)
        # an arrow and a ring round a word
        _scrawl('arrow', [(-0.25, 1.52), (-0.15, 1.48), (-0.06, 1.45)], face + 0.001, M.ink_red, 0.004)
        _scrawl('arrow_head', [(-0.08, 1.475), (-0.06, 1.45), (-0.09, 1.442)], face + 0.001, M.ink_red, 0.004)
        ring = [(0.3 + 0.045 * math.cos(math.tau * i / 14), 1.2 + 0.03 * math.sin(math.tau * i / 14)) for i in range(15)]
        _scrawl('ring', ring, face + 0.001, M.ink_blue, 0.0035)
        for k, h in enumerate((0.08, 0.14, 0.11, 0.19)):
            F.box(f'bar{k}', 0.035, h, 0.002, (-0.08 - 0.06 * k, 1.17 + h / 2, face + 0.001), M.ink_red)
        F.box('axis', 0.28, 0.004, 0.002, (-0.17, 1.168, face + 0.0012), M.ink_black)
        F.box('axis_up', 0.004, 0.24, 0.002, (-0.03, 1.29, face + 0.0012), M.ink_black)
    scribble = G.hinged(G.join(parts, 'scribble'), F.at(left, y, face))
    scribble.scale[0] = scribble.scale[2] = 0.001
    G.settle(centre=False, floor=False)
    G.grow(scribble, 'scribble', seconds=1.2, axis='X')
    G.grow(scribble, 'scribble', seconds=1.2, axis='Z')
    return G.meshes()


def _scrawl(name, pts, face, material, w):
    """A pen stroke on a board through game (x, height) points at depth `face`: a flat ribbon `w` wide."""
    return F.sweep(name, [(x, yy, face) for x, yy in pts], None, material,
                   profile=[(-w / 2, 0.0), (0.0, -0.0006), (w / 2, 0.0), (0.0, 0.0006)], up=(0, 0, 1), caps=True)


def _pushpin(name, x, y, face, material):
    """A push pin in a board at depth `face`: a round grip on a collar, its point in the cork."""
    G.revolve(name, [(0, 0), (0.0025, 0), (0.0025, 0.003), (0.006, 0.004), (0.005, 0.008), (0.008, 0.01),
                     (0.0085, 0.0128), (0.006, 0.0144), (0, 0.0149)], at=F.at(x, y, face), material=material, segments=10,
              rot=(90, 0, 0))


def _note(name, x, y, face, w, h, material, turn, curl=0.012):
    """A note pinned at its top: a flat sheet hanging down the board, its foot curling away from it."""
    c, s = math.cos(math.radians(turn)), math.sin(math.radians(turn))
    pts = []
    for i in range(5):
        t = i / 4
        dy = h / 2 - h * t
        out = curl * max(0.0, (t - 0.55) / 0.45) ** 2
        pts.append((x - s * dy, y + c * dy, face + 0.0012 + out))
    return F.sweep(name, pts, None, material, profile=[(-0.0006, -w / 2), (-0.0006, w / 2), (0.0006, w / 2),
                                                        (0.0006, -w / 2)], up=(0, 1, 0), angle=30)


def corkboard():
    y, z = 1.45, WALL
    face = z + 0.016           # the cork's face
    with G.new_parts():
        F.box('cork', 0.68, 0.46, 0.016, (0, y, z + 0.008), M.cork)
        frame = [(0.37, y - 0.26), (0.37, y + 0.26), (-0.37, y + 0.26), (-0.37, y - 0.26)]
        G.kit.moulding(F, 'frame', frame, [(0.0, 0.0), (0.0, 0.022), (0.008, 0.03), (0.026, 0.03), (0.035, 0.022),
                                           (0.035, 0.0)], M.cork_frame, at=z, angle=50)
        for k, (x, yy, m, r, pin) in enumerate((
                (-0.2, 1.55, M.note, 6, M.pin), (0.0, 1.5, M.note2, -4, M.ink_blue), (0.2, 1.56, M.note3, 3, M.pin),
                (-0.15, 1.35, M.note4, -6, M.note), (0.18, 1.34, M.note2, 4, M.pin))):
            _note(f'note{k}', x, yy, face, 0.14, 0.12, m, r)
            _pushpin(f'pin{k}', x - G.sind(r) * 0.045, yy + G.cosd(r) * 0.045, face + 0.0015, pin)
        # a photo, its white border round the picture, and a ticket
        F.box('photo', 0.1, 0.12, 0.002, (-0.02, 1.33, face + 0.002), M.label, rot=(0, 8, 0))
        F.box('photo_picture', 0.084, 0.084, 0.0006, (-0.0185, 1.337, face + 0.0033), M.ink_blue, rot=(0, 8, 0))
        F.box('photo_hill', 0.084, 0.03, 0.0006, (-0.0175, 1.31, face + 0.0036), M.leafy, rot=(0, 8, 0))
        _pushpin('pin_photo', -0.025, 1.38, face + 0.0022, M.ink_red)
        F.box('ticket', 0.12, 0.05, 0.0015, (0.06, 1.62, face + 0.001), M.note4, rot=(0, -10, 0))
        _pushpin('pin_ticket', 0.06, 1.63, face + 0.002, M.note)
        # a few lines on the notes
        for k, (x, yy, r) in enumerate(((-0.2, 1.55, 6), (0.0, 1.5, -4), (0.18, 1.34, 4))):
            for j in range(3):
                dy = 0.02 - 0.025 * j
                F.box(f'scrawl{k}{j}', 0.09 - 0.02 * (j == 2), 0.004, 0.001,
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
     G.Model('deskCorner', deskCorner, size=(1.86, 1.009, 1.86), key='model.deskCorner',
             paint=[('laminate', 'Top'), ('oak_panel', 'Frame'), ('drawers', 'Drawers'), ('chair', 'Chair')]),
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
    comment='Blender-built office models (tools/art/blender/families/office.py): the desks, the desk '
            'chairs, the filing cabinet and the boards. Listed in the base manifest\'s packs. The '
            'materials map onto the game\'s finishes so a model costs one draw call per finish group. The '
            'keyboard shelves slide out (`tray_out`), the writing desk\'s drawer opens (`drawer_open`), '
            'the laptop opens (`lid_open`), the gaming PC\'s fans spin (`fans`), the chairs swivel '
            '(`swivel`) and the whiteboard fills with a plan (`scribble`) while residents use them.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
