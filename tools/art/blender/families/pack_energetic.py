# The energetic pack (P4): the home bouldering wall, the interactive spin bike, the rebound trampoline, the
# streetball hoop, the power blender station, the contrast rain shower and the recovery pod. Notes: pack_energetic.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The bouldering wall is still (its holds are the shape); the bike's cranks go round while it is
# ridden (`pedals`, a loop the rider rides), the trampoline's mat dips with each bounce (`bounce`, a loop), the
# hoop's net sways (`swish`), the blender's lid lifts and settles (`lid`), the shower's rain grows (`water_on`)
# and the recovery pod's canopy lowers over the resident (`canopy`).
#
# Built with the forms and the kit (README "The toolkit"): bent tube in one sweep (`kit.fillet`), bodies lofted
# through rounded rings, frame-and-panel fronts (`kit.shaker`) with bar pulls, stuffed and piped pads (`F.soft`),
# lumpy holds (`G.jitter`), a net of strands spiralling down.
#
# Run:  cargo art pack_energetic [--only trampoline] [--preview]
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
    # added with the detail pass
    stem=('#6E5A3A', 'satin'),            # fruit stalks
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


def _paint(obj, material, test):
    """Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material, with no more
    triangles."""
    obj.data.materials.append(material)
    k = len(obj.data.materials) - 1
    for p in obj.data.polygons:
        if test(p.center):
            p.material_index = k
    return obj


def _lofted(F, name, x, y, z, rows, material, verts=12, square=2.0):
    """A body standing on `y` at (x, z), lofted through rings (height, half width, half depth) from its foot up."""
    bx, by, bz = F.at(x, y, z)
    return G.loft(name, [G.ring(rx, rz, bz + h, verts, square=square, at=(bx, by)) for h, rx, rz in rows], material=material)


# --- the home bouldering wall (2x1): an overhanging panel with holds on an A-frame, a crash mat before it ------
LEAN = 12          # the panel's top leans forward by this much (an overhang)
FOOT = (0.02, -0.38)   # the panel's foot (height, depth)
PANEL_H = 2.3


def _on_panel(x, up, out=0.0):
    """A game point `up` metres up the panel's face from its foot and `out` metres out of it."""
    a = math.radians(LEAN)
    return (x, FOOT[0] + up * math.cos(a) - out * math.sin(a), FOOT[1] + up * math.sin(a) + out * math.cos(a))


def _hold(F, i, x, up, s, material):
    """A climbing hold on the panel: a lumpy, flattened lump (a jug, a crimp or a sloper by its shape), its bolt."""
    shape = ((1.3, 0.55, 0.8), (0.8, 0.5, 1.2), (1.0, 0.7, 1.0), (1.5, 0.45, 0.6))[i % 4]
    hold = F.sphere(_name('hold'), s, _on_panel(x, up, 0.012), material, scale=shape, rot=(LEAN, 0, (i * 47) % 90 - 45),
                    segments=8, rings=5)
    G.jitter(hold, s * 0.18, size=s * 1.1, seed=i * 0.77)
    F.cyl(_name('bolt'), 0.008, 0.004, _on_panel(x, up, 0.012 + s * shape[1]), M.bolt, rot=(LEAN + 90, 0, 0), verts=6)


def boulderingWall():
    F = G.GameFrame()
    rot = (LEAN, 0, 0)
    F.box('panel', 1.84, PANEL_H, 0.04, _on_panel(0, PANEL_H / 2), M.plywood, rot=rot, bevel=0.006, bevel_segments=1)
    # the timber frame round its face, one piece each side
    for x in (-0.9, 0.9):
        F.box(_name('edge'), 0.06, PANEL_H, 0.08, _on_panel(x, PANEL_H / 2, -0.015), M.timber, rot=rot, bevel=0.01,
              bevel_segments=1)
    F.box('top_edge', 1.86, 0.08, 0.08, _on_panel(0, PANEL_H - 0.04, -0.015), M.timber, rot=rot, bevel=0.01,
          bevel_segments=1)
    F.box('foot_edge', 1.86, 0.06, 0.08, _on_panel(0, 0.03, -0.015), M.timber, rot=rot, bevel=0.01, bevel_segments=1)
    F.box('seam', 1.8, 0.006, 0.004, _on_panel(0, 1.15, 0.021), M.timber, rot=rot)
    # the T-nut holes in a grid over the face, where holds can be bolted
    for i in range(10):
        for j in range(12):
            x, up = -0.81 + 0.18 * i + 0.09 * (j % 2), 0.12 + 0.18 * j
            if abs(x) < 0.84:
                F.box(_name('tnut'), 0.012, 0.012, 0.004, _on_panel(x, up, 0.0205), M.bolt, rot=rot)
    # the steel A-frame behind it, on the wall: posts on base plates, braces bent onto the panel's back
    for x in (-0.84, 0.84):
        F.box(_name('post'), 0.06, 2.2, 0.06, (x, 1.1, WALL + 0.03), M.climb_frame, bevel=0.008, bevel_segments=1)
        for y, up in ((2.12, 2.18), (1.1, 1.15)):
            G.kit.tube(F, _name('brace'), [(x, y, WALL + 0.03), _on_panel(x, up, -0.05)], 0.022, M.climb_frame, verts=8,
                       round=1)
            F.box(_name('brace_plate'), 0.08, 0.1, 0.012, _on_panel(x, up, -0.026), M.climb_frame, rot=rot)
        F.box(_name('post_foot'), 0.12, 0.02, 0.12, (x, 0.01, WALL + 0.06), M.climb_frame, bevel=0.004, bevel_segments=1)
    # the holds, and three volumes (triangular wedges) bolted on the face
    holds = [(-0.62, 0.35), (-0.18, 0.42), (0.35, 0.3), (0.7, 0.55), (-0.45, 0.75), (0.05, 0.8), (0.52, 0.95),
             (-0.75, 1.1), (-0.25, 1.2), (0.28, 1.3), (0.72, 1.42), (-0.55, 1.55), (0.0, 1.62), (0.45, 1.75),
             (-0.3, 1.95), (0.2, 2.05), (0.66, 2.1), (-0.7, 1.9), (-0.05, 0.15), (0.6, 1.25)]
    for i, (x, up) in enumerate(holds):
        _hold(F, i, x, up, 0.035 + (i * 37 % 5) * 0.006, getattr(M, HOLDS[i % len(HOLDS)]))
    for x, up, m, a in ((-0.1, 1.0, 'hold_b', 0), (0.5, 1.95, 'hold_c', 30), (-0.6, 0.55, 'hold_f', -20)):
        tri = [(0.14 * G.cosd(90 + 120 * k + a), 0.14 * G.sind(90 + 120 * k + a)) for k in range(3)]
        F.extrude(_name('volume'), tri, 0.08, _on_panel(x, up, 0.06), getattr(M, m), rot=(LEAN, 0, 0), bevel=0.012,
                  bevel_segments=1)
    # the crash mat folded in two halves, piped in yellow, carry straps on its front; a chalk bag and a towel on it
    for s in (-1, 1):
        F.soft(_name('crash_mat'), 0.935, 0.22, 0.811, (s * 0.4675, 0.11, 0.0405), M.crash_mat, round=0.05, piping=0.008,
               piping_material=M.mat_band, flat=1, round_segments=1)
        G.kit.tube(F, _name('strap'), G.kit.fillet([(s * 0.465 - 0.12, 0.218, 0.36), (s * 0.465 - 0.1, 0.236, 0.36),
                                                    (s * 0.465 + 0.1, 0.236, 0.36), (s * 0.465 + 0.12, 0.218, 0.36)], 0.02),
                   0.008, M.mat_band, verts=5, round=1)
    _lofted(F, 'chalk_bag', 0.82, 0.22, 0.32, [(0, 0.05, 0.05), (0.01, 0.062, 0.062), (0.1, 0.065, 0.065),
                                                (0.12, 0.058, 0.058), (0.14, 0.06, 0.06)], M.chalk, verts=12)
    G.torus('chalk_rim', 0.06, 0.008, at=F.at(0.82, 0.36, 0.32), material=M.mat_band, major_segments=12, minor_segments=3)
    F.sweep('chalk_brush', [(0.84, 0.33, 0.33), (0.9, 0.42, 0.36)], 0.009, M.timber, verts=6)
    F.soft('towel', 0.2, 0.03, 0.12, (-0.7, 0.235, 0.3), M.towel, round=0.012, puff=0.004, flat=1)
    G.join_all('boulderingWall')
    return G.settle(centre=False)


