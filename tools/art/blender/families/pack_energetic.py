# The energetic pack (P4): the home bouldering wall, the interactive spin bike, the rebound trampoline, the
# streetball hoop, the power blender station, the contrast rain shower and the recovery pod, over their
# primitive-part placeholders (`tools/art/pack_energetic.py`, `packs/energetic/manifest.json`), which stay the
# fallback. Notes: pack_energetic.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The bouldering wall is still (its holds are the shape); the bike's cranks go round while it is
# ridden (`pedals`, a loop the rider rides), the trampoline's mat dips with each bounce (`bounce`, a loop), the
# hoop's net sways (`swish`), the blender's lid lifts and settles (`lid`), the shower's rain grows (`water_on`)
# and the recovery pod's canopy lowers over the resident (`canopy`).
#
# Run:  python3 -B tools/art/blender/build.py pack_energetic [--only trampoline] [--preview]
import math
import os
import sys

import bmesh
import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # the bouldering wall
    plywood=('#E3C79C', 'wood'),          # the climbing panel; its main part
    timber=('#C9A274', 'wood'),
    climb_frame=('#6C7A89', 'metal'),     # the steel A-frame behind
    crash_mat=('#2F6FA8', 'fabric'),
    mat_band=('#F2C14E', 'fabric'),
    chalk=('#E8692E', 'fabric'),
    hold_a=('#E4572E', 'satin'),
    hold_b=('#F2C14E', 'satin'),
    hold_c=('#4CB5AE', 'satin'),
    hold_d=('#7B5EA7', 'satin'),
    hold_e=('#7FB800', 'satin'),
    hold_f=('#F25F9C', 'satin'),
    hold_g=('#3A86FF', 'satin'),
    bolt=('#3A3D41', 'metal'),
    # the spin bike
    bike_frame=('#F2F2F0', 'gloss'),      # the frame; its main part
    bike_pad=('#2F3B4A', 'fabric'),       # the seat and backrest
    bike_grey=('#9EA3A8', 'satin'),
    flywheel=('#E8692E', 'satin'),
    steel=('#C3C7CB', 'metal'),
    dark=('#3A3D41', 'satin'),
    route=('#1E4F7A', 'gloss'),
    route_line=('#7FB800', 'gloss'),
    bottle=('#3A86FF', 'gloss'),
    # the trampoline
    tramp_frame=('#8A9096', 'metal'),     # the frame and legs; its main part
    tramp_pad=('#2F8F83', 'fabric'),      # the safety pad over the springs
    tramp_mat=('#4F6585', 'matte'),
    ball=('#E8692E', 'gloss'),
    ball_seam=('#3A2A20', 'matte'),
    # the streetball hoop
    hoop_base=('#2F5D8A', 'satin'),
    hoop_pole=('#7D858D', 'metal'),       # the pole; its main part
    board=('#F4F4F2', 'gloss'),
    pad_red=('#D7263D', 'satin'),         # the board's border and the pole pad
    rim=('#E8692E', 'metal'),
    net=('#F4F4F2', 'satin'),
    cap=('#E8E8E5', 'satin'),
    # the blender station
    blend_cab=('#DDEFE6', 'satin'),       # the cabinet; its main part
    blend_door=('#CDE6DA', 'satin'),
    plinth=('#B9C7C0', 'satin'),
    worktop=('#F4F4F2', 'gloss'),
    chrome=('#D0D3D6', 'chrome'),
    motor=('#F2F2F0', 'gloss'),
    lime=('#8CC63F', 'gloss'),
    jar=('#E6F0F2', 'glass'),             # the jug and the glasses (glass: the smoothies show through)
    smoothie=('#9BCB5A', 'gloss'),
    drink_pink=('#F07AA0', 'gloss'),
    drink_orange=('#F2A93B', 'gloss'),
    bowl=('#C8956B', 'satin'),
    chopping=('#D9B98C', 'satin'),
    orange=('#E8692E', 'gloss'),
    apple=('#D7263D', 'gloss'),
    banana=('#F2C94C', 'satin'),
    lemon=('#F2E05C', 'satin'),
    # the contrast shower
    shower_frame=('#8E979E', 'metal'),    # the posts and rails; its main part
    tray=('#EEF3F2', 'gloss'),
    grout=('#BFD6D2', 'satin'),
    tile=('#E2F0EE', 'satin'),
    glass=('#D6E8EC', 'glass'),
    fittings=('#D0D3D6', 'chrome'),
    rain_face=('#B9DDF0', 'gloss'),
    hot=('#D9483B', 'gloss'),
    cold=('#3A86FF', 'gloss'),
    shower_mat=('#7FC4C0', 'satin'),
    towel=('#F2F2F0', 'satin'),
    water=('#BFE3F2', 'gloss'),
    # the recovery pod
    pod_shell=('#F2F2F0', 'gloss'),       # the shell; its main part
    pod_glow=('#3FA7A0', 'satin'),
    pod_pad=('#D8DDE2', 'satin'),         # the mattress
    pod_head=('#ECEFF2', 'satin'),
    pod_boots=('#3FA7A0', 'satin'),
    canopy_glass=('#8FB9C9', 'glass'),
    canopy_rim=('#F2F2F0', 'gloss'),
    pod_screen=('#1E4F7A', 'gloss'),
    pod_line=('#7FE0D8', 'gloss'),
)

