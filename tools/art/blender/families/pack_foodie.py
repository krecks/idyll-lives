# The foodie pack (P6): the windowsill herb garden, the deli pantry, the espresso bar, the cookbook lectern, the
# wood-fired pizza oven and the chef's island. Notes: pack_foodie.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). A fresh sprig grows in a pot (`sprig`), the pantry's doors swing open (`doors_open`), the coffee runs
# and the steam wand swings out (`brew`), a page of the cookbook turns over (`page`), the oven door lifts aside
# and the peel slides in (`door_off`) as the fire grows (`flames`), and the island's drawer pulls out
# (`drawer_open`).
#
# Built with the forms and the kit (README "The toolkit"): frame-and-panel fronts (`kit.shaker`) with bar pulls,
# crown mouldings, herbs of extruded leaves, lofted jars, bottles, salami and china, logs and loaves wobbled by
# `G.jitter`, bent iron and steel in one sweep.
#
# Run:  cargo art pack_foodie [--only pizzaOven] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit.motion import _clip, last_frame  # noqa: E402

M = G.Palette(
    # the herb garden
    cedar=('#C49A6C', 'wood'),       # the stand and the trough; its main part
    cedar_dark=('#A07A52', 'wood'),  # posts, rails and the trough's inside
    soil=('#5A4030', 'matte'),
    basil=('#5E9E3E', 'satin'),      # herbs are plain satin, so they cost no draw call of their own
    basil_light=('#6FB04A', 'satin'),
    thyme=('#86A06E', 'satin'),
    rosemary=('#4E7C52', 'satin'),
    terracotta=('#C46A43', 'matte'),
    chalk=('#F2EDE2', 'matte'),      # chalk tags, towels
    steel=('#9EA3A8', 'metal'),
    wicker=('#C9A46E', 'satin'),
    # the deli pantry
    cream=('#EDE4D0', 'satin'),      # the pantry's body; its main part
    cream_panel=('#E4D9C1', 'satin'),
    honey=('#A8754A', 'satin'),      # the pantry's shelves, plinth and crown (plain: the doors take the draws)
    back=('#E2D6BC', 'matte'),
    marble=('#F3EFE8', 'gloss'),     # the pantry's ledge, the island's top
    brass=('#B8925A', 'metal'),
    jar_a=('#C98A3A', 'gloss'),
    jar_b=('#9DB86A', 'gloss'),
    jar_c=('#B5653F', 'gloss'),
    cheese=('#D6A64A', 'satin'),
    cheese_light=('#E9C46A', 'satin'),
    bread=('#C8964F', 'matte'),
    wine=('#6E2B33', 'gloss'),
    bottle=('#3E5A3A', 'gloss'),
    salami=('#8E3B2E', 'matte'),
    tomato=('#C9483B', 'gloss'),
    board=('#B08E66', 'satin'),
    # the espresso bar
    teal=('#2F5D62', 'satin'),       # the cart's frame; its main part
    oak_top=('#CDB08A', 'satin'),    # the cart's top and shelves (plain: the stream and the wand take the draws)
    machine=('#EFE6D4', 'gloss'),
    copper=('#C27A4A', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    black=('#2B2C2E', 'gloss'),
    cup=('#F6F3EA', 'gloss'),
    coffee=('#5A3420', 'gloss'),
    beans=('#5A3A26', 'satin'),
    dark=('#3A3D41', 'satin'),
    sack=('#C9A46E', 'satin'),
    # the cookbook lectern
    walnut=('#7A5234', 'wood'),      # the lectern; its main part
    honey_wood=('#A8754A', 'wood'),  # its ledge and the stool
    page=('#F6F1E4', 'matte'),
    cover=('#B33A32', 'satin'),
    photo=('#E9C46A', 'matte'),
    book_teal=('#4E7A7F', 'satin'),
    book_mustard=('#D6A64A', 'satin'),
    book_sage=('#9DAF8C', 'satin'),
    # the pizza oven
    plaster=('#E8DAC4', 'matte'),    # the dome; its main part
    brick=('#A4614A', 'matte'),
    brick_light=('#B8735A', 'matte'),
    brick_dark=('#6E4535', 'matte'),
    stone=('#C9C1B4', 'satin'),
    worktop=('#E9E3D8', 'gloss'),
    iron=('#3E4146', 'metal'),
    log=('#8B6B4E', 'satin'),        # plain, not bark: the door, the peel and the fire take the draws
    mouth=('#2B2220', 'matte'),
    ember=('#F28C28', 'gloss'),
    flame=('#F28C28', 'gloss'),
    flame_core=('#FFC24A', 'gloss'),
    dough=('#E8C27A', 'matte'),
    mozzarella=('#F4EBD5', 'satin'),
    oil=('#7A8F3A', 'gloss'),
    peel=('#B08E66', 'satin'),
    door_oak=('#CDB08A', 'satin'),
    # the chef's island
    cabinet=('#5F9095', 'satin'),    # the island's cabinets; its main part
    oak=('#CDB08A', 'satin'),        # the stools, plain (the hob's hot ring takes the third draw)
    oak_dark=('#B08E66', 'satin'),
    seat=('#C9A46E', 'satin'),
    hob=('#22262B', 'gloss'),
    ring=('#C9483B', 'gloss'),
    heat=('#D2402A', 'gloss'),       # the hob's ring glowing hot under the pan while it cooks
    carrot=('#E2832E', 'satin'),
    plate=('#F6F3EA', 'gloss'),
    # added with the detail pass
    crust=('#D9A35A', 'matte'),      # bread crusts, the pizza's rim
    log_end=('#C4A27A', 'satin'),    # the logs' cut ends
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _paint(obj, material, test):
    """Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a label,
    a rind, with no more triangles."""
    obj.data.materials.append(material)
    k = len(obj.data.materials) - 1
    for p in obj.data.polygons:
        if test(p.center):
            p.material_index = k
    return obj


def _lofted(F, name, x, y, z, rows, material, verts=12, square=2.0):
    """A body standing on `y` at (x, z), lofted through rings (height, radius) from its foot up."""
    bx, by, bz = F.at(x, y, z)
    return G.loft(name, [G.ring(r, r, bz + h, verts, square=square, at=(bx, by)) for h, r in rows], material=material)


# --- herbs and pots, shared ------------------------------------------------------------------------------------
LEAF = G.smooth_outline([(0.0, 0.0), (0.35, 0.25), (0.42, 0.6), (0.2, 0.92), (0.0, 1.0), (-0.2, 0.92), (-0.42, 0.6),
                         (-0.35, 0.25)], 1)


def _leaf(F, name, x, y, z, size, turn, tilt, material):
    """A leaf `size` long from its stalk at game point (x, y, z), pointing out at `turn` degrees round the up axis and
    lifted `tilt` degrees from flat."""
    pts = [(u * size, v * size) for u, v in LEAF]
    return F.extrude(name, pts, 0.003, (x, y, z), material, plane='top', rot=(-tilt, 0, turn))


def _basil(F, x, z, y, s=1.0):
    """A basil plant on the soil at `y`: pairs of broad leaves in tiers up a short stem, a tuft at its top."""
    G.kit.tube(F, _name('basil_stem'), [(x, y, z), (x + 0.004 * s, y + 0.12 * s, z)], 0.005 * s, M.basil, verts=4, round=1)
    for k, (h, size, m) in enumerate(((0.03, 0.07, M.basil), (0.065, 0.065, M.basil_light), (0.095, 0.055, M.basil),
                                      (0.12, 0.04, M.basil_light))):
        for j in range(2 if k < 3 else 3):
            a = k * 55 + j * (180 if k < 3 else 120)
            _leaf(F, _name('basil'), x, y + h * s, z, size * s, a, 30 - 8 * k, m)


def _rosemary(F, x, z, y, s=1.0):
    """A rosemary bush on the soil at `y`: woody stems rising and splaying, tapering to their tips."""
    for i in range(7):
        a = i * 137.5
        c, sn = G.cosd(a), G.sind(a)
        h = (0.213 + 0.05 * (i % 3)) * s
        F.sweep(_name('rosemary'), [(x + 0.02 * s * c, y, z + 0.02 * s * sn), (x + 0.05 * s * c, y + h * 0.5, z + 0.05 * s * sn),
                                    (x + 0.07 * s * c, y + h, z + 0.07 * s * sn)], 0.014 * s, M.rosemary, verts=5,
                resolution=2, scale=[1.0, 0.85, 0.35])


def _thyme(F, x, z, y, s=1.0, seed=0.0):
    """A low thyme mound on the soil at `y`: lumpy and fine-leaved."""
    mound = G.boulder(_name('thyme'), 0.07 * s, at=F.at(x, y - 0.01, z), material=M.thyme, scale=(1.0, 0.9, 0.7),
                      seed=seed, noise=0.25, subdivisions=2)
    G.jitter(mound, 0.006 * s, size=0.02, seed=seed)


def _pot(F, x, z, y=0.0, r=0.09, h=0.16, material=None):
    """A terracotta pot standing on `y` with soil in it, its rim rolled; returns the soil's top."""
    G.revolve(_name('pot'), [(0, 0), (r * 0.72, 0), (r * 0.88, h * 0.75), (r, h * 0.76), (r * 1.02, h * 0.82), (r * 1.02, h),
                             (r * 0.9, h), (r * 0.86, h * 0.9), (0, h * 0.9)], at=F.at(x, y, z), material=material or M.terracotta,
              segments=14)
    G.jitter(F.cyl(_name('pot_soil'), r * 0.84, 0.008, (x, y + h * 0.9 + 0.003, z), M.soil, verts=12), 0.002, size=0.02)
    return y + h * 0.9 + 0.006


# --- the windowsill herb garden (1x1): a slatted cedar trough on a stand, pots on the shelf below ------------
def herbGarden():
    F = G.GameFrame()
    leg = G.rounded_rect(0.045, 0.045, 0.008, corner=1)
    with G.new_parts() as still:
        for x in (-0.38, 0.38):
            for z in (-0.40, -0.06):
                F.sweep(_name('leg'), [(x, 0.0, z), (x, 0.615, z)], None, M.cedar_dark, profile=leg, up=(1, 0, 0))
            F.box(_name('side_rail'), 0.04, 0.03, 0.36, (x, 0.6, -0.23), M.cedar_dark, bevel=0.004, bevel_segments=1)
            F.box(_name('shelf_rail'), 0.03, 0.025, 0.36, (x, 0.165, -0.23), M.cedar_dark)
        for z in (-0.36, -0.23, -0.10):
            F.box(_name('shelf_slat'), 0.78, 0.02, 0.09, (0, 0.185, z), M.cedar, bevel=0.004, bevel_segments=1)
        for z in (-0.40, -0.06):
            F.box(_name('top_rail'), 0.8, 0.03, 0.05, (0, 0.6, z), M.cedar_dark, bevel=0.004, bevel_segments=1)
        # the trough: a dark inside, corner posts, three slats a side with a gap between, a moulded rim; soil, herbs
        y0, w, h, d = 0.615, 0.84, 0.39, 0.38
        z0, z1 = WALL, WALL + d
        zc = (z0 + z1) / 2
        F.box('trough_inside', w - 0.05, h - 0.06, d - 0.05, (0, y0 + (h - 0.06) / 2, zc), M.cedar_dark)
        for px in (-w / 2 + 0.025, w / 2 - 0.025):
            for pz in (z0 + 0.025, z1 - 0.025):
                F.box(_name('trough_post'), 0.05, h, 0.05, (px, y0 + h / 2, pz), M.cedar_dark, bevel=0.006, bevel_segments=1)
        sh = h / 3
        for k in range(3):
            y = y0 + sh * (k + 0.5)
            for z in (z0 + 0.01, z1 - 0.01):
                F.box(_name('trough_slat'), w - 0.02, sh - 0.014, 0.02, (0, y, z), M.cedar, bevel=0.004, bevel_segments=1)
            for x in (-w / 2 + 0.01, w / 2 - 0.01):
                F.box(_name('trough_slat'), 0.02, sh - 0.014, d - 0.02, (x, y, zc), M.cedar, bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'trough_rim', [(-w / 2 + 0.01, z0 + 0.0145), (w / 2 - 0.01, z0 + 0.0145), (w / 2 - 0.01, z1 - 0.005),
                                         (-w / 2 + 0.01, z1 - 0.005)],
                       [(0.03, 0.0), (-0.02, 0.0), (-0.02, 0.014), (-0.014, 0.02), (0.03, 0.02)], M.cedar_dark, at=y0 + h,
                       plane='top')
        soil = y0 + h - 0.03
        G.jitter(F.box('trough_soil', w - 0.06, 0.02, d - 0.06, (0, soil - 0.01, zc), M.soil), 0.004, size=0.05)
        _basil(F, -0.26, zc, soil, 1.3)
        _basil(F, -0.18, zc + 0.08, soil, 0.9)
        _rosemary(F, 0.0, zc, soil, 1.0)
        _thyme(F, 0.27, zc, soil, 1.1, seed=1.0)
        _thyme(F, 0.2, zc + 0.09, soil, 0.7, seed=2.0)
        for x in (-0.26, 0.0, 0.27):                   # chalk tags on the trough's front
            F.box(_name('tag'), 0.09, 0.05, 0.006, (x, y0 + h - 0.08, z1 + 0.004), M.chalk, bevel=0.002, bevel_segments=1)
            F.box(_name('tag_line'), 0.05, 0.006, 0.002, (x, y0 + h - 0.08, z1 + 0.008), M.soil)
        # the shelf below: the sprig's pot on a saucer, a pot of basil and a trowel
        F.cyl('saucer', 0.085, 0.01, (-0.2, 0.2, -0.24), M.terracotta, verts=14)
        _pot(F, -0.2, -0.24, 0.205, r=0.08, h=0.14)
        top = _pot(F, 0.18, -0.25, 0.195, r=0.07, h=0.13)
        _basil(F, 0.18, -0.25, top, 0.8)
        F.extrude('trowel_blade', [(0.0, 0.0), (0.03, 0.02), (0.028, 0.08), (0.0, 0.12), (-0.028, 0.08), (-0.03, 0.02)], 0.006,
                  (0.34, 0.199, -0.12), M.steel, plane='top', rot=(0, 0, -155), bevel=0.002, bevel_segments=1)
        F.sweep('trowel_grip', [(0.33, 0.204, -0.15), (0.29, 0.21, -0.24)], 0.014, M.cedar_dark, verts=8, scale=[0.8, 1.1])
        # a woven basket of picked herbs on the floor in front, its handle arching over
        bz = 0.236
        G.revolve('basket', [(0, 0), (0.09, 0), (0.12, 0.09), (0.11, 0.09), (0.08, 0.012), (0, 0.012)],
                  at=F.at(-0.2, 0, bz), material=M.wicker, segments=16, scale=(1, 0.72, 1))
        for y in (0.03, 0.06):
            G.torus(_name('weave'), 1.0, 0.004, at=F.at(-0.2, y, bz), material=M.wicker, major_segments=16,
                    minor_segments=3).scale = (0.09 + y / 3 + 0.003, (0.09 + y / 3 + 0.003) * 0.72, 1.0)
        G.kit.tube(F, 'basket_handle', [(-0.2, 0.085, bz - 0.084), (-0.2, 0.2, bz), (-0.2, 0.085, bz + 0.084)], 0.008,
                   M.wicker, verts=5, round=4)
        _basil(F, -0.2, bz, 0.04, 0.8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sprig, a stem and leaves in the left pot, built full size and scaled to nothing on the soil:
    # `sprig` grows it while a resident tends the herbs.
    sx, sz, sy = -0.2, -0.24, 0.205 + 0.14 * 0.9 + 0.006
    with G.new_parts() as sprig:
        F.sweep('sprig_stem', [(sx, sy, sz), (sx + 0.006, sy + 0.09, sz), (sx + 0.01, sy + 0.17, sz)], 0.005, M.rosemary,
                verts=4, resolution=2, scale=[1.0, 0.8, 0.5])
        for k, (dy, a) in enumerate(((0.06, 0), (0.09, 140), (0.12, 260), (0.15, 30), (0.175, 200))):
            _leaf(F, _name('sprig_leaf'), sx + 0.005, sy + dy, sz, 0.045 - 0.004 * k, a, 25, M.basil_light)
    sprig = G.join(sprig, 'sprig', origin=F.at(sx, sy, sz))
    sprig.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    for axis in 'XYZ':
        G.grow(sprig, 'sprig', seconds=0.9, axis=axis)
    return objs


# --- the deli pantry (1x1): a cream cupboard under a marble ledge, open shelves of deli things above -------
def _jar(F, x, y, z, m, r=0.045, h=0.15):
    """A preserving jar standing on `y`: a lofted body with shoulders, its label band, a brass lid."""
    jar = _lofted(F, _name('jar'), x, y, z, [(0, r * 0.9), (0.006, r), (h * 0.8, r), (h * 0.92, r * 0.86), (h, r * 0.86)],
                  m, verts=12)
    _paint(jar, M.chalk, lambda c: 0.3 * h < c.z - F.at(x, y, z)[2] < 0.6 * h)
    G.revolve(_name('jar_lid'), [(0, 0), (r * 0.9, 0), (r * 0.92, 0.02), (r * 0.8, 0.026), (0, 0.026)],
              at=F.at(x, y + h, z), material=M.brass, segments=12)


def _loaf(F, x, y, z, length=0.28, r=0.06, turn=0, seed=0.0):
    """A loaf lying on `y`: a lumpy, domed body with three slashes across its top."""
    loaf = F.sphere(_name('loaf'), r, (x, y + r * 0.8, z), M.bread, scale=(length / (2 * r), 1, 0.8), rot=(0, 0, turn),
                    segments=12, rings=7)
    G.jitter(loaf, r * 0.06, size=r, seed=seed)
    c, s = G.cosd(turn), G.sind(turn)
    for k in (-1, 0, 1):
        u = k * length * 0.25
        F.sweep(_name('loaf_slash'), [(x + u * c - 0.6 * r * s, y + r * 1.48, z - u * s - 0.6 * r * c),
                                      (x + u * c + 0.6 * r * s, y + r * 1.48, z - u * s + 0.6 * r * c)], r * 0.12, M.crust,
                verts=4)


def _wheel(F, name, x, y, z, r, h, m, cut=0.0):
    """A cheese wheel on `y`, its rind round it, a wedge `cut` degrees wide taken out of its front."""
    a0 = 90 + cut / 2
    pts = ([(0.0, 0.0)] if cut else []) + [(r * G.cosd(a0 + (360 - cut) * k / 20), r * G.sind(a0 + (360 - cut) * k / 20))
                                            for k in range(21 if cut else 20)]
    w = F.extrude(name, pts, h, (x, y + h / 2, z), m, plane='top', bevel=0.006, bevel_segments=1)
    return _paint(w, M.cheese, lambda c: math.hypot(c.x, c.y) > r * 0.92 or abs(c.z) > h * 0.45) if m != M.cheese else w


def _salami(F, x, y, z, length):
    """A salami hanging from its string at `y`: a tapered, lofted sausage tied at the top."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.006), (0.012, 0.02), (0.03, 0.03), (length * 0.6, 0.032), (length - 0.02, 0.026), (length, 0.008)]
    G.loft(_name('salami'), [G.ring(r, r, bz - hh, 10, at=(bx, by)) for hh, r in rows], material=M.salami)
    G.kit.tube(F, _name('string'), [(x, y, z), (x, y + 0.05, z)], 0.0025, M.chalk, verts=4, round=1)


def _pantry_door(F, x, hinge_x, zf):
    """A frame-and-panel door, its bar pull upright at the edge it opens by, hinged on its outer edge."""
    with G.new_parts() as parts:
        G.kit.shaker(F, _name('door'), 0.395, 0.78, (x, 0.5, zf + 0.01), 0.02, M.cream, rail=0.06, panel=M.cream_panel,
                     raised=True)
        px = x + (0.15 if hinge_x < x else -0.15)
        G.kit.pull(F, _name('door_pull'), px, 0.62, zf + 0.02, 0.1, M.brass, r=0.006, out=0.024, upright=True)
        for y in (0.2, 0.8):
            F.cyl(_name('hinge'), 0.008, 0.06, (hinge_x + (0.005 if hinge_x > 0 else -0.005), y, zf + 0.012), M.brass, verts=8)
    return G.hinged(G.join(parts, _name('door')), F.at(hinge_x, 0.5, zf + 0.01))


def deliPantry():
    F = G.GameFrame()
    rnd = random.Random(2)
    zf = 0.06                       # the carcass's front
    with G.new_parts() as still:
        F.box('plinth', 0.8, 0.08, 0.4, (0, 0.04, WALL + 0.2), M.honey, bevel=0.004, bevel_segments=1)
        for x in (-0.41, 0.41):
            F.box(_name('side'), 0.02, 0.84, zf - WALL, (x, 0.5, (zf + WALL) / 2), M.cream, bevel=0.004, bevel_segments=1)
        F.box('back', 0.8, 0.84, 0.01, (0, 0.5, WALL + 0.005), M.back)
        F.box('bottom', 0.8, 0.02, zf - WALL, (0, 0.09, (zf + WALL) / 2), M.cream)
        F.box('mid_shelf', 0.8, 0.02, 0.4, (0, 0.5, WALL + 0.2), M.cream)
        F.box('stile', 0.02, 0.84, 0.02, (0, 0.5, zf - 0.01), M.cream)
        # inside, behind the doors: jars and a crock below, tins, oil and a cheese above
        for k, (x, m) in enumerate(((-0.3, M.jar_a), (-0.2, M.jar_b), (-0.1, M.jar_c))):
            _jar(F, x, 0.1, -0.2, m, h=0.18)
        _lofted(F, 'crock', 0.2, 0.1, -0.22, [(0, 0.09), (0.01, 0.1), (0.17, 0.1), (0.2, 0.09)], M.cream_panel, verts=14)
        G.revolve('crock_lid', [(0, 0), (0.1, 0), (0.1, 0.012), (0.04, 0.02), (0.03, 0.035), (0, 0.04)],
                  at=F.at(0.2, 0.3, -0.22), material=M.honey, segments=14)
        for x in (-0.28, -0.18):
            F.cyl(_name('tin'), 0.045, 0.12, (x, 0.57, -0.2), M.jar_c, verts=12)
        _lofted(F, 'oil', 0.0, 0.51, -0.25, [(0, 0.03), (0.005, 0.035), (0.2, 0.035), (0.25, 0.014), (0.31, 0.012)], M.oil,
                verts=10)
        _wheel(F, 'cheese_in', 0.22, 0.51, -0.2, 0.11, 0.08, M.cheese)
        # the marble ledge, its front edge rounded
        F.extrude('ledge', G.rounded_rect(0.88, 0.56, 0.02, corner=2), 0.04, (0, 0.94, WALL + 0.28), M.marble, plane='top',
                  bevel=0.012, bevel_segments=2)
        # the open shelves above, a crown moulding round their top
        for x in (-0.405, 0.405):
            F.box(_name('upper_side'), 0.03, 0.92, 0.4, (x, 1.42, WALL + 0.2), M.cream, bevel=0.005, bevel_segments=1)
        F.box('upper_back', 0.78, 0.92, 0.01, (0, 1.42, WALL + 0.005), M.back)
        for y in (1.2, 1.5):
            F.box(_name('shelf'), 0.78, 0.02, 0.36, (0, y, WALL + 0.19), M.honey, bevel=0.003, bevel_segments=1)
            F.box(_name('shelf_lip'), 0.78, 0.03, 0.012, (0, y + 0.005, WALL + 0.374), M.honey)
        F.box('upper_top', 0.84, 0.03, 0.42, (0, 1.895, WALL + 0.21), M.cream)
        # (its back 1 cm into the wall, as the old part's crown was: the model keeps its centre)
        G.kit.moulding(F, 'crown', [(-0.41, WALL + 0.02), (0.41, WALL + 0.02), (0.41, WALL + 0.43), (-0.41, WALL + 0.43)],
                       [(0.02, 0.0), (0.0, 0.0), (-0.008, 0.012), (-0.02, 0.03), (-0.03, 0.046), (-0.03, 0.06), (0.02, 0.06)],
                       M.honey, at=1.93, plane='top')
        F.box('header', 0.78, 0.07, 0.025, (0, 1.845, -0.045), M.cream, bevel=0.004, bevel_segments=1)
        for k, (x, m) in enumerate(((-0.3, M.jar_a), (-0.19, M.jar_b), (-0.08, M.jar_c))):
            _jar(F, x, 1.21, -0.24, m, r=0.045, h=0.14)
        _wheel(F, 'wheel_a', 0.2, 1.21, -0.25, 0.11, 0.08, M.cheese_light, cut=50)
        _wheel(F, 'wheel_b', 0.21, 1.29, -0.27, 0.095, 0.07, M.cheese)
        _loaf(F, -0.14, 1.51, -0.24, 0.3, 0.055, seed=1.0)
        _loaf(F, 0.16, 1.51, -0.22, 0.24, 0.05, turn=25, seed=2.0)
        for x, m in ((-0.33, M.wine), (-0.26, M.bottle)):
            _lofted(F, _name('bottle'), x, 1.51, -0.28, [(0, 0.03), (0.005, 0.032), (0.19, 0.032), (0.23, 0.014),
                                                         (0.3, 0.012), (0.31, 0.013)], m, verts=10)
        # hanging salami on a brass rail under the header
        G.kit.tube(F, 'rail', [(-0.37, 1.79, -0.1), (0.37, 1.79, -0.1)], 0.008, M.brass, verts=8, round=1)
        for x in (-0.24, -0.08, 0.1, 0.25):
            _salami(F, x, 1.74, -0.1, rnd.uniform(0.16, 0.22))
        # on the ledge: a board with cheese and bread, two tomatoes
        F.extrude('board', G.rounded_rect(0.4, 0.22, 0.03, corner=2) , 0.02, (0.08, 0.97, -0.01), M.board, plane='top',
                  holes=[G.rounded_rect(0.03, 0.012, 0.006, corner=2)], bevel=0.004, bevel_segments=1)
        F.extrude('wedge', [(0.0, 0.0), (0.08, -0.025), (0.08, 0.025)], 0.06, (-0.02, 1.01, -0.01), M.cheese_light,
                  plane='top', bevel=0.004, bevel_segments=1)
        _loaf(F, 0.17, 0.98, 0.0, 0.18, 0.04, turn=70, seed=3.0)
        for x, z in ((-0.3, 0.02), (-0.24, 0.06)):
            F.sphere(_name('tomato'), 0.028, (x, 0.988, z), M.tomato, scale=(1, 1, 0.9), segments=10, rings=6)
            F.cone(_name('tomato_calyx'), 0.012, 0.0, 0.008, (x, 1.016, z), M.basil, verts=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two doors, hinged on their outer edges: `doors_open` swings them out.
    left = _pantry_door(F, -0.2025, -0.40, zf)
    right = _pantry_door(F, 0.2025, 0.40, zf)
    objs = G.settle(centre=False)
    G.swing(left, 'doors_open', 100, seconds=0.8)
    G.swing(right, 'doors_open', -100, seconds=0.8)
    return objs


# --- the espresso bar (1x1): a lever espresso machine on a teal-and-oak cart ----------------------------------
GROUPS = (-0.16, 0.04)   # the group heads' x


def _cup(F, x, y, z, m=None):
    """An espresso cup on its saucer on `y`, crema in it, its ear handle to the side."""
    m = m or M.cup
    G.revolve(_name('saucer'), [(0.0, 0.0), (0.026, 0.0), (0.042, 0.007), (0.04, 0.009), (0.0, 0.004)],
              at=F.at(x, y, z), material=M.cup, segments=12)
    G.revolve(_name('cup'), [(0, 0), (0.02, 0), (0.028, 0.016), (0.03, 0.044), (0.027, 0.044), (0.025, 0.026), (0, 0.026)],
              at=F.at(x, y + 0.005, z), material=m, segments=12)
    F.cyl(_name('crema'), 0.026, 0.002, (x, y + 0.045, z), M.coffee, verts=10)
    F.sweep(_name('cup_handle'), [(x + 0.028, y + 0.04, z), (x + 0.044, y + 0.036, z), (x + 0.042, y + 0.02, z),
                                  (x + 0.028, y + 0.016, z)], 0.0035, m, verts=4, resolution=2)


def espressoBar():
    F = G.GameFrame()
    zc = WALL + 0.25
    with G.new_parts() as still:
        F.extrude('top', G.rounded_rect(0.76, 0.5, 0.02, corner=2), 0.04, (0, 0.88, zc), M.oak_top, plane='top', bevel=0.008,
                  bevel_segments=1)
        for y in (0.42, 0.13):
            F.box(_name('shelf'), 0.7, 0.03, 0.44, (0, y, zc), M.oak_top, bevel=0.005, bevel_segments=1)
            F.box(_name('shelf_lip'), 0.7, 0.03, 0.012, (0, y + 0.025, zc + 0.214), M.oak_top)
        post = G.rounded_rect(0.04, 0.04, 0.008, corner=1)
        for x in (-0.35, 0.35):
            for z in (WALL + 0.03, zc + 0.22):
                F.sweep(_name('post'), [(x, 0.07, z), (x, 0.87, z)], None, M.teal, profile=post, up=(1, 0, 0))
                F.cyl(_name('fork'), 0.012, 0.03, (x, 0.07, z), M.chrome, verts=8)
                F.cyl(_name('castor'), 0.037, 0.022, (x, 0.037, z), M.dark, rot=(0, 90, 0), verts=12)
        G.kit.tube(F, 'towel_rail', [(-0.33, 0.75, 0.064), (0.33, 0.75, 0.064)], 0.008, M.brass, verts=8, round=1)
        towel = F.sweep('towel', G.kit.fillet([(0.16, 0.6, 0.06), (0.16, 0.75, 0.056), (0.16, 0.762, 0.064), (0.16, 0.75, 0.072),
                                               (0.16, 0.66, 0.074)], 0.008, steps=2), None, M.chalk,
                        profile=G.rounded_rect(0.006, 0.16, 0.003, corner=1), up=(0, -1, 0))
        G.jitter(towel, 0.003, size=0.06, seed=2.0, up=0.3)
        # the machine: a cream body on a chrome base, a copper boiler dome with a brass eagle, gauge, group heads, levers
        F.extrude('machine_base', G.rounded_rect(0.46, 0.32, 0.03, corner=2), 0.03, (-0.06, 0.915, -0.26), M.chrome,
                  plane='top', bevel=0.008, bevel_segments=1)
        F.extrude('machine', G.rounded_rect(0.42, 0.3, 0.05, corner=3), 0.3, (-0.06, 1.08, -0.26), M.machine, plane='top',
                  bevel=0.025, bevel_segments=2)
        boiler = G.revolve('boiler', [(0, 0), (0.12, 0), (0.122, 0.02), (0.12, 0.06), (0.1, 0.15), (0.06, 0.21), (0.02, 0.23),
                                      (0, 0.23)], at=F.at(-0.06, 1.23, -0.27), material=M.copper, segments=20)
        _paint(boiler, M.brass, lambda c: c.z < 0.025)
        G.revolve('eagle_post', [(0, 0), (0.016, 0), (0.01, 0.02), (0.006, 0.03), (0, 0.03)], at=F.at(-0.06, 1.455, -0.27),
                  material=M.brass, segments=8)
        F.sphere('eagle', 0.025, (-0.06, 1.482, -0.27), M.brass, scale=(1, 0.8, 1.1), segments=8, rings=5)
        for s in (-1, 1):
            F.extrude(_name('eagle_wing'), [(0.0, 0.0), (0.06, 0.02), (0.05, 0.04), (0.0, 0.015)], 0.004,
                      (-0.06 + s * 0.012, 1.482, -0.27), M.brass, rot=(0, 0, 90 - 90 * s))
        F.cyl('gauge', 0.032, 0.012, (0.1, 1.13, -0.105), M.chrome, rot=(90, 0, 0), verts=14, bevel=0.003, bevel_segments=1)
        F.cyl('gauge_face', 0.026, 0.004, (0.1, 1.13, -0.099), M.cup, rot=(90, 0, 0), verts=14)
        F.box('gauge_needle', 0.004, 0.02, 0.002, (0.105, 1.135, -0.096), M.tomato, rot=(0, -40, 0))
        F.extrude('drip_tray', G.rounded_rect(0.4, 0.13, 0.015, corner=2), 0.02, (-0.06, 0.915, -0.05), M.chrome, plane='top',
                  holes=[G.rounded_rect(0.36, 0.1, 0.01, corner=2)])
        F.box('drip_grate', 0.36, 0.006, 0.1, (-0.06, 0.912, -0.05), M.dark)
        for x in GROUPS:
            F.box(_name('group_neck'), 0.05, 0.04, 0.05, (x, 1.12, -0.09), M.chrome, bevel=0.008, bevel_segments=1)
            G.revolve(_name('group'), [(0, 0), (0.03, 0), (0.036, 0.008), (0.036, 0.034), (0.03, 0.04), (0, 0.04)],
                      at=F.at(x, 1.07, -0.07), material=M.chrome, segments=12)
            G.revolve(_name('portafilter'), [(0, 0), (0.02, 0), (0.032, 0.014), (0.034, 0.02), (0, 0.02)],
                      at=F.at(x, 1.05, -0.07), material=M.chrome, segments=12)
            F.cone(_name('spout'), 0.012, 0.006, 0.02, (x, 1.042, -0.07), M.chrome, verts=8)
            F.sweep(_name('pf_handle'), [(x, 1.06, -0.04), (x, 1.058, 0.0), (x, 1.054, 0.07)], 0.011, M.black, verts=8,
                    scale=[0.8, 1.0, 1.1])
            G.kit.tube(F, _name('lever'), [(x, 1.16, -0.1), (x, 1.25, -0.06), (x, 1.34, 0.0)], 0.011, M.chrome, verts=8,
                       round=2)
            F.sphere(_name('lever_knob'), 0.025, (x, 1.35, 0.005), M.black, scale=(1, 1, 1.3), segments=10, rings=6)
            _cup(F, x, 0.935, -0.07)
        # the grinder at the back right, a milk jug before it
        F.extrude('grinder_base', G.rounded_rect(0.12, 0.14, 0.02, corner=2), 0.12, (0.26, 0.96, -0.33), M.black,
                  plane='top', bevel=0.01, bevel_segments=1)
        G.revolve('hopper', [(0, 0), (0.035, 0), (0.035, 0.03), (0.07, 0.14), (0.072, 0.15), (0, 0.15)],
                  at=F.at(0.26, 1.02, -0.33), material=M.machine, segments=14)
        G.jitter(F.sphere('hopper_beans', 0.068, (0.26, 1.165, -0.33), M.beans, scale=(1, 1, 0.35), segments=12, rings=6),
                 0.003, size=0.015)
        G.revolve('milk_jug', [(0, 0), (0.042, 0), (0.046, 0.02), (0.04, 0.08), (0.042, 0.11), (0.038, 0.11), (0.034, 0.08),
                               (0, 0.08)], at=F.at(0.24, 0.9, -0.1), material=M.chrome, segments=14)
        F.sweep('milk_handle', G.kit.fillet([(0.278, 1.0, -0.1), (0.3, 0.99, -0.1), (0.296, 0.93, -0.1), (0.275, 0.925, -0.1)],
                                            0.012, steps=2), 0.005, M.chrome, verts=5)
        # below: cups on the middle shelf, bean sacks and a syrup bottle
        for i, x in enumerate((-0.26, -0.18, -0.1)):
            _cup(F, x, 0.435, -0.18, M.book_teal if i == 1 else None)
        _lofted(F, 'syrup', 0.2, 0.435, -0.25, [(0, 0.03), (0.005, 0.035), (0.17, 0.035), (0.2, 0.014), (0.24, 0.012)],
                M.bottle, verts=10)
        F.cyl('syrup_pump', 0.006, 0.05, (0.2, 0.7, -0.25), M.chrome, verts=6)
        F.box('syrup_nozzle', 0.04, 0.008, 0.008, (0.215, 0.722, -0.25), M.chrome)
        for name, w, hh, d, x, z, m in (('sack_a', 0.24, 0.2, 0.16, 0.17, -0.24, M.sack),
                                        ('sack_b', 0.2, 0.16, 0.14, -0.16, -0.22, M.wicker)):
            sack = F.soft(name, w, hh, d, (x, 0.145 + hh / 2, z), m, round=0.05, puff=0.02, flat=2)
            G.jitter(sack, 0.008, size=0.06, seed=x)
            F.sweep(f'{name}_tie', [(x - w * 0.3, 0.145 + hh, z), (x, 0.145 + hh + 0.025, z), (x + w * 0.3, 0.145 + hh, z)],
                    0.015, m, verts=6, resolution=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The coffee running from the left group's spout into its cup, built full length and scaled to nothing
    # at the spout: `brew` grows it.
    gx = GROUPS[0]
    with G.new_parts() as stream:
        for dx in (-0.006, 0.006):
            F.cyl(_name('stream'), 0.004, 0.08, (gx + dx, 0.992, -0.07), M.coffee, verts=6)
    stream = G.join(stream, 'stream', origin=F.at(gx, 1.032, -0.07))
    stream.scale[2] = 0.001
    # The steam wand on the machine's right side, hanging from its ball joint: `brew` swings it out.
    with G.new_parts() as wand:
        F.sphere('wand_joint', 0.018, (0.165, 1.12, -0.16), M.chrome, segments=10, rings=6)
        G.kit.tube(F, 'wand', G.kit.fillet([(0.165, 1.12, -0.16), (0.168, 1.0, -0.15), (0.176, 0.97, -0.13)], 0.02), 0.007,
                   M.chrome, verts=6, round=1)
        F.cone('wand_tip', 0.008, 0.004, 0.02, (0.176, 0.955, -0.128), M.chrome, verts=8)
    wand = G.hinged(G.join(wand, 'wand'), F.at(0.165, 1.12, -0.16))
    objs = G.settle(centre=False)
    G.grow(stream, 'brew', seconds=0.4)
    G.swing(wand, 'brew', 40, seconds=0.6, axis='Y')
    return objs


# --- the cookbook lectern (1x1): a turned walnut lectern with an open cookbook; a stool of cookbooks -------
DESK, TILT = (0.0, 1.02, -0.04), 25     # the reading desk's middle (game) and its slope (degrees, front down)


def _on_desk(lx, ly, lz):
    """A point on the desk, `ly` up off it and `lz` down its slope towards the front, as game coordinates."""
    c, s = G.cosd(TILT), G.sind(TILT)
    x, y, z = DESK
    return (x + lx, y + ly * c - lz * s, z + ly * s + lz * c)


def _pages(obj, lying=True):
    """A book's box with its page edges cream on the same mesh: the three sides but its spine (front) when lying."""
    obj.data.materials.append(M.page)
    for p in obj.data.polygons:
        n = p.normal
        if (abs(n.z) < 0.5 and n.y > -0.5) if lying else n.z > 0.5:
            p.material_index = 1
    return obj


def cookbookLectern():
    F = G.GameFrame()
    with G.new_parts() as still:
        # three feet curving out from the column's foot, the turned column, the sloped desk with its book ledge
        for k in range(3):
            a = 120 * k + 90
            c, s = G.cosd(a), G.sind(a)
            F.sweep(_name('foot'), [(0.03 * c, 0.12, -0.05 + 0.03 * s), (0.11 * c, 0.06, -0.05 + 0.11 * s),
                                    (0.18 * c, 0.02, -0.05 + 0.18 * s), (0.2 * c, 0.016, -0.05 + 0.2 * s)], None, M.walnut,
                    profile=G.rounded_rect(0.03, 0.035, 0.01, corner=1), up=(0, 0, 1), resolution=2)
            F.sphere(_name('foot_pad'), 0.018, (0.2 * c, 0.016, -0.05 + 0.2 * s), M.walnut, scale=(1, 1, 0.9), segments=8,
                     rings=4)
        G.revolve('column', [(0, 0.08), (0.05, 0.08), (0.055, 0.1), (0.04, 0.14), (0.03, 0.18), (0.034, 0.3), (0.03, 0.42),
                             (0.04, 0.46), (0.03, 0.5), (0.026, 0.72), (0.04, 0.78), (0.06, 0.86), (0.06, 0.88), (0, 0.88)],
                  at=F.at(0, 0.05, -0.05), material=M.walnut, segments=14)
        F.extrude('desk', G.rounded_rect(0.56, 0.42, 0.02, corner=2), 0.04, DESK, M.walnut, plane='top', rot=(TILT, 0, 0),
                  bevel=0.01, bevel_segments=2)
        F.box('ledge', 0.5, 0.03, 0.03, _on_desk(0, 0.03, 0.2), M.honey_wood, rot=(TILT, 0, 0), bevel=0.008, bevel_segments=1)
        # a scalloped apron under the desk's front edge, and the bracket the desk sits on
        edge = [(-0.22 + 0.44 * i / 12, -0.04 + 0.012 * math.cos(math.tau * i / 4)) for i in range(13)]
        F.extrude('apron', [(0.22, 0.0), (-0.22, 0.0)] + edge, 0.016, _on_desk(0, -0.03, 0.16), M.walnut, rot=(TILT, 0, 0),
                  bevel=0.003, bevel_segments=1)
        F.box('desk_bracket', 0.14, 0.05, 0.2, _on_desk(0, -0.04, 0.0), M.walnut, rot=(TILT, 0, 0), bevel=0.008,
              bevel_segments=1)
        # two brass page clips holding the book open
        for s in (-1, 1):
            G.kit.tube(F, _name('clip'), [_on_desk(s * 0.245, 0.0, 0.17), _on_desk(s * 0.245, 0.07, 0.17), _on_desk(s * 0.2, 0.07, 0.12)],
                       0.003, M.brass, verts=4, round=2)
            F.sphere(_name('clip_end'), 0.007, _on_desk(s * 0.2, 0.07, 0.12), M.brass, segments=6, rings=4)
        # the open cookbook: its cover, two page blocks curving up from the spine, a recipe photo and a ribbon
        F.box('book_cover', 0.48, 0.012, 0.34, _on_desk(0, 0.026, 0), M.cover, rot=(TILT, 0, 0), bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            prof = [(s * 0.005, 0.0), (s * 0.22, 0.0), (s * 0.225, 0.02), (s * 0.15, 0.03), (s * 0.06, 0.028), (s * 0.012, 0.012)]
            pts = [_on_desk(0, 0.032, -0.155), _on_desk(0, 0.032, 0.155)]
            F.sweep(_name('pages'), pts, None, M.page, profile=[(v, u) for u, v in prof], up=(0, 0, 1))
        F.box('photo', 0.13, 0.002, 0.09, _on_desk(0.11, 0.064, -0.06), M.photo, rot=(TILT, 0, 0))
        for k in range(4):
            F.box(_name('text'), 0.15, 0.002, 0.008, _on_desk(0.11, 0.064, 0.03 + 0.025 * k), M.board, rot=(TILT, 0, 0))
        F.sweep('ribbon', [_on_desk(-0.02, 0.064, -0.16), _on_desk(-0.02, 0.064, 0.15), _on_desk(-0.02, 0.03, 0.18),
                           _on_desk(-0.02, -0.03, 0.2)], None, M.tomato, profile=G.rounded_rect(0.002, 0.012, 0.001, corner=1),
                up=(0, 0, 1), resolution=2)
        # a turned stool with a pile of cookbooks, and a jar of wooden spoons on the floor
        G.revolve('stool', [(0, 0), (0.13, 0), (0.13, 0.02), (0.11, 0.04), (0.1, 0.2), (0.11, 0.36), (0.13, 0.38),
                            (0.15, 0.4), (0.15, 0.42), (0, 0.42)], at=F.at(0.29, 0, -0.27), material=M.honey_wood, segments=16)
        y = 0.42
        for w, hh, m, r in ((0.26, 0.05, M.book_teal, 6), (0.24, 0.04, M.book_mustard, -8), (0.25, 0.06, M.cover, 3),
                            (0.22, 0.04, M.book_sage, 12)):
            _pages(F.box(_name('cookbook'), w, hh, w * 0.75, (0.29, y + hh / 2, -0.27), m, rot=(0, 0, r)))
            y += hh
        # a recipe-card box on the floor, its lid propped open on its cards
        F.box('card_box', 0.16, 0.09, 0.11, (-0.12, 0.045, 0.1), M.honey_wood, rot=(0, 0, 12), bevel=0.004, bevel_segments=1)
        F.box('card_box_lid', 0.16, 0.008, 0.11, (-0.12, 0.13, 0.04), M.honey_wood, rot=(-60, 0, 12), bevel=0.003,
              bevel_segments=1)
        for k in range(4):
            F.box(_name('card'), 0.14, 0.06, 0.002, (-0.12 + 0.004 * k, 0.1, 0.09 - 0.018 * k), (M.page, M.photo)[k % 2],
                  rot=(-10, 0, 12))
        G.revolve('spoon_jar', [(0, 0), (0.045, 0), (0.05, 0.14), (0.045, 0.14), (0.04, 0.012), (0, 0.012)],
                  at=F.at(-0.31, 0, -0.3), material=M.cup, segments=12)
        for dx, dz, lean in ((-0.015, 0, -10), (0.015, 0.01, 8), (0, -0.015, 3)):
            tip = (-0.31 + dx * 3 + lean * 0.002, 0.3, -0.3 + dz * 2)
            F.sweep(_name('spoon'), [(-0.31 + dx, 0.03, -0.3 + dz), tip], 0.007, M.honey_wood, verts=5, scale=[0.8, 1.0])
            F.sphere(_name('spoon_bowl'), 0.022, (tip[0], 0.31, tip[2]), M.honey_wood, scale=(1, 0.5, 1.3), segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The page on the right-hand block (game -x), curling gently up from the spine, its origin on the spine, then
    # tilted onto the desk; `page` turns it over about the spine (its own Y, after the tilt).
    hx, hy, hz = _on_desk(0, 0.0625, 0)
    with G.new_parts() as page:
        curl = [(hx - 0.215 * k / 6, hy + 0.012 * math.sin(math.pi * k / 6) * (1 - k / 12), hz) for k in range(7)]
        F.sweep('page_leaf', curl, None, M.page, profile=G.rounded_rect(0.003, 0.3, 0.001, corner=1), up=(0, 0, 1))
        for k in range(5):
            F.box(_name('page_text'), 0.15, 0.002, 0.008, (hx - 0.11, hy + 0.0105, hz - 0.08 + 0.035 * k), M.board)
    page = G.join(page, 'page', origin=F.at(hx, hy, hz))
    page.rotation_mode = 'YXZ'            # the tilt (X) after the turn about the spine (Y)
    page.rotation_euler = (math.radians(TILT), 0, 0)
    objs = G.settle(centre=False)
    _clip(page, 'page', 'rotation_euler', 1, 0.0, math.radians(-172), 0.9)
    return objs


# --- the wood-fired pizza oven (2x1): a brick stand and plastered dome, a prep counter at its right ---------
OX = -0.38          # the oven's middle (game x)
HEARTH = 0.92       # the oven floor and the mouth's sill
MOUTH_Z = 0.403     # the mouth's dark face


def _bricks(F, x0, x1, y0, y1, z, rows, length=0.15):
    """Brick faces in running bond on a front at depth `z`, over the box x0..x1, y0..y1."""
    h = (y1 - y0) / rows
    for r in range(rows):
        y = y0 + h * (r + 0.5)
        x = x0 - (length / 2 if r % 2 else 0)
        k = 0
        while x < x1 - 0.01:
            a, b = max(x, x0), min(x + length, x1)
            if b - a > 0.02:
                m = (M.brick, M.brick_light, M.brick)[(r + k) % 3]
                F.box(_name('brick'), b - a - 0.008, h - 0.01, 0.01, ((a + b) / 2, y, z), m)
            x += length
            k += 1


def _log(F, name, a, b, r, seed):
    """A split log from game point `a` to `b`: a crooked, barked round, its cut ends pale."""
    mid = tuple((a[i] + b[i]) / 2 + (0.008 if i == 1 else 0.0) for i in range(3))
    G.jitter(F.sweep(name, [a, mid, b], r, M.log, verts=8, resolution=2, scale=[1.0, 1.05, 0.95]), r * 0.12, size=0.04,
             seed=seed)
    for k, p in enumerate((a, b)):
        q = tuple(p[i] + (p[i] - mid[i]) * 0.002 / max(1e-6, math.dist(p, mid)) for i in range(3))
        F.sweep(f'{name}_end{k}', [p, q], r * 0.86, M.log_end, verts=8)


def _flame(F, name, x, y, z, r, h, material, flat=0.25, turn=0.0):
    """A tongue of flame standing on `y`: a body lofted from a round foot, swelling, then drawn up into a tip that
    leans; `flat` thins it front to back (flames behind a grille)."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.7), (0.15, 1.0), (0.4, 0.85), (0.65, 0.5), (0.85, 0.22), (1.0, 0.02)]
    secs = []
    for t, s in rows:
        lean = 0.3 * r * t * t
        secs.append(G.ring(r * s, r * s * flat, bz + h * t, 8, at=(bx - lean * G.cosd(turn), by)))
    return G.loft(name, secs, material=material)


def pizzaOven():
    F = G.GameFrame()
    front = 0.43
    with G.new_parts() as still:
        # the stand: two brick pillars, a lintel over the log niche, the hearth slab on top
        for s in (-1, 1):
            x = OX + s * 0.45
            F.box(_name('pillar'), 0.16, 0.7, front - WALL, (x, 0.35, (front + WALL) / 2), M.brick)
            _bricks(F, x - 0.08, x + 0.08, 0.0, 0.7, front + 0.004, 9, 0.16)
        F.box('lintel', 1.06, 0.16, front - WALL, (OX, 0.78, (front + WALL) / 2), M.brick)
        _bricks(F, OX - 0.53, OX + 0.53, 0.7, 0.86, front + 0.004, 2, 0.2)
        F.box('niche_back', 0.76, 0.7, 0.02, (OX, 0.35, WALL + 0.01), M.brick_dark)
        F.box('niche_floor', 0.76, 0.03, front - WALL - 0.02, (OX, 0.015, (front + WALL) / 2 - 0.01), M.stone)
        for i in range(3):
            for j in range(4 - i):
                x, y = OX - 0.2 + j * 0.13 + i * 0.065, 0.08 + i * 0.088
                _log(F, _name('log'), (x, y, -0.26), (x + 0.01 * ((i + j) % 3 - 1), y, 0.34), 0.05, i * 4 + j)
        F.extrude('hearth', G.rounded_rect(1.12, front - WALL + 0.02, 0.02, corner=2), 0.06,
                  (OX, HEARTH - 0.03, (front + WALL) / 2 + 0.01), M.stone, plane='top', bevel=0.01, bevel_segments=1)
        # the dome, the brick arch of its mouth and the flue
        dome = G.revolve('dome', [(0, 0), (0.46, 0), (0.458, 0.06), (0.448, 0.14), (0.42, 0.24), (0.38, 0.33), (0.32, 0.42),
                                  (0.24, 0.5), (0.14, 0.565), (0.06, 0.594), (0, 0.6)], at=F.at(OX, HEARTH, -0.06),
                         material=M.plaster, segments=32, scale=(1, 0.82, 1))
        G.jitter(dome, 0.006, size=0.12, seed=3.0)
        F.extrude('arch', [(0.3 * G.cosd(a), 0.3 * G.sind(a)) for a in range(0, 181, 10)] +
                  [(0.2 * G.cosd(a), 0.2 * G.sind(a)) for a in range(180, -1, -10)], 0.14, (OX, HEARTH, 0.33), M.brick,
                  bevel=0.006, bevel_segments=1)
        F.cyl('mouth', 0.2, 0.006, (OX, HEARTH, MOUTH_Z), M.mouth, rot=(90, 0, 0), verts=24)
        for k in range(9):
            a = 10 + 20 * k
            F.box(_name('voussoir'), 0.075, 0.09, 0.012,
                  (OX + 0.25 * G.cosd(a), HEARTH + 0.25 * G.sind(a), MOUTH_Z + 0.003),
                  (M.brick_light, M.brick_dark)[k % 2], rot=(0, a - 90, 0), bevel=0.002, bevel_segments=1)
        for k, (dx, r) in enumerate(((-0.07, 0.04), (0.0, 0.05), (0.08, 0.035))):
            G.jitter(F.sphere(_name('ember'), r, (OX + dx, HEARTH + 0.004, MOUTH_Z - 0.01), M.ember, scale=(1, 0.3, 0.45),
                              segments=8, rings=4), r * 0.1, size=0.02, seed=k)
        _lofted(F, 'flue', OX + 0.12, 1.42, -0.2, [(0, 0.08), (0.04, 0.075), (0.06, 0.07), (0.42, 0.07)], M.iron, verts=14)
        G.revolve('flue_cap', [(0, 0), (0.075, 0), (0.075, 0.03), (0.13, 0.05), (0.13, 0.06), (0.04, 0.1), (0, 0.11)],
                  at=F.at(OX + 0.12, 1.79, -0.2), material=M.iron, segments=14)
        # the prep counter: a stone cabinet with an oak frame-and-panel door, its pull, a worktop
        cx = 0.56
        F.box('counter', 0.74, 0.84, 0.62, (cx, 0.42, WALL + 0.31), M.stone, bevel=0.008, bevel_segments=1)
        F.extrude('counter_top', G.rounded_rect(0.78, 0.66, 0.02, corner=2), 0.05, (cx, 0.865, WALL + 0.32), M.worktop,
                  plane='top', bevel=0.01, bevel_segments=2)
        G.kit.shaker(F, 'counter_door', 0.6, 0.62, (cx, 0.43, WALL + 0.63), 0.02, M.door_oak, rail=0.07)
        G.kit.pull(F, 'counter_pull', cx, 0.66, WALL + 0.64, 0.12, M.iron, r=0.006, out=0.025)
        # on it: a ball of dough on a floured board, a bowl of sauce, oil, a pot of basil
        G.jitter(F.sphere('dough_ball', 0.07, (cx + 0.04, 0.915, -0.25), M.dough, scale=(1, 1, 0.55), segments=12, rings=7),
                 0.004, size=0.04)
        F.cyl('flour_board', 0.16, 0.012, (cx + 0.04, 0.896, -0.25), M.board, verts=18, bevel=0.004, bevel_segments=1)
        G.revolve('bowl_sauce', [(0, 0), (0.04, 0), (0.06, 0.04), (0.062, 0.05), (0.056, 0.05), (0.05, 0.044), (0, 0.044)],
                  at=F.at(cx + 0.26, 0.89, -0.08), material=M.cup, segments=14)
        F.cyl('sauce', 0.05, 0.004, (cx + 0.26, 0.93, -0.08), M.tomato, verts=12)
        _lofted(F, 'oil', cx + 0.14, 0.89, -0.37, [(0, 0.03), (0.005, 0.035), (0.15, 0.035), (0.19, 0.014), (0.24, 0.012)],
                M.oil, verts=10)
        top = _pot(F, cx + 0.27, -0.33, 0.89, r=0.06, h=0.1)
        _basil(F, cx + 0.27, -0.33, top, 0.75)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door: an iron grille over the mouth, the fire showing through it; `door_off` lifts it aside.
    r, dz = 0.19, MOUTH_Z + 0.012
    with G.new_parts() as door:
        # its rim in one bent bar: round the arch and back along the sill
        rim = [(OX + r * G.cosd(a), HEARTH + 0.012 + r * G.sind(a), dz) for a in range(180, -1, -15)]
        G.kit.tube(F, 'door_arc', rim + [(OX - r + 0.004, HEARTH + 0.012, dz)], 0.009, M.iron, verts=6, round=1)
        for dx in (-0.12, -0.06, 0.0, 0.06, 0.12):
            top = HEARTH + 0.01 + math.sqrt(r * r - dx * dx)
            G.kit.tube(F, _name('door_bar'), [(OX + dx, HEARTH + 0.012, dz), (OX + dx, top, dz)], 0.006, M.iron, verts=6, round=1)
        G.kit.tube(F, 'door_rail', [(OX - 0.17, HEARTH + 0.1, dz), (OX + 0.17, HEARTH + 0.1, dz)], 0.006, M.iron, verts=6,
                   round=1)
        G.kit.pull(F, 'door_handle', OX, HEARTH + 0.1, dz, 0.16, M.door_oak, r=0.011, out=0.05)
    door = G.join(door, 'oven_door', origin=F.at(OX, HEARTH, dz))
    # The peel, a pizza on it, lying on the counter: `door_off` carries it round to the mouth and in.
    px, pz = 0.48, -0.1
    with G.new_parts() as peel:
        blade = G.smooth_outline([(-0.15, -0.15), (0.15, -0.15), (0.15, 0.1), (0.04, 0.16), (-0.04, 0.16), (-0.15, 0.1)], 1)
        F.extrude('peel_blade', blade, 0.01, (px, 0.896, pz), M.peel, plane='top', bevel=0.003, bevel_segments=1)
        F.sweep('peel_handle', [(px, 0.9, pz + 0.15), (px, 0.903, pz + 0.48)], None, M.peel,
                profile=G.rounded_rect(0.02, 0.035, 0.008, corner=1), up=(0, 0, 1), scale=[1.0, 0.8])
        G.revolve('pizza', [(0, 0), (0.13, 0), (0.135, 0.008), (0.128, 0.018), (0.116, 0.016), (0.11, 0.01), (0, 0.01)],
                  at=F.at(px, 0.901, pz), material=M.crust, segments=20)
        F.cyl('pizza_sauce', 0.11, 0.004, (px, 0.912, pz), M.tomato, verts=20)
        for k in range(7):
            a, d = k * 51, 0.04 + 0.04 * (k % 2)
            G.jitter(F.cyl(_name('mozzarella'), 0.018, 0.006, (px + d * G.cosd(a), 0.915, pz + d * G.sind(a)), M.mozzarella,
                           verts=8), 0.002, size=0.01, seed=k)
        for k in range(4):
            a = k * 90 + 30
            _leaf(F, _name('pizza_basil'), px + 0.05 * G.cosd(a), 0.918, pz + 0.05 * G.sind(a), 0.03, a, 0, M.basil)
    peel = G.join(peel, 'peel', origin=F.at(px, 0.896, pz))
    # The fire in the mouth, flat flames behind the grille, built full height and flattened onto the sill:
    # `flames` grows them.
    with G.new_parts() as fire:
        for k, (dx, rr, h, m) in enumerate(((-0.1, 0.035, 0.12, M.flame), (-0.04, 0.045, 0.2, M.flame),
                                            (0.03, 0.045, 0.17, M.flame), (0.1, 0.035, 0.11, M.flame),
                                            (-0.02, 0.03, 0.12, M.flame_core), (0.05, 0.025, 0.09, M.flame_core))):
            _flame(F, _name('flame'), OX + dx, HEARTH, MOUTH_Z + 0.004 + 0.002 * (k > 3), rr, h, m, turn=60 * k)
    fire = G.join(fire, 'flames', origin=F.at(OX, HEARTH, MOUTH_Z))
    fire.scale[2] = 0.001
    objs = G.settle(centre=False)
    # door_off: the door lifts off the mouth and is set down on the ground before the stand's left pillar; the
    # peel follows, round in front of the oven and in through the mouth (Blender axes: x = -game x, y = -game
    # depth, z = height).
    f = last_frame
    for i, (lift, down) in enumerate(((0.0, 0.42), (-0.06, -0.06), (0.03, -(HEARTH - 0.005)))):
        rest = door.location[i]
        _clip(door, 'door_off', 'location', i, [(1, rest), (f(0.2), rest + lift), (f(0.6), rest + down), (f(1.4), rest + down)])
    lx, ly, lz = peel.location
    # game path: up off the counter, forward, along to the oven's middle, back into the mouth
    gx, gz = OX - px, 0.25 - pz
    keys_x = [(1, lx), (f(0.35), lx), (f(0.6), lx), (f(1.0), lx - gx), (f(1.4), lx - gx)]
    keys_y = [(1, ly), (f(0.35), ly), (f(0.6), ly - 0.62), (f(1.0), ly - 0.62), (f(1.4), ly - gz)]
    keys_z = [(1, lz), (f(0.35), lz + 0.06), (f(0.6), lz + 0.06), (f(1.0), lz + 0.06), (f(1.4), lz + 0.03)]
    for i, keys in enumerate((keys_x, keys_y, keys_z)):
        _clip(peel, 'door_off', 'location', i, keys)
    G.grow(fire, 'flames', seconds=0.8)
    return objs


# --- the chef's island (2x1): a marble-topped island, a hob on it, drawers on the cook's side, two stools ----
TOP = 0.9   # the content's top (`surface.height`)


STOOL = 0.65         # the counter stools' seat top: 25 cm under the top (counter stools are 0.58-0.71 for it)
STOOL_Z = -0.235     # their middles' depth, the knees under the top's back overhang (`furniture.json`)


def _stool(F, x, z):
    """A counter stool with its seat at STOOL, a foot ring and a low curved back, centred on (x, z), facing the front:
    four splayed legs, each one tapering sweep, a saddle-dished seat with a cushion."""
    y = STOOL - 0.045                                             # the wooden seat's middle
    F.cyl(_name('stool_seat'), 0.18, 0.03, (x, y, z), M.oak, verts=20, bevel=0.008, bevel_segments=1)
    F.cyl(_name('stool_cushion'), 0.165, 0.03, (x, STOOL - 0.015, z), M.seat, verts=18, bevel=0.01, bevel_segments=2)
    for k in range(4):
        a = 45 + 90 * k
        c, s = G.cosd(a), G.sind(a)
        F.sweep(_name('stool_leg'), [(x + 0.12 * c, y - 0.015, z + 0.12 * s), (x + 0.19 * c, 0.008, z + 0.19 * s)], 0.017,
                M.oak_dark, verts=8, scale=[1.0, 0.7])
    G.torus(_name('stool_ring'), 0.165, 0.009, at=F.at(x, 0.3, z), material=M.oak_dark, major_segments=16, minor_segments=4)
    for dx in (-0.12, 0.12):
        G.kit.tube(F, _name('stool_back_post'), [(x + dx, STOOL - 0.03, z - 0.13), (x + dx, STOOL + 0.17, z - 0.17)], 0.012,
                   M.oak_dark, verts=6, round=1)
    F.sweep(_name('stool_back'), [(x - 0.16, STOOL + 0.21, z - 0.15), (x, STOOL + 0.21, z - 0.183), (x + 0.16, STOOL + 0.21, z - 0.15)],
            None, M.oak, profile=G.rounded_rect(0.1, 0.022, 0.008, corner=1), up=(0, 0, 1), resolution=3)


def chefsIsland():
    F = G.GameFrame()
    zf = 0.43            # the cook's side
    with G.new_parts() as still:
        F.extrude('top', G.rounded_rect(1.86, 0.46, 0.02, corner=2), 0.04, (0, TOP - 0.02, 0.225), M.marble, plane='top',
                  bevel=0.01, bevel_segments=2)
        for s in (-1, 1):
            x = s * 0.8
            F.box(_name('end'), 0.26, 0.78, zf, (x, 0.47, zf / 2), M.cabinet, bevel=0.006, bevel_segments=1)
            F.box(_name('end_plinth'), 0.24, 0.08, zf - 0.03, (x, 0.04, zf / 2 - 0.015), M.dark)
            G.kit.shaker(F, _name('end_drawer'), 0.22, 0.16, (x, 0.75, zf + 0.007), 0.014, M.cabinet, rail=0.03)
            G.kit.shaker(F, _name('end_door'), 0.22, 0.54, (x, 0.37, zf + 0.007), 0.014, M.cabinet, rail=0.045)
            for y in (0.79, 0.6):
                G.kit.pull(F, _name('end_pull'), x, y, zf + 0.014, 0.09, M.brass, r=0.005, out=0.012, verts=6)
        # between the ends, the drawer bank under the top; below it, the sitters' legroom open right through
        F.box('apron', 1.34, 0.16, 0.02, (0, 0.8, 0.3), M.cabinet)                 # the drawers' back
        F.box('apron_under', 1.34, 0.02, zf - 0.3, (0, 0.73, (zf + 0.3) / 2), M.cabinet)
        for x in (-0.45, 0.45):
            G.kit.shaker(F, _name('side_drawer'), 0.42, 0.13, (x, 0.8, zf + 0.007), 0.014, M.cabinet, rail=0.03)
            G.kit.pull(F, _name('side_pull'), x, 0.8, zf + 0.014, 0.12, M.brass, r=0.005, out=0.012, verts=6)
        # the sitters' side: their knee space under the top, a brass foot rail on brackets
        G.kit.tube(F, 'foot_rail', G.kit.fillet([(-0.67, 0.25, 0.2), (-0.67, 0.25, 0.06), (0.67, 0.25, 0.06), (0.67, 0.25, 0.2)],
                                                0.04), 0.012, M.brass, verts=8, round=1)
        # on the top: the hob with a copper pan and a stock pot, a board of vegetables, two tasting plates
        F.extrude('hob', G.rounded_rect(0.5, 0.28, 0.02, corner=2), 0.008, (0, TOP + 0.004, 0.27), M.hob, plane='top',
                  bevel=0.002, bevel_segments=1)
        for dx in (-0.12, 0.12):
            G.torus(_name('hob_ring'), 0.08, 0.003, at=F.at(dx, TOP + 0.009, 0.27), material=M.ring, major_segments=16,
                    minor_segments=3)
            F.cyl(_name('hob_mark'), 0.05, 0.001, (dx, TOP + 0.0085, 0.27), M.ring, verts=12)
        G.revolve('pan', [(0, 0), (0.1, 0), (0.112, 0.01), (0.118, 0.05), (0.112, 0.05), (0.106, 0.012), (0, 0.012)],
                  at=F.at(-0.12, TOP + 0.008, 0.27), material=M.copper, segments=18)
        F.sweep('pan_handle', [(-0.22, TOP + 0.045, 0.274), (-0.3, TOP + 0.05, 0.288), (-0.4, TOP + 0.056, 0.3)], None,
                M.oak_dark, profile=G.rounded_rect(0.014, 0.022, 0.006, corner=1), up=(0, 0, 1), resolution=2)
        G.revolve('stock_pot', [(0, 0), (0.11, 0), (0.12, 0.01), (0.12, 0.18), (0.112, 0.18), (0.112, 0.012), (0, 0.012)],
                  at=F.at(0.12, TOP + 0.008, 0.27), material=M.worktop, segments=18)
        G.revolve('pot_lid', [(0, 0), (0.122, 0), (0.12, 0.008), (0.08, 0.02), (0, 0.024)], at=F.at(0.12, TOP + 0.188, 0.27),
                  material=M.worktop, segments=18)
        G.revolve('pot_knob', [(0, 0), (0.008, 0), (0.008, 0.01), (0.016, 0.016), (0.012, 0.024), (0, 0.025)],
                  at=F.at(0.12, TOP + 0.21, 0.27), material=M.black, segments=8)
        for s in (-1, 1):
            G.kit.tube(F, _name('pot_handle'), G.kit.fillet([(0.12 + s * 0.118, TOP + 0.16, 0.25), (0.12 + s * 0.15, TOP + 0.16, 0.255),
                                                             (0.12 + s * 0.15, TOP + 0.16, 0.285), (0.12 + s * 0.118, TOP + 0.16, 0.29)],
                                                            0.008), 0.005, M.chrome, verts=5, round=1)
        F.extrude('chop_board', G.rounded_rect(0.34, 0.22, 0.02, corner=2), 0.02, (0.62, TOP + 0.01, 0.3), M.oak, plane='top',
                  bevel=0.004, bevel_segments=1)
        F.sphere('tomato', 0.035, (0.56, TOP + 0.052, 0.3), M.tomato, scale=(1, 1, 0.9), segments=10, rings=6)
        F.cone('tomato_calyx', 0.014, 0.0, 0.008, (0.56, TOP + 0.087, 0.3), M.basil, verts=5)
        F.sweep('carrot', [(0.6, TOP + 0.04, 0.24), (0.7, TOP + 0.038, 0.3)], 0.018, M.carrot, verts=8, scale=[1.0, 0.15])
        for k in range(3):
            _leaf(F, _name('carrot_top'), 0.6, TOP + 0.04, 0.24, 0.05, 210 + 25 * k, 20, M.basil)
        F.box('knife', 0.16, 0.003, 0.025, (0.6, TOP + 0.022, 0.37), M.chrome)
        F.box('knife_grip', 0.08, 0.016, 0.02, (0.72, TOP + 0.028, 0.37), M.black, bevel=0.005, bevel_segments=1)
        for x in (-0.5, 0.5):
            G.revolve(_name('plate'), [(0, 0), (0.07, 0), (0.11, 0.012), (0.105, 0.016), (0.068, 0.006), (0, 0.006)],
                      at=F.at(x, TOP, 0.11), material=M.plate, segments=16)
            F.sphere(_name('bite'), 0.03, (x - 0.02, TOP + 0.02, 0.11), M.salami, scale=(1, 1, 0.5), segments=8, rings=5)
            F.extrude(_name('bite'), [(0.0, 0.0), (0.05, -0.016), (0.05, 0.016)], 0.025, (x + 0.01, TOP + 0.02, 0.1),
                      M.cheese_light, plane='top', rot=(0, 0, 30), bevel=0.003, bevel_segments=1)
        F.box('menu', 0.14, 0.18, 0.006, (0.0, TOP + 0.08, 0.06), M.chalk, rot=(-12, 0, 0), bevel=0.002, bevel_segments=1)
        F.box('menu_stand', 0.02, 0.12, 0.006, (0.0, TOP + 0.055, 0.03), M.oak_dark, rot=(18, 0, 0))
        _stool(F, -0.5, STOOL_Z)
        _stool(F, 0.5, STOOL_Z)
    G.join(still, 'body', origin=(0, 0, 0))
    # The middle drawer on the cook's side: `drawer_open` pulls it out 12 cm.
    with G.new_parts() as drawer:
        G.kit.shaker(F, 'drawer_front', 0.42, 0.13, (0, 0.8, zf + 0.007), 0.014, M.cabinet, rail=0.03)
        G.kit.pull(F, 'drawer_pull', 0, 0.8, zf + 0.014, 0.12, M.brass, r=0.005, out=0.012)
        F.box('drawer_floor', 0.38, 0.01, 0.12, (0, 0.745, zf - 0.06), M.cabinet)
        for x in (-0.19, 0.19):
            F.box(_name('drawer_side'), 0.01, 0.08, 0.12, (x, 0.785, zf - 0.06), M.cabinet)
        for k, x in enumerate((-0.12, -0.05, 0.03, 0.1)):
            F.box(_name('utensil'), 0.02, 0.008, 0.11, (x, 0.755, zf - 0.06), (M.chrome, M.oak_top)[k % 2], bevel=0.003,
                  bevel_segments=1)
    drawer = G.join(drawer, 'drawer')
    # The ring under the copper pan glowing hot, scaled to nothing at rest: `burner_on` grows it while it cooks.
    with G.new_parts() as heat:
        F.cyl('hot_ring', 0.13, 0.003, (-0.12, TOP + 0.0085, 0.27), M.heat, verts=24)
    heat = G.join(heat, 'hot_ring', origin=F.at(-0.12, TOP + 0.0085, 0.27))
    heat.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.12, 0), seconds=0.6)
    for axis in 'XYZ':
        G.grow(heat, 'burner_on', seconds=0.5, axis=axis)
    return objs


FAMILY = G.Family(
    'pack_foodie', M,
    [G.Model('herbGarden', herbGarden, size=(0.86, 1.293, 0.763), key='model.foodie.herbGarden', centre=(0, -0.054),
             motions={'uses': {'tendHerbs': 'sprig'}},
             paint=[('cedar', 'Body'), ('terracotta', 'Pots')]),
     G.Model('deliPantry', deliPantry, size=(0.88, 1.99, 0.57), key='model.foodie.deliPantry', centre=(0, -0.155),
             motions={'uses': {'nibble': 'doors_open', 'cheeseBoard': 'doors_open'}, 'hold': 2.5},
             paint=[('cream', 'Body'), ('marble', 'Worktop'), ('honey', 'Shelves')]),
     G.Model('espressoBar', espressoBar, size=(0.76, 1.509, 0.514), key='model.foodie.espressoBar', centre=(0, -0.18),
             motions={'uses': {'espresso': 'brew', 'latteArt': 'brew', 'pour': 'brew'}, 'hold': 4.0},
             paint=[('teal', 'Body'), ('oak_top', 'Worktop'), ('machine', 'Machine')]),
     G.Model('cookbookLectern', cookbookLectern, size=(0.836, 1.147, 0.588), key='model.foodie.cookbookLectern',
             centre=(0.022, -0.123), motions={'uses': {'studyRecipes': 'page', 'foodMagazines': 'page'}},
             paint=[('walnut', 'Body'), ('honey_wood', 'Stool & ledge'), ('cover', 'Cookbook')]),
     G.Model('pizzaOven', pizzaOven, size=(1.89, 1.9, 0.916), key='model.foodie.pizzaOven', centre=(0.005, 0.018),
             # a bake (the pizza, or any `bake` step of a recipe at it) takes the door off and lights the fire
             motions={'uses': {'bakePizza': ['door_off', 'flames'], 'bake': ['door_off', 'flames'], 'warmUp': 'flames'}},
             paint=[('plaster', 'Body'), ('worktop', 'Worktop'), ('brick', 'Brick')]),
     G.Model('chefsIsland', chefsIsland, size=(1.86, 1.128, 0.889), key='model.foodie.chefsIsland', centre=(0, 0.0154),
             # the drawer opens for the prep step's utensils (chop), the ring under the pan glows while it cooks (stir):
             # keyed by the recipe's step tags, so each step at it plays its own (A-systems-03)
             motions={'uses': {'chop': {'clip': 'drawer_open', 'hold': 2.5}, 'stir': 'burner_on'}},
             paint=[('cabinet', 'Body'), ('marble', 'Worktop'), ('oak', 'Stools')])],
    comment='Blender-built foodie pack models (tools/art/blender/families/pack_foodie.py). Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw '
            'call per finish group. A sprig grows, the pantry doors swing, the coffee runs, a page turns, '
            'the oven door lifts aside for the peel as the fire grows, and the island\'s drawer pulls out.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
