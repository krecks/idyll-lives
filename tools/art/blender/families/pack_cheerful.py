# The cheerful pack (P2): the sunburst jukebox, the spotlight karaoke machine, the fizz & sunshine mocktail cart,
# the rainbow hammock, the sunrise stretch lamp and the confetti party booth, over their primitive-part
# placeholders (`tools/art/pack_cheerful.py`, `packs/cheerful/manifest.json`), which stay the fallback.
# Notes: pack_cheerful.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the jukebox's record swings up into the window (`record`), the microphone
# rises on its stand (`mic_up`), the cocktail shaker tips (`shake`), the hammock sways with its sleeper
# (`sway`, a loop they ride), the sun rises in the lamp's frame (`sunrise`; its sky glows at night) and the
# booth's side curtain is drawn aside (`curtain`).
#
# Run:  python3 -B tools/art/blender/build.py pack_cheerful [--only jukebox] [--preview]
import math
import os
import sys

import bmesh
import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # the jukebox
    cherry=('#D7263D', 'gloss'),       # the cabinet; its main part
    amber=('#F6A609', 'gloss'),        # the arch band and the bubble tubes
    sunny=('#FFE29A', 'gloss'),        # the inner arch band
    window=('#5B3A8C', 'gloss'),       # the record window's back
    grille=('#E9D8B0', 'fabric'),      # the speaker cloth
    cards=('#FFF2C8', 'gloss'),        # the song cards
    vinyl=('#2B2433', 'gloss'),
    label=('#F6C744', 'gloss'),
    coral=('#F28D8D', 'gloss'),
    mint=('#9ED8C6', 'gloss'),         # the buttons; the mocktail cart's shelves (its main part)
    sky=('#6EC1E4', 'gloss'),
    chrome=('#D0D3D6', 'chrome'),
    brass=('#C9A54A', 'metal'),
    # the karaoke machine
    speaker=('#8A6BC0', 'gloss'),      # the speaker cabinet; its main part
    speaker_trim=('#54456E', 'satin'),
    cone=('#3B2F4E', 'satin'),
    steel=('#C9CCD0', 'metal'),
    dark=('#2B2433', 'satin'),         # the screen's case, the mic, the tablet
    lyrics=('#3B2A6B', 'gloss'),
    white=('#F4F4F2', 'gloss'),
    stand=('#3A3D41', 'satin'),        # stands and poles
    lens=('#FFF4B0', 'gloss'),
    stage=('#6A3FA0', 'fabric'),       # the round stage rug
    star=('#F6C744', 'satin'),
    # the mocktail cart
    canopy=('#F28D8D', 'fabric'),
    canopy_white=('#F4F4F2', 'fabric'),
    tyre=('#F2F2F0', 'satin'),
    porcelain=('#F6F3EA', 'gloss'),
    juice=('#FF9A3C', 'gloss'),
    ice=('#E6F7FF', 'gloss'),
    tumbler=('#E6F0F2', 'gloss'),      # the glasses' rims, opaque (no glass draw)
    pineapple=('#E8B33C', 'satin'),
    leaf=('#4F9A3E', 'satin'),
    lemon=('#F2E05C', 'gloss'),
    lime=('#8CC63F', 'gloss'),
    orange=('#F28C28', 'gloss'),
    drink_pink=('#F07AA0', 'gloss'),
    drink_sun=('#F6C744', 'gloss'),
    drink_teal=('#7FD4C1', 'gloss'),
    fridge=('#F2F2F0', 'satin'),
    # the hammock
    wood=('#C9905A', 'wood'),          # the stand
    spreader=('#B07A4A', 'satin'),     # the spreader bars
    rope=('#EDE4D3', 'matte'),
    stripe_red=('#E4572E', 'fabric'),
    stripe_orange=('#F28C28', 'fabric'),
    stripe_yellow=('#F6C744', 'fabric'),
    stripe_green=('#8CC63F', 'fabric'),
    stripe_blue=('#3FA7D6', 'fabric'),
    stripe_violet=('#7B5EA7', 'fabric'),
    pillow=('#F4F4F2', 'satin'),
    # the sunrise lamp
    housing=('#F4F1EA', 'gloss'),      # the disc's case and the base; its main part
    lamp_metal=('#E9DCC7', 'metal'),   # the ring and the stem
    dawn=('#FFCF9E', 'satin', {'glow': '#FFB070'}),   # the sky behind the sun: the lamp glows at night
    hills=('#F28C5A', 'satin'),
    hills_far=('#F6A97A', 'satin'),
    sun=('#FFE08A', 'satin'),          # plain: a glow on it shows through the hills while it is set
    yoga=('#F29E8E', 'matte'),
    yoga_roll=('#E9876F', 'matte'),
    # the party booth
    booth=('#F28D8D', 'fabric'),       # the upholstery; its main part
    piping=('#F6C744', 'satin'),
    panel=('#7A5FB8', 'satin'),
    curtain=('#6EC1E4', 'fabric'),
    tablet_face=('#E84F9A', 'gloss'),
    popcorn=('#FFF3C4', 'matte'),
    bulb_a=('#F6C744', 'gloss'),
    bulb_b=('#6EC1E4', 'gloss'),
    bulb_c=('#F28D8D', 'gloss'),
    bulb_d=('#8CC63F', 'gloss'),
    bulb_e=('#B8A1E3', 'gloss'),
    bulb_f=('#F28C28', 'gloss'),
)

BULBS = ('bulb_a', 'bulb_b', 'bulb_c', 'bulb_d', 'bulb_e', 'bulb_f')   # by name: made anew after each reset
STRIPES = ('stripe_red', 'stripe_orange', 'stripe_yellow', 'stripe_green', 'stripe_blue', 'stripe_violet')
WALL = -0.43    # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93   # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