HOLDS = ('hold_a', 'hold_b', 'hold_c', 'hold_d', 'hold_e', 'hold_f', 'hold_g')   # by name: made anew after each reset
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _obj(name, verts, faces, mats, sharp=35):
    """A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in mats:
        mesh.materials.append(m)
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.set_sharp_from_angle(angle=math.radians(sharp))
    return obj


def _cushion(F, name, w, h, d, at, mat, rot=(0, 0, 0), r=0.05):
    return F.box(name, w, h, d, at, mat, rot=rot, bevel=min(r, w / 2.2, h / 2.2, d / 2.2), bevel_segments=3)


# --- the home bouldering wall (2x1): an overhanging panel with holds on an A-frame, a crash mat before it ------
LEAN = 12          # the panel's top leans forward by this much (an overhang)
FOOT = (0.02, -0.38)   # the panel's foot (height, depth)
PANEL_H = 2.3


def _on_panel(x, up, out=0.0):
    """A game point `up` metres up the panel's face from its foot and `out` metres out of it."""
    a = math.radians(LEAN)
    return (x, FOOT[0] + up * math.cos(a) - out * math.sin(a), FOOT[1] + up * math.sin(a) + out * math.cos(a))


def boulderingWall():
    F = G.GameFrame()
    rot = (LEAN, 0, 0)
    F.box('panel', 1.84, PANEL_H, 0.04, _on_panel(0, PANEL_H / 2), M.plywood, rot=rot, bevel=0.006)
    for x in (-0.9, 0.9):
        F.box(_name('edge'), 0.06, PANEL_H, 0.08, _on_panel(x, PANEL_H / 2, -0.015), M.timber, rot=rot, bevel=0.01)
    F.box('top_edge', 1.86, 0.08, 0.08, _on_panel(0, PANEL_H - 0.04, -0.015), M.timber, rot=rot, bevel=0.01)
    F.box('foot_edge', 1.86, 0.06, 0.08, _on_panel(0, 0.03, -0.015), M.timber, rot=rot, bevel=0.01)
    # the steel A-frame behind it, on the wall
    for x in (-0.84, 0.84):
        F.box(_name('post'), 0.06, 2.2, 0.06, (x, 1.1, WALL + 0.03), M.climb_frame, bevel=0.008)
        for y, up in ((2.12, 2.18), (1.1, 1.15)):
            G.rod(_name('brace'), F.at(x, y, WALL + 0.03), F.at(*_on_panel(x, up, -0.05)), 0.024, material=M.climb_frame,
                  verts=8)
        F.box(_name('post_foot'), 0.12, 0.02, 0.12, (x, 0.01, WALL + 0.06), M.climb_frame, bevel=0.004)
    # the holds: flattened lumps bolted to the face, and three volumes (pyramids) on it
    holds = [(-0.62, 0.35), (-0.18, 0.42), (0.35, 0.3), (0.7, 0.55), (-0.45, 0.75), (0.05, 0.8), (0.52, 0.95),
             (-0.75, 1.1), (-0.25, 1.2), (0.28, 1.3), (0.72, 1.42), (-0.55, 1.55), (0.0, 1.62), (0.45, 1.75),
             (-0.3, 1.95), (0.2, 2.05), (0.66, 2.1), (-0.7, 1.9)]
    for i, (x, up) in enumerate(holds):
        s = 0.035 + (i * 37 % 5) * 0.006
        sx, sy = (1.2, 0.85) if i % 3 else (0.8, 1.15)
        F.sphere(_name('hold'), s, _on_panel(x, up, 0.02), getattr(M, HOLDS[i % len(HOLDS)]), scale=(sx, 0.55, sy),
                 rot=rot, segments=10, rings=6)
        F.cyl(_name('bolt'), 0.008, 0.004, _on_panel(x, up, 0.02 + s * 0.55), M.bolt, rot=(LEAN + 90, 0, 0), verts=6)
    for x, up, m in ((-0.1, 1.0, 'hold_b'), (0.5, 1.95, 'hold_c'), (-0.6, 0.55, 'hold_f')):
        F.cone(_name('volume'), 0.14, 0.0, 0.1, _on_panel(x, up, 0.07), getattr(M, m), rot=(LEAN + 90, 0, 45), verts=3)
    # the crash mat, a chalk bag and a towel on it
    _cushion(F, 'crash_mat', 1.86, 0.22, 0.8, (0, 0.11, 0.04), M.crash_mat, r=0.06)
    F.box('mat_band', 1.87, 0.02, 0.81, (0, 0.11, 0.04), M.mat_band, bevel=0.008)
    G.revolve('chalk_bag', [(0, 0), (0.06, 0), (0.065, 0.12), (0.055, 0.14), (0, 0.14)], at=F.at(0.82, 0.22, 0.32),
              material=M.chalk, segments=14)
    F.box('towel', 0.2, 0.03, 0.12, (-0.7, 0.235, 0.3), M.towel, bevel=0.01)
    G.join_all('boulderingWall')
    return G.settle(centre=False)