# --- the interactive spin bike (1x2): a recumbent trainer, seat at 0.45, cranks in the flywheel's hub --------------
CRANK = (0.36, 0.66)    # the crank axle (height, depth)


def spinBike():
    F = G.GameFrame()
    cy, cz = CRANK
    with G.new_parts() as still:
        F.soft('seat', 0.44, 0.08, 0.42, (0, 0.4, 0.0), M.bike_pad, round=0.03, puff=0.01, piping=0.005, flat=2)
        F.soft('backrest', 0.42, 0.52, 0.08, (0, 0.72, -0.27), M.bike_pad, rot=(-18, 0, 0), round=0.035, puff=0.015, flat=2)
        F.extrude('seat_pan', G.rounded_rect(0.46, 0.3, 0.06, corner=2), 0.03, (0, 0.345, -0.02), M.bike_frame,
                  plane='top', bevel=0.01, bevel_segments=1)
        F.sweep('seat_post', [(0, 0.1, -0.02), (0, 0.34, -0.02)], None, M.bike_frame,
                profile=G.rounded_rect(0.1, 0.12, 0.035, corner=2), up=(1, 0, 0))
        F.extrude('back_shell', G.rounded_rect(0.4, 0.5, 0.06, corner=2), 0.02, (0, 0.71, -0.31), M.bike_frame,
                  rot=(-18, 0, 0), bevel=0.006, bevel_segments=1)
        G.kit.tube(F, 'back_strut', G.kit.fillet([(0, 0.34, -0.14), (0, 0.4, -0.3), (0, 0.6, -0.36)], 0.06), 0.022,
                   M.bike_frame, verts=8, round=1)
        # the main beam, curving up from the rear stabiliser to the flywheel's foot
        F.sweep('beam', [(0, 0.05, -0.66), (0, 0.08, -0.2), (0, 0.13, 0.3), (0, 0.18, 0.62)], None, M.bike_frame,
                profile=G.rounded_rect(0.1, 0.11, 0.04, corner=2), up=(1, 0, 0), resolution=2)
        for z, w in ((-0.66, 0.6), (0.86, 0.56)):
            F.sweep(_name('stab'), [(-w / 2 + 0.02, 0.03, z), (w / 2 - 0.02, 0.03, z)], None, M.bike_grey,
                    profile=G.rounded_rect(0.05, 0.08, 0.02, corner=2), up=(0, 0, 1))
            for x in (-w / 2 + 0.0065, w / 2 - 0.0065):
                F.cyl(_name('stab_cap'), 0.032, 0.03, (x, 0.03, z), M.dark, rot=(0, 90, 0), verts=10)
        F.sweep('front_leg', [(0, 0.04, 0.86), (0, 0.12, 0.76), (0, 0.18, 0.66)], None, M.bike_frame,
                profile=G.rounded_rect(0.1, 0.1, 0.035, corner=2), up=(1, 0, 0), resolution=2)
        # the flywheel's rounded housing, the orange wheel showing in each side with its spokes
        G.revolve('housing', [(0, -0.08), (0.2, -0.08), (0.228, -0.074), (0.24, -0.055), (0.24, 0.055), (0.228, 0.074),
                              (0.2, 0.08), (0, 0.08)], at=F.at(0, cy, cz), material=M.bike_frame, segments=28,
                  rot=(0, 90, 0))
        for s in (-1, 1):
            F.cyl(_name('flywheel'), 0.19, 0.006, (s * 0.081, cy, cz), M.flywheel, rot=(0, 90, 0), verts=24)
            for k in range(5):
                a = 72 * k + 18
                F.box(_name('spoke'), 0.004, 0.022, 0.15, (s * 0.085, cy + 0.1 * G.sind(a), cz + 0.1 * G.cosd(a)), M.steel,
                      rot=(-a, 0, 0))
        F.sweep('housing_foot', [(0, 0.04, cz), (0, cy - 0.12, cz)], None, M.bike_frame,
                profile=G.rounded_rect(0.2, 0.22, 0.05, corner=2), up=(1, 0, 0))
        for s in (-1, 1):                                   # the side grips by the seat: bent tube, foam grips
            G.kit.tube(F, _name('grip_post'), G.kit.fillet([(s * 0.2, 0.34, 0.1), (s * 0.24, 0.36, 0.12),
                                                            (s * 0.24, 0.5, 0.12), (s * 0.24, 0.5, -0.12)], 0.03),
                       0.012, M.steel, verts=8, round=1)
            F.sweep(_name('grip'), [(s * 0.24, 0.5, 0.06), (s * 0.24, 0.5, -0.12)], 0.017, M.dark, verts=8)
        # the console on its stalk, its route screen facing the rider, the handlebar and a bottle in its cage
        G.kit.tube(F, 'stalk', [(0, 0.5, 0.74), (0, 0.75, 0.69), (0, 1.0, 0.63)], 0.025, M.bike_frame, verts=10, round=2)
        cc, a = (0, 1.08, 0.6), math.radians(35)
        F.extrude('console', G.rounded_rect(0.42, 0.28, 0.04, corner=2), 0.04, cc, M.bike_frame, rot=(35, 0, 0), bevel=0.01,
                  bevel_segments=1)
        n = (0, math.sin(a), -math.cos(a))                   # the screen's normal, towards the rider
        F.box('route', 0.38, 0.23, 0.004, (0, cc[1] + n[1] * 0.021, cc[2] + n[2] * 0.021), M.route, rot=(35, 0, 0))
        F.box('route_line', 0.2, 0.05, 0.003, (-0.05, cc[1] + n[1] * 0.024 + 0.03 * math.cos(a), cc[2] + n[2] * 0.024
                                               + 0.03 * math.sin(a)), M.route_line, rot=(35, 0, 0))
        G.kit.tube(F, 'handlebar', G.kit.fillet([(-0.24, 1.02, 0.5), (-0.22, 0.98, 0.56), (0.22, 0.98, 0.56),
                                                 (0.24, 1.02, 0.5)], 0.04), 0.014, M.dark, verts=8, round=1)
        _lofted(F, 'bottle', 0.2, 0.33, 0.8, [(0, 0.03, 0.03), (0.005, 0.035, 0.035), (0.14, 0.035, 0.035),
                                              (0.17, 0.022, 0.022), (0.2, 0.02, 0.02)], M.bottle, verts=10)
        F.cyl('bottle_cap', 0.022, 0.03, (0.2, 0.545, 0.8), M.dark, verts=10)
        G.kit.tube(F, 'cage', G.kit.fillet([(0.16, 0.36, 0.8), (0.165, 0.36, 0.84), (0.235, 0.36, 0.84), (0.24, 0.36, 0.8)],
                                           0.02), 0.004, M.steel, verts=4, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cranks and pedals on the flywheel's axle: `pedals` turns them while a resident rides.
    with G.new_parts() as cranks:
        F.cyl('axle', 0.025, 0.24, (0, cy, cz), M.steel, rot=(0, 90, 0), verts=12)
        for s, ang in ((-1, 60), (1, 240)):
            ey, ez = cy + 0.13 * G.sind(ang), cz + 0.13 * G.cosd(ang)
            F.sweep(_name('crank'), [(s * 0.12, cy, cz), (s * 0.122, ey, ez)], None, M.steel,
                    profile=G.rounded_rect(0.014, 0.034, 0.007, corner=1), up=(1, 0, 0), scale=[1.0, 0.75])
            F.cyl(_name('spindle'), 0.007, 0.05, (s * 0.145, ey, ez), M.steel, rot=(0, 90, 0), verts=6)
            F.extrude(_name('pedal'), G.rounded_rect(0.09, 0.06, 0.012, corner=2), 0.022, (s * 0.2, ey, ez), M.dark,
                      plane='top', bevel=0.004, bevel_segments=1)
            F.box(_name('strap'), 0.07, 0.022, 0.014, (s * 0.2, ey + 0.02, ez + 0.01), M.bike_grey, bevel=0.005,
                  bevel_segments=1)
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
        G.torus('frame', 0.92, 0.022, at=F.at(0, 0.56, 0), material=M.tramp_frame, major_segments=40, minor_segments=8)
        # three W legs: each down from the frame, along the grass and up again, in one bent tube
        for k in range(3):
            a0 = k * 120 + 30
            pts = []
            for a, top in ((a0, True), (a0 + 60, False)):
                c, s = G.cosd(a), G.sind(a)
                rim = (0.92 * c, 0.555, 0.92 * s)
                foot = (0.92 * c, 0.02, 0.92 * s)
                pts += [rim, foot] if top else [foot, rim]
            G.kit.tube(F, _name('leg'), G.kit.fillet(pts, 0.06), 0.02, M.tramp_frame, verts=8, round=1)
            for a in (a0, a0 + 60):
                F.cyl(_name('leg_foot'), 0.03, 0.012, (0.92 * G.cosd(a), 0.006, 0.92 * G.sind(a)), M.dark, verts=8)
        for i in range(36):                                          # the springs, under the pad's edge
            a = i * 10
            c, s = G.cosd(a), G.sind(a)
            G.kit.tube(F, _name('spring'), [(0.8 * c, MAT_RIM - 0.004, 0.8 * s), (0.915 * c, 0.565, 0.915 * s)], 0.007,
                       M.steel, verts=4, round=1)
        prof = [(0.78, 0.6), (0.93, 0.6), (0.948, 0.607), (0.952, 0.617), (0.944, 0.628), (0.92, 0.634), (0.79, 0.634),
                (0.776, 0.626), (0.774, 0.608), (0.78, 0.6)]
        G.revolve('pad', prof, at=F.at(0, 0, 0), material=M.tramp_pad, segments=40)
        # the ladder at the front: two bent stiles hooked over the frame, two treads
        for x in (-0.13, 0.13):
            G.kit.tube(F, _name('ladder'), G.kit.fillet([(x, 0.014, 0.94), (x, 0.5, 0.875), (x, 0.59, 0.895), (x, 0.585, 0.93)],
                                                        0.025), 0.014, M.tramp_frame, verts=8, round=1)
        for y in (0.18, 0.36):
            F.extrude(_name('step'), G.rounded_rect(0.3, 0.07, 0.02, corner=2), 0.025, (0, y, 0.94 - y / 0.5 * 0.065),
                      M.dark, plane='top', bevel=0.005, bevel_segments=1)
        F.sphere('ball', 0.11, (0.8, 0.11, 0.76), M.ball, segments=16, rings=10)
        for r in ((90, 0, 30), (0, 90, 30)):
            G.torus(_name('ball_seam'), 0.11, 0.003, at=F.at(0.8, 0.11, 0.76), material=M.ball_seam, rot=r,
                    major_segments=20, minor_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The mat, a shallow dish from the springs in, its origin at the rim, a stitched band round it and a target in
    # its middle: `bounce` deepens it and lets it go.
    with G.new_parts() as mat:
        y0 = MAT_RIM
        prof = [(0, y0 - MAT_SAG - 0.008)] + [(r, y0 - 0.008 - MAT_SAG * (1 - r / 0.8)) for r in (0.08, 0.1, 0.7, 0.73, 0.8)]
        prof += [(0.8, y0)] + [(r, y0 - MAT_SAG * (1 - r / 0.8)) for r in (0.73, 0.7, 0.1, 0.08)] + [(0, y0 - MAT_SAG)]
        dish = G.revolve('mat', prof, at=F.at(0, 0, 0), material=M.tramp_mat, segments=40)
        _paint(dish, M.dark, lambda c: c.z > y0 - MAT_SAG - 0.004 and (0.705 < math.hypot(c.x, c.y) < 0.725 or
                                                                      0.082 < math.hypot(c.x, c.y) < 0.098))
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
        # the blow-moulded base: rounded, tapering up, its fill cap, two wheels at its back
        _lofted(F, 'base', 0, 0, WALL + 0.25, [(0.0, 0.29, 0.24), (0.02, 0.3, 0.25), (0.16, 0.28, 0.23), (0.21, 0.25, 0.2),
                                                (0.23, 0.22, 0.17)], M.hoop_base, verts=24, square=4)
        G.revolve('fill_cap', [(0, 0), (0.06, 0), (0.06, 0.03), (0.05, 0.04), (0, 0.04)], at=F.at(0.18, 0.21, -0.3),
                  material=M.cap, segments=12)
        for s in (-1, 1):
            F.cyl(_name('wheel'), 0.05, 0.04, (s * 0.24, 0.05, WALL + 0.05), M.dark, rot=(0, 90, 0), verts=14)
            F.cyl(_name('hub'), 0.022, 0.044, (s * 0.24, 0.05, WALL + 0.05), M.cap, rot=(0, 90, 0), verts=8)
        # the pole in two lengths, a collar where they meet, its pad, the arm braces up to the board
        G.kit.tube(F, 'pole', [(0, 0.2, -0.28), (0, 1.3, -0.215)], 0.048, M.hoop_pole, verts=12, round=1)
        G.kit.tube(F, 'pole_upper', [(0, 1.26, -0.217), (0, 2.32, -0.16)], 0.04, M.hoop_pole, verts=12, round=1)
        G.revolve('collar', [(0, 0), (0.058, 0), (0.06, 0.01), (0.06, 0.05), (0.055, 0.06), (0, 0.06)],
                  at=F.at(0, 1.27, -0.216), material=M.dark, segments=12)
        G.kit.tube(F, 'collar_knob', [(0, 1.3, -0.16), (0, 1.3, -0.12)], 0.012, M.dark, verts=6, round=1)
        F.soft('pole_pad', 0.12, 0.9, 0.12, (0, 0.7, -0.252), M.pad_red, rot=(3.24, 0, 0), round=0.03, flat=1)
        for x in (-0.1, 0.1):
            G.kit.tube(F, _name('arm'), [(0, 2.1, -0.18), (x, 2.3, -0.06), (x, 2.42, -0.045)], 0.016, M.hoop_pole, verts=8,
                       round=1)
        G.kit.tube(F, 'arm_top', [(0, 2.25, -0.2), (0, 2.42, -0.05)], 0.028, M.hoop_pole, verts=10, round=1)
        # the board: white, a red border round it, the target square over the rim
        F.extrude('board', G.rounded_rect(0.84, 0.54, 0.03, corner=2), 0.035, (0, 2.555, -0.02), M.board, bevel=0.006,
                  bevel_segments=1)
        F.extrude('border', G.rounded_rect(0.86, 0.56, 0.035, corner=2), 0.045, (0, 2.555, -0.02), M.pad_red,
                  holes=[G.rounded_rect(0.79, 0.49, 0.01, corner=1)], bevel=0.008, bevel_segments=1)
        F.extrude('target', G.rounded_rect(0.3, 0.2, 0.004, corner=1), 0.044, (0, 2.51, -0.016), M.pad_red,
                  holes=[G.rounded_rect(0.25, 0.155, 0.002, corner=1)])
        # the rim on its bracket, little hooks round it for the net
        G.torus('rim', rr, 0.011, at=F.at(0, ry, rz), material=M.rim, major_segments=28, minor_segments=6)
        F.extrude('rim_bracket', [(-0.06, 0.0), (0.06, 0.0), (0.04, 0.06), (-0.04, 0.06)], 0.022, (0, ry - 0.011, 0.0),
                  M.rim, plane='top', bevel=0.004, bevel_segments=1)
        F.box('rim_plate', 0.14, 0.12, 0.012, (0, ry - 0.04, 0.0), M.rim, bevel=0.004, bevel_segments=1)
        F.sphere('ball', 0.12, (0.28, 0.12, 0.3), M.ball, segments=16, rings=10)
        for r in ((90, 0, 0), (0, 90, 0), (0, 0, 0)):
            G.torus(_name('ball_seam'), 0.12, 0.003, at=F.at(0.28, 0.12, 0.3), material=M.ball_seam, rot=r,
                    major_segments=20, minor_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The net: strands spiralling down from the rim each way, crossing into diamonds, drawn in at its foot by two
    # rings; `swish` sways it under the rim.
    with G.new_parts() as net:
        levels = [(ry - 0.01, rr - 0.012), (ry - 0.12, rr * 0.88), (ry - 0.23, rr * 0.74), (ry - 0.34, rr * 0.64)]
        for k in range(12):
            for t in (-1, 1):
                pts = [(r * G.cosd(k * 30 + t * 15 * j), y, rz + r * G.sind(k * 30 + t * 15 * j))
                       for j, (y, r) in enumerate(levels)]
                F.sweep(_name('strand'), pts, 0.0035, M.net, verts=4)
        for y, r in levels[1:]:
            G.torus(_name('net_ring'), r, 0.0035, at=F.at(0, y, rz), material=M.net, major_segments=24, minor_segments=3)
    net = G.hinged(G.join(net, 'net'), F.at(0, ry, rz))
    objs = G.settle(centre=False)
    G.rock(net, 'swish', 9, seconds=1.0, axis='X')
    return objs


# --- the power blender station (1x1): a kitchen base unit, the blender, fruit and two smoothies ----------------
TOP = 0.9
JAR = (-0.18, -0.12)   # the blender's middle (x, depth)


def _glass(F, x, z, drink, y=TOP, h=0.12, r=0.035, straw=None):
    """A glass (see-through, like the jug: the same draw) with a drink in it and a straw."""
    G.revolve(_name('drink'), [(0, 0), (r * 0.8, 0), (r * 0.85, h * 0.72), (0, h * 0.72)], at=F.at(x, y + 0.006, z),
              material=drink, segments=12)
    G.revolve(_name('glass'), [(0, 0), (r * 0.88, 0), (r, h), (r * 0.92, h), (r * 0.84, 0.008), (0, 0.008)],
              at=F.at(x, y, z), material=M.jar, segments=14)
    if straw:
        G.kit.tube(F, _name('straw'), [(x + 0.008, y + 0.02, z), (x + 0.02, y + h + 0.05, z - 0.008)], 0.004, straw,
                   verts=5, round=1)


def _banana(F, name, a, b, sag, r, material):
    """A banana from game point `a` to `b`, curving up by `sag` in the middle, tapering to its ends, a dark stalk."""
    mid = tuple((a[i] + b[i]) / 2 + (sag if i == 1 else 0.0) for i in range(3))
    F.sweep(name, [a, mid, b], r, material, verts=6, resolution=3, scale=lambda f: 0.35 + 0.65 * math.sin(math.pi * f))
    F.sweep(f'{name}_stalk', [b, tuple(b[i] + (b[i] - mid[i]) * 0.15 for i in range(3))], r * 0.3, M.stem, verts=4)


def powerBlender():
    F = G.GameFrame()
    w, h, d = 0.82, TOP, 0.59
    z0, t = WALL, 0.035
    zc = z0 + d - 0.035
    with G.new_parts() as still:
        F.box('plinth', w - 0.06, 0.08, d - 0.1, (0, 0.04, z0 + (d - 0.1) / 2), M.plinth, bevel=0.006, bevel_segments=1)
        F.box('carcass', w - 0.01, h - t - 0.08, zc - z0, (0, 0.08 + (h - t - 0.08) / 2, (z0 + zc) / 2), M.blend_cab,
              bevel=0.006, bevel_segments=1)
        # the worktop, its front rolled (postformed)
        F.sweep('worktop', [(-w / 2, h - t / 2, z0 + d / 2), (w / 2, h - t / 2, z0 + d / 2)], None, M.worktop,
                profile=[(-t / 2, d / 2), (t / 2, d / 2), (t / 2, -d / 2 + 0.016), (t / 2 - 0.006, -d / 2 + 0.004),
                         (0.0, -d / 2), (-t / 2 + 0.006, -d / 2 + 0.004), (-t / 2, -d / 2 + 0.014)], up=(0, 0, 1))
        G.kit.shaker(F, 'drawer', w - 0.03, 0.15, (0, h - t - 0.095, zc + 0.01), 0.02, M.blend_door, rail=0.04)
        G.kit.pull(F, 'drawer_pull', 0, h - t - 0.095, zc + 0.02, 0.16, M.chrome, r=0.006, out=0.026)
        dw = (w - 0.03) / 2
        for s in (-1, 1):
            cx = s * (dw / 2 + 0.005)
            dh = h - t - 0.29
            G.kit.shaker(F, _name('door'), dw, dh, (cx, 0.1 + dh / 2, zc + 0.01), 0.02, M.blend_door, rail=0.055)
            hx = cx - s * (dw / 2 - 0.045)
            G.kit.pull(F, _name('handle'), hx, h - t - 0.29, zc + 0.02, 0.12, M.chrome, r=0.006, out=0.026, upright=True)
        # the blender: a motor base with its control strip and dials, the glass jug with its handle and spout
        jx, jz = JAR
        _lofted(F, 'motor', jx, TOP, jz, [(0.0, 0.09, 0.08), (0.06, 0.088, 0.078), (0.1, 0.075, 0.068), (0.11, 0.07, 0.064)],
                M.motor, verts=16, square=4)
        F.box('motor_strip', 0.1, 0.03, 0.004, (jx, TOP + 0.05, jz + 0.08), M.lime, bevel=0.002, bevel_segments=1)
        for dx in (-0.04, 0.0, 0.04):
            G.revolve(_name('dial'), [(0, 0), (0.011, 0), (0.011, 0.008), (0.008, 0.012), (0, 0.012)],
                      at=F.at(jx + dx, TOP + 0.025, jz + 0.08), material=M.chrome, segments=8, rot=(90, 0, 0))
        F.cyl('jug_seat', 0.07, 0.02, (jx, TOP + 0.11, jz), M.dark, verts=16)
        bx, by, bz = F.at(jx, TOP + 0.12, jz)
        G.loft('jug', [G.ring(r, r, bz + hh, 16, square=3.2, at=(bx, by)) for hh, r in ((0.0, 0.058), (0.03, 0.074),
                                                                                       (0.22, 0.074), (0.25, 0.07))],
               material=M.jar)
        F.cone('jug_spout', 0.02, 0.006, 0.03, (jx - 0.075, TOP + 0.36, jz), M.jar, rot=(0, 55, 0), verts=8)
        F.sweep('jug_handle', G.kit.fillet([(jx + 0.072, TOP + 0.33, jz), (jx + 0.12, TOP + 0.31, jz), (jx + 0.125, TOP + 0.2, jz),
                                            (jx + 0.075, TOP + 0.16, jz)], 0.03, steps=2), None, M.dark,
                profile=G.rounded_rect(0.024, 0.014, 0.005, corner=1), up=(0, 1, 0))
        # a fruit bowl with oranges, an apple, a lime and two bananas; a board with lemon halves and a knife
        bowl = [(0.0, 0.05, 0.05), (0.008, 0.06, 0.06), (0.05, 0.13, 0.13), (0.09, 0.16, 0.16), (0.1, 0.162, 0.162)]
        _lofted(F, 'bowl', 0.16, TOP, -0.1, bowl, M.bowl, verts=18)
        for x, y, z, r, m in ((0.1, 0.12, -0.08, 0.04, M.orange), (0.2, 0.12, -0.14, 0.04, M.apple),
                              (0.16, 0.14, -0.04, 0.035, M.lime), (0.22, 0.12, -0.05, 0.038, M.orange)):
            F.sphere(_name('fruit'), r, (x, TOP + y, z), m, segments=10, rings=6)
            F.cyl(_name('fruit_stalk'), 0.003, 0.012, (x, TOP + y + r, z), M.stem, verts=4)
        _banana(F, 'banana_a', (0.06, TOP + 0.15, -0.14), (0.24, TOP + 0.16, -0.1), 0.03, 0.02, M.banana)
        _banana(F, 'banana_b', (0.07, TOP + 0.17, -0.12), (0.25, TOP + 0.18, -0.07), 0.035, 0.018, M.banana)
        F.extrude('board', G.rounded_rect(0.26, 0.16, 0.02, corner=2), 0.015, (0.12, TOP + 0.008, 0.12), M.chopping,
                  plane='top', holes=[G.rounded_rect(0.03, 0.012, 0.006, corner=2)], bevel=0.003, bevel_segments=1)
        for x in (0.08, 0.16):
            F.cyl(_name('lemon_rind'), 0.032, 0.012, (x, TOP + 0.021, 0.12 + (x - 0.12) * 0.25), M.lemon, verts=12)
            F.cyl(_name('lemon_face'), 0.027, 0.002, (x, TOP + 0.0275, 0.12 + (x - 0.12) * 0.25), M.worktop, verts=12)
        F.box('knife', 0.13, 0.003, 0.022, (0.15, TOP + 0.017, 0.17), M.chrome, rot=(0, 0, 8))
        F.box('knife_grip', 0.07, 0.014, 0.018, (0.26, TOP + 0.022, 0.155), M.dark, rot=(0, 0, 8), bevel=0.005,
              bevel_segments=1)
        _glass(F, -0.04, 0.12, M.drink_pink, straw=M.lime)
        _glass(F, -0.32, 0.12, M.drink_orange, straw=M.apple)
    G.join(still, 'body', origin=(0, 0, 0))
    # The jug's lid: `lid` lifts it off and, after `hold`, settles it back.
    jx, jz = JAR
    with G.new_parts() as lid:
        G.revolve('lid', [(0, 0), (0.072, 0), (0.074, 0.012), (0.07, 0.022), (0.03, 0.026), (0, 0.026)],
                  at=F.at(jx, TOP + 0.368, jz), material=M.dark, segments=16)
        G.revolve('lid_cap', [(0, 0), (0.026, 0), (0.026, 0.024), (0.02, 0.032), (0, 0.032)],
                  at=F.at(jx, TOP + 0.392, jz), material=M.dark, segments=10)
    lid = G.hinged(G.join(lid, 'lid'), F.at(jx, TOP + 0.37, jz))
    # The green smoothie in the glass jug, with streaks of fruit through it and bits at the top, so its swirl shows:
    # `blades` turns it once the lid is back on (the blades themselves would be hidden in it).
    with G.new_parts() as swirl:
        bx, by, bz = F.at(jx, TOP + 0.13, jz)
        G.loft('smoothie', [G.ring(r, r, bz + hh, 14, square=3.2, at=(bx, by)) for hh, r in ((0.0, 0.054), (0.02, 0.066),
                                                                                           (0.16, 0.066))],
               material=M.smoothie)
        for k, (a, m) in enumerate(((20, M.orange), (150, M.apple), (260, M.lime))):
            r = 0.062
            F.box(_name('streak'), 0.016, 0.12, 0.006, (jx + r * G.cosd(a), TOP + 0.21, jz + r * G.sind(a)), m,
                  rot=(0, -a + 90, 0))
            F.sphere(_name('bit'), 0.014, (jx + 0.035 * G.cosd(a + 60), TOP + 0.29, jz + 0.035 * G.sind(a + 60)), m,
                     segments=6, rings=4)
    swirl = G.join(swirl, 'smoothie', origin=F.at(jx, TOP + 0.13, jz))
    objs = G.settle(centre=False)
    G.slide(lid, 'lid', (0, 0, 0.09), seconds=0.5)
    G.spin(swirl, 'blades', turns=1, seconds=1.0)
    return objs


# --- the contrast rain shower (1x1): a cabin of its own tile, a square rain head, hot and cold dials -----------------
def _tiled_walls(F, x0, x1, z0, z1, y0, y1, t=0.03, size=0.3, gap=0.005):
    """The back wall (game x0..x1 at the back z0) and the left wall (at x0, z0..z1): grout backing, square tiles with
    softened edges."""
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
            F.box(_name('tile'), tw - gap, th - gap, 0.008, (x0 + t + tw * (i + 0.5), y, z0 + t + 0.004), M.tile,
                  bevel=0.002, bevel_segments=1)
        for i in range(deep):
            F.box(_name('tile'), 0.008, th - gap, td - gap, (x0 + t + 0.004, y, z0 + t + td * (i + 0.5)), M.tile,
                  bevel=0.002, bevel_segments=1)


HEAD = (0.0, 2.0, -0.02)   # the rain head's underside
DOOR_HALF = 0.28           # half the glass door's width, across the middle of the front


def contrastShower():
    F = G.GameFrame()
    top, s = 2.15, 0.45
    with G.new_parts() as still:
        F.extrude('tray', G.rounded_rect(0.9, 0.9, 0.03, corner=2), 0.058, (0, 0.029, 0), M.tray, plane='top', bevel=0.012,
                  bevel_segments=1)
        G.revolve('drain', [(0, 0), (0.045, 0), (0.045, 0.004), (0.038, 0.006), (0, 0.006)], at=F.at(0.1, 0.057, 0.1),
                  material=M.fittings, segments=14)
        for k in range(-2, 3):
            F.box(_name('drain_slot'), 0.05, 0.002, 0.004, (0.1, 0.0635, 0.1 + 0.012 * k), M.grout)
        F.extrude('mat', G.rounded_rect(0.6, 0.6, 0.04, corner=2), 0.012, (0, 0.064, 0.02), M.shower_mat, plane='top',
                  bevel=0.003, bevel_segments=1)
        for k in range(5):
            F.box(_name('mat_rib'), 0.54, 0.004, 0.02, (0, 0.071, -0.22 + k * 0.11), M.tray)
        _tiled_walls(F, -s, s, -s, s, 0.058, top)
        F.box('glass_side', 0.01, 2.0, 0.83, (0.435, 1.09, -0.005), M.glass)
        # a narrow fixed pane either side of the door across the middle (the door its own part, below)
        for x0, x1 in ((-0.414, -DOOR_HALF - 0.004), (DOOR_HALF + 0.004, 0.4225)):
            F.box(_name('glass_fixed'), x1 - x0, 2.0, 0.01, ((x0 + x1) / 2, 1.09, 0.437), M.glass)
        prof = G.rounded_rect(0.025, 0.025, 0.006, corner=1)
        for x, z in ((0.435, 0.435), (0.435, -0.42)):
            F.sweep(_name('post'), [(x, 0.058, z), (x, top, z)], None, M.shower_frame, profile=prof, up=(1, 0, 0))
        F.sweep('rail_top', [(-0.42, top - 0.015, 0.435), (0.435, top - 0.015, 0.435), (0.435, top - 0.015, -0.42)], None,
                M.shower_frame, profile=G.rounded_rect(0.03, 0.03, 0.006, corner=1), up=(0, 0, 1))
        F.box('rail_bottom', 0.87, 0.02, 0.03, (0.015, 0.068, 0.435), M.shower_frame, bevel=0.004, bevel_segments=1)
        for y in (0.3, 1.85):                                           # the door's hinges on the right pane
            F.extrude(_name('hinge'), G.rounded_rect(0.05, 0.05, 0.01, corner=2), 0.026, (DOOR_HALF + 0.01, y, 0.437),
                      M.shower_frame, bevel=0.003, bevel_segments=1)
        # the square rain head on its arm from the wall, the hot and cold dials on their plate, a shelf of bottles
        hx, hy, hz = HEAD
        G.kit.tube(F, 'arm', G.kit.fillet([(0, 2.1, -0.415), (0, 2.1, -0.3), (0, 2.04, -0.25), (0, 2.04, hz)], 0.04), 0.014,
                   M.fittings, verts=8, round=1)
        G.revolve('arm_rose', [(0, 0), (0.035, 0), (0.035, 0.008), (0.02, 0.016), (0, 0.016)], at=F.at(0, 2.1, -0.415),
                  material=M.fittings, segments=12, rot=(90, 0, 0))
        F.extrude('rain_head', G.rounded_rect(0.4, 0.4, 0.05, corner=2), 0.025, (hx, hy + 0.0125, hz), M.fittings,
                  plane='top', bevel=0.006, bevel_segments=1)
        F.extrude('rain_face', G.rounded_rect(0.36, 0.36, 0.04, corner=2), 0.004, (hx, hy - 0.001, hz), M.rain_face,
                  plane='top')
        for i in range(5):
            for k in range(5):
                F.box(_name('nozzle'), 0.01, 0.003, 0.01, (hx - 0.14 + i * 0.07, hy - 0.003, hz - 0.14 + k * 0.07), M.fittings)
        F.extrude('dial_plate', G.rounded_rect(0.34, 0.12, 0.05, corner=3), 0.012, (0, 1.15, -0.414), M.fittings,
                  bevel=0.004, bevel_segments=1)
        for x, m in ((-0.1, M.hot), (0.1, M.cold)):
            G.revolve(_name('dial'), [(0, 0), (0.042, 0), (0.042, 0.012), (0.036, 0.026), (0.03, 0.03), (0, 0.03)],
                      at=F.at(x, 1.15, -0.408), material=m, segments=14, rot=(90, 0, 0))
            F.box(_name('dial_mark'), 0.008, 0.05, 0.006, (x, 1.165, -0.376), M.fittings)
        F.box('niche_shelf', 0.3, 0.015, 0.1, (-0.2, 1.4, -0.365), M.tile, bevel=0.003, bevel_segments=1)
        for k, (x, m, hh) in enumerate(((-0.29, M.hot, 0.16), (-0.22, M.cold, 0.13), (-0.15, M.shower_mat, 0.15))):
            _lofted(F, _name('bottle'), x, 1.4075, -0.37, [(0, 0.024, 0.016), (0.004, 0.026, 0.018), (hh - 0.02, 0.026, 0.018),
                                                           (hh, 0.014, 0.012)], m, verts=10, square=3)
            F.cyl(_name('bottle_cap'), 0.012, 0.02, (x, 1.4075 + hh + 0.01, -0.37), M.towel, verts=8)
        # a towel hanging from a hook on the back wall
        F.cyl('hook', 0.008, 0.03, (0.36, 1.42, -0.405), M.fittings, rot=(90, 0, 0), verts=8)
        towel = F.sweep('towel', G.kit.fillet([(0.36, 1.06, -0.4), (0.36, 1.42, -0.402), (0.36, 1.43, -0.38),
                                                (0.36, 1.12, -0.36)], 0.012, steps=2), None, M.towel,
                        profile=G.rounded_rect(0.012, 0.16, 0.005, corner=1), up=(0, -1, 0))
        G.jitter(towel, 0.004, size=0.08, seed=5.0, up=0.3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: a grid of thin streams under the head, origin on the head, scaled to nothing: `water_on` grows them.
    with G.new_parts() as rain:
        for i in range(5):
            for k in range(5):
                F.cyl(_name('rain'), 0.004, hy - 0.07, (hx - 0.14 + i * 0.07, (hy + 0.07) / 2, hz - 0.14 + k * 0.07), M.water,
                      verts=6, smooth=False)
    rain = G.join(rain, 'water', origin=F.at(hx, hy, hz))
    rain.scale[2] = 0.001
    # The glass door across the middle of the front, hinged on its right: `door_open` swings it out to let the
    # bather in and out (a fourth draw: Q-05). All glass, no handle, to keep it one draw.
    with G.new_parts() as door:
        F.box('glass_door', 2 * DOOR_HALF, 1.98, 0.01, (0, 1.08, 0.437), M.glass)
    door = G.hinged(G.join(door, 'door'), F.at(DOOR_HALF, 0.09, 0.437))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.grow(rain, 'water_on', seconds=0.5)
    G.swing(door, 'door_open', -95, seconds=0.8)
    return objs


# --- the recovery pod (1x2): a padded lounger (top 0.5) in a shell, a tinted canopy over the head end -------------
CANOPY = (0.42, -0.35, 0.4, 0.4, 0.5)   # the canopy's rim height, middle depth, half-width, height, half-length
CANOPY_UP = 40                          # how far it stands open at rest (degrees)


def _boot(F, x, y, z):
    """A compression boot lying along the mattress, toe to the front: a lofted leg sleeve, quilted in rings."""
    bx, by, bz = F.at(x, y, z)
    rows = [(-0.31, 0.04, 0.035), (-0.29, 0.1, 0.055), (-0.1, 0.11, 0.06), (0.12, 0.1, 0.06), (0.24, 0.09, 0.07),
            (0.29, 0.07, 0.065), (0.31, 0.02, 0.03)]
    secs = [[(bx + rx * math.cos(math.tau * k / 12), by - u, bz + rz * math.sin(math.tau * k / 12)) for k in range(12)]
            for u, rx, rz in rows]
    G.loft(_name('boot'), secs, material=M.pod_boots)
    for u in (-0.2, -0.06, 0.08):
        G.torus(_name('boot_quilt'), 1.0, 0.006, at=(bx, by - u, bz), material=M.pod_boots, rot=(90, 0, 0),
                major_segments=12, minor_segments=3).scale = (0.112, 0.062, 1.0)


def recoveryPod():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the shell: a tub lofted through rounded rings, a glowing strip round its foot
        _lofted(F, 'shell', 0, 0.04, 0.0, [(0.0, 0.39, 0.91), (0.02, 0.405, 0.925), (0.16, 0.41, 0.93), (0.27, 0.4, 0.92),
                                           (0.29, 0.385, 0.905)], M.pod_shell, verts=32, square=7)
        F.sweep('glow_strip', [(x, 0.025, z) for x, z in G.rounded_rect(0.78, 1.82, 0.08, corner=2)], 0.015, M.pod_glow,
                verts=5, closed=True)
        F.soft('mattress', 0.68, 0.132, 1.66, (0, 0.426, 0.04), M.pod_pad, round=0.05, puff=0.008, piping=0.006, flat=2)
        F.soft('headrest', 0.5, 0.1, 0.3, (0, 0.53, -0.7), M.pod_head, round=0.04, puff=0.012, flat=2)
        for x in (-0.13, 0.13):
            _boot(F, x, 0.56, 0.5)
        F.extrude('control', G.rounded_rect(0.3, 0.3, 0.05, corner=2), 0.06, (0.38, 0.5, 0.55), M.pod_shell, plane='side',
                  bevel=0.015, bevel_segments=1)
        F.box('control_screen', 0.004, 0.16, 0.2, (0.412, 0.54, 0.55), M.pod_screen)
        F.box('control_line', 0.004, 0.03, 0.12, (0.414, 0.6, 0.55), M.pod_line)
        for k in range(3):
            F.cyl(_name('control_key'), 0.012, 0.004, (0.412, 0.44, 0.49 + 0.06 * k), M.pod_line, rot=(0, 90, 0), verts=8)
        for x in (-0.33, 0.33):
            F.extrude(_name('hinge_block'), G.rounded_rect(0.06, 0.07, 0.02, corner=2), 0.06, (x, 0.42, -0.86), M.pod_shell,
                      plane='side', bevel=0.008, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The canopy: a tinted glass half-shell with a white rim, built lowered, hinged at the head end and turned up
    # to stand open at rest; `canopy` lowers it over the resident.
    ry, cz, hw, hh, hl = CANOPY
    with G.new_parts() as can:
        shell = F.sphere('canopy', 1.0, (0, ry, cz), M.canopy_glass, scale=(hw, hl, hh), segments=28, rings=14)
        bm = bmesh.new()
        bm.from_mesh(shell.data)
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context='VERTS')
        bm.to_mesh(shell.data)
        bm.free()
        sol = shell.modifiers.new('shell', 'SOLIDIFY')
        sol.thickness = 0.008
        rim = G.torus('canopy_rim', 1.0, 0.06, at=F.at(0, ry, cz), material=M.canopy_rim, major_segments=32,
                      minor_segments=5)
        rim.scale = (hw + 0.005, hl + 0.005, 0.4)
        for x in (-0.33, 0.33):
            F.box(_name('canopy_arm'), 0.02, 0.03, 0.12, (x, ry + 0.01, cz - hl + 0.05), M.canopy_rim, bevel=0.006,
                  bevel_segments=1)
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
             # the lid comes off for the fruit and goes back on (`hold`), then it blends: the smoothie swirls, a
             # turn each second (`every` a hair under it) from the lid's back on till the use ends
             centre=(0, -0.115),
             motions={'uses': {'blend': ['lid', {'clip': 'blades', 'delay': 1.5, 'every': 0.9}]}, 'hold': 1.0},
             paint=[('blend_cab', 'Frame'), ('blend_door', 'Doors'), ('motor', 'Blender')]),
     G.Model('contrastShower', contrastShower, size=(0.9, 2.15, 0.9), key='model.energetic.contrastShower',
             # the door lets the bather in, shuts behind them and opens again to let them out (as the glasshouse's)
             centre=(0, 0.02), motions={'uses': {'rinse': [{'clip': 'door_open', 'hold': 1.5, 'out': 1.5}, 'water_on']}},
             paint=[('shower_frame', 'Frame'), ('tile', 'Tiles'), ('shower_mat', 'Mat')]),
     G.Model('recoveryPod', recoveryPod, size=(0.859, 1.175, 1.86), key='model.energetic.recoveryPod',
             centre=(0, 0), motions={'uses': {'recover': 'canopy'}},
             paint=[('pod_shell', 'Frame'), ('pod_pad', 'Pads'), ('pod_boots', 'Boots')])],
    comment='Blender-built energetic pack models (tools/art/blender/families/pack_energetic.py). Listed '
            'in the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs '
            'one draw call per finish group. The bike\'s cranks turn and the trampoline\'s mat dips while '
            'used (loops), the net sways, the blender\'s lid lifts, the rain grows and the pod\'s canopy '
            'lowers; the bouldering wall is still.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