# --- meshes from points ------------------------------------------------------------------------------------------
def _obj(name, verts, faces, mats, face_mats=None, sharp=35, bevel=0.0):
    """A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    for m in mats:
        mesh.materials.append(m)
    if face_mats:
        for p, i in zip(mesh.polygons, face_mats):
            p.material_index = i
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.set_sharp_from_angle(angle=math.radians(sharp))
    if bevel:
        m = obj.modifiers.new('bevel', 'BEVEL')
        m.width, m.segments, m.limit_method, m.angle_limit = bevel, 2, 'ANGLE', math.radians(40)
    return obj


def _slab(F, name, pts, z0, z1, mat, bevel=0.0):
    """A flat shape: the outline `pts` [(x, y)] (game, in order round it) between depths `z0` and `z1`."""
    n = len(pts)
    verts = [F.at(x, y, z1) for x, y in pts] + [F.at(x, y, z0) for x, y in pts]
    faces = [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    faces += [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    return _obj(name, verts, faces, [mat], bevel=bevel)


def _ring(F, name, outer, inner, z0, z1, mat):
    """A flat frame between two outlines of as many points, `outer` round `inner`, from depth `z0` to `z1`."""
    n = len(outer)
    verts = [F.at(x, y, z1) for x, y in outer] + [F.at(x, y, z1) for x, y in inner]
    verts += [F.at(x, y, z0) for x, y in outer] + [F.at(x, y, z0) for x, y in inner]
    faces = []
    for i in range(n):
        j = (i + 1) % n
        faces += [(i, j, n + j, n + i), (2 * n + i, 3 * n + i, 3 * n + j, 2 * n + j),
                  (i, 2 * n + i, 2 * n + j, j), (n + i, n + j, 3 * n + j, 3 * n + i)]
    return _obj(name, verts, faces, [mat])


def _tomb(hw, yb, yc, n=18):
    """An arch-topped outline: straight sides `hw` either side of x = 0 from `yb` up to `yc`, a half circle over."""
    return [(-hw, yb), (hw, yb)] + [(hw * math.cos(math.pi * k / n), yc + hw * math.sin(math.pi * k / n))
                                    for k in range(n + 1)]


def _pleats(F, name, x, y0, y1, z0, z1, folds, amp, mat, t=0.012):
    """A pleated curtain hanging in the plane `x` from `y1` down to `y0`, along the depth `z0`..`z1`."""
    steps = folds * 4
    rows = []
    for i in range(steps + 1):
        z = z0 + (z1 - z0) * i / steps
        dx = amp * math.sin(math.pi * 2 * i / 4)
        rows.append([F.at(x + dx - t / 2, y1, z), F.at(x + dx + t / 2, y1, z),
                     F.at(x + dx + t / 2, y0, z), F.at(x + dx - t / 2, y0, z)])
    verts = [v for r in rows for v in r]
    faces = [(0, 1, 2, 3), (4 * steps + 3, 4 * steps + 2, 4 * steps + 1, 4 * steps)]
    for i in range(steps):
        a, b = 4 * i, 4 * (i + 1)
        faces += [(a + k, a + (k + 1) % 4, b + (k + 1) % 4, b + k) for k in range(4)]
    return _obj(name, verts, faces, [mat], sharp=60)


def _bulb(F, x, y, z, k, r=0.028):
    F.sphere(_name('bulb'), r, (x, y, z), getattr(M, BULBS[k % len(BULBS)]), segments=8, rings=6)


# --- the sunburst jukebox (1x1): an arch-topped cabinet, a record window, bubble tubes either side -------------
ARCH = 1.08     # the height of the arch's centre


def jukebox():
    F = G.GameFrame()
    zf = WALL + 0.46      # the cabinet's front
    with G.new_parts() as still:
        F.box('plinth', 0.74, 0.08, 0.48, (0, 0.04, WALL + 0.24), M.chrome, bevel=0.015)
        F.box('cabinet', 0.7, 0.8, 0.46, (0, 0.48, WALL + 0.23), M.cherry, bevel=0.03, bevel_segments=3)
        F.box('belt', 0.71, 0.03, 0.47, (0, 0.865, WALL + 0.235), M.chrome, bevel=0.008)
        # the arched top round the record window, its bands stepping out, a cavity behind it
        _ring(F, 'arch', _tomb(0.35, 0.88, ARCH), _tomb(0.2, 0.9, ARCH), WALL, zf, M.cherry)
        _ring(F, 'arch_band', _tomb(0.3, 0.885, ARCH), _tomb(0.25, 0.895, ARCH), zf, zf + 0.022, M.amber)
        _ring(F, 'arch_inner', _tomb(0.25, 0.895, ARCH), _tomb(0.2, 0.9, ARCH), zf, zf + 0.012, M.sunny)
        _slab(F, 'window', _tomb(0.2, 0.9, ARCH), WALL, WALL + 0.04, M.window)
        for k in range(3):                                           # the record stack in the window's foot
            F.cyl(_name('stack'), 0.13, 0.008, (0, 0.906 + 0.009 * k, -0.17), M.vinyl, verts=28)
        F.cyl('stack_label', 0.04, 0.03, (0, 0.92, -0.17), M.label, verts=16)
        # the front: song cards under a chrome bar, four buttons, a speaker cloth in a brass frame
        F.box('cards', 0.46, 0.1, 0.02, (0, 0.71, zf + 0.008), M.cards, bevel=0.006)
        F.box('cards_bar', 0.48, 0.014, 0.026, (0, 0.768, zf + 0.01), M.chrome, bevel=0.005)
        for k, (x, m) in enumerate(((-0.15, M.label), (-0.05, M.coral), (0.05, M.mint), (0.15, M.sky))):
            F.cyl(_name('button'), 0.017, 0.03, (x, 0.625, zf + 0.012), m, rot=(90, 0, 0), verts=12)
        F.box('grille_frame', 0.52, 0.32, 0.016, (0, 0.4, zf + 0.004), M.brass, bevel=0.01)
        F.box('grille', 0.48, 0.28, 0.016, (0, 0.4, zf + 0.01), M.grille, bevel=0.006)
        for x in (-0.12, 0.0, 0.12):                                 # chrome slats over the cloth
            F.box(_name('slat'), 0.012, 0.28, 0.012, (x, 0.4, zf + 0.022), M.chrome, bevel=0.004)
        # the bubble tubes at the front corners, chrome capped
        for s in (-1, 1):
            G.revolve(_name('tube'), [(0, 0), (0.035, 0), (0.035, 0.86), (0, 0.86)], at=F.at(s * 0.33, 0.12, zf - 0.01),
                      material=M.amber, segments=16)
            F.cyl(_name('tube_foot'), 0.045, 0.05, (s * 0.33, 0.12, zf - 0.01), M.chrome, verts=16, bevel=0.01)
            F.sphere(_name('tube_cap'), 0.045, (s * 0.33, 1.0, zf - 0.01), M.chrome, segments=14, rings=8)
            for k in range(5):                                       # bubbles rising in the tubes
                F.sphere(_name('bubble'), 0.012, (s * 0.33, 0.25 + 0.15 * k + 0.04 * (k % 2), zf + 0.022),
                         M.sunny, segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The record on top of the stack, lying flat, hinged on its front edge: `record` swings it up into the window.
    zr = -0.17 + 0.13
    with G.new_parts() as rec:
        F.cyl('record', 0.13, 0.008, (0, 0.937, -0.17), M.vinyl, verts=32)
        G.torus('record_groove', 0.095, 0.004, at=F.at(0, 0.941, -0.17), material=M.vinyl, major_segments=28, minor_segments=4)
    rec = G.hinged(G.join(rec, 'record'), F.at(0, 0.937, zr))
    # Its label, a part inside it: `spin` turns it (and a coral stripe across it, to see it turn) while the record plays.
    with G.new_parts() as lab:
        F.cyl('record_label', 0.045, 0.01, (0, 0.937, -0.17), M.label, verts=16)
        F.box('label_stripe', 0.012, 0.0125, 0.07, (0, 0.937, -0.17), M.coral)
    lab = G.hinged(G.join(lab, 'record_label'), F.at(0, 0.937, -0.17))
    objs = G.settle(centre=False)
    G.attach(lab, rec)
    G.swing(rec, 'record', 90, seconds=0.8, axis='X')
    G.spin(lab, 'spin', turns=-1, seconds=1.8, axis='Z')   # 33 1/3 a minute
    return objs


# --- the spotlight karaoke machine (1x1): a party speaker with a lyrics screen, a mic stand and a spotlight -----
def _tilt(c, ang, dy, dz):
    """A point `dy` up and `dz` out from `c` (game) on a face leaning back by `ang` degrees."""
    a = math.radians(ang)
    return (c[0], c[1] + dy * math.cos(a) + dz * math.sin(a), c[2] - dy * math.sin(a) + dz * math.cos(a))


MIC = (0.18, 0.2)     # the mic stand's foot (x, depth)


def karaokeMachine():
    F = G.GameFrame()
    w, h, d = 0.5, 0.86, 0.415
    zf = WALL + d
    with G.new_parts() as still:
        F.box('cabinet', w, h - 0.03, d, (0, 0.03 + (h - 0.03) / 2, WALL + d / 2), M.speaker, bevel=0.035, bevel_segments=3)
        F.box('band', w + 0.006, 0.04, d + 0.004, (0, h - 0.12, WALL + d / 2 + 0.001), M.speaker_trim, bevel=0.01)
        for fx in (-w / 2 + 0.06, w / 2 - 0.06):
            for fz in (WALL + 0.06, zf - 0.06):
                F.cyl(_name('foot'), 0.025, 0.03, (fx, 0.015, fz), M.speaker_trim, verts=10)
        for y, r in ((0.36, 0.17), (0.66, 0.075)):                       # woofer and tweeter
            G.torus(_name('surround'), r, 0.016, at=F.at(0, y, zf + 0.004), material=M.speaker_trim, rot=(90, 0, 0),
                    major_segments=28, minor_segments=6)
            F.cone(_name('cone'), r * 0.95, r * 0.3, 0.05, (0, y, zf - 0.02), M.cone, rot=(-90, 0, 0), verts=28)
            F.sphere(_name('dust_cap'), r * 0.32, (0, y, zf - 0.0), M.steel, scale=(1, 0.5, 1), segments=14, rings=8)
        F.box('port', 0.22, 0.04, 0.012, (0, 0.11, zf + 0.002), M.cone, bevel=0.012)
        F.box('light_strip', 0.3, 0.012, 0.006, (0, 0.15, zf + 0.004), M.label, bevel=0.003)
        for s in (-1, 1):
            F.box(_name('handle'), 0.012, 0.05, 0.14, (s * (w / 2 + 0.004), h - 0.25, WALL + d / 2), M.speaker_trim, bevel=0.005)
        # the lyrics screen on a neck, leaning back
        sc = (0, 1.08, -0.2)
        F.box('neck', 0.06, 0.1, 0.04, (0, 0.9, -0.22), M.dark, bevel=0.008)
        F.box('screen', 0.46, 0.34, 0.04, sc, M.dark, rot=(-10, 0, 0), bevel=0.012)
        F.box('screen_face', 0.42, 0.29, 0.004, _tilt(sc, 10, 0, 0.021), M.lyrics, rot=(-10, 0, 0))
        for dy, wl, m in ((0.06, 0.3, M.white), (0.01, 0.24, M.label), (-0.04, 0.28, M.white), (-0.09, 0.18, M.white)):
            F.box(_name('lyric'), wl, 0.022, 0.003, _tilt(sc, 10, dy, 0.024), m, rot=(-10, 0, 0))
        # the round stage rug with its star, the mic stand's foot and lower tube
        F.cyl('stage', 0.3, 0.01, (0.02, 0.005, 0.1), M.stage, verts=40)
        star = [(0.02 + (0.13 if k % 2 == 0 else 0.055) * math.sin(math.pi * k / 5),
                 0.1 + (0.13 if k % 2 == 0 else 0.055) * math.cos(math.pi * k / 5)) for k in range(10)]
        verts = [(-x, -z, 0.0105) for x, z in star] + [(-x, -z, 0.006) for x, z in star]
        faces = [tuple(range(10)), tuple(range(19, 9, -1))] + [(i, (i + 1) % 10, 10 + (i + 1) % 10, 10 + i) for i in range(10)]
        _obj('star', verts, faces, [M.star])
        mx, mz = MIC
        G.revolve('mic_foot', [(0, 0), (0.13, 0), (0.125, 0.012), (0.03, 0.03), (0, 0.032)], at=F.at(mx, 0.01, mz),
                  material=M.stand, segments=24)
        G.rod('mic_tube', F.at(mx, 0.03, mz), F.at(mx, 1.0, mz), 0.013, material=M.chrome, verts=12)
        F.cyl('mic_clutch', 0.02, 0.04, (mx, 1.0, mz), M.stand, verts=12)
        # the spotlight on its pole, aimed at the stage
        px, pz = -0.32, 0.3
        G.revolve('spot_foot', [(0, 0), (0.12, 0), (0.115, 0.012), (0.025, 0.028), (0, 0.03)], at=F.at(px, 0, pz),
                  material=M.stand, segments=24)
        G.rod('spot_pole', F.at(px, 0.02, pz), F.at(px, 1.62, pz), 0.012, material=M.stand, verts=10)
        head, aim = (px, 1.66, pz), (0.1, 1.2, 0.9)
        v = [aim[i] - head[i] for i in range(3)]
        n = math.sqrt(sum(c * c for c in v))
        u = [c / n for c in v]
        back = [head[i] - u[i] * 0.09 for i in range(3)]
        front = [head[i] + u[i] * 0.09 for i in range(3)]
        G.rod('spot_can', F.at(*back), F.at(*front), 0.07, material=M.dark, verts=20)
        lens_end = [front[i] + u[i] * 0.006 for i in range(3)]
        G.rod('spot_lens', F.at(*front), F.at(*lens_end), 0.06, material=M.lens, verts=20)
        G.torus('spot_yoke', 0.085, 0.008, at=F.at(*head), material=M.stand, rot=(0, 90, 0), major_segments=16,
                minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The upper tube, boom and microphone, lowered 25 cm into the tube: `mic_up` raises them to singing height.
    mx, mz = MIC
    drop = 0.25
    with G.new_parts() as mic:
        top = 1.35 - drop
        G.rod('mic_upper', F.at(mx, 0.75 - drop, mz), F.at(mx, top, mz), 0.008, material=M.chrome, verts=10)
        tip = (mx - 0.05, top + 0.07, mz + 0.08)
        G.rod('mic_boom', F.at(mx, top, mz), F.at(*tip), 0.007, material=M.chrome, verts=8)
        G.rod('mic_handle', F.at(*tip), F.at(tip[0] - 0.015, tip[1] + 0.05, tip[2] + 0.07), 0.017, material=M.dark, verts=12)
        F.sphere('mic_grille', 0.03, (tip[0] - 0.02, tip[1] + 0.068, tip[2] + 0.095), M.steel, segments=12, rings=8)
    mic = G.hinged(G.join(mic, 'mic'), F.at(mx, 1.0, mz))
    objs = G.settle(centre=False)
    G.slide(mic, 'mic_up', (0, 0, drop), seconds=0.8)
    return objs


# --- the fizz & sunshine mocktail cart (1x1): a brass bar cart under a striped canopy ---------------------------
TOP = 0.815     # the top shelf's top


def _tumbler(F, x, z, drink, y=TOP, h=0.11, r=0.034):
    """A tumbler full of `drink` (opaque: no glass draw), its rim and foot clear, with a straw."""
    F.cyl(_name('tumbler_foot'), r * 0.85, 0.012, (x, y + 0.006, z), M.tumbler, verts=16)
    G.revolve(_name('tumbler'), [(0, 0), (r * 0.86, 0), (r * 0.98, h - 0.012), (0, h - 0.012)],
              at=F.at(x, y + 0.012, z), material=drink, segments=16)
    G.torus(_name('rim'), r * 0.98, 0.004, at=F.at(x, y + h, z), material=M.tumbler, major_segments=16, minor_segments=4)
    G.rod(_name('straw'), F.at(x + 0.01, y + 0.03, z), F.at(x + 0.03, y + h + 0.06, z - 0.01), 0.004, material=M.coral, verts=6)


SHAKER = (0.06, 0.08)   # the shaker's foot (x, depth), tipping over to the left


def mocktailCart():
    F = G.GameFrame()
    zb, zf = -0.29, 0.17
    with G.new_parts() as still:
        for y in (0.3, 0.8):
            F.box(_name('shelf'), 0.78, 0.03, 0.46, (0, y, -0.06), M.mint, bevel=0.012)
            for z in (zb, zf):
                G.rod(_name('gallery'), F.at(-0.39, y + 0.06, z), F.at(0.39, y + 0.06, z), 0.006, material=M.brass, verts=8)
            for x in (-0.39, 0.39):
                G.rod(_name('gallery_end'), F.at(x, y + 0.06, zb), F.at(x, y + 0.06, zf), 0.006, material=M.brass, verts=8)
        for x in (-0.39, 0.39):
            for z in (zb, zf):
                y0 = 0.07 if z == zf else 0.0
                G.rod(_name('post'), F.at(x, y0, z), F.at(x, 1.6, z), 0.012, material=M.brass, verts=10)
                F.sphere(_name('finial'), 0.018, (x, 1.6, z), M.brass, segments=10, rings=6)
            G.torus(_name('wheel'), 0.055, 0.016, at=F.at(x, 0.071, zf), material=M.tyre, rot=(0, 90, 0),
                    major_segments=18, minor_segments=8)
            F.cyl(_name('hub'), 0.02, 0.05, (x, 0.071, zf), M.brass, rot=(0, 90, 0), verts=10)
            F.cyl(_name('cap'), 0.016, 0.02, (x, 0.01, zb), M.brass, verts=10)
        G.rod('handle', F.at(-0.39, 0.95, zb), F.at(-0.39, 0.95, zf), 0.012, material=M.brass, verts=10)
        # the striped canopy with a scalloped valance at the front and sides
        for i in range(6):
            m = M.canopy if i % 2 else M.canopy_white
            F.box(_name('canopy'), 0.14, 0.03, 0.56, (-0.35 + i * 0.14, 1.62, -0.05), m, bevel=0.006)
            pts = [(-0.07, 1.605), (0.07, 1.605)] + [(0.07 * math.cos(math.pi * k / 8), 1.56 - 0.07 * math.sin(math.pi * k / 8))
                                                     for k in range(9)]
            _slab(F, _name('scallop'), [(-0.35 + i * 0.14 + px, py) for px, py in pts], 0.218, 0.23, m)
        for s in (-1, 1):
            for i in range(4):
                z = -0.26 + i * 0.14
                m = M.canopy if i % 2 else M.canopy_white
                pts = [(z - 0.07, 1.605), (z + 0.07, 1.605)] + [(z + 0.07 * math.cos(math.pi * k / 8),
                                                                1.56 - 0.07 * math.sin(math.pi * k / 8)) for k in range(9)]
                verts = [F.at(s * 0.41 + s * 0.006 * j, y, zz) for j in (1, -1) for zz, y in pts]
                nn = len(pts)
                faces = [tuple(range(nn)), tuple(range(2 * nn - 1, nn - 1, -1))]
                faces += [(k, (k + 1) % nn, nn + (k + 1) % nn, nn + k) for k in range(nn)]
                _obj(_name('scallop_side'), verts, faces, [m])
        # the top: a jug of juice, an ice bucket, a pineapple, citrus and three drinks
        G.revolve('jug', [(0, 0), (0.05, 0), (0.06, 0.08), (0.05, 0.17), (0.055, 0.2), (0.045, 0.2), (0, 0.19)],
                  at=F.at(-0.25, TOP, -0.17), material=M.porcelain, segments=18)
        F.cyl('jug_juice', 0.047, 0.01, (-0.25, TOP + 0.19, -0.17), M.juice, verts=16)
        G.torus('jug_handle', 0.04, 0.008, at=F.at(-0.31, TOP + 0.11, -0.17), material=M.porcelain, rot=(90, 0, 0),
                major_segments=12, minor_segments=6)
        G.revolve('bucket', [(0, 0), (0.06, 0), (0.08, 0.12), (0.075, 0.12), (0.055, 0.01), (0, 0.01)],
                  at=F.at(0.2, TOP, -0.17), material=M.chrome, segments=20)
        for k, (dx, dz) in enumerate(((-0.02, 0.0), (0.02, 0.015), (0.0, -0.025), (0.03, -0.01))):
            F.box(_name('ice'), 0.03, 0.03, 0.03, (0.2 + dx, TOP + 0.11 + 0.008 * (k % 2), -0.17 + dz), M.ice,
                  rot=(10 * k, 20, 15 * k), bevel=0.005)
        F.sphere('pineapple', 0.055, (0.3, TOP + 0.08, 0.06), M.pineapple, scale=(1, 1, 1.45), segments=14, rings=10)
        for k in range(5):
            a = 72 * k
            F.cone(_name('crown'), 0.022, 0.0, 0.11, (0.3 + 0.012 * G.cosd(a), TOP + 0.2, 0.06 + 0.012 * G.sind(a)), M.leaf,
                   rot=(18 * G.sind(a), 18 * G.cosd(a), 0), verts=5)
        for x, z, r, m in ((0.15, 0.1, 0.034, M.lemon), (0.21, 0.13, 0.03, M.lime), (0.17, 0.02, 0.038, M.orange)):
            F.sphere(_name('citrus'), r, (x, TOP + r, z), m, segments=12, rings=8)
        _tumbler(F, -0.1, 0.1, M.drink_pink)
        _tumbler(F, -0.22, 0.06, M.drink_sun)
        _tumbler(F, -0.33, 0.1, M.drink_teal)
        # below: a little fridge drawer and three syrup bottles
        F.box('fridge', 0.3, 0.2, 0.22, (-0.15, 0.415, -0.1), M.fridge, bevel=0.025)
        F.box('fridge_handle', 0.12, 0.012, 0.012, (-0.15, 0.47, 0.016), M.chrome, bevel=0.004)
        for i, m in enumerate((M.drink_pink, M.drink_teal, M.drink_sun)):
            G.revolve(_name('syrup'), [(0, 0), (0.032, 0), (0.034, 0.15), (0.012, 0.2), (0.012, 0.23), (0, 0.23)],
                      at=F.at(0.12 + i * 0.09, 0.315, -0.12), material=m, segments=14)
            F.cyl(_name('syrup_cap'), 0.014, 0.02, (0.12 + i * 0.09, 0.555, -0.12), M.chrome, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cocktail shaker, hinged on its foot's left edge: `shake` tips it over the pink drink.
    sx, sz = SHAKER
    with G.new_parts() as shaker:
        G.revolve('shaker', [(0, 0), (0.036, 0), (0.042, 0.13), (0.036, 0.15), (0.02, 0.18), (0.012, 0.2), (0, 0.205)],
                  at=F.at(sx, TOP, sz), material=M.chrome, segments=20)
        G.torus('shaker_band', 0.04, 0.004, at=F.at(sx, TOP + 0.12, sz), material=M.chrome, major_segments=20,
                minor_segments=4)
    shaker = G.hinged(G.join(shaker, 'shaker'), F.at(sx - 0.036, TOP, sz))
    objs = G.settle(centre=False)
    G.swing(shaker, 'shake', 35, seconds=0.6, axis='Y')
    return objs


# --- the rainbow hammock (2x1): a striped sling between spreader bars, on a wooden stand -----------------------
SLING_END, SLING_LOW, SLING_W = 0.86, 0.49, 0.62     # the sling's ends' and middle's heights, its width
HOOK = 1.0                                           # the hooks' height: the sway's pivot


def _sling_y(u, v):
    """The sling's top at a fraction `u` along it and `v` across it: sagging along, cupped across in the middle."""
    s = 2 * u - 1
    return SLING_LOW + (SLING_END - SLING_LOW) * s * s + 0.07 * (2 * v - 1) ** 2 * (1 - s * s)