# --- the interactive spin bike (1x2): a recumbent trainer, seat at 0.45, cranks in the flywheel's hub --------------
CRANK = (0.36, 0.66)    # the crank axle (height, depth)


def spinBike():
    F = G.GameFrame()
    with G.new_parts() as still:
        _cushion(F, 'seat', 0.44, 0.09, 0.42, (0, 0.405, 0.0), M.bike_pad, r=0.035)
        _cushion(F, 'backrest', 0.42, 0.52, 0.09, (0, 0.72, -0.27), M.bike_pad, rot=(-18, 0, 0), r=0.04)
        F.box('seat_pan', 0.46, 0.04, 0.3, (0, 0.34, -0.02), M.bike_frame, bevel=0.012)
        F.box('seat_post', 0.1, 0.3, 0.12, (0, 0.18, -0.02), M.bike_frame, bevel=0.02)
        G.rod('back_strut', F.at(0, 0.34, -0.15), F.at(0, 0.6, -0.33), 0.022, material=M.bike_frame, verts=10)
        length = math.hypot(1.24, 0.12)
        F.box('beam', 0.1, 0.12, length, (0, 0.12, 0.0), M.bike_frame, rot=(-math.degrees(math.atan2(0.12, 1.24)), 0, 0),
              bevel=0.03)
        F.box('stab_rear', 0.6, 0.05, 0.08, (0, 0.025, -0.66), M.bike_grey, bevel=0.02)
        F.box('stab_front', 0.56, 0.05, 0.08, (0, 0.025, 0.86), M.bike_grey, bevel=0.02)
        for x in (-0.28, 0.28):
            F.cyl(_name('stab_cap'), 0.03, 0.06, (x, 0.03, -0.66), M.dark, rot=(90, 0, 0), verts=10)
        cy, cz = CRANK
        F.cyl('housing', 0.24, 0.16, (0, cy, cz), M.bike_frame, rot=(0, 90, 0), verts=36, bevel=0.02)
        F.cyl('flywheel', 0.19, 0.17, (0, cy, cz), M.flywheel, rot=(0, 90, 0), verts=32)
        F.box('housing_foot', 0.2, 0.32, 0.22, (0, 0.18, cz), M.bike_frame, bevel=0.04)
        for s in (-1, 1):                                   # the side grips by the seat
            G.rod(_name('grip_post'), F.at(s * 0.24, 0.34, 0.12), F.at(s * 0.24, 0.5, 0.12), 0.013, material=M.steel, verts=10)
            G.rod(_name('grip'), F.at(s * 0.24, 0.5, 0.13), F.at(s * 0.24, 0.5, -0.12), 0.017, material=M.dark, verts=10)
        # the console on its stalk, its route screen facing the rider, the handlebar and a bottle
        G.rod('stalk', F.at(0, 0.5, 0.74), F.at(0, 1.0, 0.63), 0.025, material=M.bike_frame, verts=12)
        cc, a = (0, 1.08, 0.6), math.radians(35)
        F.box('console', 0.42, 0.28, 0.04, cc, M.bike_frame, rot=(35, 0, 0), bevel=0.015)
        n = (0, math.sin(a), -math.cos(a))                   # the screen's normal, towards the rider
        F.box('route', 0.38, 0.23, 0.004, (0, cc[1] + n[1] * 0.021, cc[2] + n[2] * 0.021), M.route, rot=(35, 0, 0))
        F.box('route_line', 0.2, 0.05, 0.003, (-0.05, cc[1] + n[1] * 0.024 + 0.03 * math.cos(a), cc[2] + n[2] * 0.024
                                               + 0.03 * math.sin(a)), M.route_line, rot=(35, 0, 0))
        G.rod('handlebar', F.at(-0.22, 0.98, 0.55), F.at(0.22, 0.98, 0.55), 0.016, material=M.dark, verts=10)
        G.revolve('bottle', [(0, 0), (0.035, 0), (0.035, 0.15), (0.02, 0.18), (0.02, 0.2), (0, 0.2)],
                  at=F.at(0.2, 0.33, 0.8), material=M.bottle, segments=14)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cranks and pedals on the flywheel's axle: `pedals` turns them while a resident rides.
    cy, cz = CRANK
    with G.new_parts() as cranks:
        F.cyl('axle', 0.025, 0.24, (0, cy, cz), M.steel, rot=(0, 90, 0), verts=12)
        for s, ang in ((-1, 60), (1, 240)):
            ey, ez = cy + 0.13 * G.sind(ang), cz + 0.13 * G.cosd(ang)
            G.rod(_name('crank'), F.at(s * 0.12, cy, cz), F.at(s * 0.12, ey, ez), 0.014, material=M.steel, verts=8)
            G.rod(_name('spindle'), F.at(s * 0.12, ey, ez), F.at(s * 0.17, ey, ez), 0.007, material=M.steel, verts=6)
            F.box(_name('pedal'), 0.09, 0.025, 0.06, (s * 0.2, ey, ez), M.dark, bevel=0.008)
    cranks = G.hinged(G.join(cranks, 'cranks'), F.at(0, cy, cz))
    objs = G.settle(centre=False)
    G.spin(cranks, 'pedals', 1, seconds=1.0, axis='X')
    return objs