def rainbowHammock():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('beam', 1.72, 0.07, 0.09, (0, 0.05, 0), M.wood, bevel=0.02)
        for s in (-1, 1):
            G.rod(_name('post'), F.at(s * 0.8, 0.04, 0), F.at(s * 0.9, 1.05, 0), 0.036, material=M.wood, verts=10)
            F.box(_name('foot'), 0.09, 0.045, 0.62, (s * 0.84, 0.0225, 0), M.wood, bevel=0.015)
            F.sphere(_name('post_top'), 0.042, (s * 0.9, 1.05, 0), M.wood, segments=12, rings=8)
            for z in (-0.18, 0.18):                                   # braces from the foot to the post
                G.rod(_name('brace'), F.at(s * 0.84, 0.04, z * 1.5), F.at(s * 0.835, 0.32, 0), 0.016, material=M.wood, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sling, its spreader bars and ropes, the hooks, the pillow and the throw: `sway` rocks them about the hooks.
    with G.new_parts() as sling:
        nu, nv, x0, x1, t = 20, 12, -0.74, 0.74, 0.018
        verts, faces, mats = [], [], []
        for layer in (0, 1):
            for i in range(nu + 1):
                for j in range(nv + 1):
                    u, v = i / nu, j / nv
                    verts.append(F.at(x0 + (x1 - x0) * u, _sling_y(u, v) - t * layer, -SLING_W / 2 + SLING_W * v))
        at = lambda layer, i, j: layer * (nu + 1) * (nv + 1) + i * (nv + 1) + j  # noqa: E731
        stripe = lambda j: min(5, j * 6 // nv)  # noqa: E731
        for i in range(nu):
            for j in range(nv):
                faces.append((at(0, i, j), at(0, i + 1, j), at(0, i + 1, j + 1), at(0, i, j + 1)))
                faces.append((at(1, i, j), at(1, i, j + 1), at(1, i + 1, j + 1), at(1, i + 1, j)))
                mats += [stripe(j)] * 2
        for i in range(nu):                                           # the long edges
            for j in (0, nv):
                faces.append((at(0, i, j), at(1, i, j), at(1, i + 1, j), at(0, i + 1, j)))
                mats.append(stripe(min(j, nv - 1)))
        for j in range(nv):                                           # the ends
            for i in (0, nu):
                faces.append((at(0, i, j), at(0, i, j + 1), at(1, i, j + 1), at(1, i, j)))
                mats.append(stripe(j))
        _obj('sling', verts, faces, [getattr(M, s) for s in STRIPES], face_mats=mats, sharp=50)
        for s in (-1, 1):
            F.box(_name('spreader'), 0.045, 0.045, SLING_W + 0.08, (s * 0.755, SLING_END - 0.01, 0), M.spreader, bevel=0.012)
            hook = (s * 0.88, HOOK, 0)
            G.torus(_name('hook'), 0.03, 0.007, at=F.at(*hook), material=M.lamp_metal, rot=(90, 0, 0),
                    major_segments=12, minor_segments=5)
            for z in (-0.33, -0.11, 0.11, 0.33):
                G.rod(_name('rope'), F.at(s * 0.87, HOOK - 0.03, 0), F.at(s * 0.755, SLING_END + 0.01, z), 0.006,
                      material=M.rope, verts=6)
        # the pillow at the left end, lying on the slope
        u = (-0.52 - x0) / (x1 - x0)
        slope = math.degrees(math.atan((SLING_END - SLING_LOW) * 4 * (2 * u - 1) / (x1 - x0)))
        F.box('pillow', 0.36, 0.11, 0.3, (-0.52, _sling_y(u, 0.5) + 0.05, 0), M.pillow, rot=(0, slope, 0),
              bevel=0.045, bevel_segments=3)
    sling = G.hinged(G.join(sling, 'sling'), F.at(0, HOOK, 0))
    objs = G.settle(centre=False)
    G.rock(sling, 'sway', 5, seconds=2.4, axis='X')
    return objs


# --- the sunrise stretch lamp (1x1): a dawn scene in a round frame on a stand, a yoga mat before it -------------
SUN_C = (0.0, 1.25, -0.3)    # the frame's middle (x, height, depth)
SUN_R = 0.33                 # the scene's radius
SUN_RISE = 0.22


def _hills(cy, r, top, waves, amp, phase, n=40):
    """The part of a disc of radius `r` (centre height `cy`) below a wavy horizon at `top` (offsets from the centre)."""
    pts = []
    for k in range(n + 1):                                         # the horizon, left to right
        x = -r + 2 * r * k / n
        y = top + amp * math.sin(waves * math.pi * (x + r) / (2 * r) + phase)
        y = min(y, math.sqrt(max(r * r - x * x, 0.0)))
        pts.append((x, y))
    lo = [(x, -math.sqrt(max(r * r - x * x, 0.0))) for x, _ in pts]
    pts = [(x, y) for x, y in pts]
    out = pts + lo[::-1][1:-1]                                     # along the top, back along the circle's foot
    return [(x, cy + y) for x, y in out]


def sunriseLamp():
    F = G.GameFrame()
    cx, cy, cz = SUN_C
    with G.new_parts() as still:
        G.revolve('base', [(0, 0), (0.13, 0), (0.13, 0.015), (0.1, 0.03), (0.03, 0.04), (0, 0.04)], at=F.at(cx, 0, cz),
                  material=M.housing, segments=28)
        G.rod('stem', F.at(cx, 0.03, cz), F.at(cx, cy - 0.3, cz), 0.016, material=M.lamp_metal, verts=12)
        F.cyl('case', 0.385, 0.06, (cx, cy, cz - 0.03), M.housing, rot=(90, 0, 0), verts=48, bevel=0.012)
        G.torus('ring', 0.37, 0.022, at=F.at(cx, cy, cz + 0.012), material=M.lamp_metal, rot=(90, 0, 0),
                major_segments=48, minor_segments=8)
        F.cyl('dawn', SUN_R + 0.01, 0.006, (cx, cy, cz + 0.003), M.dawn, rot=(90, 0, 0), verts=48)
        _slab(F, 'hills_far', _hills(cy, SUN_R, -0.06, 3, 0.03, 0.4), cz + 0.032, cz + 0.038, M.hills_far)
        _slab(F, 'hills', _hills(cy, SUN_R, -0.12, 2, 0.035, 2.2), cz + 0.038, cz + 0.046, M.hills)
        # the yoga mat, its far end rolled up
        F.box('mat', 0.6, 0.008, 0.38, (0, 0.004, 0.13), M.yoga, bevel=0.002)
        F.cyl('mat_roll', 0.06, 0.6, (0, 0.06, 0.38), M.yoga_roll, rot=(0, 90, 0), verts=20)
        F.cyl('mat_roll_end', 0.035, 0.602, (0, 0.06, 0.38), M.yoga, rot=(0, 90, 0), verts=16)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sun, set behind the hills: `sunrise` raises it into the sky, which glows at night (`dawn`'s `glow`).
    with G.new_parts() as sun:
        F.cyl('sun', 0.13, 0.004, (cx, cy - 0.19, cz + 0.011), M.sun, rot=(90, 0, 0), verts=36)
    sun = G.hinged(G.join(sun, 'sun'), F.at(cx, cy - 0.19, cz + 0.011))
    objs = G.settle(centre=False)
    G.slide(sun, 'sunrise', (0, 0, SUN_RISE), seconds=1.5)
    return objs


# --- the confetti party booth (2x2): a horseshoe booth for two under a bulb-lit frame, a table with a tablet ----
def partyBooth():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('seat_base', 1.3, 0.38, 0.62, (0, 0.19, -0.05), M.booth, bevel=0.06, bevel_segments=3)
        F.box('seat', 1.3, 0.1, 0.58, (0, 0.4, -0.04), M.booth, bevel=0.045, bevel_segments=3)
        F.box('backrest', 1.86, 0.62, 0.22, (0, 0.72, -0.44), M.booth, rot=(-8, 0, 0), bevel=0.07, bevel_segments=3)
        for x in (-0.31, 0.31):                                       # tufting lines on the backrest
            F.box(_name('tuft'), 0.02, 0.5, 0.02, (x, 0.74, -0.32), M.booth, rot=(-8, 0, 0), bevel=0.008)
        F.box('backrest_piping', 1.86, 0.03, 0.22, (0, 1.05, -0.48), M.piping, bevel=0.012)
        F.box('back_panel', 1.9, 1.6, 0.06, (0, 0.8, WALL2 + 0.03), M.panel, bevel=0.015)
        for s in (-1, 1):
            F.box(_name('wing'), 0.3, 0.72, 1.0, (s * 0.8, 0.36, 0.4), M.booth, bevel=0.07, bevel_segments=3)
            F.box(_name('wing_piping'), 0.3, 0.03, 1.0, (s * 0.8, 0.73, 0.4), M.piping, bevel=0.012)
            F.box(_name('wing_base'), 0.28, 0.06, 0.96, (s * 0.8, 0.03, 0.4), M.booth, bevel=0.02)
        # the bulb-lit frame over it: posts on the wings, rails along the sides and the front
        zr, xr, yr = 0.82, 0.88, 1.58
        for s in (-1, 1):
            G.rod(_name('post'), F.at(s * xr, 0.74, zr), F.at(s * xr, yr, zr), 0.018, material=M.brass, verts=10)
            G.rod(_name('rail'), F.at(s * xr, yr, WALL2 + 0.06), F.at(s * xr, yr, zr), 0.014, material=M.brass, verts=10)
            F.sphere(_name('corner'), 0.03, (s * xr, yr, zr), M.brass, segments=12, rings=8)
        F.box('marquee', 1.76, 0.12, 0.05, (0, yr, zr), M.panel, bevel=0.012)
        for i in range(9):
            _bulb(F, -0.72 + i * 0.18, yr, zr + 0.03, i, r=0.026)
        # the marquee bulbs and confetti on the back panel
        for i in range(13):
            x = -0.84 + i * 0.14
            _bulb(F, x, 1.53, WALL2 + 0.065, i, r=0.03)
            _bulb(F, x + 0.07, 1.28 + 0.06 * math.sin(i), WALL2 + 0.063, i + 3, r=0.024)
        for i, (x, y) in enumerate([(-0.6, 1.08), (-0.3, 1.18), (0.1, 1.1), (0.45, 1.15), (0.7, 1.05), (-0.05, 1.4),
                                    (0.3, 1.38), (-0.45, 1.4), (0.62, 1.42), (-0.75, 1.22)]):
            F.box(_name('confetti'), 0.05, 0.05, 0.008, (x, y, WALL2 + 0.064), getattr(M, BULBS[i % 6]), rot=(0, 30 * i, 0))
        # the round table with the party tablet, two drinks and a bowl of popcorn
        tz = 0.52
        F.cyl('table', 0.31, 0.035, (0, 0.72, tz), M.white, verts=40, bevel=0.01)
        G.revolve('table_stem', [(0.2, 0), (0.2, 0.025), (0.05, 0.05), (0.035, 0.1), (0.035, 0.7), (0, 0.705)],
                  at=F.at(0, 0, tz), material=M.brass, segments=24)
        F.box('tablet_foot', 0.12, 0.012, 0.08, (0, 0.743, 0.62), M.dark, bevel=0.004)
        tc = (0, 0.86, 0.6)
        F.box('tablet', 0.34, 0.22, 0.02, tc, M.dark, rot=(12, 0, 0), bevel=0.008)
        F.box('tablet_face', 0.31, 0.19, 0.004, _tilt(tc, -12, 0, -0.011), M.tablet_face, rot=(12, 0, 0))
        _tumbler(F, -0.18, 0.4, M.drink_sun, y=0.7375)
        _tumbler(F, 0.18, 0.4, M.drink_teal, y=0.7375)
        G.revolve('popcorn_bowl', [(0, 0), (0.05, 0), (0.09, 0.05), (0.085, 0.052), (0.045, 0.006), (0, 0.006)],
                  at=F.at(0.14, 0.7375, 0.66), material=M.white, segments=18)
        for k in range(9):
            a = 40 * k
            r = 0.04 if k % 3 else 0.015
            F.sphere(_name('popcorn'), 0.022, (0.14 + r * G.cosd(a), 0.79 + 0.008 * (k % 2), 0.66 + r * G.sind(a)),
                     M.popcorn, segments=7, rings=5)
        # the left curtain, gathered at the back
        _pleats(F, 'curtain_left', -xr, 0.77, yr - 0.02, WALL2 + 0.07, WALL2 + 0.45, 4, 0.03, M.curtain)
    G.join(still, 'body', origin=(0, 0, 0))
    # The right curtain, hanging closed along the rail: `curtain` draws it aside to the back.
    with G.new_parts() as cur:
        _pleats(F, 'curtain', xr, 0.77, yr - 0.02, WALL2 + 0.07, zr - 0.04, 9, 0.022, M.curtain)
    cur = G.hinged(G.join(cur, 'curtain'), F.at(xr, yr, WALL2 + 0.07))
    objs = G.settle(centre=False)
    G.squash(cur, 'curtain', (1, 0.25, 1), seconds=0.8)
    return objs


FAMILY = G.Family(
    'pack_cheerful', M,
    [G.Model('jukebox', jukebox, size=(0.75, 1.43, 0.495), key='model.cheerful.jukebox',
             centre=(0, -0.183), motions={'uses': {'dance': 'record'}, 'loops': {'dance': 'spin'}},
             paint=[('cherry', 'Body'), ('amber', 'Trim')]),
     G.Model('karaokeMachine', karaokeMachine, size=(0.76, 1.763, 0.851), key='model.cheerful.karaokeMachine',
             centre=(-0.06, -0.005), motions={'uses': {'sing': 'mic_up'}},
             paint=[('speaker', 'Body'), ('speaker_trim', 'Trim'), ('stage', 'Rug')]),
     G.Model('mocktailCart', mocktailCart, size=(0.84, 1.635, 0.57), key='model.cheerful.mocktailCart',
             centre=(0, -0.045), motions={'uses': {'mix': 'shake'}},
             paint=[('mint', 'Body'), ('canopy', 'Trim'), ('brass', 'Frame')]),
     G.Model('rainbowHammock', rainbowHammock, size=(1.88, 1.09, 0.7), key='model.cheerful.rainbowHammock',
             centre=(0, 0), motions={'loops': {'swing': 'sway', 'doze': 'sway'}, 'ride': 'sway'},
             paint=[('wood', 'Body'), ('spreader', 'Trim'), ('pillow', 'Pillow')]),
     G.Model('sunriseLamp', sunriseLamp, size=(0.784, 1.642, 0.87), key='model.cheerful.sunriseLamp',
             centre=(0, 0.005), motions={'uses': {'stretch': 'sunrise'}},
             paint=[('housing', 'Body'), ('lamp_metal', 'Trim'), ('yoga', 'Mat')]),
     G.Model('partyBooth', partyBooth, size=(1.9, 1.64, 1.83), key='model.cheerful.partyBooth',
             centre=(0, -0.015), motions={'uses': {'party': 'curtain'}, 'hold': 1.5},
             paint=[('booth', 'Body'), ('piping', 'Trim'), ('curtain', 'Curtain')])],
    comment='Blender-built cheerful pack models (tools/art/blender/families/pack_cheerful.py). Replaces the '
            'primitive-part placeholders of packs/cheerful by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. Each has one moving part: the record swings up, the microphone rises, the shaker tips, '
            'the hammock sways (a loop its sleeper rides), the sun rises (its sky glowing at night) and the curtain is drawn.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