# --- the rebound trampoline (2x2): a round frame on six legs, a padded rim and the mat, a ladder at the front ---
MAT_RIM = 0.6       # the mat's edge, on the springs under the pad
MAT_SAG = 0.02      # its middle sits this much lower at rest; `bounce` stretches the sag to 0.1 (the feet at 0.5)


def trampoline():
    F = G.GameFrame()
    with G.new_parts() as still:
        G.torus('frame', 0.92, 0.022, at=F.at(0, 0.56, 0), material=M.tramp_frame, major_segments=48, minor_segments=8)
        for i in range(6):
            a = i * 60 + 30
            x, z = 0.86 * G.cosd(a), 0.86 * G.sind(a)
            G.rod(_name('leg'), F.at(x * 1.04, 0.03, z * 1.04), F.at(x * 1.07, 0.56, z * 1.07), 0.02, material=M.tramp_frame,
                  verts=10)
            nx, nz = 0.86 * G.cosd(a + 60), 0.86 * G.sind(a + 60)
            G.rod(_name('base'), F.at(x * 1.04, 0.03, z * 1.04), F.at(nx * 1.04, 0.03, nz * 1.04), 0.018,
                  material=M.tramp_frame, verts=8)
        for i in range(36):                                          # the springs, under the pad's edge
            a = i * 10
            G.rod(_name('spring'), F.at(0.8 * G.cosd(a), MAT_RIM - 0.004, 0.8 * G.sind(a)),
                  F.at(0.915 * G.cosd(a), 0.565, 0.915 * G.sind(a)), 0.006, material=M.steel, verts=5)
        prof = [(0.78, 0.6), (0.94, 0.6), (0.95, 0.615), (0.94, 0.632), (0.78, 0.632), (0.775, 0.616), (0.78, 0.6)]
        G.revolve('pad', prof, at=F.at(0, 0, 0), material=M.tramp_pad, segments=48)
        for x in (-0.13, 0.13):                                      # the ladder at the front
            G.rod(_name('ladder'), F.at(x, 0.0, 0.94), F.at(x, 0.56, 0.86), 0.014, material=M.tramp_frame, verts=8)
        for y in (0.18, 0.38):
            F.box(_name('step'), 0.28, 0.025, 0.06, (0, y, 0.94 - y / 0.56 * 0.08), M.dark, bevel=0.008)
        F.sphere('ball', 0.11, (0.8, 0.11, 0.76), M.ball, segments=18, rings=12)
        G.torus('ball_seam', 0.11, 0.003, at=F.at(0.8, 0.11, 0.76), material=M.ball_seam, rot=(90, 0, 30),
                major_segments=20, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The mat, a shallow dish from the springs in, its origin at the rim: `bounce` deepens it and lets it go.
    with G.new_parts() as mat:
        y0 = MAT_RIM
        G.revolve('mat', [(0, y0 - MAT_SAG - 0.008), (0.8, y0 - 0.008), (0.8, y0), (0, y0 - MAT_SAG)], at=F.at(0, 0, 0),
                  material=M.tramp_mat, segments=48)
    mat = G.hinged(G.join(mat, 'mat'), F.at(0, MAT_RIM, 0))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.squash(mat, 'bounce', (1, 1, 5), seconds=0.625, back=True)   # one bounce (the residents bounce 1.6 a second)
    return objs


# --- the streetball hoop (1x1): a weighted base at the back, the pole, the board and the rim over the front -------
RIM = (2.4, 0.22, 0.225)    # the rim's height, depth and radius


def streetballHoop():
    F = G.GameFrame()
    ry, rz, rr = RIM
    with G.new_parts() as still:
        _cushion(F, 'base', 0.6, 0.22, 0.5, (0, 0.11, WALL + 0.25), M.hoop_base, r=0.06)
        F.box('base_top', 0.5, 0.02, 0.4, (0, 0.225, WALL + 0.25), M.hoop_base, bevel=0.01)
        F.cyl('fill_cap', 0.06, 0.04, (0.18, 0.24, -0.3), M.cap, verts=16)
        G.rod('pole', F.at(0, 0.2, -0.28), F.at(0, 2.32, -0.16), 0.045, material=M.hoop_pole, verts=12)
        _cushion(F, 'pole_pad', 0.12, 0.9, 0.12, (0, 0.7, -0.252), M.pad_red, rot=(3.24, 0, 0), r=0.03)
        G.rod('arm', F.at(0, 2.25, -0.2), F.at(0, 2.42, -0.05), 0.03, material=M.hoop_pole, verts=10)
        F.box('board', 0.86, 0.56, 0.04, (0, 2.55, -0.02), M.board, bevel=0.012)
        for y, h in ((2.81, 0.04), (2.29, 0.04)):
            F.box(_name('border'), 0.86, h, 0.045, (0, y, -0.02), M.pad_red, bevel=0.008)
        for x in (-0.41, 0.41):
            F.box(_name('border'), 0.04, 0.56, 0.045, (x, 2.55, -0.02), M.pad_red, bevel=0.008)
        for y in (2.6, 2.42):                                        # the target square
            F.box(_name('target'), 0.3, 0.025, 0.044, (0, y, -0.016), M.pad_red, bevel=0.004)
        for x in (-0.14, 0.14):
            F.box(_name('target'), 0.025, 0.2, 0.044, (x, 2.51, -0.016), M.pad_red, bevel=0.004)
        G.torus('rim', rr, 0.011, at=F.at(0, ry, rz), material=M.rim, major_segments=28, minor_segments=8)
        F.box('rim_bracket', 0.1, 0.03, 0.06, (0, ry, 0.02 + 0.03), M.rim, bevel=0.008)
        F.sphere('ball', 0.12, (0.28, 0.12, 0.3), M.ball, segments=18, rings=12)
        G.torus('ball_seam', 0.12, 0.003, at=F.at(0.28, 0.12, 0.3), material=M.ball_seam, rot=(90, 0, 0),
                major_segments=20, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The net: crossing cords from the rim down to its narrow foot, two rings; `swish` sways it under the rim.
    with G.new_parts() as net:
        top, mid, low = (ry - 0.01, rr - 0.012), (ry - 0.17, rr * 0.82), (ry - 0.34, rr * 0.62)
        for k in range(12):
            a = k * 30
            for t in (-1, 1):
                pts = [(top[0], top[1], a), (mid[0], mid[1], a + t * 15), (low[0], low[1], a)]
                for (y1, r1, a1), (y2, r2, a2) in zip(pts, pts[1:]):
                    G.rod(_name('cord'), F.at(r1 * G.cosd(a1), y1, rz + r1 * G.sind(a1)),
                          F.at(r2 * G.cosd(a2), y2, rz + r2 * G.sind(a2)), 0.004, material=M.net, verts=4)
        for y, r in (mid, low):
            G.torus(_name('net_ring'), r, 0.004, at=F.at(0, y, rz), material=M.net, major_segments=24, minor_segments=4)
    net = G.hinged(G.join(net, 'net'), F.at(0, ry, rz))
    objs = G.settle(centre=False)
    G.rock(net, 'swish', 9, seconds=1.0, axis='X')
    return objs


# --- the power blender station (1x1): a kitchen base unit, the blender, fruit and two smoothies ----------------
TOP = 0.9
JAR = (-0.18, -0.12)   # the blender's middle (x, depth)


def _glass(F, x, z, drink, y=TOP, h=0.12, r=0.035):
    """A glass (see-through, like the jug: the same draw) with a drink in it."""
    G.revolve(_name('drink'), [(0, 0), (r * 0.8, 0), (r * 0.85, h * 0.72), (0, h * 0.72)], at=F.at(x, y + 0.006, z),
              material=drink, segments=14)
    G.revolve(_name('glass'), [(0, 0), (r * 0.88, 0), (r, h), (r * 0.92, h), (r * 0.84, 0.008), (0, 0.008)],
              at=F.at(x, y, z), material=M.jar, segments=16)


def powerBlender():
    F = G.GameFrame()
    w, h, d = 0.82, TOP, 0.59
    z0, t = WALL, 0.035
    zc = z0 + d - 0.035
    with G.new_parts() as still:
        F.box('plinth', w - 0.04, 0.08, d - 0.1, (0, 0.04, z0 + (d - 0.1) / 2), M.plinth, bevel=0.006)
        F.box('carcass', w - 0.01, h - t - 0.08, zc - z0, (0, 0.08 + (h - t - 0.08) / 2, (z0 + zc) / 2), M.blend_cab,
              bevel=0.01)
        F.box('worktop', w, t, d, (0, h - t / 2, z0 + d / 2), M.worktop, bevel=0.008)
        F.box('drawer', w - 0.03, 0.15, 0.02, (0, h - t - 0.095, zc + 0.01), M.blend_door, bevel=0.008)
        F.box('drawer_pull', 0.22, 0.016, 0.016, (0, h - t - 0.095, zc + 0.04), M.chrome, bevel=0.006)
        for x in (-0.08, 0.08):
            F.box(_name('pull_post'), 0.012, 0.012, 0.022, (x, h - t - 0.095, zc + 0.03), M.chrome)
        dw = (w - 0.03) / 2
        for s in (-1, 1):
            cx = s * (dw / 2 + 0.005)
            F.box(_name('door'), dw, h - t - 0.29, 0.02, (cx, 0.1 + (h - t - 0.29) / 2, zc + 0.01), M.blend_door, bevel=0.008)
            hx = cx - s * (dw / 2 - 0.045)
            F.box(_name('handle'), 0.016, 0.14, 0.016, (hx, h - t - 0.29, zc + 0.048), M.chrome, bevel=0.006)
            for y in (h - t - 0.33, h - t - 0.25):
                F.box(_name('handle_post'), 0.012, 0.012, 0.022, (hx, y, zc + 0.03), M.chrome)
        # the blender: motor base, the jug with a green smoothie in it
        jx, jz = JAR
        F.box('motor', 0.18, 0.1, 0.16, (jx, TOP + 0.05, jz), M.motor, bevel=0.03)
        F.box('motor_strip', 0.1, 0.02, 0.004, (jx, TOP + 0.05, jz + 0.081), M.lime, bevel=0.002)
        for dx in (-0.04, 0.0, 0.04):
            F.cyl(_name('dial'), 0.008, 0.008, (jx + dx, TOP + 0.025, jz + 0.082), M.chrome, rot=(90, 0, 0), verts=8)
        F.cyl('jug_seat', 0.07, 0.02, (jx, TOP + 0.11, jz), M.dark, verts=20)
        G.revolve('smoothie', [(0, 0), (0.055, 0), (0.066, 0.16), (0, 0.16)], at=F.at(jx, TOP + 0.13, jz),
                  material=M.smoothie, segments=18)
        G.revolve('jug', [(0, 0), (0.06, 0), (0.075, 0.03), (0.075, 0.22), (0.068, 0.25), (0.064, 0.25),
                          (0.068, 0.22), (0.068, 0.034), (0.054, 0.006), (0, 0.006)], at=F.at(jx, TOP + 0.12, jz),
                  material=M.jar, segments=20)
        G.rod('jug_handle', F.at(jx + 0.075, TOP + 0.33, jz), F.at(jx + 0.12, TOP + 0.24, jz), 0.012, material=M.dark,
              verts=8)
        G.rod('jug_handle_low', F.at(jx + 0.12, TOP + 0.24, jz), F.at(jx + 0.075, TOP + 0.16, jz), 0.012, material=M.dark,
              verts=8)
        # a bowl of fruit, a board with lemon halves, two smoothies in glasses
        G.revolve('bowl', [(0, 0), (0.05, 0), (0.15, 0.08), (0.16, 0.1), (0.15, 0.1), (0.045, 0.012), (0, 0.012)],
                  at=F.at(0.16, TOP, -0.1), material=M.bowl, segments=20)
        for x, y, z, r, m in ((0.1, 0.05, -0.08, 0.04, M.orange), (0.2, 0.05, -0.14, 0.04, M.apple),
                              (0.16, 0.08, -0.05, 0.035, M.lime)):
            F.sphere(_name('fruit'), r, (x, TOP + y, z), m, segments=12, rings=8)
        G.rod('banana_a', F.at(0.1, TOP + 0.1, -0.12), F.at(0.18, TOP + 0.12, -0.08), 0.02, material=M.banana, verts=10)
        G.rod('banana_b', F.at(0.18, TOP + 0.12, -0.08), F.at(0.25, TOP + 0.13, -0.1), 0.018, material=M.banana, verts=10)
        F.box('board', 0.26, 0.015, 0.16, (0.12, TOP + 0.008, 0.12), M.chopping, bevel=0.005)
        for x in (0.08, 0.16):
            F.sphere(_name('lemon'), 0.03, (x, TOP + 0.016, 0.12 + (x - 0.12) * 0.25), M.lemon, scale=(1, 1, 0.5),
                     segments=12, rings=6)
        _glass(F, -0.04, 0.12, M.drink_pink)
        _glass(F, -0.32, 0.12, M.drink_orange)
    G.join(still, 'body', origin=(0, 0, 0))
    # The jug's lid: `lid` lifts it off and, after `hold`, settles it back.
    jx, jz = JAR
    with G.new_parts() as lid:
        F.cyl('lid', 0.07, 0.025, (jx, TOP + 0.38, jz), M.dark, verts=20, bevel=0.006)
        F.cyl('lid_cap', 0.025, 0.03, (jx, TOP + 0.405, jz), M.dark, verts=12, bevel=0.005)
    lid = G.hinged(G.join(lid, 'lid'), F.at(jx, TOP + 0.37, jz))
    objs = G.settle(centre=False)
    G.slide(lid, 'lid', (0, 0, 0.09), seconds=0.5)
    return objs


# --- the contrast rain shower (1x1): a cabin of its own tile, a square rain head, hot and cold dials -----------------
def _tiled_walls(F, x0, x1, z0, z1, y0, y1, t=0.03, size=0.3, gap=0.005):
    """The back wall (game x0..x1 at the back z0) and the left wall (at x0, z0..z1): grout backing, square tiles."""
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
            F.box(_name('tile'), tw - gap, th - gap, 0.006, (x0 + t + tw * (i + 0.5), y, z0 + t + 0.003), M.tile)
        for i in range(deep):
            F.box(_name('tile'), 0.006, th - gap, td - gap, (x0 + t + 0.003, y, z0 + t + td * (i + 0.5)), M.tile)


HEAD = (0.0, 2.0, -0.02)   # the rain head's underside


def contrastShower():
    F = G.GameFrame()
    top, s = 2.15, 0.45
    with G.new_parts() as still:
        F.box('tray', 0.9, 0.058, 0.9, (0, 0.029, 0), M.tray, bevel=0.015)
        F.cyl('drain', 0.04, 0.006, (0.1, 0.06, 0.1), M.fittings, verts=16)
        F.box('mat', 0.6, 0.012, 0.6, (0, 0.064, 0.02), M.shower_mat, bevel=0.004)
        for k in range(5):
            F.box(_name('mat_rib'), 0.56, 0.004, 0.02, (0, 0.071, -0.22 + k * 0.11), M.tray)
        _tiled_walls(F, -s, s, -s, s, 0.058, top)
        F.box('glass_side', 0.01, 2.0, 0.83, (0.435, 1.09, -0.005), M.glass)
        F.box('glass_fixed', 0.46, 2.0, 0.01, (-0.195, 1.09, 0.437), M.glass)
        F.box('glass_door', 0.44, 1.98, 0.01, (0.205, 1.08, 0.423), M.glass)
        for x, z in ((0.435, 0.435), (0.435, -0.42)):
            F.box(_name('post'), 0.025, top - 0.058, 0.025, (x, (top + 0.058) / 2, z), M.shower_frame, bevel=0.004)
        F.box('rail_front', 0.87, 0.03, 0.03, (0.015, top - 0.015, 0.435), M.shower_frame, bevel=0.004)
        F.box('rail_side', 0.03, 0.03, 0.84, (0.435, top - 0.015, 0.0), M.shower_frame, bevel=0.004)
        F.box('rail_bottom', 0.87, 0.02, 0.03, (0.015, 0.068, 0.435), M.shower_frame, bevel=0.004)
        F.box('door_handle', 0.02, 0.4, 0.02, (0.03, 1.05, 0.405), M.shower_frame, bevel=0.006)
        # the square rain head on its arm, the hot and cold dials, a towel on a hook
        hx, hy, hz = HEAD
        G.rod('arm', F.at(0, 2.04, -0.415), F.at(0, 2.04, hz), 0.014, material=M.fittings, verts=10)
        F.box('rain_head', 0.4, 0.025, 0.4, (hx, hy + 0.0125, hz), M.fittings, bevel=0.008)
        F.box('rain_face', 0.36, 0.004, 0.36, (hx, hy - 0.001, hz), M.rain_face, bevel=0.002)
        F.box('dial_plate', 0.34, 0.12, 0.012, (0, 1.15, -0.414), M.fittings, bevel=0.01)
        for x, m in ((-0.1, M.hot), (0.1, M.cold)):
            F.cyl(_name('dial'), 0.04, 0.03, (x, 1.15, -0.395), m, rot=(90, 0, 0), verts=16, bevel=0.006)
            F.box(_name('dial_mark'), 0.008, 0.05, 0.004, (x, 1.165, -0.379), M.fittings)
        F.cyl('hook', 0.008, 0.03, (0.36, 1.42, -0.405), M.fittings, rot=(90, 0, 0), verts=8)
        F.box('towel', 0.04, 0.3, 0.14, (0.36, 1.25, -0.33), M.towel, bevel=0.015)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: a grid of thin streams under the head, origin on the head, scaled to nothing: `water_on` grows them.
    hx, hy, hz = HEAD
    with G.new_parts() as rain:
        for i in range(5):
            for k in range(5):
                F.cyl(_name('rain'), 0.004, hy - 0.07, (hx - 0.14 + i * 0.07, (hy + 0.07) / 2, hz - 0.14 + k * 0.07), M.water,
                      verts=6, smooth=False)
    rain = G.join(rain, 'water', origin=F.at(hx, hy, hz))
    rain.scale[2] = 0.001
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.grow(rain, 'water_on', seconds=0.5)
    return objs


# --- the recovery pod (1x2): a padded lounger (top 0.5) in a shell, a tinted canopy over the head end -------------
CANOPY = (0.42, -0.35, 0.4, 0.4, 0.5)   # the canopy's rim height, middle depth, half-width, height, half-length
CANOPY_UP = 40                          # how far it stands open at rest (degrees)


def recoveryPod():
    F = G.GameFrame()
    with G.new_parts() as still:
        _cushion(F, 'shell', 0.82, 0.29, 1.86, (0, 0.185, 0.0), M.pod_shell, r=0.1)
        F.box('glow_strip', 0.78, 0.03, 1.82, (0, 0.025, 0.0), M.pod_glow, bevel=0.012)
        _cushion(F, 'mattress', 0.68, 0.14, 1.66, (0, 0.43, 0.04), M.pod_pad, r=0.06)
        _cushion(F, 'headrest', 0.5, 0.1, 0.3, (0, 0.53, -0.7), M.pod_head, r=0.045)
        for x in (-0.13, 0.13):
            _cushion(F, _name('boot'), 0.24, 0.12, 0.62, (x, 0.54, 0.5), M.pod_boots, r=0.05)
        F.box('control', 0.06, 0.3, 0.3, (0.38, 0.5, 0.55), M.pod_shell, bevel=0.025)
        F.box('control_screen', 0.004, 0.16, 0.2, (0.412, 0.54, 0.55), M.pod_screen)
        F.box('control_line', 0.004, 0.03, 0.12, (0.414, 0.6, 0.55), M.pod_line)
        for x in (-0.33, 0.33):
            F.box(_name('hinge_block'), 0.06, 0.06, 0.06, (x, 0.42, -0.86), M.pod_shell, bevel=0.012)
    G.join(still, 'body', origin=(0, 0, 0))
    # The canopy: a tinted glass half-shell with a white rim, built lowered, hinged at the head end and turned up
    # to stand open at rest; `canopy` lowers it over the resident.
    ry, cz, hw, hh, hl = CANOPY
    with G.new_parts() as can:
        shell = F.sphere('canopy', 1.0, (0, ry, cz), M.canopy_glass, scale=(hw, hl, hh), segments=32, rings=16)
        bm = bmesh.new()
        bm.from_mesh(shell.data)
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context='VERTS')
        bm.to_mesh(shell.data)
        bm.free()
        sol = shell.modifiers.new('shell', 'SOLIDIFY')
        sol.thickness = 0.008
        rim = G.torus('canopy_rim', 1.0, 0.06, at=F.at(0, ry, cz), material=M.canopy_rim, major_segments=40,
                      minor_segments=6)
        rim.scale = (hw + 0.005, hl + 0.005, 0.4)
    can = G.hinged(G.join(can, 'canopy'), F.at(0, ry, cz - hl))
    G.turn(can, -CANOPY_UP, 'X')
    objs = G.settle(centre=False)
    G.swing(can, 'canopy', CANOPY_UP, seconds=1.2, axis='X')
    return objs


FAMILY = G.Family(
    'pack_energetic', M,
    [G.Model('boulderingWall', boulderingWall, size=(1.87, 2.279, 0.877), key='model.energetic.boulderingWall',
             centre=(0, 0.007), paint=[('climb_frame', 'Frame'), ('crash_mat', 'Pads'), ('plywood', 'Panel')]),
     G.Model('spinBike', spinBike, size=(0.617, 1.2, 1.6), key='model.energetic.spinBike',
             centre=(0, 0.1), motions={'loops': {'ride': 'pedals'}},
             paint=[('bike_frame', 'Frame'), ('bike_pad', 'Pads'), ('flywheel', 'Flywheel')]),
     G.Model('trampoline', trampoline, size=(1.9, 0.635, 1.904), key='model.energetic.trampoline',
             centre=(0, 0.022), motions={'loops': {'bounce': 'bounce'}},
             paint=[('tramp_frame', 'Frame'), ('tramp_pad', 'Pads'), ('tramp_mat', 'Mat')]),
     G.Model('streetballHoop', streetballHoop, size=(0.86, 2.833, 0.886), key='model.energetic.streetballHoop',
             centre=(0, 0.013), motions={'uses': {'shoot': {'clip': 'swish', 'delay': 1.4, 'every': 3.2}}},
             paint=[('hoop_pole', 'Frame'), ('pad_red', 'Pads'), ('hoop_base', 'Base')]),
     G.Model('powerBlender', powerBlender, size=(0.82, 1.32, 0.63), key='model.energetic.powerBlender',
             centre=(0, -0.115), motions={'uses': {'blend': 'lid'}, 'hold': 1.0},
             paint=[('blend_cab', 'Frame'), ('blend_door', 'Doors'), ('motor', 'Blender')]),
     G.Model('contrastShower', contrastShower, size=(0.9, 2.15, 0.9), key='model.energetic.contrastShower',
             centre=(0, 0.02), motions={'uses': {'rinse': 'water_on'}},
             paint=[('shower_frame', 'Frame'), ('tile', 'Tiles'), ('shower_mat', 'Mat')]),
     G.Model('recoveryPod', recoveryPod, size=(0.859, 1.175, 1.86), key='model.energetic.recoveryPod',
             centre=(0, 0), motions={'uses': {'recover': 'canopy'}},
             paint=[('pod_shell', 'Frame'), ('pod_pad', 'Pads'), ('pod_boots', 'Boots')])],
    comment='Blender-built energetic pack models (tools/art/blender/families/pack_energetic.py). Replaces the '
            'primitive-part placeholders of packs/energetic by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The bike\'s cranks turn and the trampoline\'s mat dips while used (loops), the net sways, '
            'the blender\'s lid lifts, the rain grows and the pod\'s canopy lowers; the bouldering wall is still.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
