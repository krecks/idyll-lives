# Model entries for the 0.20 catalogue (`model.<id>[@modern|@cozy|@minimal]`): the furniture, lighting,
# decor, kids', hobby and garden things in web/public/content/furniture.json.
#
# Every model is drawn here from primitive parts (box, cylinder, cone, sphere, capsule, blob, lathe, torus;
# original, CC0), several with the shared shapes of catalog2.py; no downloads. Same conventions as
# furniture.py: 1 unit = 1 m, origin at the footprint centre on the floor, front +Z, back against the
# wall at -Z. Things on a wall (content `layer: wall`) hang with their back on the wall face
# (z = -0.43 in a 1-deep footprint: half a wall's thickness in front of the tile edge); things on the
# ceiling hang from 2.8 m. Sizes, seat heights (render/babylon/characters.ts SEATS / BEDS) and tops
# (content `surface`) are those of the downloaded models these replaced, so content still lines up.
#
# Style direction as in furniture.py: modern = white, light grey, white oak, black-metal accents;
# cozy = warm woods, cream, earthy fabrics; minimal = white, birch, pale linen.
#
# Run: python3 -I -B catalog.py   (writes web/public/assets/catalog/manifest.json, a manifest listed in the base
# manifest's `packs`; one entry per line keeps it small)
import json, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import objects_extra as OX                                          # noqa: E402
from objects_extra import box, cyl, cone, sph, rod, part            # noqa: E402
from furniture import (placeholder, cushion, lathe, torus, office_chair,  # noqa: E402
                       OAK, OAKD, BIRCH, HONEY, WALNUT, BLACKMETAL, STEEL,
                       CHROME, BRASS, TEAL, COGNAC, MUSTARD, SAGE, RUST, CREAM, SCREEN)
import catalog2 as C2                                               # noqa: E402
from catalog2 import flare, handle_h, handle_v, books_row, case, VASE, BOWL  # noqa: E402

E = {}
FOOT = {}
STRICT = set()        # models held to the full checks (at most 4 finishes)
CEILING = 2.8
WALL = -0.43          # a wall thing's back (the wall face)
LINEN, SAND, CLAY, NAVY, BLUSH, OLIVE = '#E7E0D4', '#D9C7A7', '#C98F6B', '#3E5470', '#E4B8AE', '#8C9468'
BULB = '#FFF1CF'
FOLIAGE = '#5E8F4E'
PLINTH, PORCELAIN, MIRROR, CHARCOAL = '#2F3236', '#F6F6F4', '#CFDCE2', '#3A3D41'


def add(key, entry, fp=(1, 1)):
    E['model.' + key] = entry
    FOOT['model.' + key] = fp


def addp(key, parts, fp=(1, 1)):
    """A primitive-part model held to the full checks."""
    add(key, placeholder(parts), fp)
    STRICT.add('model.' + key)


def fin_of(color, metal_fin='metal'):
    """The finish a handle or trim colour is drawn with."""
    return 'chrome' if color == CHROME else metal_fin


def facing_back(parts):
    """The parts turned half way round (a thing built facing +Z, now facing -Z)."""
    out = []
    for q in parts:
        q = json.loads(json.dumps(q))
        q['at'] = [round(-q['at'][0], 4), q['at'][1], round(-q['at'][2], 4)]
        r = q.get('rotate', [0, 0, 0])
        q['rotate'] = [r[0], round(r[1] + 180, 2), r[2]]
        out.append(q)
    return out


def on_plane(centre, rx, dx, dy, dz=0.0):
    """A point on a panel centred on `centre` and tilted by `rx` degrees about X (local x, y across it, z out)."""
    return OX.add(centre, OX.rot([dx, dy, dz], [rx, 0, 0]))


def books(x0, x1, y, z, depth=0.2, seed=1, colors=(TEAL, RUST, MUSTARD, SAGE, CREAM, NAVY, CLAY)):
    """A row of books standing on a shelf from x0 to x1 (bottom at y)."""
    p, x, k = [], x0, seed
    while x < x1 - 0.03:
        k = (k * 7 + 3) % 11
        w = 0.025 + (k % 4) * 0.008
        h = 0.17 + (k % 5) * 0.02
        if x + w > x1: break
        p.append(box([w, h, depth], [x + w / 2, y + h / 2, z], colors[k % len(colors)], 'matte', radius=0.003))
        x += w + 0.004
        if k % 7 == 0: x += 0.06
    return p


def chair_facing_front(x, z, wood, seat, fin='wood', seat_h=0.46, back_h=0.92):
    """A simple dining chair whose sitter faces +Z (seat centred on (x, z))."""
    return [
        box([0.42, 0.04, 0.4], [x, seat_h - 0.04, z], wood, fin, radius=0.012),
        cushion([0.38, 0.04, 0.36], [x, seat_h - 0.005, z + 0.01], seat, radius=0.018),
        box([0.035, seat_h - 0.05, 0.035], [x - 0.18, (seat_h - 0.05) / 2, z + 0.17], wood, fin, radius=0.008),
        box([0.035, seat_h - 0.05, 0.035], [x + 0.18, (seat_h - 0.05) / 2, z + 0.17], wood, fin, radius=0.008),
        box([0.035, back_h, 0.035], [x - 0.18, back_h / 2, z - 0.18], wood, fin, radius=0.008),
        box([0.035, back_h, 0.035], [x + 0.18, back_h / 2, z - 0.18], wood, fin, radius=0.008),
        box([0.4, 0.1, 0.035], [x, back_h - 0.07, z - 0.18], wood, fin, radius=0.015),
        box([0.4, 0.04, 0.03], [x, back_h - 0.28, z - 0.18], wood, fin, radius=0.01),
    ]


# =============================================================================================
# Kitchen: appliances
# =============================================================================================
# Mini fridge: a white box with a chrome-handled door, a chopping board and a snack on top.
p = [box([0.6, 0.82, 0.56], [0, 0.43, -0.18], '#F3F3F1', 'gloss', radius=0.03),
     box([0.56, 0.04, 0.52], [0, 0.02, -0.2], PLINTH, 'satin', radius=0.006),
     box([0.58, 0.6, 0.025], [0, 0.39, 0.1], '#F3F3F1', 'gloss', radius=0.02),                          # door
     box([0.58, 0.16, 0.025], [0, 0.765, 0.1], '#F3F3F1', 'gloss', radius=0.02),                        # freezer flap
     box([0.5, 0.018, 0.026], [0, 0.67, 0.112], '#C9CBCC', 'satin', radius=0.006),
     box([0.018, 0.24, 0.03], [0.24, 0.48, 0.125], CHROME, 'chrome', radius=0.008),
     box([0.12, 0.012, 0.03], [0.2, 0.765, 0.125], CHROME, 'chrome', radius=0.006),
     box([0.36, 0.04, 0.012], [0, 0.06, 0.1], '#C9CBCC', 'satin', radius=0.004),                         # vent
     box([0.5, 0.02, 0.4], [0, 0.87, -0.12], OAK, 'wood', radius=0.006),                                 # board on top
     cyl([0.1, 0.12, 0.1], [-0.14, 0.94, -0.14], '#E9E6E0', 'gloss', taper=0.8),                         # jar
     sph([0.09, 0.08, 0.09], [0.12, 0.92, -0.1], '#D9483B', 'satin', segments=8)]                        # apple
addp('fridgeMini', p)


def builtin_fridge(cab, cfin, door, dfin, handle, hfin):
    """A tall fridge-freezer behind cabinet fronts: a fridge door over a freezer drawer, long bar handles."""
    p = [box([0.82, 1.92, 0.6], [0, 1.06, -0.16], cab, cfin, radius=0.012),
         box([0.78, 0.1, 0.56], [0, 0.05, -0.18], PLINTH, 'satin', radius=0.004),
         box([0.84, 0.05, 0.62], [0, 2.0, -0.15], cab, cfin, radius=0.008),                                # cornice
         box([0.78, 1.18, 0.025], [0, 1.39, 0.152], door, dfin, radius=0.01),                               # fridge door
         box([0.78, 0.6, 0.025], [0, 0.48, 0.152], door, dfin, radius=0.01),                                # freezer drawer
         box([0.022, 0.62, 0.032], [0.33, 1.38, 0.18], handle, hfin, radius=0.008),
         box([0.5, 0.022, 0.032], [0, 0.72, 0.18], handle, hfin, radius=0.008),
         box([0.14, 0.2, 0.01], [-0.24, 1.42, 0.168], '#2B2C2E', 'gloss', radius=0.012),                   # water and ice
         box([0.08, 0.025, 0.004], [-0.24, 1.48, 0.174], '#7FC8E8', 'gloss', radius=0.003),
         box([0.6, 0.05, 0.01], [0, 0.07, 0.11], '#3A3D41', 'satin', radius=0.004)]                         # vent grille
    return p


addp('fridgeBuiltIn', builtin_fridge('#EDEBE6', 'satin', '#E2DFD8', 'satin', CHROME, 'chrome'))
addp('fridgeBuiltIn@modern', builtin_fridge('#F4F4F2', 'gloss', OAK, 'wood', BLACKMETAL, 'metal'))
addp('fridgeBuiltIn@cozy', builtin_fridge(SAGE, 'satin', '#8FA27E', 'satin', BRASS, 'metal'))
addp('fridgeBuiltIn@minimal', builtin_fridge(BIRCH, 'wood', '#F2F1EE', 'satin', '#C9C6BE', 'satin'))


def gas_range(body, fin='satin', steel=STEEL, knob='#3A3D41'):
    """A gas cooker: four burners under cast-iron grates, an oven with a window, knobs on a strip."""
    p = [box([0.9, 0.8, 0.6], [0, 0.5, -0.16], body, fin, radius=0.014),
         box([0.86, 0.1, 0.56], [0, 0.05, -0.18], PLINTH, 'satin', radius=0.004),
         box([0.92, 0.03, 0.64], [0, 0.915, -0.15], steel, 'metal', radius=0.01),                         # hob
         box([0.92, 0.12, 0.04], [0, 0.99, -0.45], steel, 'metal', radius=0.01),                          # back guard
         box([0.84, 0.48, 0.025], [0, 0.42, 0.152], body, fin, radius=0.012),                              # oven door
         box([0.56, 0.24, 0.008], [0, 0.43, 0.168], '#1D2228', 'gloss', radius=0.012),                     # window
         box([0.64, 0.022, 0.03], [0, 0.62, 0.19], steel, 'metal', radius=0.01),                           # handle
         box([0.84, 0.1, 0.02], [0, 0.82, 0.15], '#2F3236', 'satin', radius=0.008),                        # knob strip
         box([0.84, 0.1, 0.02], [0, 0.12, 0.15], body, fin, radius=0.008)]                                 # drawer
    for i in range(5):
        p.append(cyl([0.045, 0.03, 0.045], [-0.3 + i * 0.15, 0.82, 0.172], knob, 'satin', rotate=[90, 0, 0], segments=12))
    for x in (-0.22, 0.22):
        for z in (-0.32, 0.0):
            r = 0.15 if (x < 0) == (z < 0) else 0.12
            p += [cyl([r, 0.02, r], [x, 0.94, z], '#22252A', 'satin', segments=16),
                  cyl([r * 0.55, 0.03, r * 0.55], [x, 0.95, z], '#3A3D41', 'satin', segments=12)]
    for x in (-0.22, 0.22):                                                                              # grates
        p += [box([0.4, 0.018, 0.018], [x, 0.968, z], '#22252A', 'satin', radius=0.004) for z in (-0.32, 0.0)]
        p += [box([0.018, 0.018, 0.62], [x + d, 0.968, -0.16], '#22252A', 'satin', radius=0.004) for d in (-0.1, 0.1)]
    p += [cyl([0.26, 0.12, 0.26], [-0.22, 1.04, -0.32], '#B9BDC1', 'metal', segments=18),                  # pot
          cyl([0.27, 0.015, 0.27], [-0.22, 1.105, -0.32], '#B9BDC1', 'metal', segments=18),
          cyl([0.28, 0.03, 0.28], [0.22, 0.99, 0.0], '#2F3236', 'satin', segments=18),                     # pan
          rod([0.34, 0.99, 0.08], [0.42, 1.0, 0.24], 0.025, '#2F3236', 'satin', shape='box')]
    return p


addp('stoveGas', gas_range('#EDEBE6'))
addp('stoveGas@modern', gas_range('#F4F4F2', 'gloss', knob=BLACKMETAL))
addp('stoveGas@cozy', gas_range('#C9B79C', steel='#D8C9A6', knob=WALNUT))
addp('stoveGas@minimal', gas_range(BIRCH, 'wood', knob='#E8E6E1'))

# Electric cooker: a freestanding white enamel cooker, black ceramic hob, a big oven window.
p = [box([0.6, 0.86, 0.6], [0, 0.47, -0.16], '#F4F3EF', 'gloss', radius=0.03),
     box([0.56, 0.04, 0.56], [0, 0.02, -0.18], PLINTH, 'satin', radius=0.006),
     box([0.6, 0.02, 0.6], [0, 0.91, -0.16], '#1B1D20', 'gloss', radius=0.012),                           # glass hob
     box([0.6, 0.06, 0.1], [0, 0.95, -0.41], '#F4F3EF', 'gloss', radius=0.02),                            # back panel
     box([0.56, 0.5, 0.025], [0, 0.5, 0.152], '#F4F3EF', 'gloss', radius=0.015),                          # oven door
     box([0.44, 0.3, 0.008], [0, 0.48, 0.168], '#1D2228', 'gloss', radius=0.02),
     cyl([0.022, 0.46, 0.022], [0, 0.71, 0.19], CHROME, 'chrome', rotate=[0, 0, 90]),
     box([0.56, 0.14, 0.025], [0, 0.13, 0.152], '#F4F3EF', 'gloss', radius=0.012),                        # store drawer
     box([0.2, 0.016, 0.02], [0, 0.17, 0.172], CHROME, 'chrome', radius=0.006),
     box([0.56, 0.06, 0.01], [0, 0.86, 0.148], '#2F3236', 'satin', radius=0.006)]
for x, z, r in ((-0.14, -0.3, 0.2), (0.14, -0.3, 0.16), (-0.14, -0.04, 0.16), (0.14, -0.04, 0.2)):
    p.append(torus([r, 0.008, r], [x, 0.922, z], '#6B6E73', 'satin', segments=20))
for i in range(4):
    p.append(cyl([0.04, 0.025, 0.04], [-0.18 + i * 0.12, 0.95, -0.355], CHROME, 'chrome', rotate=[90, 0, 0], segments=12))
p.append(sph([0.015, 0.015, 0.01], [0.24, 0.86, 0.155], '#E9853B', 'gloss', segments=6))
addp('stoveElectric', p)


class Kit:
    """A kitchen style: carcass, fronts (doors and drawers), pulls and worktop."""
    def __init__(self, body, bfin, front, ffin, handle, hfin, top, tfin, pull, shaker=False):
        self.body, self.bfin, self.front, self.ffin = body, bfin, front, ffin
        self.handle, self.hfin, self.top, self.tfin, self.pull, self.shaker = handle, hfin, top, tfin, pull, shaker


KITCHEN = {
    '': Kit('#EDEBE6', 'satin', '#E2DFD8', 'satin', CHROME, 'chrome', '#E8E6E1', 'gloss', 'classic'),
    '@modern': Kit('#F4F4F2', 'gloss', '#F4F4F2', 'gloss', BLACKMETAL, 'metal', '#C9CBCC', 'satin', 'bar'),
    '@cozy': Kit(SAGE, 'satin', SAGE, 'satin', BRASS, 'metal', HONEY, 'wood', 'knob', shaker=True),
    '@minimal': Kit(BIRCH, 'wood', BIRCH, 'wood', '#8C8780', 'satin', '#F2F2F0', 'satin', 'slot'),
}


def pull(s, x, y, z, horizontal, w):
    """A handle on a front whose face is at z: classic bars, long modern bars, cozy knobs or minimal finger slots."""
    if s.pull == 'knob':
        return sph([0.036, 0.036, 0.03], [x, y, z + 0.006], s.handle, s.hfin, segments=10)
    if s.pull == 'slot':
        return box([min(0.3, w * 0.6), 0.014, 0.006] if horizontal else [0.014, 0.16, 0.006], [x, y, z - 0.001], s.handle, s.hfin, radius=0.003)
    if s.pull == 'bar':
        return box([min(0.36, w * 0.7), 0.016, 0.022], [x, y, z + 0.008], s.handle, s.hfin, radius=0.007)
    return handle_h(x, y, z + 0.008, s.handle, s.hfin, w=min(0.18, w * 0.4)) if horizontal else handle_v(x, y, z + 0.008, s.handle, s.hfin)


def front(s, x, w, y0, y1, z, side=0, drawer=False, handle_y=None):
    """A door or drawer front `w` wide from y0 to y1, its back on the carcass face at z. A door's pull goes
    near its `side` edge (-1 / 1) and its top (or at `handle_y`)."""
    h = y1 - y0
    p = [box([w - 0.006, h, 0.02], [x, (y0 + y1) / 2, z + 0.01], s.front, s.ffin, radius=0.008)]
    zf = z + 0.02
    if s.shaker and h > 0.18:                                                   # a raised frame round the panel
        t = 0.06 if h > 0.3 else 0.04
        p += [box([t, h - 0.02, 0.012], [x + d * (w / 2 - t / 2 - 0.013), (y0 + y1) / 2, zf + 0.006], s.front, s.ffin, radius=0.004) for d in (-1, 1)]
        p += [box([w - 0.03, t, 0.012], [x, yy, zf + 0.006], s.front, s.ffin, radius=0.004) for yy in (y0 + t / 2 + 0.01, y1 - t / 2 - 0.01)]
        zf += 0.012
    if handle_y is None:
        handle_y = (y0 + y1) / 2 if drawer else (y1 - 0.05 if s.pull in ('bar', 'slot') else y1 - 0.1)
    centred = drawer or not side or s.pull in ('bar', 'slot')
    p.append(pull(s, x if centred else x + side * (w / 2 - 0.06), handle_y, zf, drawer or s.pull in ('bar', 'slot'), w))
    return p


def k_base(s, w=1.0, top=True):
    """A base unit: carcass y 0.1..0.92 (front face z = 0.14), a dark plinth and a worktop to 0.96 (z -0.48..0.18)."""
    p = [box([w - 0.004, 0.82, 0.6], [0, 0.51, -0.16], s.body, s.bfin, radius=0.01),
         box([w - 0.04, 0.1, 0.56], [0, 0.05, -0.18], PLINTH, 'satin', radius=0.004)]
    if top:
        p.append(box([w, 0.04, 0.66], [0, 0.94, -0.15], s.top, s.tfin, radius=0.006))
    return p


def counter(s):
    """A drawer over two doors."""
    return (k_base(s) + front(s, 0, 0.98, 0.73, 0.9, 0.14, drawer=True)
            + front(s, -0.245, 0.49, 0.12, 0.72, 0.14, side=1) + front(s, 0.245, 0.49, 0.12, 0.72, 0.14, side=-1))


def corner_counter(s):
    """A deep corner unit (fills its tile, for the corner of an L): doors at the front, a worktop over all."""
    p = [box([1.0, 0.82, 0.96], [0, 0.51, -0.02], s.body, s.bfin, radius=0.012),
         box([0.96, 0.1, 0.92], [0, 0.05, -0.04], PLINTH, 'satin', radius=0.006),
         box([1.0, 0.04, 1.0], [0, 0.94, 0.0], s.top, s.tfin, radius=0.006)]
    p += front(s, -0.245, 0.49, 0.12, 0.9, 0.46, side=1) + front(s, 0.245, 0.49, 0.12, 0.9, 0.46, side=-1)
    p += [cyl([0.14, 0.18, 0.14], [-0.3, 1.05, -0.3], '#E8E6E1', 'gloss', taper=0.85),            # utensil pot
          cyl([0.02, 0.2, 0.02], [-0.32, 1.17, -0.3], OAK, 'wood'), cyl([0.02, 0.2, 0.02], [-0.28, 1.16, -0.31], OAK, 'wood')]
    return p


def wall_cabinet(s):
    """Two doors, pulls at the bottom, a light strip under it."""
    y0, h, d = 1.45, 0.72, 0.34
    p = [box([0.98, h, d], [0, y0 + h / 2, WALL + d / 2], s.body, s.bfin, radius=0.01)]
    for x, side in ((-0.245, 1), (0.245, -1)):
        p += front(s, x, 0.49, y0 + 0.01, y0 + h - 0.01, WALL + d, side=side, handle_y=y0 + (0.05 if s.pull in ('bar', 'slot') else 0.1))
    p.append(box([0.9, 0.012, 0.03], [0, y0 - 0.006, WALL + 0.07], BULB, 'gloss', radius=0.004))
    return p


def pantry(s):
    """A tall larder: two doors under a cornice, a basket on top."""
    p = [box([0.92, 1.92, 0.6], [0, 1.06, -0.16], s.body, s.bfin, radius=0.012),
         box([0.88, 0.1, 0.56], [0, 0.05, -0.18], PLINTH, 'satin', radius=0.004),
         box([0.94, 0.04, 0.62], [0, 2.04, -0.15], s.body, s.bfin, radius=0.008)]
    p += front(s, -0.228, 0.456, 0.12, 2.0, 0.14, side=1, handle_y=1.05) + front(s, 0.228, 0.456, 0.12, 2.0, 0.14, side=-1, handle_y=1.05)
    p.append(box([0.36, 0.2, 0.3], [-0.2, 2.16, -0.2], '#C9A979', 'fabric', radius=0.04))
    return p


def microwave_counter(s):
    x, z = 0.18, -0.24
    p = counter(s)
    p += [box([0.5, 0.3, 0.36], [x, 1.11, z], CHARCOAL, 'satin', radius=0.02),
          box([0.36, 0.26, 0.014], [x - 0.06, 1.11, z + 0.186], '#2A2C30', 'satin', radius=0.01),          # door
          box([0.28, 0.17, 0.006], [x - 0.07, 1.11, z + 0.195], '#1D2228', 'gloss', radius=0.008),         # window
          box([0.1, 0.24, 0.01], [x + 0.19, 1.11, z + 0.183], '#22252A', 'satin', radius=0.006),
          box([0.06, 0.02, 0.006], [x + 0.19, 1.2, z + 0.19], '#5EC28B', 'gloss', radius=0.003)]           # display
    p += [box([0.03, 0.02, 0.006], [x + 0.175 + (i % 2) * 0.035, 1.15 - (i // 2) * 0.035, z + 0.19], '#C9CBCC', 'satin', radius=0.004) for i in range(6)]
    return p


def coffee_counter(s):
    x, z = -0.14, -0.26
    steel = CHROME if s.hfin == 'chrome' else STEEL
    p = counter(s)
    p += [box([0.34, 0.3, 0.32], [x, 1.11, z], CHARCOAL, 'satin', radius=0.03),
          box([0.26, 0.14, 0.03], [x, 1.04, z + 0.15], '#22252A', 'satin', radius=0.01),                   # cup bay
          box([0.34, 0.03, 0.32], [x, 1.275, z], steel, s.hfin, radius=0.01),                              # warming top
          cyl([0.08, 0.04, 0.08], [x, 1.16, z + 0.15], steel, s.hfin, segments=14),                        # group head
          box([0.03, 0.025, 0.12], [x, 1.13, z + 0.22], '#22252A', 'satin', radius=0.01),                  # portafilter handle
          box([0.28, 0.02, 0.1], [x, 0.975, z + 0.15], steel, s.hfin, radius=0.006),                       # drip tray
          cyl([0.07, 0.06, 0.07], [x, 1.015, z + 0.15], '#F2F1EE', 'gloss', segments=14),                  # cup
          cyl([0.06, 0.012, 0.06], [x + 0.08, 1.22, z + 0.162], '#F2F1EE', 'gloss', rotate=[90, 0, 0], segments=12),  # gauge
          cyl([0.11, 0.12, 0.11], [x + 0.08, 1.35, z - 0.06], '#5A3A26', 'gloss', segments=14, taper=1.15),  # beans
          cyl([0.08, 0.1, 0.08], [0.32, 1.01, 0.05], TEAL, 'gloss'), cyl([0.08, 0.1, 0.08], [0.42, 1.01, 0.0], CREAM, 'gloss')]
    return p


def breakfast_counter(s, toaster):
    p = counter(s)
    p += [box([0.28, 0.18, 0.16], [0.3, 1.05, -0.3], toaster, 'gloss', radius=0.05),                     # toaster
          box([0.18, 0.012, 0.03], [0.3, 1.137, -0.33], '#22252A', 'satin', radius=0.004),
          box([0.18, 0.012, 0.03], [0.3, 1.137, -0.27], '#22252A', 'satin', radius=0.004),
          box([0.02, 0.03, 0.03], [0.45, 1.06, -0.3], '#22252A', 'satin', radius=0.006),
          box([0.16, 0.12, 0.16], [0.0, 1.02, -0.32], CHARCOAL, 'satin', radius=0.03),                     # blender
          lathe([0.15, 0.26, 0.15], [0.0, 1.21, -0.32], '#DCE6EA', 'gloss', [[0, -0.5], [0.34, -0.5], [0.5, 0.5], [0, 0.5]], segments=16),
          cyl([0.15, 0.03, 0.15], [0.0, 1.355, -0.32], CHARCOAL, 'satin', segments=16),
          cyl([0.26, 0.06, 0.26], [-0.28, 0.99, 0.05], '#E9E6E0', 'gloss'),                                  # fruit bowl
          sph([0.08, 0.07, 0.08], [-0.3, 1.04, 0.04], '#F2C14E', 'satin', segments=8),
          sph([0.08, 0.07, 0.08], [-0.24, 1.04, 0.08], '#D9483B', 'satin', segments=8)]
    return p


for sfx, s in KITCHEN.items():
    addp('counter' + sfx, counter(s))
    add('counterCorner' + sfx, placeholder(corner_counter(s)))
    addp('wallCabinet' + sfx, wall_cabinet(s))
    addp('pantry' + sfx, pantry(s))
    addp('microwave' + sfx, microwave_counter(s))
    addp('coffeeMachine' + sfx, coffee_counter(s))
    addp('breakfastCounter' + sfx, breakfast_counter(s, {'': CREAM, '@modern': '#F4F4F2', '@cozy': RUST, '@minimal': '#F2F1EE'}[sfx]))




def dishwasher(body, panel, fin='satin', handle=CHROME, top=OAK, top_fin='wood'):
    return placeholder([
        box([0.98, 0.84, 0.58], [0, 0.48, -0.13], body, fin, radius=0.012),
        box([0.98, 0.08, 0.56], [0, 0.04, -0.14], '#2F3236', 'satin', radius=0.006),               # plinth
        box([0.9, 0.72, 0.02], [0, 0.48, 0.165], panel, 'metal' if fin == 'metal' else fin, radius=0.012),  # door
        box([0.9, 0.08, 0.02], [0, 0.89, 0.165], '#2F3236', 'satin', radius=0.01),                # control strip
        box([0.5, 0.02, 0.03], [0, 0.8, 0.19], handle, 'chrome' if handle == CHROME else 'metal', radius=0.008),
        sph([0.025, 0.025, 0.012], [0.32, 0.89, 0.177], '#5EC28B', 'gloss', segments=6),
        box([1.0, 0.04, 0.66], [0, 0.94, -0.15], top, top_fin, radius=0.006),
    ])


add('dishwasher', dishwasher('#EDEBE6', STEEL, 'satin', CHROME, '#E8E6E1', 'gloss'))
add('dishwasher@modern', dishwasher('#F4F4F2', '#F4F4F2', 'gloss', BLACKMETAL, '#C9CBCC', 'satin'))
add('dishwasher@cozy', dishwasher(SAGE, SAGE, 'satin', BRASS, HONEY, 'wood'))
add('dishwasher@minimal', dishwasher('#F2F2F0', BIRCH, 'satin', '#F2F2F0', '#F2F2F0', 'satin'))


def farmhouse_sink(cab, top=OAK, top_fin='wood', tap=CHROME):
    return placeholder([
        box([0.98, 0.84, 0.58], [0, 0.48, -0.13], cab, 'satin', radius=0.012),
        box([0.98, 0.08, 0.56], [0, 0.04, -0.14], '#2F3236', 'satin', radius=0.006),
        box([0.4, 0.62, 0.02], [-0.22, 0.42, 0.165], cab, 'satin', radius=0.01),                    # doors
        box([0.4, 0.62, 0.02], [0.22, 0.42, 0.165], cab, 'satin', radius=0.01),
        box([0.012, 0.12, 0.02], [-0.04, 0.62, 0.18], tap, 'chrome', radius=0.004),
        box([0.012, 0.12, 0.02], [0.04, 0.62, 0.18], tap, 'chrome', radius=0.004),
        box([1.0, 0.04, 0.66], [-0.0, 0.94, -0.15], top, top_fin, radius=0.006),
        box([0.66, 0.26, 0.5], [0, 0.83, -0.08], '#F6F5F2', 'gloss', radius=0.03),                 # apron basin
        box([0.58, 0.02, 0.4], [0, 0.955, -0.1], '#D9DEE0', 'gloss', radius=0.02),                 # water/basin floor
        cyl([0.04, 0.3, 0.04], [0, 1.1, -0.38], tap, 'chrome'),                                      # gooseneck tap
        rod([0, 1.25, -0.38], [0, 1.22, -0.2], 0.03, tap, 'chrome'),
        cyl([0.05, 0.03, 0.05], [0.12, 0.975, -0.4], tap, 'chrome'),
        box([0.2, 0.1, 0.06], [0.36, 1.01, -0.36], '#E8E6E1', 'gloss', radius=0.02),                # soap dish
        cyl([0.06, 0.16, 0.06], [-0.36, 1.04, -0.38], TEAL, 'gloss', taper=0.8),
    ])


add('farmhouseSink', farmhouse_sink('#EDEBE6'))
add('farmhouseSink@modern', farmhouse_sink('#F4F4F2', '#C9CBCC', 'satin', BLACKMETAL))
add('farmhouseSink@cozy', farmhouse_sink(SAGE, HONEY, 'wood', BRASS))
add('farmhouseSink@minimal', farmhouse_sink('#F2F2F0', '#F2F2F0', 'satin'))


def range_hood(canopy, cfin, trim, tfin, stepped=False):
    """A chimney hood on the wall: a canopy (stepped, for the cozy plaster one) under a flue, a light under it."""
    y0 = 1.55
    zc = WALL + 0.25
    p = [box([0.9, 0.12, 0.5], [0, y0 + 0.06, zc], canopy, cfin, radius=0.02),
         box([0.9, 0.025, 0.02], [0, y0 + 0.03, zc + 0.255], trim, tfin, radius=0.006),
         box([0.84, 0.012, 0.44], [0, y0 - 0.002, zc], '#2F3236', 'satin', radius=0.004),                   # filter
         box([0.5, 0.008, 0.04], [0, y0 - 0.008, zc + 0.12], BULB, 'gloss', radius=0.003)]
    if stepped:
        p[0] = box([0.9, 0.2, 0.5], [0, y0 + 0.1, zc], canopy, cfin, radius=0.03)                      # deeper plaster lip
        p += [box([0.94, 0.04, 0.54], [0, y0 + 0.22, zc + 0.005], trim, tfin, radius=0.008),             # mantel shelf
              box([0.6, 0.16, 0.34], [0, y0 + 0.32, WALL + 0.17], canopy, cfin, radius=0.04),
              box([0.42, 0.3, 0.28], [0, y0 + 0.55, WALL + 0.14], canopy, cfin, radius=0.02),
              lathe([0.1, 0.12, 0.1], [0.37, y0 + 0.3, zc + 0.12], CREAM, 'gloss', VASE, segments=14),       # on the shelf
              cyl([0.07, 0.1, 0.07], [-0.37, y0 + 0.29, zc + 0.12], '#C98F6B', 'gloss', segments=12)]
    else:
        p += [box([0.3, 0.58, 0.3], [0, y0 + 0.41, WALL + 0.15], canopy, cfin, radius=0.01),
              box([0.16, 0.03, 0.01], [0.28, y0 + 0.08, zc + 0.252], '#2F3236', 'satin', radius=0.004)]     # touch keys
    return p


addp('rangeHood', range_hood(STEEL, 'metal', '#3A3D41', 'satin'))
addp('rangeHood@cozy', range_hood('#E8DFCF', 'satin', HONEY, 'wood', stepped=True))


def island(body, top, top_fin, stool, legs):
    p = [box([1.84, 0.86, 0.56], [0, 0.47, -0.16], body, 'satin', radius=0.014),
         box([1.84, 0.08, 0.54], [0, 0.04, -0.16], '#2F3236', 'satin', radius=0.006),
         box([1.94, 0.05, 0.82], [0, 0.925, -0.04], top, top_fin, radius=0.01)]            # worktop with overhang at the front
    for x in (-0.68, -0.23, 0.23, 0.68):
        p.append(box([0.42, 0.6, 0.02], [x, 0.5, -0.45], body, 'satin', radius=0.01))     # (back doors, seen from the room behind)
    p += [cyl([0.3, 0.02, 0.3], [-0.45, 0.96, -0.1], '#F2F1EE', 'gloss'),                   # fruit bowl
          sph([0.08, 0.07, 0.08], [-0.48, 1.0, -0.1], '#F2C14E', 'satin', segments=8),
          sph([0.08, 0.07, 0.08], [-0.42, 1.0, -0.06], '#D9483B', 'satin', segments=8),
          box([0.4, 0.03, 0.28], [0.4, 0.965, -0.2], OAK, 'wood', radius=0.01)]              # chopping board
    for x in (-0.5, 0.5):                                                                   # two stools at the front
        p += [cyl([0.36, 0.05, 0.36], [x, 0.7, 0.28], stool, 'fabric', segments=16),
              cyl([0.05, 0.66, 0.05], [x, 0.36, 0.28], legs, 'metal'),
              torus([0.3, 0.02, 0.3], [x, 0.26, 0.28], legs, 'metal'),
              cyl([0.32, 0.02, 0.32], [x, 0.01, 0.28], legs, 'metal')]
    return p


add('kitchenIsland', placeholder(island('#EDEBE6', '#E8E6E1', 'gloss', SAGE, CHROME)), (2, 1))
add('kitchenIsland@modern', placeholder(island('#F4F4F2', '#C9CBCC', 'satin', '#9AA4AB', BLACKMETAL)), (2, 1))
add('kitchenIsland@cozy', placeholder(island(SAGE, HONEY, 'wood', RUST, BRASS)), (2, 1))
add('kitchenIsland@minimal', placeholder(island('#F2F2F0', BIRCH, 'wood', LINEN, '#D9D9D6')), (2, 1))


def bar_stool(seat, frame, ffin, kind='round', back=False):
    """A counter stool, seat top at 0.74 (the sitter at 0.72), centred: `round` on four splayed legs and a
    foot ring, `square` on a welded frame, `three` on three legs with a saddle seat."""
    h = 0.74
    p = []
    if kind == 'square':
        p += [box([0.4, 0.035, 0.4], [0, h - 0.055, 0], frame, ffin, radius=0.008),
              cushion([0.38, 0.05, 0.38], [0, h - 0.025, 0], seat, radius=0.02)]
        p += [box([0.03, h - 0.07, 0.03], [sx * 0.175, (h - 0.07) / 2, sz * 0.175], frame, ffin, radius=0.006) for sx in (-1, 1) for sz in (-1, 1)]
        p += [box([0.33, 0.022, 0.022], [0, 0.24, s * 0.175], frame, ffin, radius=0.006) for s in (-1, 1)]
        p += [box([0.022, 0.022, 0.33], [s * 0.175, 0.24, 0], frame, ffin, radius=0.006) for s in (-1, 1)]
    else:
        n = 3 if kind == 'three' else 4
        p.append(cyl([0.4, 0.04, 0.4], [0, h - 0.045 if kind != 'three' else h - 0.02, 0], frame, ffin, segments=24))
        if kind != 'three':
            p.append(cyl([0.37, 0.05, 0.37], [0, h - 0.025, 0], seat, 'fabric', segments=24))
        else:
            p.append(cyl([0.3, 0.012, 0.3], [0, h + 0.004, 0], seat, 'fabric', segments=20))
        for i in range(n):
            a = math.radians(45 + i * 360 / n)
            p.append(rod([math.cos(a) * 0.21, 0, math.sin(a) * 0.21], [math.cos(a) * 0.12, h - 0.06, math.sin(a) * 0.12], 0.034, frame, ffin, segments=8))
        p.append(torus([0.37, 0.024, 0.37], [0, 0.26, 0], frame, ffin, segments=20))
    if back:
        p += [box([0.36, 0.12, 0.03], [0, h + 0.2, -0.16], frame, ffin, radius=0.02, rotate=[-10, 0, 0]),
              rod([-0.12, h - 0.02, -0.14], [-0.14, h + 0.17, -0.165], 0.026, frame, ffin, segments=8),
              rod([0.12, h - 0.02, -0.14], [0.14, h + 0.17, -0.165], 0.026, frame, ffin, segments=8)]
    return p


addp('barStool', bar_stool(SAGE, OAK, 'wood'))
addp('barStool@modern', bar_stool('#9AA4AB', BLACKMETAL, 'metal', 'square'))
addp('barStool@cozy', bar_stool(RUST, HONEY, 'wood', back=True))
addp('barStool@minimal', bar_stool(LINEN, BIRCH, 'wood', 'three'))

# =============================================================================================
# Dining
# =============================================================================================
def bistro(top, frame, seat, top_fin='wood', frame_fin='metal'):
    p = [cyl([0.62, 0.035, 0.62], [0, 0.735, 0.17], top, top_fin, segments=28),
         cyl([0.06, 0.7, 0.06], [0, 0.37, 0.17], frame, frame_fin),
         cyl([0.42, 0.03, 0.42], [0, 0.015, 0.17], frame, frame_fin, segments=20),
         cyl([0.12, 0.12, 0.12], [0.12, 0.81, 0.24], '#F2F1EE', 'gloss', taper=0.8),            # cup
         cyl([0.16, 0.012, 0.16], [0.12, 0.756, 0.24], '#F2F1EE', 'gloss'),
         cyl([0.08, 0.14, 0.08], [-0.15, 0.82, 0.1], '#DCE6EA', 'gloss', taper=0.7),             # little vase
         sph([0.06, 0.05, 0.06], [-0.15, 0.91, 0.1], '#D9483B', 'satin', segments=6)]
    p += chair_facing_front(0, -0.24, frame, seat, frame_fin)
    return p


add('bistroSet', placeholder(bistro(OAK, BLACKMETAL, SAGE)))
add('bistroSet@modern', placeholder(bistro('#F1F0EC', BLACKMETAL, '#9AA4AB', 'gloss')))
add('bistroSet@cozy', placeholder(bistro(HONEY, WALNUT, RUST, 'wood', 'wood')))
add('bistroSet@minimal', placeholder(bistro(BIRCH, '#F2F2F0', LINEN, 'wood', 'satin')))


def dining_long(top, frame, seat, top_fin='wood', frame_fin='wood', glass=False, runner='#E8DCC0'):
    """2x1 table for two seated at the back half facing +Z, with two more chairs drawn across the table."""
    p = []
    if glass:
        p += [box([1.7, 0.02, 0.62], [0, 0.75, 0.16], '#DCE6E8', 'glass', radius=0.01)]
        for x in (-0.7, 0.7):
            p += [box([0.05, 0.74, 0.5], [x, 0.37, 0.16], frame, frame_fin, radius=0.01)]       # sled legs
            p += [box([0.05, 0.05, 0.56], [x, 0.025, 0.16], frame, frame_fin, radius=0.01)]
    else:
        p += [box([1.76, 0.06, 0.66], [0, 0.735, 0.15], top, top_fin, radius=0.012),
              box([1.62, 0.08, 0.5], [0, 0.66, 0.15], frame, frame_fin, radius=0.006)]
        for x in (-0.8, 0.8):
            for z in (-0.12, 0.42):
                p.append(box([0.08, 0.7, 0.08], [x, 0.35, z], frame, frame_fin, radius=0.01))
        p.append(box([0.3, 0.005, 0.62], [0, 0.768, 0.15], runner, 'fabric', radius=0.002))
    p += [cyl([0.14, 0.24, 0.14], [0, 0.88, 0.18], '#E8E6E1', 'gloss', taper=0.6),             # vase with stems
          sph([0.16, 0.12, 0.16], [0, 1.04, 0.18], SAGE, 'foliage', segments=8)]
    for s in (-1, 1):
        x = s * 0.5
        p += [cyl([0.26, 0.015, 0.26], [x, 0.775, 0.06], '#F5F3EE', 'gloss'),
              cyl([0.06, 0.11, 0.06], [x + s * 0.18, 0.83, 0.18], '#DCE6EA', 'gloss', taper=1.1),
              cyl([0.26, 0.015, 0.26], [x, 0.775, 0.36], '#F5F3EE', 'gloss')]
        p += chair_facing_front(x, -0.22, frame if not glass else '#3A3C40', seat, 'wood' if frame_fin == 'wood' else 'metal')
    return p


def far_chairs(p, wood, seat, fin):
    """Two chairs across the table (facing -Z, their backs to the room): drawn, not sat on."""
    out = list(p)
    for x in (-0.5, 0.5):
        out += [box([0.42, 0.04, 0.34], [x, 0.42, 0.29], wood, fin, radius=0.012),
                cushion([0.38, 0.035, 0.3], [x, 0.455, 0.28], seat, radius=0.016),
                box([0.4, 0.42, 0.035], [x, 0.66, 0.44], wood, fin, radius=0.015),
                box([0.035, 0.42, 0.035], [x - 0.18, 0.21, 0.14], wood, fin, radius=0.008),
                box([0.035, 0.42, 0.035], [x + 0.18, 0.21, 0.14], wood, fin, radius=0.008)]
    return out


add('diningGlass', placeholder(dining_long('#DCE6E8', CHROME, '#C9CBCC', frame_fin='chrome', glass=True)), (2, 1))
add('diningGlass@modern', placeholder(dining_long('#DCE6E8', BLACKMETAL, '#9AA4AB', frame_fin='metal', glass=True)), (2, 1))
add('diningGlass@minimal', placeholder(dining_long('#DCE6E8', '#F2F2F0', LINEN, frame_fin='satin', glass=True)), (2, 1))
add('diningOak', placeholder(dining_long(OAK, OAKD, SAGE)), (2, 1))
add('diningOak@modern', placeholder(dining_long('#F1F0EC', OAK, '#9AA4AB', top_fin='satin')), (2, 1))
add('diningOak@cozy', placeholder(dining_long(WALNUT, HONEY, RUST, runner='#D9C7A7')), (2, 1))
add('diningOak@minimal', placeholder(dining_long(BIRCH, '#E3CFA8', LINEN, runner='#F2F0EA')), (2, 1))

def chair_ladder(paint, pad):
    """A painted ladder-back chair on turned legs, a tie-on seat pad; seat top 0.46, centred."""
    p = [box([0.44, 0.04, 0.42], [0, 0.44, 0.02], paint, 'satin', radius=0.012),
         cushion([0.38, 0.025, 0.36], [0, 0.472, 0.03], pad, radius=0.012)]
    for x in (-0.19, 0.19):
        p += [cyl([0.038, 0.42, 0.038], [x, 0.21, 0.19], paint, 'satin', taper=0.75, segments=10),
              rod([x, 0, -0.17], [x * 0.95, 0.96, -0.21], 0.036, paint, 'satin', segments=10),
              sph([0.045, 0.05, 0.045], [x * 0.95, 0.975, -0.21], paint, 'satin', segments=10),
              box([0.022, 0.022, 0.34], [x, 0.2, 0.01], paint, 'satin', radius=0.006)]
    for y in (0.6, 0.73, 0.86):
        p.append(box([0.36, 0.06, 0.02], [0, y, -0.175 - (y - 0.46) * 0.08], paint, 'satin', radius=0.01, rotate=[-4, 0, 0]))
    p += [box([0.36, 0.022, 0.022], [0, 0.16, z], paint, 'satin', radius=0.006) for z in (-0.16, 0.19)]
    return p


def chair_tube(frame, pad):
    """A black tube frame with an upholstered seat and back pad."""
    p = [cushion([0.44, 0.07, 0.42], [0, 0.425, 0.02], pad, radius=0.03),
         cushion([0.42, 0.28, 0.06], [0, 0.72, -0.2], pad, radius=0.03, rotate=[-8, 0, 0])]
    for x in (-0.2, 0.2):
        p += [box([0.024, 0.39, 0.024], [x, 0.195, 0.2], frame, 'metal', radius=0.008),
              rod([x, 0, -0.19], [x, 0.88, -0.25], 0.024, frame, 'metal', segments=8),
              box([0.024, 0.024, 0.42], [x, 0.39, 0.0], frame, 'metal', radius=0.008)]
    p += [box([0.4, 0.024, 0.024], [0, 0.39, 0.2], frame, 'metal', radius=0.008),
          box([0.42, 0.024, 0.024], [0, 0.86, -0.245], frame, 'metal', radius=0.008)]
    return p


def chair_spindle(wood, seat):
    """A farmhouse chair: a spindle back under a curved top rail, a cushion on the seat."""
    p = [box([0.46, 0.045, 0.43], [0, 0.418, 0.02], wood, 'wood', radius=0.014),
         cushion([0.4, 0.04, 0.38], [0, 0.46, 0.03], seat, radius=0.018)]
    for x in (-0.19, 0.19):
        p += [cyl([0.04, 0.4, 0.04], [x, 0.2, 0.19], wood, 'wood', taper=0.7, segments=10),
              rod([x, 0, -0.17], [x * 1.04, 0.92, -0.22], 0.04, wood, 'wood', segments=10)]
    p.append(box([0.46, 0.09, 0.035], [0, 0.89, -0.22], wood, 'wood', radius=0.015, rotate=[-6, 0, 0]))
    for x in (-0.11, -0.055, 0.0, 0.055, 0.11):
        p.append(rod([x, 0.44, -0.17], [x * 1.04, 0.85, -0.215], 0.018, wood, 'wood', segments=8))
    p += [box([0.36, 0.024, 0.024], [0, 0.18, z], wood, 'wood', radius=0.006) for z in (-0.16, 0.19)]
    return p


def chair_round(wood, pad):
    """A Scandinavian chair: a round seat on splayed legs, a rounded back rail on two posts."""
    p = [cyl([0.44, 0.04, 0.42], [0, 0.44, 0.02], wood, 'wood', segments=28),
         cyl([0.38, 0.016, 0.36], [0, 0.466, 0.03], pad, 'fabric', segments=24)]
    for sx, sz in ((-1, 1), (1, 1), (-1, -1), (1, -1)):
        p.append(rod([sx * 0.21, 0, 0.02 + sz * 0.2], [sx * 0.15, 0.43, 0.02 + sz * 0.14], 0.034, wood, 'wood', segments=10))
    p += [rod([s * 0.14, 0.44, -0.14], [s * 0.17, 0.8, -0.2], 0.028, wood, 'wood', segments=8) for s in (-1, 1)]
    p += [part('capsule', [0.1, 0.46, 0.04], [0, 0.79, -0.2], wood, 'wood', rotate=[-8, 0, 90]),
          part('capsule', [0.05, 0.38, 0.03], [0, 0.63, -0.175], wood, 'wood', rotate=[-8, 0, 90]),
          torus([0.34, 0.022, 0.34], [0, 0.19, 0.02], wood, 'wood', segments=20)]
    return p


addp('diningChair', chair_ladder('#E4E0D6', SAGE))
addp('diningChair@modern', chair_tube(BLACKMETAL, '#B9BCBE'))
addp('diningChair@cozy', chair_spindle(HONEY, RUST))
addp('diningChair@minimal', chair_round(BIRCH, LINEN))

# =============================================================================================
# Living: seating, tables, media, storage
# =============================================================================================
ARMS = {'box': 0.17, 'slim': 0.09, 'roll': 0.17, 'low': 0.14}


def sofa(w, fabric, legs, leg_fin='wood', arm='box', n=2, depth=0.84, z=-0.04, back_h=0.82, leg='taper', pillows=(), soft=True):
    """An upholstered sofa or armchair `w` wide (seat cushions to 0.45, sitters at 0.44): `box` arms (classic),
    `slim` track arms (modern), `roll` rolled arms (cozy) or `low` arms (minimal); legs `taper`, `post` or `bun`.
    `pillows`: (x, colour, turn)."""
    z0, z1 = z - depth / 2, z + depth / 2
    lh, bt, aw = 0.1, (0.16 if arm == 'slim' else 0.2), ARMS[arm]
    rad = (lambda s: round(min(s) * 0.42, 3)) if soft else (lambda s: 0.02)
    p = [box([w, 0.32 - lh, depth], [0, (lh + 0.32) / 2, z], fabric, 'fabric', radius=0.04),
         box([w, back_h - 0.08 - 0.3, bt], [0, (back_h - 0.08 + 0.3) / 2, z0 + bt / 2], fabric, 'fabric', radius=0.06 if soft else 0.025)]
    for s in (-1, 1):
        x = s * (w / 2 - aw / 2)
        if arm == 'roll':
            p += [box([aw - 0.02, 0.52 - lh, depth - 0.02], [x, (lh + 0.52) / 2, z + 0.01], fabric, 'fabric', radius=0.05),
                  cyl([0.2, depth - 0.02, 0.2], [s * (w / 2 - 0.1), 0.56, z + 0.01], fabric, 'fabric', rotate=[90, 0, 0], segments=18)]
        else:
            top = {'box': 0.62, 'slim': 0.6, 'low': 0.56}[arm]
            p.append(box([aw, top - lh, depth], [x, (lh + top) / 2, z], fabric, 'fabric', radius=0.07 if arm == 'box' else (0.03 if arm == 'slim' else 0.05)))
    sx0, sx1, sz0 = -w / 2 + aw, w / 2 - aw, z0 + bt
    cw = (sx1 - sx0) / n
    for i in range(n):
        cx = sx0 + cw * (i + 0.5)
        size = [cw - 0.015, 0.13, z1 - sz0 - 0.02]
        p.append(cushion(size, [cx, 0.385, (sz0 + z1) / 2 + 0.005], fabric, radius=rad(size)))
        size = [cw - 0.025, back_h - 0.44, 0.16]
        p.append(cushion(size, [cx, (back_h + 0.44) / 2 + 0.01, sz0 + 0.07], fabric, rotate=[-9, 0, 0], radius=rad(size)))
    for px, c, ry in pillows:
        p.append(cushion([0.4, 0.36, 0.12], [px, 0.66, sz0 + 0.14], c, rotate=[-14, ry, 0]))
    for lx in (-w / 2 + 0.07, w / 2 - 0.07):
        for lz in (z0 + 0.07, z1 - 0.07):
            if leg == 'bun':
                p.append(sph([0.09, 0.11, 0.09], [lx, 0.055, lz], legs, leg_fin, segments=10))
            elif leg == 'post':
                p.append(box([0.035, lh, 0.035], [lx, lh / 2, lz], legs, leg_fin, radius=0.006))
            else:
                p.append(cyl([0.045, lh, 0.045], [lx, lh / 2, lz], legs, leg_fin, taper=0.75, segments=10))
    return p


addp('loveseat', sofa(1.84, SAGE, OAK, pillows=[(-0.55, CREAM, 10)]), (2, 1))
addp('loveseat@modern', sofa(1.84, '#B9BCBE', BLACKMETAL, 'metal', 'slim', back_h=0.76, leg='post', pillows=[(-0.55, COGNAC, 8)], soft=False), (2, 1))
addp('loveseat@cozy', sofa(1.84, RUST, WALNUT, arm='roll', back_h=0.86, leg='bun', pillows=[(-0.55, MUSTARD, 12), (0.55, CREAM, -12)]), (2, 1))
addp('loveseat@minimal', sofa(1.84, LINEN, BIRCH, arm='low', back_h=0.78), (2, 1))


def sofa3(fabric, base, cushion_col, base_fin='wood', legs=None):
    """A 3-wide sofa with three seat cushions; two sitters at x = +-0.75 (seat ~0.44 m)."""
    p = [box([2.82, 0.2, 0.86], [0, 0.22, -0.03], fabric, 'fabric', radius=0.05),            # base
         box([2.82, 0.5, 0.22], [0, 0.62, -0.35], fabric, 'fabric', radius=0.08),             # back
         box([0.2, 0.32, 0.86], [-1.31, 0.48, -0.03], fabric, 'fabric', radius=0.08),         # arms
         box([0.2, 0.32, 0.86], [1.31, 0.48, -0.03], fabric, 'fabric', radius=0.08)]
    for x in (-0.8, 0.0, 0.8):
        p += [cushion([0.78, 0.12, 0.62], [x, 0.38, 0.06], fabric),
              cushion([0.74, 0.4, 0.16], [x, 0.62, -0.2], fabric, rotate=[-10, 0, 0])]
    p += [cushion([0.4, 0.34, 0.12], [-0.98, 0.66, -0.12], cushion_col, rotate=[-12, 10, 0]),
          cushion([0.4, 0.34, 0.12], [0.98, 0.66, -0.12], cushion_col, rotate=[-12, -10, 0]),
          box([0.7, 0.04, 0.5], [0.95, 0.5, -0.05], cushion_col, 'fabric', radius=0.02, rotate=[0, 0, 4])]  # throw
    for x in (-1.32, 1.32):
        for z in (-0.38, 0.32):
            p.append(cyl([0.05, 0.12, 0.05], [x, 0.06, z], legs or base, base_fin, taper=0.8))
    return p


add('sofaThree', placeholder(sofa3('#C9C6BE', OAKD, SAGE)), (3, 1))
add('sofaThree@modern', placeholder(sofa3('#9AA4AB', BLACKMETAL, COGNAC, 'metal')), (3, 1))
add('sofaThree@cozy', placeholder(sofa3('#B98F6A', WALNUT, MUSTARD)), (3, 1))
add('sofaThree@minimal', placeholder(sofa3(LINEN, BIRCH, '#F2F0EA')), (3, 1))

def design_sofa(fabric, frame, ffin):
    """A mid-century sofa: a slim tufted body on tall tapered legs, a frame rail along the front (seat 0.42)."""
    z = -0.04
    p = [box([1.86, 0.1, 0.8], [0, 0.27, z], fabric, 'fabric', radius=0.03),                              # seat deck
         box([1.86, 0.4, 0.14], [0, 0.52, z - 0.33], fabric, 'fabric', radius=0.05, rotate=[-8, 0, 0]),  # back
         box([1.8, 0.03, 0.03], [0, 0.22, z + 0.38], frame, ffin, radius=0.008)]                            # front rail
    for s in (-1, 1):
        p += [box([0.1, 0.32, 0.8], [s * 0.88, 0.38, z], fabric, 'fabric', radius=0.04),                  # slim arms
              box([0.11, 0.025, 0.7], [s * 0.88, 0.555, z + 0.02], frame, ffin, radius=0.008)]              # arm caps
        for lz in (z - 0.3, z + 0.3):
            p.append(rod([s * 0.8, 0.0, lz + (0.03 if lz > z else -0.03)], [s * 0.78, 0.23, lz], 0.04, frame, ffin, segments=10))
    for x in (-0.42, 0.42):
        p.append(cushion([0.82, 0.11, 0.66], [x, 0.37, z + 0.06], fabric, radius=0.03))
        for bx in (-0.2, 0.0, 0.2):                                                                        # buttons in the back
            p.append(sph([0.03, 0.03, 0.02], [x + bx, 0.58, z - 0.255], fabric, 'fabric', segments=6))
    return p


addp('sofaDesign', design_sofa(TEAL, BRASS, 'metal'), (2, 1))
addp('sofaDesign@modern', design_sofa('#D2D4D5', BLACKMETAL, 'metal'), (2, 1))
addp('sofaDesign@cozy', design_sofa(COGNAC, WALNUT, 'wood'), (2, 1))
addp('sofaDesign@minimal', design_sofa('#ECE7DD', BIRCH, 'wood'), (2, 1))

addp('armchairLounge', sofa(0.82, SAGE, OAK, n=1))
addp('armchairLounge@modern', sofa(0.82, '#B9BCBE', BLACKMETAL, 'metal', 'slim', n=1, back_h=0.76, leg='post', soft=False))
addp('armchairLounge@cozy', sofa(0.82, RUST, WALNUT, arm='roll', n=1, back_h=0.86, leg='bun', pillows=[(0, MUSTARD, 0)]))
addp('armchairLounge@minimal', sofa(0.82, LINEN, BIRCH, arm='low', n=1, back_h=0.78))

# Leather armchair: black leather cushions in an open teak frame.
LEATHER, TEAKF = '#2B2826', '#9A6B45'
p = [cushion([0.58, 0.12, 0.6], [0, 0.38, 0.03], LEATHER, 'satin', radius=0.04),                          # seat (0.44)
     cushion([0.58, 0.5, 0.12], [0, 0.69, -0.3], LEATHER, 'satin', radius=0.04, rotate=[-14, 0, 0]),
     box([0.64, 0.05, 0.04], [0, 0.3, 0.34], TEAKF, 'wood', radius=0.01),
     box([0.6, 0.04, 0.04], [0, 0.3, -0.33], TEAKF, 'wood', radius=0.01)]
for s in (-1, 1):
    x = s * 0.33
    p += [rod([x, 0, 0.37], [x, 0.62, 0.32], 0.045, TEAKF, 'wood', segments=10),                             # front leg into arm
          rod([x, 0, -0.38], [x, 0.3, -0.33], 0.045, TEAKF, 'wood', segments=10),
          rod([x, 0.3, -0.33], [x, 0.94, -0.44], 0.04, TEAKF, 'wood', segments=10),                          # back post
          box([0.07, 0.035, 0.64], [x, 0.635, -0.01], TEAKF, 'wood', radius=0.012),                         # arm rest
          box([0.04, 0.05, 0.68], [x, 0.3, 0.0], TEAKF, 'wood', radius=0.01)]
p.append(box([0.66, 0.05, 0.04], [0, 0.92, -0.435], TEAKF, 'wood', radius=0.012))
addp('armchairLeather', p)

# Design lounge chair: walnut shells with leather cushions on a five-star swivel base.
PLY, HIDE, ALU = '#6B4630', '#5A3A2A', '#C9CBCC'
p = [cyl([0.08, 0.2, 0.08], [0, 0.17, 0.0], ALU, 'metal', segments=12),
     box([0.66, 0.04, 0.6], [0, 0.32, 0.02], PLY, 'wood', radius=0.015, rotate=[-5, 0, 0]),                 # seat shell
     cushion([0.62, 0.1, 0.58], [0, 0.39, 0.03], HIDE, 'satin', radius=0.04, rotate=[-5, 0, 0]),           # seat (0.45)
     box([0.66, 0.42, 0.04], [0, 0.6, -0.36], PLY, 'wood', radius=0.015, rotate=[-18, 0, 0]),              # back shell
     cushion([0.6, 0.38, 0.11], [0, 0.62, -0.3], HIDE, 'satin', radius=0.04, rotate=[-18, 0, 0]),
     box([0.6, 0.2, 0.04], [0, 0.86, -0.43], PLY, 'wood', radius=0.015, rotate=[-12, 0, 0]),               # head shell
     cushion([0.56, 0.17, 0.1], [0, 0.86, -0.38], HIDE, 'satin', radius=0.04, rotate=[-12, 0, 0])]
for s in (-1, 1):
    p += [box([0.08, 0.05, 0.4], [s * 0.34, 0.57, -0.02], HIDE, 'satin', radius=0.02),                      # arm pads
          rod([s * 0.31, 0.34, 0.08], [s * 0.34, 0.54, 0.02], 0.025, ALU, 'metal', segments=8),
          rod([s * 0.31, 0.4, -0.26], [s * 0.34, 0.54, -0.18], 0.025, ALU, 'metal', segments=8)]
for i in range(5):
    a = math.radians(i * 72 + 90)
    tip = [math.cos(a) * 0.36, 0.03, math.sin(a) * 0.36]
    p += [rod([0, 0.08, 0], tip, 0.04, ALU, 'metal', shape='box', t2=0.03, radius=0.01),
          cyl([0.05, 0.02, 0.05], [tip[0], 0.01, tip[2]], '#2B2C2E', 'satin', segments=8)]
addp('armchairDesign', p)

# Ottoman (a pouf to sit on): buttoned dark leather on short legs.
p = [cushion([0.72, 0.34, 0.5], [0, 0.26, 0], '#3A2E2A', 'satin', radius=0.06),
     cushion([0.7, 0.05, 0.48], [0, 0.435, 0], '#3A2E2A', 'satin', radius=0.02)]
p += [sph([0.03, 0.02, 0.03], [x, 0.462, z], '#2A201D', 'satin', segments=6) for x in (-0.2, 0.0, 0.2) for z in (-0.1, 0.1)]
p += [cyl([0.045, 0.09, 0.045], [x, 0.045, z], WALNUT, 'wood', taper=0.7, segments=10) for x in (-0.3, 0.3) for z in (-0.19, 0.19)]
addp('pouf', p)


def coffee_table(top, tfin, legc, lfin, style):
    """2x1, its top at 0.42 (1.3 x 0.74): `classic` tapered legs and a slatted shelf, `sled` black frames,
    `chunky` square legs and a shelf of baskets, `splay` thin splayed legs."""
    t = {'chunky': 0.06, 'splay': 0.03}.get(style, 0.04)
    p = [box([1.3, t, 0.74], [0, 0.42 - t / 2, 0], top, tfin, radius=0.012 if style != 'sled' else 0.006)]
    h = 0.42 - t
    if style == 'classic':
        p += [box([1.12, 0.06, 0.02], [0, h - 0.03, s * 0.28], legc, lfin, radius=0.005) for s in (-1, 1)]
        p += [cyl([0.05, h, 0.05], [x, h / 2, z], legc, lfin, taper=0.65, segments=10) for x in (-0.58, 0.58) for z in (-0.3, 0.3)]
        p += [box([1.1, 0.02, 0.09], [0, 0.12, z], legc, lfin, radius=0.004) for z in (-0.2, -0.07, 0.07, 0.2)]
    elif style == 'sled':
        for x in (-0.56, 0.56):
            p += [box([0.03, h, 0.03], [x, h / 2, z], legc, lfin, radius=0.006) for z in (-0.3, 0.3)]
            p += [box([0.03, 0.03, 0.63], [x, y, 0], legc, lfin, radius=0.006) for y in (0.015, h - 0.015)]
    elif style == 'chunky':
        p += [box([0.08, h, 0.08], [x, h / 2, z], legc, lfin, radius=0.012) for x in (-0.58, 0.58) for z in (-0.3, 0.3)]
        p += [box([1.12, 0.03, 0.6], [0, 0.11, 0], legc, lfin, radius=0.006),
              box([0.34, 0.16, 0.32], [-0.3, 0.205, 0.0], '#C9A979', 'fabric', radius=0.03),
              box([0.34, 0.16, 0.32], [0.12, 0.205, 0.0], '#C9A979', 'fabric', radius=0.03)]
    else:
        for sx in (-1, 1):
            for sz in (-1, 1):
                p.append(rod([sx * 0.6, 0, sz * 0.32], [sx * 0.54, h, sz * 0.27], 0.035, legc, lfin, segments=10))
    if style == 'splay':
        p += [lathe([0.26, 0.08, 0.26], [-0.32, 0.46, 0.0], '#E9E6E0', 'gloss', BOWL, segments=20)]
    else:
        p += [box([0.3, 0.03, 0.22], [0.3, 0.435, 0.05], TEAL, 'matte', radius=0.004, rotate=[0, 10, 0]),   # magazines
              box([0.3, 0.02, 0.22], [0.31, 0.46, 0.04], CREAM, 'matte', radius=0.004, rotate=[0, -6, 0]),
              cyl([0.18, 0.06, 0.18], [-0.32, 0.45, 0.0], '#E9E6E0', 'gloss')]
    return p


addp('coffeeTable', coffee_table(OAK, 'wood', OAK, 'wood', 'classic'), (2, 1))
addp('coffeeTable@modern', coffee_table('#F1F0EC', 'gloss', BLACKMETAL, 'metal', 'sled'), (2, 1))
addp('coffeeTable@cozy', coffee_table(WALNUT, 'wood', WALNUT, 'wood', 'chunky'), (2, 1))
addp('coffeeTable@minimal', coffee_table(BIRCH, 'wood', BIRCH, 'wood', 'splay'), (2, 1))


def glass_table(frame, ffin, deco):
    p = [box([1.3, 0.02, 0.74], [0, 0.41, 0], '#DCE6E8', 'glass', radius=0.01),
         box([1.2, 0.012, 0.64], [0, 0.12, 0], '#DCE6E8', 'glass', radius=0.008)]
    p += [cyl([0.03, 0.4, 0.03], [x, 0.2, z], frame, ffin, segments=10) for x in (-0.6, 0.6) for z in (-0.32, 0.32)]
    for y in (0.105, 0.39):
        p += [box([1.2, 0.022, 0.022], [0, y, s * 0.32], frame, ffin, radius=0.006) for s in (-1, 1)]
        p += [box([0.022, 0.022, 0.64], [s * 0.6, y, 0], frame, ffin, radius=0.006) for s in (-1, 1)]
    return p + deco


addp('coffeeTableGlass', glass_table(CHROME, 'chrome', [
    sph([0.2, 0.14, 0.2], [-0.3, 0.49, 0.0], FOLIAGE, 'foliage', segments=8),
    cyl([0.14, 0.08, 0.14], [-0.3, 0.46, 0.0], '#F2F2F0', 'gloss'),
    cyl([0.1, 0.18, 0.1], [0.36, 0.22, 0.05], '#E8E6E1', 'gloss', taper=0.6)]), (2, 1))
addp('coffeeTableGlass@modern', glass_table(BLACKMETAL, 'metal', [
    lathe([0.3, 0.09, 0.3], [-0.3, 0.465, 0.0], '#3A3D41', 'gloss', BOWL, segments=20),
    box([0.26, 0.04, 0.2], [0.3, 0.44, 0.05], '#C9CBCC', 'matte', radius=0.004, rotate=[0, 8, 0]),
    box([0.24, 0.03, 0.18], [0.3, 0.475, 0.05], COGNAC, 'matte', radius=0.004, rotate=[0, -5, 0])]), (2, 1))

# Round marble table on a brass tulip foot.
p = [cyl([0.9, 0.04, 0.9], [0, 0.4, 0], '#EEECE8', 'gloss', segments=36),
     lathe([0.56, 0.38, 0.56], [0, 0.19, 0], BRASS, 'metal', [[0, -0.5], [0.5, -0.5], [0.5, -0.43], [0.2, -0.25], [0.11, 0.2], [0.18, 0.5], [0, 0.5]], segments=28),
     box([0.5, 0.003, 0.02], [-0.05, 0.4215, 0.06], '#C9CBCC', 'gloss', radius=0.001, rotate=[0, 32, 0]),        # veins
     box([0.3, 0.003, 0.012], [0.16, 0.4215, -0.12], '#C9CBCC', 'gloss', radius=0.001, rotate=[0, -20, 0]),
     cyl([0.1, 0.16, 0.1], [0.12, 0.5, 0.05], '#E8E6E1', 'gloss', taper=0.6),
     box([0.24, 0.03, 0.18], [-0.12, 0.435, -0.08], NAVY, 'matte', radius=0.004, rotate=[0, 14, 0])]
addp('coffeeTableMarble', p)


def side_table(wood, fin, style, knob=BRASS, legs=None):
    """Top at 0.55: `classic` (tapered legs, a shelf), `cabinet` (a drawer box on thin legs), `cube` (drawer and cubby)."""
    if style == 'classic':
        p = [box([0.55, 0.03, 0.45], [0, 0.535, 0], wood, fin, radius=0.01),
             box([0.46, 0.08, 0.36], [0, 0.48, 0], wood, fin, radius=0.006),                                   # drawer box
             box([0.3, 0.06, 0.012], [0, 0.48, 0.185], wood, fin, radius=0.004),
             sph([0.03, 0.03, 0.025], [0, 0.48, 0.195], knob, 'metal', segments=8),
             box([0.46, 0.02, 0.36], [0, 0.16, 0], wood, fin, radius=0.004)]
        p += [cyl([0.04, 0.52, 0.04], [x, 0.26, z], wood, fin, taper=0.7, segments=10) for x in (-0.23, 0.23) for z in (-0.18, 0.18)]
        return p
    p = [box([0.6, 0.3 if style == 'cabinet' else 0.42, 0.45], [0, 0.4 if style == 'cabinet' else 0.34, 0], wood, fin, radius=0.012)]
    if style == 'cabinet':
        p += [box([0.03, 0.25, 0.03], [x, 0.125, z], legs, 'metal', radius=0.006) for x in (-0.26, 0.26) for z in (-0.18, 0.18)]
        p += [box([0.56, 0.005, 0.01], [0, 0.4, 0.226], '#C9CBCC', 'gloss', radius=0.002),
              box([0.2, 0.016, 0.02], [0, 0.47, 0.235], knob, 'metal', radius=0.006)]
    else:
        p += [box([0.52, 0.12, 0.015], [0, 0.47, 0.228], wood, fin, radius=0.004),
              box([0.16, 0.012, 0.006], [0, 0.5, 0.237], '#8C8780', 'satin', radius=0.003),
              box([0.52, 0.22, 0.01], [0, 0.26, 0.222], '#7A6A55', 'matte', radius=0.004)]                       # open cubby
        p += [cyl([0.035, 0.13, 0.035], [x, 0.065, z], wood, fin, segments=10) for x in (-0.26, 0.26) for z in (-0.18, 0.18)]
    return p


addp('sideTable', side_table(OAK, 'wood', 'classic'))
addp('sideTable@modern', side_table('#F1F0EC', 'gloss', 'cabinet', BLACKMETAL, BLACKMETAL))
addp('sideTable@minimal', side_table(BIRCH, 'wood', 'cube'))


def tv_wall(frame='#2A2C30', w=1.24, h=0.72):
    y = 0.92
    return placeholder([
        box([w, h, 0.05], [0, y + h / 2, -0.43 + 0.04], frame, 'satin', radius=0.012),
        box([w - 0.03, h - 0.03, 0.004], [0, y + h / 2, -0.43 + 0.066], SCREEN, 'gloss', radius=0.004),
        box([w * 0.7, h * 0.55, 0.002], [0, y + h / 2 + 0.03, -0.43 + 0.0685], '#2E5470', 'gloss', radius=0.004),  # picture glow
        box([0.3, 0.2, 0.03], [0, y + h / 2, -0.43 + 0.012], '#3A3D41', 'metal', radius=0.006),                  # bracket
        box([0.9, 0.06, 0.08], [0, y - 0.12, -0.43 + 0.05], '#3A3D41', 'satin', radius=0.02),                    # soundbar
    ])


add('tvWall', tv_wall(), (2, 1))
# Retro TV: a honey-wood console on splayed legs, a bulging CRT screen, a speaker grille and rabbit ears.
z = -0.24
p = [box([0.92, 0.62, 0.44], [0, 0.5, z], HONEY, 'wood', radius=0.03),
     box([0.94, 0.03, 0.46], [0, 0.825, z], HONEY, 'wood', radius=0.01),
     box([0.56, 0.46, 0.02], [-0.14, 0.52, z + 0.225], '#2A2C30', 'gloss', radius=0.008),                 # bezel
     sph([0.5, 0.38, 0.07], [-0.14, 0.52, z + 0.23], '#22303E', 'gloss', segments=20),                    # screen
     sph([0.3, 0.2, 0.05], [-0.18, 0.56, z + 0.245], '#3B5E7A', 'gloss', segments=16),                    # glow
     box([0.26, 0.3, 0.012], [0.3, 0.46, z + 0.222], '#3A3530', 'fabric', radius=0.01)]                    # grille
p += [cyl([0.06, 0.03, 0.06], [x, 0.7, z + 0.232], STEEL, 'metal', rotate=[90, 0, 0], segments=12) for x in (0.24, 0.36)]
p += [rod([x * 0.42, 0, z + s * 0.17], [x * 0.38, 0.2, z + s * 0.15], 0.04, HONEY, 'wood', segments=8) for x in (-1, 1) for s in (-1, 1)]
p += [sph([0.1, 0.05, 0.1], [0.24, 0.86, z - 0.06], '#2A2C30', 'gloss', segments=10),                     # rabbit ears
      rod([0.24, 0.87, z - 0.06], [0.1, 1.12, z - 0.12], 0.008, STEEL, 'metal', segments=6),
      rod([0.24, 0.87, z - 0.06], [0.4, 1.1, z - 0.1], 0.008, STEEL, 'metal', segments=6)]
addp('tvRetro', p)


def home_cinema(unit, top, fin='wood'):
    p = [box([1.86, 0.42, 0.44], [0, 0.25, -0.22], unit, fin, radius=0.012),
         box([1.88, 0.03, 0.45], [0, 0.475, -0.22], top, fin, radius=0.008)]
    for x in (-0.62, 0.0, 0.62):
        p.append(box([0.58, 0.3, 0.012], [x, 0.25, 0.006], top, fin, radius=0.006))
    for x in (-0.86, 0.86):
        p += [box([0.04, 0.06, 0.4], [x, 0.03, -0.22], BLACKMETAL, 'metal', radius=0.006)]
    p += [box([1.7, 0.96, 0.04], [0, 1.08, -0.4], '#2A2C30', 'satin', radius=0.012),                  # big screen
          box([1.66, 0.92, 0.004], [0, 1.08, -0.378], SCREEN, 'gloss', radius=0.004),
          box([1.2, 0.6, 0.002], [0, 1.1, -0.375], '#3B5E7A', 'gloss', radius=0.004),
          box([1.0, 0.07, 0.09], [0, 0.53, -0.12], '#3A3D41', 'satin', radius=0.025)]                  # soundbar
    for x in (-0.9, 0.9):                                                                               # tower speakers
        p += [box([0.16, 0.9, 0.2], [x, 0.94, -0.3], '#3A3D41', 'satin', radius=0.02),
              cyl([0.11, 0.02, 0.11], [x, 1.2, -0.195], '#22252A', 'satin', rotate=[90, 0, 0]),
              cyl([0.11, 0.02, 0.11], [x, 0.9, -0.195], '#22252A', 'satin', rotate=[90, 0, 0])]
    p += [box([0.36, 0.05, 0.26], [0.45, 0.525, -0.25], '#22252A', 'satin', radius=0.01),             # player
          sph([0.03, 0.03, 0.01], [0.6, 0.53, -0.115], '#5EC28B', 'gloss', segments=6)]
    return p


add('homeCinema', placeholder(home_cinema(OAKD, OAK)), (2, 1))
add('homeCinema@modern', placeholder(home_cinema('#F1F0EC', '#F1F0EC', 'gloss')), (2, 1))
add('homeCinema@cozy', placeholder(home_cinema(WALNUT, HONEY)), (2, 1))
add('homeCinema@minimal', placeholder(home_cinema(BIRCH, '#F2F2F0', 'wood')), (2, 1))

def stereo(wood, dark, metal, mfin):
    """A hi-fi cabinet: an amplifier and records in the open front, a turntable between two speakers on top."""
    z = -0.25
    p = [box([0.92, 0.46, 0.42], [0, 0.37, z], wood, 'wood', radius=0.012),
         box([0.86, 0.38, 0.012], [0, 0.37, z + 0.206], '#2A2620', 'satin', radius=0.004),                   # open front
         box([0.4, 0.1, 0.03], [-0.2, 0.47, z + 0.2], dark, 'satin', radius=0.01),                           # amplifier
         box([0.1, 0.03, 0.006], [-0.28, 0.48, z + 0.217], '#7FC8E8', 'gloss', radius=0.003)]
    p += [cyl([0.035, 0.02, 0.035], [x, 0.47, z + 0.22], metal, mfin, rotate=[90, 0, 0], segments=10) for x in (-0.14, -0.06)]
    p += [box([0.012, 0.28, 0.26], [0.08 + i * 0.022, 0.32, z + 0.06], c, 'satin', radius=0.003, rotate=[0, 0, -4 if i > 6 else 0])
          for i, c in enumerate((RUST, '#2A2C30', MUSTARD, TEAL, CREAM, '#2A2C30', NAVY, CLAY))]                  # records
    p += [cyl([0.04, 0.14, 0.04], [x, 0.07, z + s * 0.16], wood, 'wood', taper=0.7, segments=8) for x in (-0.4, 0.4) for s in (-1, 1)]
    p += [box([0.4, 0.07, 0.32], [0, 0.635, z], wood, 'wood', radius=0.01),                                    # turntable
          cyl([0.3, 0.02, 0.3], [-0.03, 0.68, z], dark, 'satin', segments=24),
          cyl([0.29, 0.006, 0.29], [-0.03, 0.692, z], '#141414', 'satin', segments=24),
          cyl([0.09, 0.008, 0.09], [-0.03, 0.694, z], RUST, 'satin', segments=12),
          rod([0.15, 0.69, z - 0.1], [0.06, 0.7, z + 0.04], 0.012, metal, mfin, segments=6)]
    for x in (-0.37, 0.37):                                                                                  # speakers
        p += [box([0.16, 0.24, 0.2], [x, 0.72, z - 0.02], wood, 'wood', radius=0.012),
              cyl([0.11, 0.012, 0.11], [x, 0.69, z + 0.08], dark, 'satin', rotate=[90, 0, 0], segments=16),
              cyl([0.04, 0.012, 0.04], [x, 0.79, z + 0.08], dark, 'satin', rotate=[90, 0, 0], segments=10)]
    return p


addp('stereo', stereo(OAK, CHARCOAL, STEEL, 'metal'))
addp('stereo@cozy', stereo(WALNUT, '#3A3530', BRASS, 'metal'))
addp('stereo@minimal', stereo(BIRCH, '#E6E4DF', '#F2F2F0', 'satin'))

# Boombox on a crate of records.
y0 = 0.42
p = [box([0.6, 0.42, 0.44], [0, 0.21, 0], HONEY, 'wood', radius=0.01),                                    # crate
     box([0.52, 0.34, 0.38], [0, 0.215, 0.0], '#9E6E43', 'wood', radius=0.01),
     box([0.24, 0.03, 0.24], [0.14, 0.435, 0.12], '#2A2C30', 'satin', radius=0.004, rotate=[0, 15, 0]),     # record sleeve
     box([0.52, 0.26, 0.14], [0, y0 + 0.13, -0.08], CHARCOAL, 'satin', radius=0.03),
     box([0.5, 0.02, 0.12], [0, y0 + 0.265, -0.08], '#C9CBCC', 'metal', radius=0.008),
     box([0.12, 0.07, 0.006], [0, y0 + 0.13, -0.008], '#1D2228', 'gloss', radius=0.006),                     # cassette
     box([0.1, 0.02, 0.006], [0, y0 + 0.21, -0.008], '#7FC8E8', 'gloss', radius=0.003)]
for x in (-0.17, 0.17):
    p += [cyl([0.15, 0.02, 0.15], [x, y0 + 0.12, -0.008], '#22252A', 'satin', rotate=[90, 0, 0], segments=20),
          torus([0.16, 0.014, 0.16], [x, y0 + 0.12, -0.006], '#C9CBCC', 'metal', rotate=[90, 0, 0], segments=20),
          cyl([0.04, 0.02, 0.04], [x, y0 + 0.12, -0.004], '#3A3D41', 'satin', rotate=[90, 0, 0], segments=10),
          rod([x * 1.1, y0 + 0.27, -0.08], [x * 1.1, y0 + 0.33, -0.08], 0.018, '#C9CBCC', 'metal', segments=8)]
p += [rod([-0.19, y0 + 0.33, -0.08], [0.19, y0 + 0.33, -0.08], 0.022, '#C9CBCC', 'metal', segments=8),          # handle
      rod([0.22, y0 + 0.27, -0.13], [0.3, y0 + 0.52, -0.16], 0.006, '#C9CBCC', 'metal', segments=6)]             # aerial
p += [box([0.03, 0.012, 0.02], [-0.06 + i * 0.04, y0 + 0.28, -0.06], '#2A2C30', 'satin', radius=0.003) for i in range(4)]
addp('boombox', p)

for sfx, (wood, fin, back, knob) in {'': (OAK, 'wood', None, OAKD), '@modern': ('#F1F0EC', 'satin', '#3A3D41', BLACKMETAL),
                                     '@cozy': (WALNUT, 'wood', '#6B4630', HONEY), '@minimal': (BIRCH, 'wood', None, '#C9B48E')}.items():
    p = C2.bookcase(0.92, 0.9, 0.42, wood, fin, 2, seed=3 if sfx != '@minimal' else 4, back=back)
    p += [cyl([0.12, 0.12, 0.12], [0.3, 0.96, -0.24], '#E8E6E1', 'gloss', taper=0.8),
          sph([0.2, 0.18, 0.2], [0.3, 1.08, -0.24], FOLIAGE, 'foliage', segments=8)]
    addp('bookcaseLow' + sfx, p)
    # Wide bookcase: cupboards under open shelves of books, a plant on top.
    d, z = 0.42, -0.25
    p = case(1.86, 2.0, d, 0, 0, z, wood, fin, back=back)
    p += [box([0.022, 1.95, d - 0.01], [0, 1.0, z], wood, fin, radius=0.003),
          box([1.82, 0.03, d], [0, 0.8, z], wood, fin, radius=0.004)]
    for x in (-0.69, -0.23, 0.23, 0.69):
        p += [box([0.445, 0.72, 0.02], [x, 0.42, z + d / 2 + 0.01], wood, fin, radius=0.008),
              sph([0.03, 0.03, 0.025], [x + (0.17 if x in (-0.69, 0.23) else -0.17), 0.66, z + d / 2 + 0.03], knob, fin, segments=8)]
    for i, (x0, y) in enumerate(((-0.9, 1.2), (0.03, 1.2), (-0.9, 1.6), (0.03, 1.6))):
        p.append(box([0.87, 0.022, d - 0.02], [x0 + 0.435, y, z], wood, fin, radius=0.003))
    p += books_row(-0.88, -0.05, 0.815, z + 0.03, depth=0.26, seed=1, hmax=0.3)
    p += books_row(0.05, 0.6, 0.815, z + 0.03, depth=0.26, seed=4, hmax=0.3)
    p += books_row(-0.88, -0.3, 1.211, z + 0.03, depth=0.26, seed=6, hmax=0.3)
    p += books_row(0.05, 0.88, 1.211, z + 0.03, depth=0.26, seed=2, hmax=0.3)
    p += books_row(-0.88, -0.05, 1.611, z + 0.03, depth=0.26, seed=8, hmax=0.3)
    p += [cyl([0.12, 0.2, 0.12], [-0.6, 1.31, z], CREAM, 'gloss', taper=0.7),
          cyl([0.12, 0.12, 0.12], [0.7, 0.88, z], '#E8E6E1', 'gloss', taper=0.8),
          sph([0.18, 0.16, 0.18], [0.7, 1.0, z], FOLIAGE, 'foliage', segments=8),
          cyl([0.14, 0.12, 0.14], [0.45, 1.67, z], CLAY if fin == 'wood' else '#E8E6E1', 'gloss', taper=0.8),
          part('blob', [0.3, 0.22, 0.24], [0.45, 1.82, z], FOLIAGE, 'foliage', noise=0.3)]
    addp('bookcaseWide' + sfx, p, (2, 1))

# Industrial shelving: oak boards on a black steel frame (the top at 1.83), books, boxes and a plant.
z = -0.18
p = []
for x in (-0.91, 0.0, 0.91):
    p += [box([0.035, 1.83, 0.035], [x, 0.915, z + s * 0.26], BLACKMETAL, 'metal', radius=0.006) for s in (-1, 1)]
for y in (0.12, 0.56, 1.0, 1.44, 1.815):
    p += [box([1.86, 0.03, 0.56], [0, y, z], OAK, 'wood', radius=0.006),
          box([1.86, 0.03, 0.025], [0, y - 0.03, z + 0.27], BLACKMETAL, 'metal', radius=0.006)]
p += [rod([-0.9, 0.15, z - 0.27], [0.0, 1.4, z - 0.27], 0.015, BLACKMETAL, 'metal', segments=6),            # cross braces
      rod([0.0, 0.15, z - 0.27], [-0.9, 1.4, z - 0.27], 0.015, BLACKMETAL, 'metal', segments=6)]
p += books_row(-0.86, -0.06, 0.575, z, depth=0.24, seed=7, hmax=0.3) + books_row(0.08, 0.6, 1.015, z, depth=0.24, seed=2, hmax=0.3)
p += books_row(-0.5, 0.0, 1.455, z, depth=0.24, seed=9, hmax=0.3)
p += [box([0.38, 0.26, 0.4], [-0.6, 0.265, z], '#C9B79C', 'matte', radius=0.02), box([0.38, 0.26, 0.4], [-0.2, 0.265, z], '#8E979E', 'matte', radius=0.02),
      box([0.36, 0.24, 0.4], [0.5, 0.255, z], '#C9B79C', 'matte', radius=0.02),
      flare([0.14, 0.14, 0.14], [0.75, 1.085, z], CLAY, 'matte', 1.18),
      part('blob', [0.26, 0.22, 0.26], [0.75, 1.24, z], FOLIAGE, 'foliage', noise=0.3),
      flare([0.16, 0.12, 0.16], [-0.6, 1.075, z], '#E8E6E1', 'matte', 1.15),
      box([0.2, 0.26, 0.03], [0.4, 1.6, z - 0.1], '#F2F0EA', 'matte', radius=0.006, rotate=[-8, 0, 0])]
addp('shelvingIndustrial', p, (2, 1))


def fireplace(stone, mantel, fin='satin', mantel_fin='wood'):
    p = [box([1.84, 1.25, 0.42], [0, 0.625, -0.27], stone, fin, radius=0.015),          # chimney breast
         box([1.84, 1.55, 0.3], [0, 2.025, -0.33], stone, fin, radius=0.015),           # flue up to the ceiling
         box([1.96, 0.07, 0.52], [0, 1.27, -0.22], mantel, mantel_fin, radius=0.01),    # mantel shelf
         box([1.96, 0.1, 0.62], [0, 0.05, -0.17], '#8E8A84', 'matte', radius=0.01),     # hearth
         box([0.92, 0.7, 0.2], [0, 0.46, -0.16], '#2A2624', 'matte', radius=0.02),      # firebox
         box([0.8, 0.6, 0.02], [0, 0.44, -0.05], '#1B1716', 'matte', radius=0.01)]
    for x, r in ((-0.18, 12), (0.12, -18), (0.0, 80)):                                   # logs
        p.append(cyl([0.1, 0.5, 0.1], [x, 0.2, -0.12], '#6B4A33', 'bark', rotate=[0, r, 90]))
    p += [part('blob', [0.42, 0.28, 0.16], [0, 0.34, -0.1], '#FF9A3C', 'gloss', noise=0.25),       # flames
          part('blob', [0.26, 0.36, 0.1], [0.04, 0.42, -0.09], '#FFD27A', 'gloss', noise=0.3)]
    p += [cyl([0.1, 0.24, 0.1], [-0.65, 1.42, -0.25], '#E8E6E1', 'gloss', taper=0.7),            # mantel things
          box([0.3, 0.38, 0.03], [0.55, 1.5, -0.38], '#F2F0EA', 'satin', radius=0.006),
          box([0.24, 0.3, 0.005], [0.55, 1.5, -0.364], TEAL, 'matte', radius=0.004),
          cyl([0.06, 0.12, 0.06], [-0.4, 1.36, -0.25], CREAM, 'satin'),
          cyl([0.06, 0.18, 0.06], [-0.3, 1.39, -0.25], CREAM, 'satin')]
    return p


add('fireplace', placeholder(fireplace('#E7E3DC', OAK)), (2, 1))
add('fireplace@modern', placeholder(fireplace('#B9BCBE', OAK, 'matte')), (2, 1))
add('fireplace@cozy', placeholder(fireplace('#B9A48C', WALNUT, 'matte')), (2, 1))
add('fireplace@minimal', placeholder(fireplace('#F4F3EF', BIRCH)), (2, 1))

# =============================================================================================
# Bedroom
# =============================================================================================
def single_bed(frame, ffin, sheet, duvet, style, accent=CREAM):
    """A single bed along Z, head at -Z, mattress top 0.5: `slat` head and foot boards (classic), a `panel`
    headboard and a folded throw (modern), `spindle` boards with ball finials (cozy), a `low` headboard (minimal)."""
    p = [box([0.96, 0.2, 1.86], [0, 0.24, 0.02], frame, ffin, radius=0.02),
         box([0.88, 0.18, 1.82], [0, 0.41, 0.03], sheet, 'fabric', radius=0.05),
         box([0.92, 0.07, 1.16], [0, 0.505, 0.36], duvet, 'fabric', radius=0.03)]
    p += [box([0.03, 0.2, 1.14], [s * 0.455, 0.43, 0.36], duvet, 'fabric', radius=0.012) for s in (-1, 1)]
    p.append(cushion([0.62, 0.13, 0.34], [0, 0.57, -0.72], '#FFFFFF', rotate=[-10, 0, 0]))
    foot = style in ('slat', 'spindle')
    if not foot:
        p.append(box([0.94, 0.24, 0.04], [0, 0.42, 0.935], duvet, 'fabric', radius=0.02))
    if style == 'spindle':
        p += [cyl([0.06, 0.9, 0.06], [x, 0.45, -0.94], frame, ffin, segments=12) for x in (-0.45, 0.45)]
        p += [cyl([0.06, 0.62, 0.06], [x, 0.31, 0.94], frame, ffin, segments=12) for x in (-0.45, 0.45)]
        p += [sph([0.085, 0.085, 0.085], [x, 0.93, -0.94], frame, ffin, segments=10) for x in (-0.45, 0.45)]
        p += [sph([0.085, 0.085, 0.085], [x, 0.65, 0.94], frame, ffin, segments=10) for x in (-0.45, 0.45)]
        p += [box([0.86, 0.07, 0.05], [0, 0.84, -0.94], frame, ffin, radius=0.02), box([0.86, 0.06, 0.05], [0, 0.56, 0.94], frame, ffin, radius=0.02)]
        p += [cyl([0.026, 0.32, 0.026], [x, 0.66, -0.94], frame, ffin, segments=8) for x in (-0.3, -0.15, 0.0, 0.15, 0.3)]
        p += [cyl([0.026, 0.18, 0.026], [x, 0.44, 0.94], frame, ffin, segments=8) for x in (-0.3, -0.15, 0.0, 0.15, 0.3)]
        p += [cushion([0.4, 0.3, 0.12], [0.1, 0.66, -0.52], accent, rotate=[-14, -8, 0])]
    elif style == 'slat':
        p += [box([0.06, 0.95, 0.06], [x, 0.475, -0.94], frame, ffin, radius=0.01) for x in (-0.45, 0.45)]
        p += [box([0.06, 0.62, 0.06], [x, 0.31, 0.94], frame, ffin, radius=0.01) for x in (-0.45, 0.45)]
        p += [box([0.9, 0.08, 0.05], [0, 0.9, -0.94], frame, ffin, radius=0.012), box([0.9, 0.22, 0.03], [0, 0.48, 0.94], frame, ffin, radius=0.01)]
        p += [box([0.1, 0.36, 0.022], [x, 0.66, -0.94], frame, ffin, radius=0.006) for x in (-0.28, -0.14, 0.0, 0.14, 0.28)]
    elif style == 'panel':
        p += [box([0.98, 0.8, 0.06], [0, 0.4, -0.95], frame, ffin, radius=0.02),
              box([0.98, 0.04, 0.4], [0, 0.56, 0.6], accent, 'fabric', radius=0.015)]                    # throw
    else:
        p += [box([0.98, 0.7, 0.05], [0, 0.35, -0.95], frame, ffin, radius=0.02)]
    p += [box([0.06, 0.14, 0.06], [x, 0.07, z], frame, ffin, radius=0.01) for x in (-0.44, 0.44) for z in (-0.86, 0.9)]
    return p


addp('bedSingle', single_bed(OAK, 'wood', '#F7F7F5', SAGE, 'slat'), (1, 2))
addp('bedSingle@modern', single_bed('#F1F0EC', 'satin', '#FFFFFF', '#9AA7B0', 'panel', accent='#5F6E7A'), (1, 2))
addp('bedSingle@cozy', single_bed(HONEY, 'wood', CREAM, RUST, 'spindle', accent=MUSTARD), (1, 2))
addp('bedSingle@minimal', single_bed(BIRCH, 'wood', '#FFFFFF', LINEN, 'low'), (1, 2))


def bunk_bed(frame, ffin, sheet, cover):
    """Two beds one above the other (mattress tops 0.44 and 1.38), a guard rail round the top, a ladder on +X."""
    p = [box([0.06, 1.9, 0.06], [x, 0.95, z], frame, ffin, radius=0.01) for x in (-0.45, 0.45) for z in (-0.93, 0.93)]
    for yb, top in ((0.22, 0.44), (1.16, 1.38)):
        p += [box([0.04, 0.12, 1.8], [x, yb, 0], frame, ffin, radius=0.008) for x in (-0.45, 0.45)]
        p += [box([0.86, 0.12, 0.04], [0, yb, z], frame, ffin, radius=0.008) for z in (-0.93, 0.93)]
        p += [box([0.86, 0.16, 1.8], [0, top - 0.08, 0], sheet, 'fabric', radius=0.04),
              box([0.88, 0.05, 1.1], [0, top + 0.01, 0.3], cover, 'fabric', radius=0.02),
              cushion([0.56, 0.11, 0.3], [0, top + 0.05, -0.7], '#FFFFFF', rotate=[-10, 0, 0])]
    p += [box([0.86, 0.3, 0.025], [0, 0.46, z], frame, ffin, radius=0.008) for z in (-0.93, 0.93)]           # end panels
    p += [box([0.86, 0.36, 0.025], [0, 1.42, z], frame, ffin, radius=0.008) for z in (-0.93, 0.93)]
    p += [box([0.04, 0.05, 1.8], [-0.45, 1.6, 0], frame, ffin, radius=0.008),                                 # guard rails
          box([0.04, 0.05, 1.26], [0.45, 1.6, -0.27], frame, ffin, radius=0.008),
          box([0.03, 0.4, 0.03], [0.45, 1.4, 0.34], frame, ffin, radius=0.006)]
    p += [box([0.04, 1.62, 0.045], [0.47, 0.81, z], frame, ffin, radius=0.008) for z in (0.42, 0.86)]       # ladder
    p += [cyl([0.03, 0.44, 0.03], [0.47, y, 0.64], frame, ffin, rotate=[90, 0, 0], segments=8) for y in (0.32, 0.62, 0.92, 1.22)]
    return p


addp('bedBunk', bunk_bed(OAK, 'wood', '#F7F7F5', TEAL), (1, 2))
addp('bedBunk@cozy', bunk_bed(HONEY, 'wood', CREAM, RUST), (1, 2))
addp('bedBunk@minimal', bunk_bed('#F2F2F0', 'satin', '#FFFFFF', LINEN), (1, 2))


def futon(base, mattress, cover):
    """Low double platform: mattress top ~0.4 m, two sleepers at x = +-0.5."""
    p = [box([1.96, 0.18, 1.96], [0, 0.11, 0.0], base, 'wood', radius=0.01),
         box([1.86, 0.2, 1.86], [0, 0.3, 0.02], mattress, 'fabric', radius=0.06),
         box([1.88, 0.06, 1.2], [0, 0.42, 0.34], cover, 'fabric', radius=0.03),
         box([1.96, 0.5, 0.06], [0, 0.42, -0.95], base, 'wood', radius=0.01)]                # low headboard
    for s in (-1, 1):
        p += [cushion([0.66, 0.14, 0.36], [s * 0.45, 0.46, -0.7], '#FFFFFF'),
              box([0.02, 0.04, 1.96], [s * 0.97, 0.03, 0], base, 'wood', radius=0.005)]
    return p


add('bedFuton', placeholder(futon(BIRCH, '#F4F2EE', LINEN)), (2, 2))
add('bedFuton@modern', placeholder(futon('#3A3D41', '#F4F2EE', '#9AA4AB')), (2, 2))
add('bedFuton@cozy', placeholder(futon(HONEY, CREAM, RUST)), (2, 2))


def king_bed(head, frame, sheet, throw, ffin='fabric'):
    p = [box([1.96, 0.3, 1.94], [0, 0.25, 0.0], frame, ffin, radius=0.05),
         box([1.98, 1.3, 0.14], [0, 0.75, -0.9], head, 'fabric', radius=0.06),               # tall headboard
         box([1.86, 0.22, 1.84], [0, 0.48, 0.04], '#F8F8F6', 'fabric', radius=0.07),          # mattress (top 0.59)
         box([1.92, 0.08, 1.3], [0, 0.6, 0.32], sheet, 'fabric', radius=0.04),               # duvet
         box([1.94, 0.04, 0.5], [0, 0.66, 0.66], throw, 'fabric', radius=0.015),
         box([0.04, 0.32, 0.5], [0.97, 0.52, 0.66], throw, 'fabric', radius=0.015),
         box([0.04, 0.32, 0.5], [-0.97, 0.52, 0.66], throw, 'fabric', radius=0.015)]
    for x in (-0.66, -0.22, 0.22, 0.66):
        p.append(box([0.42, 1.1, 0.04], [x, 0.78, -0.82], head, 'fabric', radius=0.04))     # channel tufts
    for s in (-1, 1):
        p += [cushion([0.66, 0.16, 0.4], [s * 0.44, 0.66, -0.66], '#FFFFFF'),
              cushion([0.5, 0.3, 0.12], [s * 0.38, 0.75, -0.46], throw, rotate=[-15, 0, 0])]
    return p


add('bedKing', placeholder(king_bed('#CFC9BD', '#D9D4CA', '#F1F0EC', SAGE)), (2, 2))
add('bedKing@modern', placeholder(king_bed('#9AA4AB', '#B9BCBE', '#F6F6F4', COGNAC)), (2, 2))
add('bedKing@cozy', placeholder(king_bed(RUST, '#B98F6A', CREAM, MUSTARD)), (2, 2))
add('bedKing@minimal', placeholder(king_bed(LINEN, '#ECE7DD', '#FFFFFF', '#E3CFA8')), (2, 2))

def night(wood, fin, knob, kfin, style):
    """A bedside table against the wall, its top at 0.56 (z -0.46..-0.02): `classic` drawer over a shelf on
    tapered legs, `legs` a drawer box on black legs, `two` two drawers on a plinth, `open` a drawer over a cubby."""
    z, d, w = -0.24, 0.44, 0.52
    zf = z + d / 2
    y0 = {'classic': 0.14, 'legs': 0.26, 'two': 0.06, 'open': 0.08}[style]
    p = [box([w, 0.56 - y0 - 0.025, d], [0, (0.56 - 0.025 + y0) / 2, z], wood, fin, radius=0.012),
         box([w + 0.02, 0.025, d + 0.01], [0, 0.5475, z], wood, fin, radius=0.008)]
    if style == 'classic':
        p += [cyl([0.04, y0, 0.04], [x, y0 / 2, zz], wood, fin, taper=0.7, segments=10) for x in (-0.22, 0.22) for zz in (z - 0.18, z + 0.18)]
        p += [box([w - 0.04, 0.14, 0.02], [0, 0.45, zf + 0.008], wood, fin, radius=0.006),
              sph([0.034, 0.034, 0.028], [0, 0.45, zf + 0.025], knob, kfin, segments=10),
              box([w - 0.06, 0.2, 0.01], [0, 0.26, zf - 0.002], '#3B302A', fin, radius=0.004)]
    elif style == 'legs':
        p += [box([0.025, y0, 0.025], [x, y0 / 2, zz], knob, kfin, radius=0.006) for x in (-0.22, 0.22) for zz in (z - 0.18, z + 0.18)]
        p += [box([w - 0.02, 0.005, 0.01], [0, 0.42, zf + 0.002], '#C9CBCC', 'gloss', radius=0.002),
              box([0.18, 0.016, 0.02], [0, 0.49, zf + 0.012], knob, kfin, radius=0.006)]
    elif style == 'two':
        p += [box([w - 0.04, y0, d - 0.04], [0, y0 / 2, z], '#3B302A', fin, radius=0.004)]
        for y in (0.39, 0.2):
            p += [box([w - 0.04, 0.17, 0.02], [0, y, zf + 0.008], wood, fin, radius=0.006),
                  sph([0.034, 0.034, 0.028], [0, y, zf + 0.025], knob, kfin, segments=10)]
    else:
        p += [cyl([0.05, y0, 0.05], [x, y0 / 2, zz], wood, fin, segments=12) for x in (-0.2, 0.2) for zz in (z - 0.16, z + 0.16)]
        p += [box([w - 0.04, 0.12, 0.02], [0, 0.47, zf + 0.008], wood, fin, radius=0.006),
              box([0.16, 0.012, 0.006], [0, 0.5, zf + 0.018], '#8C8780', 'satin', radius=0.003),
              box([w - 0.06, 0.28, 0.01], [0, 0.25, zf - 0.002], '#7A6A55', fin, radius=0.004)]
    return p


def bedside_lamp(base, bfin, shade, stem, sfin, square=False, x=0.0, z=-0.26, y=0.56):
    """A lamp standing at y: a ceramic or wooden base, a stem and a fabric shade round its bulb (~y + 0.4)."""
    p = [box([0.13, 0.18, 0.13], [x, y + 0.09, z], base, bfin, radius=0.012) if square else lathe([0.16, 0.18, 0.16], [x, y + 0.09, z], base, bfin, VASE, segments=18),
         cyl([0.016, 0.12, 0.016], [x, y + 0.24, z], stem, sfin, segments=8),
         sph([0.07, 0.08, 0.07], [x, y + 0.38, z], BULB, 'gloss', segments=10)]
    if square:
        p.append(cyl([0.3, 0.22, 0.3], [x, y + 0.42, z], shade, 'fabric', taper=0.8, segments=4, rotate=[0, 45, 0]))
    else:
        p += [cyl([0.3, 0.22, 0.3], [x, y + 0.41, z], shade, 'fabric', taper=0.85, segments=24),
              cyl([0.27, 0.004, 0.27], [x, y + 0.31, z], BULB, 'gloss', segments=20)]
    return p


NIGHT = {'': (OAK, 'wood', BRASS, 'metal', 'classic'), '@modern': ('#F1F0EC', 'gloss', BLACKMETAL, 'metal', 'legs'),
         '@cozy': (WALNUT, 'wood', BRASS, 'metal', 'two'), '@minimal': (BIRCH, 'wood', '#F2F2F0', 'satin', 'open')}
for sfx, (wood, fin, knob, kfin, style) in NIGHT.items():
    addp('nightstand' + sfx, night(wood, fin, knob, kfin, style) + [
        box([0.16, 0.03, 0.22], [0.1, 0.575, -0.16], TEAL, 'matte', radius=0.004, rotate=[0, -12, 0]),
        cyl([0.08, 0.1, 0.08], [-0.12, 0.61, -0.12], '#E8E6E1', 'gloss')])
    lamp = {'': ('#E8E2D6', 'gloss', '#F4EBD8', BRASS, 'metal', False), '@modern': ('#F4F4F2', 'gloss', '#FAF8F3', BLACKMETAL, 'metal', True),
            '@cozy': (CLAY, 'gloss', '#EFD9B4', BRASS, 'metal', False), '@minimal': (BIRCH, 'wood', '#FAF8F3', '#F2F2F0', 'satin', True)}[sfx]
    addp('nightstandLamp' + sfx, night(wood, fin, knob, kfin, style) + bedside_lamp(*lamp[:5], square=lamp[5]))


def dresser(wood, fin, knob):
    """A low chest of drawers (three rows of two), things on top."""
    p = [box([1.4, 0.8, 0.5], [0, 0.46, -0.18], wood, fin, radius=0.012),
         box([1.44, 0.03, 0.52], [0, 0.875, -0.17], wood, fin, radius=0.008)]
    for x in (-0.68, 0.68):
        for z in (-0.38, 0.02):
            p.append(box([0.04, 0.08, 0.04], [x, 0.04, z], wood, fin, radius=0.008))
    for y in (0.24, 0.48, 0.72):
        for x in (-0.35, 0.35):
            p += [box([0.66, 0.2, 0.02], [x, y, 0.075], wood, fin, radius=0.008),
                  box([0.12, 0.018, 0.02], [x, y + 0.03, 0.092], knob, 'metal', radius=0.006)]
    p += [box([0.36, 0.46, 0.03], [0.32, 1.12, -0.3], '#F2F0EA', 'satin', radius=0.006, rotate=[-6, 0, 0]),   # framed photo
          box([0.3, 0.38, 0.005], [0.32, 1.12, -0.283], TEAL, 'matte', radius=0.004, rotate=[-6, 0, 0]),
          cyl([0.12, 0.22, 0.12], [-0.45, 1.0, -0.2], CREAM, 'gloss', taper=0.7),
          box([0.22, 0.06, 0.16], [-0.12, 0.92, -0.16], '#E6DFD3', 'satin', radius=0.02)]
    return p


for sfx, (wood, fin, knob) in {'': (OAK, 'wood', BRASS), '@modern': ('#F1F0EC', 'gloss', BLACKMETAL), '@cozy': (WALNUT, 'wood', BRASS),
                               '@minimal': (BIRCH, 'wood', '#D9D9D6')}.items():
    add('dresser' + sfx, placeholder(dresser(wood, fin, knob)), (2, 1))


def wardrobe(body, door, fin, handle):
    p = [box([1.84, 2.06, 0.6], [0, 1.08, -0.14], body, fin, radius=0.012),
         box([1.84, 0.06, 0.58], [0, 0.03, -0.14], '#2F3236', 'satin', radius=0.006)]
    for x in (-0.46, 0.46):
        p += [box([0.9, 1.96, 0.02], [x, 1.08, 0.165], door, fin, radius=0.01),
              box([0.02, 0.4, 0.03], [x - 0.04 if x > 0 else x + 0.04, 1.1, 0.185], handle, 'metal', radius=0.008)]
    p += [box([0.4, 0.3, 0.32], [-0.5, 2.27, -0.18], '#C9B79C', 'matte', radius=0.02),       # boxes on top
          box([0.32, 0.22, 0.3], [-0.08, 2.23, -0.18], '#E6DFD3', 'matte', radius=0.02)]
    return p


add('wardrobe', placeholder(wardrobe('#EDEBE6', OAK, 'satin', BRASS)), (2, 1))
add('wardrobe@modern', placeholder(wardrobe('#F4F4F2', '#F4F4F2', 'gloss', BLACKMETAL)), (2, 1))
add('wardrobe@cozy', placeholder(wardrobe(WALNUT, HONEY, 'wood', BRASS)), (2, 1))
add('wardrobe@minimal', placeholder(wardrobe(BIRCH, '#F2F2F0', 'wood', '#D9D9D6')), (2, 1))


def dressing_table(wood, fin, seat, frame):
    p = [box([0.86, 0.04, 0.4], [0, 0.74, 0.22], wood, fin, radius=0.01),                     # top
         box([0.3, 0.14, 0.38], [-0.26, 0.65, 0.22], wood, fin, radius=0.01),                  # drawers
         box([0.3, 0.14, 0.38], [0.26, 0.65, 0.22], wood, fin, radius=0.01)]
    for x in (-0.4, 0.4):
        for z in (0.06, 0.38):
            p.append(cyl([0.035, 0.72, 0.035], [x, 0.36, z], frame, 'metal' if frame == BRASS else 'wood', taper=0.8))
    p += [box([0.62, 0.76, 0.03], [0, 1.16, 0.41], frame, 'metal' if frame == BRASS else 'wood', radius=0.12),
          box([0.54, 0.66, 0.006], [0, 1.15, 0.38], '#CFDCE2', 'gloss', radius=0.1),            # mirror
          cyl([0.05, 0.1, 0.05], [0.3, 0.81, 0.3], BLUSH, 'gloss'), cyl([0.04, 0.12, 0.04], [0.35, 0.82, 0.32], '#E8E6E1', 'gloss'),
          box([0.16, 0.04, 0.1], [-0.28, 0.78, 0.18], '#2A2C30', 'satin', radius=0.01),
          cyl([0.42, 0.36, 0.42], [0, 0.22, -0.22], seat, 'fabric', segments=20),                 # pouf stool
          cyl([0.44, 0.06, 0.44], [0, 0.42, -0.22], seat, 'fabric', segments=20)]
    return p


add('dressingTable', placeholder(dressing_table(OAK, 'wood', BLUSH, BRASS)))
add('dressingTable@modern', placeholder(dressing_table('#F1F0EC', 'gloss', '#9AA4AB', BLACKMETAL)))
add('dressingTable@cozy', placeholder(dressing_table(WALNUT, 'wood', RUST, BRASS)))
add('dressingTable@minimal', placeholder(dressing_table(BIRCH, 'wood', LINEN, BIRCH)))

def mirror(frame, ffin, shape, w, h, y0, t=0.035, rim=0.04):
    """A mirror on the wall from y0, `w` x `h`: a `rect` frame with a little ledge, a `pill` or an `arch` (ornate)."""
    zf, zm = WALL + t / 2, WALL + t + 0.003

    def outline(ww, hh, yy, z, depth, color, fin):
        if shape == 'rect':
            return [box([ww, hh, depth], [0, yy + hh / 2, z], color, fin, radius=min(0.015, depth * 0.45))]
        if shape == 'pill':
            return [box([ww, hh - ww, depth], [0, yy + hh / 2, z], color, fin, radius=0.002),
                    cyl([ww, depth, ww], [0, yy + ww / 2, z], color, fin, rotate=[90, 0, 0], segments=32),
                    cyl([ww, depth, ww], [0, yy + hh - ww / 2, z], color, fin, rotate=[90, 0, 0], segments=32)]
        return [box([ww, hh - ww / 2, depth], [0, yy + (hh - ww / 2) / 2, z], color, fin, radius=0.002),            # arch
                cyl([ww, depth, ww], [0, yy + hh - ww / 2, z], color, fin, rotate=[90, 0, 0], segments=32)]
    p = outline(w, h, y0, zf, t, frame, ffin) + outline(w - 2 * rim, h - 2 * rim, y0 + rim, zm, 0.006, MIRROR, 'gloss')
    if shape == 'rect':
        p += [box([w - 0.1, 0.02, 0.09], [0, y0 - 0.03, WALL + 0.045], frame, ffin, radius=0.006),                 # ledge
              cyl([0.06, 0.09, 0.06], [-0.16, y0 + 0.025, WALL + 0.05], '#9DC6CF', 'gloss', segments=12),
              cyl([0.012, 0.12, 0.012], [-0.155, y0 + 0.08, WALL + 0.05], '#F2F1EE', 'gloss', segments=6, rotate=[0, 0, 8])]
    elif shape == 'arch':
        p += [sph([0.09, 0.09, 0.05], [0, y0 + h + 0.03, zf + 0.01], frame, ffin, segments=12),                  # crest and beads
              sph([0.06, 0.06, 0.04], [-w / 2 + 0.02, y0 + h - w / 2, zf + 0.01], frame, ffin, segments=10),
              sph([0.06, 0.06, 0.04], [w / 2 - 0.02, y0 + h - w / 2, zf + 0.01], frame, ffin, segments=10),
              box([w * 0.5, 0.05, 0.045], [0, y0 + 0.01, zf + 0.005], frame, ffin, radius=0.02)]
    return p


addp('mirrorWall', mirror(BLACKMETAL, 'metal', 'rect', 0.62, 0.9, 1.0, rim=0.03))
addp('mirrorWall@cozy', mirror(BRASS, 'metal', 'arch', 0.53, 0.82, 1.05, rim=0.05))
addp('mirrorWall@minimal', mirror(BIRCH, 'wood', 'pill', 0.6, 0.9, 1.0, rim=0.035))

# =============================================================================================
# Bathroom
# =============================================================================================
# Compact toilet: a slim cistern on the wall, a round bowl with the seat up-to-sit (0.43), chrome flush.
p = [box([0.4, 0.4, 0.17], [0, 0.58, -0.355], PORCELAIN, 'gloss', radius=0.03),                         # cistern
     box([0.42, 0.03, 0.18], [0, 0.795, -0.355], PORCELAIN, 'gloss', radius=0.012),
     cyl([0.07, 0.012, 0.07], [0, 0.814, -0.355], CHROME, 'chrome', segments=14),                       # flush button
     lathe([0.36, 0.4, 0.46], [0, 0.2, -0.06], PORCELAIN, 'gloss',
           [[0, -0.5], [0.27, -0.5], [0.3, -0.2], [0.42, 0.2], [0.5, 0.45], [0.5, 0.5], [0, 0.5]], segments=28),  # bowl
     box([0.14, 0.2, 0.16], [0, 0.4, -0.25], PORCELAIN, 'gloss', radius=0.04),                          # neck to the cistern
     torus([0.38, 0.03, 0.46], [0, 0.415, -0.06], PORCELAIN, 'gloss', segments=28),                     # seat ring
     cyl([0.26, 0.01, 0.32], [0, 0.39, -0.05], '#CFE6EA', 'gloss', segments=24),                        # water
     box([0.36, 0.38, 0.03], [0, 0.6, -0.255], PORCELAIN, 'gloss', radius=0.014, rotate=[-6, 0, 0]),   # lid, up
     cyl([0.11, 0.1, 0.11], [0.3, 0.62, -0.4], '#FFFFFF', 'gloss', rotate=[0, 0, 90], segments=14),     # paper on its holder
     cyl([0.02, 0.14, 0.02], [0.3, 0.62, -0.42], CHROME, 'chrome', rotate=[0, 0, 90], segments=8)]
addp('toiletCompact', p)


def smart_toilet(panel, fin):
    return placeholder([
        box([0.9, 1.2, 0.16], [0, 0.6, -0.36], panel, fin, radius=0.01),                     # wall panel (cistern)
        box([0.18, 0.12, 0.02], [0, 1.0, -0.275], '#2A2C30', 'gloss', radius=0.01),          # flush plate
        sph([0.03, 0.03, 0.01], [0.04, 1.0, -0.263], '#5EC28B', 'gloss', segments=6),
        part('lathe', [0.42, 0.34, 0.56], [0, 0.26, -0.03], '#F6F6F4', 'gloss',
             profile=[[0, -0.5], [0.36, -0.45], [0.5, 0.2], [0.48, 0.5], [0, 0.5]], segments=24),   # wall-hung bowl
        cyl([0.44, 0.04, 0.58], [0, 0.44, -0.03], '#F6F6F4', 'gloss', segments=24),          # seat (0.44)
        box([0.12, 0.1, 0.1], [0, 0.34, -0.27], '#F6F6F4', 'gloss', radius=0.04),
        box([0.14, 0.06, 0.06], [0.36, 0.7, -0.25], '#3A3D41', 'satin', radius=0.02),        # remote
        cyl([0.12, 0.11, 0.12], [-0.34, 0.8, -0.28], '#FFFFFF', 'fabric', rotate=[0, 0, 90]),  # paper roll
    ])


add('toiletSmart', smart_toilet('#E4E3DF', 'satin'))
add('toiletSmart@modern', smart_toilet('#3A3D41', 'matte'))
add('toiletSmart@cozy', smart_toilet(HONEY, 'wood'))

# Built-in bathtub (2x1): rim at 0.6, a hollow basin with water (the bather sits at 0.25), chrome taps.
z = -0.03
p = [box([1.82, 0.44, 0.86], [0, 0.22, z], PORCELAIN, 'gloss', radius=0.03),                               # body under the basin
     box([1.82, 0.16, 0.13], [0, 0.52, z - 0.365], PORCELAIN, 'gloss', radius=0.03),                     # rim walls
     box([1.82, 0.16, 0.13], [0, 0.52, z + 0.365], PORCELAIN, 'gloss', radius=0.03),
     box([0.14, 0.16, 0.62], [-0.84, 0.52, z], PORCELAIN, 'gloss', radius=0.03),
     box([0.14, 0.16, 0.62], [0.84, 0.52, z], PORCELAIN, 'gloss', radius=0.03),
     box([1.56, 0.012, 0.62], [0, 0.445, z], '#E4ECEE', 'gloss', radius=0.04),                             # basin floor
     box([1.56, 0.02, 0.62], [0, 0.5, z], '#BFE0E6', 'glass', radius=0.04),                                # water
     box([1.7, 0.42, 0.012], [0, 0.25, z + 0.432], '#ECECE8', 'gloss', radius=0.006),                     # front panel
     cyl([0.035, 0.14, 0.035], [-0.84, 0.67, z - 0.36], CHROME, 'chrome', segments=10),                  # mixer and spout
     rod([-0.84, 0.73, z - 0.36], [-0.72, 0.71, z - 0.36], 0.03, CHROME, 'chrome', segments=10),
     cyl([0.05, 0.03, 0.05], [-0.84, 0.615, z - 0.27], CHROME, 'chrome', segments=12),
     cyl([0.05, 0.03, 0.05], [-0.84, 0.615, z - 0.41], CHROME, 'chrome', segments=12),
     box([0.4, 0.05, 0.12], [0.62, 0.625, z + 0.365], '#FFFFFF', 'fabric', radius=0.02),                 # towel on the rim
     sph([0.07, 0.06, 0.08], [0.2, 0.525, z + 0.05], '#F2C14E', 'gloss', segments=10),                   # duck
     sph([0.045, 0.045, 0.045], [0.2, 0.57, z + 0.08], '#F2C14E', 'gloss', segments=10),
     cone([0.025, 0.02, 0.03], [0.2, 0.565, z + 0.11], '#E9853B', 'gloss', rotate=[90, 0, 0], segments=8)]
addp('bathtub', p, (2, 1))


def clawfoot(body, outer, feet):
    p = [part('lathe', [1.7, 0.56, 0.78], [0, 0.42, -0.02], outer, 'gloss',
              profile=[[0, -0.5], [0.42, -0.5], [0.48, -0.2], [0.5, 0.5], [0.46, 0.5], [0.44, -0.3], [0, -0.3]], segments=28),
         part('lathe', [1.6, 0.06, 0.7], [0, 0.6, -0.02], '#F6F6F4', 'gloss', profile=[[0.46, -0.5], [0.5, 0.5], [0.44, 0.5], [0.44, -0.5]], segments=28),
         box([1.36, 0.02, 0.5], [0, 0.5, -0.02], '#BFE0E6', 'glass', radius=0.12)]
    for x in (-0.6, 0.6):
        for z in (-0.24, 0.2):
            p.append(sph([0.1, 0.16, 0.1], [x, 0.08, z], feet, 'metal', segments=8))
    p += [cyl([0.035, 0.5, 0.035], [-0.78, 0.85, -0.02], feet, 'metal'),                     # tap column
          rod([-0.78, 1.08, -0.02], [-0.66, 1.06, -0.02], 0.03, feet, 'metal'),
          box([0.36, 0.06, 0.24], [0.62, 0.65, -0.05], '#FFFFFF', 'fabric', radius=0.02),     # folded towel
          cyl([0.06, 0.1, 0.06], [0.5, 0.67, -0.02], CREAM, 'satin')]                         # candle
    return p


add('bathtubClawfoot', placeholder(clawfoot('#F6F6F4', '#F6F6F4', BRASS)), (2, 1))
add('bathtubClawfoot@modern', placeholder(clawfoot('#F6F6F4', '#3A3D41', BLACKMETAL)), (2, 1))
add('bathtubClawfoot@cozy', placeholder(clawfoot('#F6F6F4', SAGE, BRASS)), (2, 1))


def shower_stall(curtain, tray='#F6F6F4', metal=CHROME):
    return placeholder([
        box([0.9, 0.08, 0.9], [0, 0.04, 0], tray, 'gloss', radius=0.03),
        box([0.86, 2.0, 0.04], [0, 1.08, -0.42], '#E4E8EA', 'satin', radius=0.006),            # tiled back
        box([0.04, 2.0, 0.86], [-0.42, 1.08, 0], '#E4E8EA', 'satin', radius=0.006),
        cyl([0.02, 0.9, 0.02], [0.42, 1.95, 0], metal, 'chrome', rotate=[90, 0, 0]),          # curtain rail
        cyl([0.02, 0.9, 0.02], [0, 1.95, 0.42], metal, 'chrome', rotate=[0, 0, 90]),
        box([0.04, 1.65, 0.58], [0.42, 1.08, -0.12], curtain, 'fabric', radius=0.02),          # curtain, half drawn
        box([0.3, 1.65, 0.04], [0.26, 1.08, 0.42], curtain, 'fabric', radius=0.02),
        cyl([0.03, 0.7, 0.03], [-0.2, 1.6, -0.39], metal, 'chrome'),                           # riser and head
        cyl([0.14, 0.03, 0.14], [-0.2, 1.96, -0.32], metal, 'chrome', rotate=[20, 0, 0]),
        box([0.2, 0.1, 0.08], [0.15, 1.2, -0.38], '#FFFFFF', 'satin', radius=0.02),            # shelf
        cyl([0.05, 0.14, 0.05], [0.12, 1.32, -0.38], TEAL, 'gloss'),
    ])


add('showerStall', shower_stall('#CFE0E4'))
add('showerStall@cozy', shower_stall('#E8D8BE', metal=BRASS))
add('showerStall@minimal', shower_stall('#F4F3EF'))


def walk_in(stone, metal):
    return placeholder([
        box([0.92, 0.04, 0.92], [0, 0.02, 0], stone, 'matte', radius=0.01),
        box([0.88, 2.2, 0.04], [0, 1.12, -0.43], stone, 'matte', radius=0.006),
        box([0.02, 2.0, 0.6], [0.42, 1.04, -0.1], '#DCE6E8', 'glass', radius=0.004),         # glass panel
        box([0.03, 2.0, 0.03], [0.42, 1.04, 0.2], metal, 'metal', radius=0.008),
        cyl([0.03, 0.5, 0.03], [0, 2.0, -0.4], metal, 'metal'),
        rod([0, 2.24, -0.4], [0, 2.24, -0.12], 0.025, metal, 'metal'),
        cyl([0.34, 0.02, 0.34], [0, 2.22, -0.08], metal, 'metal', segments=20),                # rain head
        box([0.06, 0.14, 0.04], [-0.25, 1.1, -0.4], metal, 'metal', radius=0.01),
        box([0.4, 0.04, 0.12], [-0.12, 1.3, -0.37], stone, 'matte', radius=0.01),             # niche shelf
        cyl([0.05, 0.16, 0.05], [-0.22, 1.4, -0.37], '#F2F1EE', 'gloss'),
        cyl([0.05, 0.12, 0.05], [-0.12, 1.38, -0.37], SAGE, 'gloss'),
        box([0.3, 0.02, 0.06], [0, 0.045, 0.2], '#3A3D41', 'metal', radius=0.004),            # drain
    ])


add('showerWalkIn', walk_in('#C9C6C0', BLACKMETAL))
add('showerWalkIn@modern', walk_in('#8E8A84', BLACKMETAL))
add('showerWalkIn@cozy', walk_in('#D9C7A7', BRASS))
add('showerWalkIn@minimal', walk_in('#F4F3EF', CHROME))

# Pedestal basin: a round basin on a column, a tap deck with a mixer, soap and a tooth mug.
z = -0.22
p = [lathe([0.24, 0.72, 0.2], [0, 0.36, z - 0.03], PORCELAIN, 'gloss', [[0, -0.5], [0.5, -0.5], [0.42, -0.42], [0.3, 0.2], [0.42, 0.5], [0, 0.5]], segments=20),
     lathe([0.56, 0.18, 0.46], [0, 0.79, z], PORCELAIN, 'gloss', BOWL, segments=28),
     box([0.5, 0.06, 0.1], [0, 0.87, -0.4], PORCELAIN, 'gloss', radius=0.025),                           # tap deck
     cyl([0.035, 0.12, 0.035], [0, 0.95, -0.4], CHROME, 'chrome', segments=10),
     rod([0, 1.0, -0.4], [0, 0.98, -0.3], 0.026, CHROME, 'chrome', segments=10),
     box([0.016, 0.07, 0.016], [0, 1.02, -0.43], CHROME, 'chrome', radius=0.005, rotate=[-30, 0, 0]),
     cyl([0.4, 0.008, 0.32], [0, 0.765, z + 0.01], '#E4ECEE', 'gloss', segments=24),                     # basin floor
     cyl([0.06, 0.12, 0.06], [0.17, 0.96, -0.41], TEAL, 'gloss', segments=12),
     cyl([0.02, 0.03, 0.02], [0.17, 1.03, -0.41], CHROME, 'chrome', segments=8),
     cyl([0.06, 0.09, 0.06], [-0.17, 0.945, -0.41], '#F2F1EE', 'gloss', segments=12, taper=1.1),
     cyl([0.012, 0.12, 0.012], [-0.165, 1.0, -0.41], MUSTARD, 'gloss', segments=6, rotate=[0, 0, 10])]
addp('sinkPedestal', p)


def double_vanity(cab, top, fin, tap):
    p = [box([1.86, 0.78, 0.52], [0, 0.45, -0.17], cab, fin, radius=0.012),
         box([1.86, 0.06, 0.5], [0, 0.03, -0.17], '#2F3236', 'satin', radius=0.006),
         box([1.9, 0.04, 0.56], [0, 0.86, -0.15], top, 'gloss', radius=0.008)]
    for x in (-0.68, -0.23, 0.23, 0.68):
        p += [box([0.43, 0.62, 0.02], [x, 0.44, 0.1], cab, fin, radius=0.01),
              box([0.14, 0.015, 0.02], [x, 0.7, 0.118], tap, 'metal', radius=0.004)]
    for x in (-0.48, 0.48):
        p += [part('lathe', [0.5, 0.14, 0.38], [x, 0.94, -0.12], '#F6F6F4', 'gloss',
                   profile=[[0, -0.5], [0.4, -0.45], [0.5, 0.5], [0.46, 0.5], [0.36, -0.3], [0, -0.3]], segments=24),
              cyl([0.03, 0.22, 0.03], [x, 0.98, -0.36], tap, 'metal'),
              rod([x, 1.08, -0.36], [x, 1.06, -0.24], 0.02, tap, 'metal'),
              box([0.66, 0.76, 0.03], [x, 1.5, -0.42], '#CFDCE2', 'gloss', radius=0.04)]           # mirrors
    p += [cyl([0.06, 0.16, 0.06], [0.0, 0.96, -0.3], TEAL, 'gloss'), cyl([0.07, 0.1, 0.07], [0.1, 0.93, -0.32], '#F2F1EE', 'gloss')]
    return p


add('vanityDouble', placeholder(double_vanity('#EDEBE6', '#F4F4F2', 'satin', CHROME)), (2, 1))
add('vanityDouble@modern', placeholder(double_vanity(OAK, '#F4F4F2', 'wood', BLACKMETAL)), (2, 1))
add('vanityDouble@cozy', placeholder(double_vanity(SAGE, '#E8E2D6', 'satin', BRASS)), (2, 1))
add('vanityDouble@minimal', placeholder(double_vanity(BIRCH, '#F4F4F2', 'wood', CHROME)), (2, 1))


def towel_rail(metal, towel):
    z = -0.43
    return placeholder([
        cyl([0.025, 0.7, 0.025], [0, 1.2, z + 0.07], metal, 'chrome' if metal == CHROME else 'metal', rotate=[0, 0, 90]),
        cyl([0.025, 0.7, 0.025], [0, 0.95, z + 0.07], metal, 'chrome' if metal == CHROME else 'metal', rotate=[0, 0, 90]),
        box([0.03, 0.06, 0.07], [-0.34, 1.2, z + 0.035], metal, 'metal', radius=0.01),
        box([0.03, 0.06, 0.07], [0.34, 1.2, z + 0.035], metal, 'metal', radius=0.01),
        box([0.03, 0.31, 0.03], [-0.34, 1.075, z + 0.07], metal, 'metal', radius=0.01),
        box([0.03, 0.31, 0.03], [0.34, 1.075, z + 0.07], metal, 'metal', radius=0.01),
        box([0.44, 0.5, 0.05], [-0.06, 1.0, z + 0.09], towel, 'fabric', radius=0.02),          # towels over the bars
        box([0.3, 0.32, 0.04], [0.16, 0.86, z + 0.1], '#FFFFFF', 'fabric', radius=0.02),
    ])


add('towelRail', towel_rail(CHROME, '#9DC6CF'))
add('towelRail@modern', towel_rail(BLACKMETAL, '#C9C6BE'))
add('towelRail@cozy', towel_rail(BRASS, RUST))
def mirror_cabinet(body, bfin, frame):
    """A wall cabinet with a mirror door, a light over it."""
    y0, d = 1.2, 0.16
    return [box([0.56, 0.72, d], [0, y0 + 0.36, WALL + d / 2], body, bfin, radius=0.012),
            box([0.54, 0.7, 0.02], [0, y0 + 0.36, WALL + d + 0.01], frame, bfin, radius=0.008),
            box([0.48, 0.64, 0.006], [0, y0 + 0.36, WALL + d + 0.022], MIRROR, 'gloss', radius=0.004),
            box([0.012, 0.1, 0.02], [0.24, y0 + 0.36, WALL + d + 0.03], frame, bfin, radius=0.004),
            box([0.46, 0.04, 0.06], [0, y0 + 0.75, WALL + 0.1], '#F2F1EE', 'gloss', radius=0.015),
            box([0.42, 0.01, 0.03], [0, y0 + 0.727, WALL + 0.11], BULB, 'gloss', radius=0.004)]


addp('mirrorCabinet', mirror_cabinet('#F2F1EE', 'satin', '#E4E3DF'))
addp('mirrorCabinet@cozy', mirror_cabinet(HONEY, 'wood', WALNUT))


def laundry(dryer=False):
    """A front-loading washer (or dryer): white body, a round door with a glass porthole, a control panel."""
    z = -0.16
    zf = z + 0.3
    p = [box([0.6, 0.84, 0.6], [0, 0.44, z], '#F4F4F2', 'gloss', radius=0.03),
         box([0.56, 0.02, 0.56], [0, 0.01, z], '#3A3D41', 'satin', radius=0.006),
         box([0.6, 0.13, 0.012], [0, 0.78, zf + 0.004], '#E6E6E3', 'gloss', radius=0.006),                  # control panel
         cyl([0.07, 0.03, 0.07], [0.18, 0.78, zf + 0.012], '#C9CBCC', 'satin', rotate=[90, 0, 0], segments=16),
         box([0.12, 0.05, 0.006], [0.02, 0.78, zf + 0.012], '#3A3D41', 'satin', radius=0.006),
         box([0.03, 0.02, 0.006], [0.02, 0.78, zf + 0.016], '#5EC28B' if not dryer else '#E9853B', 'gloss', radius=0.003),
         torus([0.46, 0.05, 0.46], [0, 0.4, zf + 0.012], '#C9CBCC' if not dryer else '#9AA4AB', 'satin', rotate=[90, 0, 0], segments=28),
         cyl([0.4, 0.012, 0.4], [0, 0.4, zf + 0.004], '#2B2C2E', 'satin', rotate=[90, 0, 0], segments=28),  # drum
         cyl([0.38, 0.02, 0.38], [0, 0.4, zf + 0.018], '#BFD3DA', 'glass', rotate=[90, 0, 0], segments=28),
         box([0.03, 0.08, 0.03], [0.22, 0.4, zf + 0.03], '#C9CBCC', 'satin', radius=0.01)]
    if dryer:
        p += [box([0.4, 0.008, 0.01], [0, 0.08 + i * 0.025, zf + 0.002], '#C9CBCC', 'satin', radius=0.003) for i in range(3)]
        p += [lathe([0.42, 0.2, 0.32], [-0.06, 0.96, z + 0.02], '#C9B79C', 'fabric',
                    [[0, -0.5], [0.44, -0.5], [0.5, 0.5], [0.46, 0.5], [0.4, -0.4], [0, -0.4]], segments=16),     # laundry basket
              part('blob', [0.32, 0.12, 0.24], [-0.06, 1.04, z + 0.02], '#E9E2D4', 'fabric', noise=0.25),
              part('blob', [0.16, 0.08, 0.14], [0.02, 1.08, z + 0.06], '#9DC6CF', 'fabric', noise=0.25)]
    else:
        p += [box([0.12, 0.03, 0.006], [-0.18, 0.78, zf + 0.012], '#C9CBCC', 'satin', radius=0.006),       # soap drawer
              part('blob', [0.3, 0.12, 0.3], [0, 0.4, zf - 0.08], '#9DC6CF', 'fabric', noise=0.3),         # washing inside
              box([0.3, 0.08, 0.22], [0.08, 0.92, -0.1], '#FFFFFF', 'fabric', radius=0.02),
              box([0.28, 0.04, 0.2], [0.08, 0.98, -0.1], '#E4B8AE', 'fabric', radius=0.015)]
    return p


addp('laundryWasher', laundry())
addp('laundryDryer', laundry(dryer=True))

# =============================================================================================
# Kids
# =============================================================================================
def kids_bed(frame, sheet, cover, rail):
    p = [box([0.96, 0.24, 1.9], [0, 0.18, 0.0], frame, 'satin', radius=0.03),
         box([0.88, 0.16, 1.82], [0, 0.36, 0.02], sheet, 'fabric', radius=0.05),             # mattress (top 0.44)
         box([0.9, 0.05, 1.1], [0, 0.45, 0.36], cover, 'fabric', radius=0.02),
         box([0.96, 0.7, 0.06], [0, 0.42, -0.95], frame, 'satin', radius=0.03),               # headboard
         cushion([0.58, 0.12, 0.3], [0, 0.49, -0.7], '#FFFFFF')]
    for z in (-0.6, -0.2, 0.2):                                                               # guard rail
        p.append(box([0.03, 0.22, 0.03], [0.46, 0.55, z], rail, 'satin', radius=0.01))
    p += [box([0.03, 0.03, 0.9], [0.46, 0.66, -0.2], rail, 'satin', radius=0.01)]
    for i, (x, c) in enumerate(((-0.2, MUSTARD), (0.0, TEAL), (0.2, RUST))):              # stars on the headboard
        p.append(part('cylinder', [0.12, 0.012, 0.12], [x, 0.6, -0.915], c, 'satin', rotate=[90, 0, 0], segments=5))
    p.append(sph([0.18, 0.2, 0.14], [0.22, 0.52, -0.5], '#C08A52', 'fabric', segments=10))   # plush toy
    return p


add('kidsBed', placeholder(kids_bed('#9DC6CF', '#F7F7F5', '#F2C14E', '#F2F2F0')), (1, 2))
add('kidsBed@cozy', placeholder(kids_bed('#E4B8AE', CREAM, SAGE, '#F2F2F0')), (1, 2))
add('kidsBed@minimal', placeholder(kids_bed('#F2F2F0', '#FFFFFF', LINEN, BIRCH)), (1, 2))


def toy_box():
    p = [box([0.8, 0.46, 0.5], [0, 0.23, -0.18], '#F2C14E', 'satin', radius=0.03),
         box([0.82, 0.05, 0.52], [0, 0.6, -0.36], '#E9A23B', 'satin', radius=0.02, rotate=[-60, 0, 0]),  # open lid
         box([0.1, 0.1, 0.1], [-0.2, 0.5, -0.15], '#D9483B', 'satin', radius=0.02),
         sph([0.16, 0.16, 0.16], [0.15, 0.52, -0.12], '#3E8EDE', 'gloss', segments=10),        # ball
         box([0.1, 0.1, 0.1], [0.32, 0.05, 0.25], '#5EC28B', 'satin', radius=0.02),             # blocks spilled
         box([0.1, 0.1, 0.1], [0.2, 0.05, 0.3], '#D9483B', 'satin', radius=0.02, rotate=[0, 30, 0]),
         box([0.1, 0.1, 0.1], [0.26, 0.15, 0.28], '#3E8EDE', 'satin', radius=0.02)]
    for x in (-0.25, 0.0, 0.25):
        p.append(box([0.16, 0.14, 0.01], [x, 0.28, 0.072], ['#D9483B', '#5EC28B', '#3E8EDE'][int((x + 0.25) / 0.25)], 'satin', radius=0.02))
    return p


add('toyBox', placeholder(toy_box()))


def play_mat():
    p = [box([0.92, 0.02, 0.92], [0, 0.01, 0], '#9DC6CF', 'fabric', radius=0.01)]
    for (x, z), c in zip(((-0.3, -0.3), (0.3, -0.3), (-0.3, 0.3), (0.3, 0.3)), ('#F2C14E', '#E4B8AE', '#B5D99C', '#F2F2F0')):
        p.append(box([0.3, 0.004, 0.3], [x, 0.022, z], c, 'fabric', radius=0.01))
    for i, (x, y, z, c) in enumerate(((-0.05, 0.06, -0.1, '#D9483B'), (0.07, 0.06, -0.1, '#3E8EDE'), (0.01, 0.16, -0.1, '#F2C14E'),
                                      (0.2, 0.06, 0.1, '#5EC28B'), (-0.25, 0.06, 0.15, '#3E8EDE'))):
        p.append(box([0.1, 0.1, 0.1], [x, y, z], c, 'satin', radius=0.02, rotate=[0, i * 17, 0]))
    p += [sph([0.12, 0.12, 0.12], [0.3, 0.08, -0.25], '#D9483B', 'gloss', segments=10)]
    return p


add('playMat', placeholder(play_mat()))
def chalkboard(frame, board):
    """A standing A-frame chalkboard: two framed boards hinged at the top, a chalk tray and drawings on the front."""
    a = -14
    front = [0, 0.5, 0.11]
    p = [box([0.62, 0.96, 0.03], front, frame, 'wood', radius=0.01, rotate=[a, 0, 0]),
         box([0.52, 0.8, 0.036], front, board, 'matte', radius=0.006, rotate=[a, 0, 0]),
         box([0.62, 0.96, 0.03], [0, 0.5, -0.11], frame, 'wood', radius=0.01, rotate=[-a, 0, 0]),
         box([0.52, 0.8, 0.036], [0, 0.5, -0.11], board, 'matte', radius=0.006, rotate=[-a, 0, 0]),
         cyl([0.03, 0.6, 0.03], [0, 0.975, 0.0], frame, 'wood', rotate=[0, 0, 90], segments=10),
         box([0.54, 0.02, 0.07], on_plane(front, a, 0, -0.42, 0.04), frame, 'wood', radius=0.006, rotate=[a, 0, 0])]
    tray = on_plane(front, a, 0, -0.4, 0.05)
    p += [cyl([0.014, 0.08, 0.014], [tray[0] + x, tray[1] + 0.01, tray[2]], c, 'matte', rotate=[0, 0, 90], segments=6)
          for x, c in ((-0.12, '#F2F2EE'), (0.0, '#F2D16B'), (0.1, '#E9A0A8'))]
    on = lambda dx, dy: on_plane(front, a, dx, dy, 0.02)                                                     # noqa: E731
    p += [cyl([0.12, 0.003, 0.12], on(-0.13, 0.24), '#F2D16B', 'matte', rotate=[90 + a, 0, 0], segments=16),   # sun
          box([0.16, 0.12, 0.003], on(0.08, -0.12), '#E9A0A8', 'matte', radius=0.002, rotate=[a, 0, 0]),       # house
          cone([0.22, 0.1, 0.003], on(0.08, -0.01), '#F2F2EE', 'matte', segments=3, rotate=[a, 0, 0]),
          box([0.04, 0.06, 0.003], on(0.1, -0.15), '#8EC5E8', 'matte', radius=0.002, rotate=[a, 0, 0]),
          box([0.4, 0.008, 0.003], on(0.0, -0.22), '#9DD18B', 'matte', radius=0.002, rotate=[a, 0, 0])]
    for i, (dx, rz) in enumerate(((-0.2, 70), (-0.14, -70), (-0.06, 0))):                                       # letters
        p.append(box([0.008, 0.08, 0.003], on(dx + 0.16, 0.26), '#F2F2EE', 'matte', radius=0.002, rotate=[a, 0, rz * 0.3]))
    return p


addp('kidsChalkboard', chalkboard('#B5784A', '#2E3532'))


def doll_house(wall, roof, trim):
    p = [box([0.8, 0.04, 0.42], [0, 0.02, -0.15], trim, 'wood', radius=0.01),
         box([0.76, 0.66, 0.38], [0, 0.37, -0.16], wall, 'satin', radius=0.01),
         box([0.72, 0.02, 0.36], [0, 0.36, -0.14], trim, 'wood', radius=0.004)]               # floor between storeys
    for s in (-1, 1):
        p.append(box([0.44, 0.03, 0.42], [s * 0.19, 0.83, -0.16], roof, 'satin', radius=0.01, rotate=[0, 0, -s * 38]))
    p += [box([0.16, 0.12, 0.004], [-0.2, 0.17, 0.032], '#2A2C30', 'matte', radius=0.004),     # open front: rooms inside
          box([0.12, 0.08, 0.1], [0.18, 0.08, -0.1], RUST, 'fabric', radius=0.02),             # tiny sofa
          box([0.14, 0.04, 0.1], [-0.18, 0.42, -0.1], '#9DC6CF', 'fabric', radius=0.01),       # tiny bed
          box([0.06, 0.12, 0.06], [0.2, 0.44, -0.1], MUSTARD, 'satin', radius=0.01),
          box([0.12, 0.16, 0.02], [0.0, 0.62, 0.02], '#BFE0E6', 'glass', radius=0.01)]
    return p


add('dollHouse', placeholder(doll_house('#F4EBD8', '#D9483B', HONEY)))
add('dollHouse@modern', placeholder(doll_house('#F4F4F2', '#3A3D41', OAK)))


def rocking_horse(body, mane, rocker):
    """A rocking horse whose rider faces +Z (saddle ~0.56 m)."""
    p = []
    for x in (-0.12, 0.12):                                                                   # curved rockers along z
        for i in range(6):
            a0, a1 = math.radians(-16 + i * 32 / 6), math.radians(-16 + (i + 1) * 32 / 6)
            pa = [x, 1.62 - math.cos(a0) * 1.6, math.sin(a0) * 1.6]
            pb = [x, 1.62 - math.cos(a1) * 1.6, math.sin(a1) * 1.6]
            p.append(rod(pa, pb, 0.04, rocker, 'wood'))
    for x in (-0.12, 0.12):
        for z in (-0.2, 0.2):
            p.append(rod([x, 0.04, z * 1.4], [x * 0.6, 0.38, z * 0.7], 0.04, rocker, 'wood'))
    p += [cyl([0.2, 0.5, 0.18], [0, 0.44, 0], body, 'satin', rotate=[90, 0, 0], segments=14),   # body
          rod([0, 0.48, 0.18], [0, 0.72, 0.3], 0.12, body, 'satin'),                            # neck
          box([0.11, 0.12, 0.24], [0, 0.74, 0.38], body, 'satin', radius=0.04, rotate=[15, 0, 0]),  # head
          box([0.04, 0.24, 0.05], [0, 0.66, 0.2], mane, 'fabric', radius=0.02, rotate=[-40, 0, 0]),
          rod([0, 0.46, -0.24], [0, 0.3, -0.34], 0.04, mane, 'fabric'),                         # tail
          box([0.2, 0.03, 0.2], [0, 0.545, -0.02], '#D9483B', 'fabric', radius=0.02),             # saddle
          cyl([0.02, 0.24, 0.02], [0, 0.66, 0.28], '#2A2C30', 'metal', rotate=[0, 0, 90])]          # handle
    return p


add('rockingHorse', placeholder(rocking_horse('#F2F0EA', '#C08A52', '#C9A27A')))


def kids_desk(top, chair):
    p = [box([0.76, 0.03, 0.44], [0, 0.56, 0.2], top, 'wood', radius=0.01)]
    for x in (-0.34, 0.34):
        for z in (0.02, 0.38):
            p.append(cyl([0.04, 0.54, 0.04], [x, 0.27, z], '#F2F2F0', 'satin'))
    p += [box([0.3, 0.005, 0.22], [-0.12, 0.578, 0.2], '#FFFFFF', 'matte', radius=0.002),         # drawing
          box([0.2, 0.004, 0.14], [-0.12, 0.581, 0.2], '#9DC6CF', 'matte', radius=0.002),
          cyl([0.08, 0.1, 0.08], [0.24, 0.62, 0.3], '#F2C14E', 'satin'),                          # crayon cup
          cyl([0.01, 0.1, 0.01], [0.23, 0.69, 0.3], '#D9483B', 'satin'), cyl([0.01, 0.1, 0.01], [0.25, 0.69, 0.31], '#3E8EDE', 'satin')]
    p += chair_facing_front(0, -0.2, chair, chair, 'satin', seat_h=0.36, back_h=0.68)
    return p


add('kidsDesk', placeholder(kids_desk(BIRCH, '#9DC6CF')))
add('kidsDesk@cozy', placeholder(kids_desk(HONEY, '#E4B8AE')))
def teddy(fur, muzzle, bow):
    """A big sitting teddy bear, facing +Z."""
    return [sph([0.5, 0.5, 0.42], [0, 0.28, -0.02], fur, 'fabric', segments=14),                 # body
            sph([0.28, 0.24, 0.1], [0, 0.3, 0.17], muzzle, 'fabric', segments=12),               # tummy
            sph([0.4, 0.38, 0.36], [0, 0.68, 0.0], fur, 'fabric', segments=14),                  # head
            sph([0.16, 0.12, 0.12], [0, 0.64, 0.17], muzzle, 'fabric', segments=10),             # snout
            sph([0.06, 0.04, 0.04], [0, 0.67, 0.23], '#2A2624', 'gloss', segments=8),            # nose
            sph([0.04, 0.04, 0.03], [-0.08, 0.74, 0.16], '#2A2624', 'gloss', segments=6),        # eyes
            sph([0.04, 0.04, 0.03], [0.08, 0.74, 0.16], '#2A2624', 'gloss', segments=6),
            sph([0.14, 0.14, 0.08], [-0.15, 0.86, -0.02], fur, 'fabric', segments=10),          # ears
            sph([0.14, 0.14, 0.08], [0.15, 0.86, -0.02], fur, 'fabric', segments=10),
            sph([0.14, 0.26, 0.14], [-0.26, 0.36, 0.06], fur, 'fabric', segments=10),           # arms
            sph([0.14, 0.26, 0.14], [0.26, 0.36, 0.06], fur, 'fabric', segments=10),
            sph([0.18, 0.14, 0.3], [-0.14, 0.08, 0.2], fur, 'fabric', segments=10),             # legs
            sph([0.18, 0.14, 0.3], [0.14, 0.08, 0.2], fur, 'fabric', segments=10),
            box([0.2, 0.08, 0.06], [0, 0.5, 0.15], bow, 'satin', radius=0.03)]                    # bow


add('teddyBear', placeholder(teddy('#C08A52', '#E9D5B5', '#D9483B')))

# =============================================================================================
# Office
# =============================================================================================
def desk(top, legs, chair, top_fin='wood', leg_fin='metal', w=1.54, laptop=False, lamp=None):
    p = [box([w, 0.035, 0.42], [0, 0.735, 0.24], top, top_fin, radius=0.008)]
    for s in (-1, 1):
        p += [box([0.04, 0.72, 0.04], [s * (w / 2 - 0.04), 0.36, 0.06], legs, leg_fin, radius=0.006),
              box([0.04, 0.72, 0.04], [s * (w / 2 - 0.04), 0.36, 0.42], legs, leg_fin, radius=0.006)]
    p.append(box([w - 0.1, 0.06, 0.02], [0, 0.69, 0.43], legs, leg_fin, radius=0.006))
    if laptop:
        p += [box([0.34, 0.015, 0.24], [0.02, 0.76, 0.2], '#C9CBCC', 'metal', radius=0.008),
              box([0.34, 0.22, 0.012], [0.02, 0.87, 0.32], '#C9CBCC', 'metal', radius=0.008, rotate=[-15, 0, 0]),
              box([0.31, 0.19, 0.003], [0.02, 0.87, 0.313], '#2E5470', 'gloss', radius=0.004, rotate=[-15, 0, 0])]
    else:
        p += [box([0.42, 0.012, 0.3], [-0.1, 0.758, 0.2], '#FFFFFF', 'matte', radius=0.002, rotate=[0, 8, 0]),   # paper
              cyl([0.012, 0.15, 0.012], [0.05, 0.76, 0.2], '#2A2C30', 'satin', rotate=[0, 0, 85]),             # pen
              box([0.2, 0.06, 0.28], [0.45, 0.78, 0.26], MUSTARD, 'matte', radius=0.006)]                    # notebooks
    if lamp:
        p += [cyl([0.13, 0.02, 0.13], [-w / 2 + 0.2, 0.763, 0.36], lamp, 'metal'),
              rod([-w / 2 + 0.2, 0.77, 0.36], [-w / 2 + 0.24, 1.1, 0.31], 0.018, lamp, 'metal'),
              rod([-w / 2 + 0.24, 1.1, 0.31], [-w / 2 + 0.35, 1.06, 0.22], 0.016, lamp, 'metal'),
              cone([0.12, 0.1, 0.12], [-w / 2 + 0.37, 1.03, 0.2], lamp, 'metal', taper=0.4, rotate=[35, 0, 0]),
              sph([0.05, 0.05, 0.05], [-w / 2 + 0.37, 0.995, 0.18], BULB, 'gloss', segments=8)]
    p += office_chair(-0.17, chair, legs if leg_fin == 'metal' else BLACKMETAL)
    return p


add('deskWriting', placeholder(desk(OAK, BLACKMETAL, SAGE, lamp=BLACKMETAL)), (2, 1))
add('deskWriting@modern', placeholder(desk('#F1F0EC', BLACKMETAL, '#8E979E', 'gloss', lamp=BLACKMETAL)), (2, 1))
add('deskWriting@cozy', placeholder(desk(HONEY, WALNUT, RUST, leg_fin='wood', lamp='#2F6B4F')), (2, 1))
add('deskWriting@minimal', placeholder(desk(BIRCH, '#F2F2F0', '#E6E2DA', leg_fin='satin', lamp='#F2F2F0')), (2, 1))
add('deskLaptop', placeholder(desk(OAK, BLACKMETAL, '#5F6E7A', w=0.9, laptop=True)))
add('deskLaptop@modern', placeholder(desk('#F1F0EC', BLACKMETAL, '#8E979E', 'gloss', w=0.9, laptop=True)))
add('deskLaptop@cozy', placeholder(desk(HONEY, WALNUT, RUST, leg_fin='wood', w=0.9, laptop=True)))
add('deskLaptop@minimal', placeholder(desk(BIRCH, '#F2F2F0', '#E6E2DA', leg_fin='satin', w=0.9, laptop=True)))


def gaming_pc(accent):
    p = desk('#2F3236', '#2F3236', '#2F3236', 'satin', w=1.7)
    p = [q for q in p if q['at'][1] < 0.74 or q['at'][2] < 0.0]           # bare desk and chair
    p += [box([1.7, 0.035, 0.44], [0, 0.735, 0.24], '#2F3236', 'satin', radius=0.008),
          box([1.6, 0.004, 0.36], [0, 0.755, 0.22], '#3A3D41', 'matte', radius=0.004),          # desk mat
          box([1.62, 0.01, 0.01], [0, 0.74, 0.465], accent, 'gloss', radius=0.003)]             # LED strip
    for x, ry in ((-0.36, 18), (0.36, -18)):                                                    # two monitors angled in
        p += [box([0.62, 0.36, 0.03], [x, 1.03, 0.38], '#22252A', 'satin', radius=0.01, rotate=[0, ry, 0]),
              box([0.6, 0.34, 0.004], [x, 1.03, 0.3635], '#3B2E6E' if x < 0 else '#2E5470', 'gloss', radius=0.004, rotate=[0, ry, 0]),
              box([0.05, 0.22, 0.03], [x, 0.84, 0.41], '#22252A', 'satin', radius=0.01)]
    p += [box([0.46, 0.02, 0.15], [0, 0.765, 0.17], '#22252A', 'satin', radius=0.006),
          box([0.44, 0.004, 0.13], [0, 0.777, 0.17], accent, 'gloss', radius=0.004),                  # backlit keys
          box([0.08, 0.025, 0.12], [0.36, 0.765, 0.17], '#22252A', 'satin', radius=0.03),
          box([0.24, 0.5, 0.48], [0.62, 0.25, 0.2], '#22252A', 'satin', radius=0.01),                 # tower under the desk
          box([0.005, 0.4, 0.38], [0.5, 0.26, 0.2], accent, 'glass', radius=0.004),
          box([0.16, 0.2, 0.14], [-0.66, 0.85, 0.36], '#22252A', 'satin', radius=0.02),               # speaker
          cyl([0.06, 0.1, 0.06], [-0.5, 0.8, 0.38], accent, 'gloss')]
    p += office_chair(-0.17, '#3A3D41', '#22252A')
    return p


add('computerGaming', placeholder(gaming_pc('#6E5CE6')), (2, 1))
add('computerGaming@cozy', placeholder(gaming_pc('#F2A23C')), (2, 1))
def desk_chair(fabric, frame, ffin, back_h=0.62, arms=True, head=False):
    """A swivel desk chair (seat 0.47, centred): padded seat and back, a gas lift on five castors."""
    p = [cushion([0.52, 0.09, 0.5], [0, 0.43, 0.02], fabric, radius=0.04),
         cushion([0.48, back_h, 0.09], [0, 0.52 + back_h / 2, -0.24], fabric, rotate=[-9, 0, 0], radius=0.04),
         box([0.1, 0.24, 0.03], [0, 0.5, -0.22], frame, ffin, radius=0.01),
         cyl([0.05, 0.28, 0.05], [0, 0.24, 0.0], CHROME if ffin != 'satin' else frame, ffin if ffin == 'satin' else 'chrome', segments=12),
         cyl([0.08, 0.05, 0.08], [0, 0.11, 0.0], frame, ffin, segments=12)]
    if head:
        p.append(cushion([0.3, 0.14, 0.08], [0, 0.6 + back_h + 0.06, -0.33], fabric, rotate=[-12, 0, 0], radius=0.035))
    if arms:
        for x in (-0.28, 0.28):
            p += [box([0.05, 0.035, 0.28], [x, 0.66, 0.02], frame, ffin, radius=0.012),
                  box([0.025, 0.2, 0.025], [x, 0.55, -0.04], frame, ffin, radius=0.006)]
    for i in range(5):
        a = math.radians(i * 72 + 18)
        tip = [math.cos(a) * 0.28, 0.05, math.sin(a) * 0.28]
        p += [rod([0, 0.1, 0], tip, 0.03, frame, ffin, shape='box', t2=0.035, radius=0.008),
              sph([0.05, 0.05, 0.05], [tip[0], 0.025, tip[2]], '#1E1F22', 'satin', segments=8)]
    return p


addp('officeChair', desk_chair('#5F6E7A', BLACKMETAL, 'metal'))
addp('officeChair@cozy', desk_chair(RUST, '#3A3530', 'metal', back_h=0.7, head=True))
addp('officeChair@minimal', desk_chair(LINEN, '#F2F2F0', 'satin', back_h=0.48, arms=False))

# =============================================================================================
# Hobbies
# =============================================================================================
def guitar_stand(metal=BLACKMETAL):
    return [rod([0, 0.02, 0.12], [0, 0.62, -0.02], 0.03, metal, 'metal'),
            rod([0, 0.06, 0.12], [-0.16, 0.0, -0.06], 0.025, metal, 'metal'),
            rod([0, 0.06, 0.12], [0.16, 0.0, -0.06], 0.025, metal, 'metal'),
            rod([-0.12, 0.2, 0.08], [0.12, 0.2, 0.08], 0.03, metal, 'metal'),
            sph([0.05, 0.05, 0.05], [0, 0.63, -0.03], '#2A2C30', 'satin', segments=8)]


def ukulele(body, top):
    """A ukulele leaning in its stand (bottom on the cradle, neck on the rest)."""
    a = 15
    up = [0, math.cos(math.radians(a)), -math.sin(math.radians(a))]
    n = [0, math.sin(math.radians(a)), math.cos(math.radians(a))]
    b = [0, 0.17, 0.15]
    at = lambda u, f=0.0: [round(b[i] + up[i] * u + n[i] * f, 4) for i in range(3)]       # noqa: E731
    return [cyl([0.23, 0.07, 0.2], at(0.1), body, 'wood', rotate=[90 - a, 0, 0], segments=24),             # lower bout
            cyl([0.18, 0.07, 0.16], at(0.25), body, 'wood', rotate=[90 - a, 0, 0], segments=24),
            cyl([0.21, 0.004, 0.18], at(0.1, 0.035), top, 'wood', rotate=[90 - a, 0, 0], segments=24),      # top
            cyl([0.16, 0.004, 0.14], at(0.25, 0.035), top, 'wood', rotate=[90 - a, 0, 0], segments=24),
            cyl([0.06, 0.004, 0.06], at(0.2, 0.038), '#2A2018', 'satin', rotate=[90 - a, 0, 0], segments=16),  # sound hole
            box([0.08, 0.015, 0.012], at(0.07, 0.04), '#4A3426', 'wood', radius=0.004, rotate=[-a, 0, 0]),   # bridge
            box([0.045, 0.24, 0.022], at(0.45, 0.005), body, 'wood', radius=0.006, rotate=[-a, 0, 0]),      # neck
            box([0.04, 0.22, 0.005], at(0.44, 0.018), '#4A3426', 'wood', radius=0.002, rotate=[-a, 0, 0]),  # fretboard
            box([0.06, 0.09, 0.02], at(0.61), '#4A3426', 'wood', radius=0.01, rotate=[-a, 0, 0])]           # head


addp('ukulele', guitar_stand() + ukulele('#C08A52', '#D9A86C'))


def electric_guitar(body, amp):
    p = guitar_stand()
    p += [box([0.36, 0.44, 0.05], [0, 0.42, 0.06], body, 'gloss', radius=0.12, rotate=[-12, 0, 0]),    # body
          box([0.2, 0.2, 0.052], [0.02, 0.36, 0.064], '#F2F2F0', 'gloss', radius=0.05, rotate=[-12, 0, 0]),  # pickguard
          box([0.05, 0.62, 0.03], [0, 0.92, -0.04], '#C9A27A', 'wood', radius=0.01, rotate=[-12, 0, 0]),     # neck
          box([0.08, 0.16, 0.03], [0, 1.28, -0.12], '#2A2C30', 'gloss', radius=0.02, rotate=[-12, 0, 0])]    # head
    p += [box([0.46, 0.4, 0.28], [0.0, 0.2, -0.28], amp, 'matte', radius=0.02),                              # amp behind
          box([0.4, 0.26, 0.004], [0, 0.18, -0.138], '#4A4D52', 'fabric', radius=0.01),
          box([0.4, 0.06, 0.004], [0, 0.36, -0.138], '#C9A27A', 'satin', radius=0.004)]
    return p


add('guitarElectric', placeholder(electric_guitar('#D9483B', '#2A2C30')))
add('guitarElectric@modern', placeholder(electric_guitar('#F2F2F0', '#3A3D41')))
add('guitarElectric@cozy', placeholder(electric_guitar('#C08A52', '#5A3F2A')))


def keyboard(body, stand, seat):
    p = [box([0.9, 0.1, 0.3], [0, 0.78, 0.22], body, 'satin', radius=0.02),
         box([0.8, 0.02, 0.15], [0, 0.835, 0.17], '#F6F3EA', 'satin', radius=0.003),
         box([0.9, 0.03, 0.06], [0, 0.845, 0.33], body, 'satin', radius=0.01),
         box([0.4, 0.22, 0.012], [0, 0.98, 0.33], '#F4EFE2', 'matte', radius=0.004, rotate=[-12, 0, 0])]
    for i in range(20):
        x = -0.4 + i * 0.039
        if i % 7 in (2, 6): continue
        p.append(box([0.011, 0.018, 0.09], [x + 0.02, 0.852, 0.2], '#141414', 'gloss', radius=0.002))
    for s in (-1, 1):                                                                       # X stand
        p += [rod([s * 0.34, 0.02, 0.08], [s * 0.34, 0.72, 0.36], 0.03, stand, 'metal'),
              rod([s * 0.34, 0.02, 0.36], [s * 0.34, 0.72, 0.08], 0.03, stand, 'metal')]
    p += [box([0.62, 0.06, 0.32], [0, 0.42, -0.2], seat, 'fabric', radius=0.03),          # bench (0.45)
          rod([-0.26, 0.0, -0.32], [-0.26, 0.39, -0.2], 0.03, stand, 'metal'), rod([0.26, 0.0, -0.32], [0.26, 0.39, -0.2], 0.03, stand, 'metal'),
          rod([-0.26, 0.0, -0.08], [-0.26, 0.39, -0.2], 0.03, stand, 'metal'), rod([0.26, 0.0, -0.08], [0.26, 0.39, -0.2], 0.03, stand, 'metal')]
    return p


add('keyboardDigital', placeholder(keyboard('#2A2C30', BLACKMETAL, '#3A3D41')))
add('keyboardDigital@modern', placeholder(keyboard('#F2F2F0', BLACKMETAL, '#9AA4AB')))
add('keyboardDigital@cozy', placeholder(keyboard(WALNUT, '#3A3530', RUST)))


def sewing_table(top, legs, seat):
    p = [box([0.9, 0.035, 0.46], [0, 0.735, 0.2], top, 'wood', radius=0.008)]
    for x in (-0.4, 0.4):
        for z in (0.02, 0.4):
            p.append(box([0.04, 0.72, 0.04], [x, 0.36, z], legs, 'metal' if legs == BLACKMETAL else 'wood', radius=0.006))
    p += [box([0.36, 0.08, 0.16], [0.02, 0.795, 0.28], '#F2F1EE', 'gloss', radius=0.02),         # sewing machine base
          box([0.08, 0.2, 0.14], [0.16, 0.92, 0.28], '#F2F1EE', 'gloss', radius=0.03),
          box([0.32, 0.08, 0.14], [0.02, 1.0, 0.28], '#F2F1EE', 'gloss', radius=0.03),
          box([0.06, 0.1, 0.06], [-0.14, 0.92, 0.28], '#F2F1EE', 'gloss', radius=0.02),
          cyl([0.08, 0.02, 0.08], [0.21, 0.94, 0.28], '#9DC6CF', 'satin', rotate=[0, 0, 90]),
          box([0.4, 0.008, 0.3], [-0.2, 0.757, 0.14], BLUSH, 'fabric', radius=0.004, rotate=[0, 12, 0]),  # fabric
          cyl([0.05, 0.06, 0.05], [0.36, 0.78, 0.12], RUST, 'satin'), cyl([0.05, 0.06, 0.05], [0.36, 0.78, 0.22], TEAL, 'satin'),
          lathe([0.22, 0.12, 0.22], [-0.32, 0.81, 0.36], '#C9B79C', 'matte',
                [[0, -0.5], [0.44, -0.5], [0.5, 0.5], [0.46, 0.5], [0.4, -0.4], [0, -0.4]])]
    p += chair_facing_front(0, -0.22, top, seat, 'wood')
    return p


add('sewingTable', placeholder(sewing_table(OAK, OAKD, SAGE)))
add('sewingTable@cozy', placeholder(sewing_table(HONEY, WALNUT, RUST)))


def pool_table(felt, wood, fin='wood'):
    p = [box([2.5, 0.1, 1.4], [0, 0.72, 0], wood, fin, radius=0.04),                          # rails
         box([2.3, 0.02, 1.2], [0, 0.775, 0], felt, 'fabric', radius=0.01),                    # cloth
         box([2.4, 0.18, 1.3], [0, 0.6, 0], wood, fin, radius=0.02)]                           # apron
    for x in (-1.12, 1.12):
        for z in (-0.52, 0.52):
            p.append(box([0.16, 0.56, 0.16], [x, 0.28, z], wood, fin, radius=0.02))           # legs
    for x in (-1.16, 0.0, 1.16):
        for z in (-0.62, 0.62):
            p.append(cyl([0.1, 0.02, 0.1], [x, 0.772, z * 0.97], '#141414', 'matte'))           # pockets
    balls = [('#F2C14E', 0.5, 0.0), ('#3E8EDE', 0.56, 0.04), ('#D9483B', 0.56, -0.04), ('#6E3FA8', 0.62, 0.08),
             ('#141414', 0.62, 0.0), ('#E9853B', 0.62, -0.08), ('#2F8F4E', 0.68, 0.04), ('#8E2A2A', 0.68, -0.04),
             ('#FFFFFF', -0.6, 0.0)]
    for c, x, z in balls:
        p.append(sph([0.057, 0.057, 0.057], [x, 0.815, z], c, 'gloss', segments=10))
    p += [cyl([0.025, 1.45, 0.025], [-0.2, 0.8, 0.3], '#C9A27A', 'wood', rotate=[0, 70, 88]),   # cue on the table
          box([0.2, 0.06, 0.03], [0, 0.11, 0.66], wood, fin, radius=0.01)]
    return p


add('poolTable', placeholder(pool_table('#2F7A57', WALNUT)), (3, 2))
add('poolTable@modern', placeholder(pool_table('#3E5470', '#F1F0EC', 'gloss')), (3, 2))
add('poolTable@cozy', placeholder(pool_table('#8E2A2A', HONEY)), (3, 2))


def foosball(body, men_a, men_b):
    p = [box([1.4, 0.24, 0.72], [0, 0.8, 0], body, 'wood', radius=0.02),
         box([1.3, 0.02, 0.62], [0, 0.72, 0], '#2F8F4E', 'matte', radius=0.004)]
    for x in (-0.6, 0.6):
        for z in (-0.3, 0.3):
            p.append(box([0.08, 0.7, 0.08], [x, 0.35, z], body, 'wood', radius=0.01))
    for i, x in enumerate((-0.45, -0.15, 0.15, 0.45)):                                        # rods with players
        c = men_a if i % 2 == 0 else men_b
        p.append(cyl([0.02, 0.96, 0.02], [x, 0.84, 0], CHROME, 'chrome', rotate=[90, 0, 0]))
        p.append(cyl([0.05, 0.08, 0.05], [x, 0.84, 0.46], '#2A2C30', 'satin', rotate=[90, 0, 0]))
        for z in (-0.18, 0.0, 0.18):
            p.append(box([0.04, 0.12, 0.05], [x, 0.8, z], c, 'gloss', radius=0.01))
    p.append(sph([0.04, 0.04, 0.04], [0.05, 0.75, 0.05], '#FFFFFF', 'gloss', segments=8))
    return p


add('foosball', placeholder(foosball(OAK, '#D9483B', '#3E8EDE')), (2, 1))
def dartboard():
    y, zb = 1.725, WALL + 0.022
    p = [box([0.62, 0.62, 0.02], [0, y, WALL + 0.012], '#2A2C30', 'matte', radius=0.02),
         cyl([0.45, 0.04, 0.45], [0, y, zb + 0.02], '#1E1F22', 'matte', rotate=[90, 0, 0], segments=32)]
    zf = zb + 0.04
    for k, (d, c) in enumerate(((0.36, '#C23B2E'), (0.335, '#EADBB8'))):
        p.append(cyl([d, 0.004, d], [0, y, zf + 0.002 * k], c, 'satin', rotate=[90, 0, 0], segments=32))
    p += [box([0.04, 0.33, 0.003], [0, y, zf + 0.005], '#1E1F22', 'satin', radius=0.001, rotate=[0, 0, k * 36]) for k in range(5)]
    for d, c, dz in ((0.23, '#2F7A4A', 0.007), (0.205, '#EADBB8', 0.009), (0.05, '#2F7A4A', 0.014), (0.024, '#C23B2E', 0.016)):
        p.append(cyl([d, 0.004, d], [0, y, zf + dz], c, 'satin', rotate=[90, 0, 0], segments=32))
    p += [box([0.026, 0.2, 0.003], [0, y, zf + 0.0115], '#1E1F22', 'satin', radius=0.001, rotate=[0, 0, k * 36]) for k in range(5)]
    for x, dy, c in ((0.02, 0.01, '#D9483B'), (-0.07, 0.08, '#3E8EDE'), (0.09, -0.06, '#F2C14E')):            # darts
        p += [cyl([0.008, 0.1, 0.008], [x, y + dy, zf + 0.06], '#C9CBCC', 'metal', rotate=[90, 0, 0], segments=6),
              box([0.04, 0.003, 0.04], [x, y + dy, zf + 0.11], c, 'satin', radius=0.001),
              box([0.003, 0.04, 0.04], [x, y + dy, zf + 0.11], c, 'satin', radius=0.001)]
    return p


addp('dartboard', dartboard())


def chemistry(top, legs):
    p = [box([0.92, 0.04, 0.5], [0, 0.88, -0.15], top, 'satin', radius=0.008)]
    for x in (-0.42, 0.42):
        for z in (-0.36, 0.06):
            p.append(box([0.04, 0.86, 0.04], [x, 0.43, z], legs, 'metal', radius=0.006))
    p += [box([0.9, 0.03, 0.44], [0, 0.3, -0.15], legs, 'metal', radius=0.006),
          box([0.8, 0.5, 0.04], [0, 1.15, -0.38], '#F2F1EE', 'satin', radius=0.008),             # back board
          box([0.8, 0.02, 0.16], [0, 1.25, -0.3], top, 'satin', radius=0.004)]                   # shelf
    flasks = [(-0.3, 0.92, -0.1, '#7FD1AE'), (-0.15, 0.92, -0.05, '#E9853B'), (0.25, 0.92, -0.08, '#6E8BE6')]
    for x, y, z, c in flasks:
        p += [part('lathe', [0.12, 0.18, 0.12], [x, y + 0.09, z], c, 'glass',
                   profile=[[0, -0.5], [0.5, -0.5], [0.5, -0.2], [0.18, 0.3], [0.18, 0.5], [0, 0.5]], segments=12)]
    for i in range(5):
        p.append(cyl([0.03, 0.14, 0.03], [-0.3 + i * 0.12, 1.33, -0.3], ['#7FD1AE', '#F2C14E', '#E9853B', '#6E8BE6', '#D9483B'][i], 'glass'))
    p += [box([0.12, 0.2, 0.12], [0.05, 1.0, -0.2], '#2A2C30', 'satin', radius=0.01),           # microscope
          cyl([0.04, 0.16, 0.04], [0.05, 1.16, -0.16], '#2A2C30', 'satin', rotate=[-20, 0, 0]),
          box([0.16, 0.02, 0.14], [0.05, 0.91, -0.12], '#2A2C30', 'satin', radius=0.004)]
    return p


add('chemistrySet', placeholder(chemistry('#E4E3DF', BLACKMETAL)))

# =============================================================================================
# Fitness
# =============================================================================================
def exercise_bike(frame, accent):
    p = [box([0.16, 0.06, 0.9], [0, 0.03, 0.0], frame, 'satin', radius=0.02),
         box([0.5, 0.05, 0.08], [0, 0.03, -0.38], frame, 'satin', radius=0.02),
         box([0.5, 0.05, 0.08], [0, 0.03, 0.38], frame, 'satin', radius=0.02),
         cyl([0.42, 0.06, 0.42], [0, 0.3, 0.22], '#3A3D41', 'metal', rotate=[0, 0, 90], segments=24),   # flywheel
         rod([0, 0.06, -0.15], [0, 0.66, -0.1], 0.07, frame, 'satin', shape='box', t2=0.07, radius=0.02),  # seat post
         rod([0, 0.06, 0.25], [0, 0.95, 0.32], 0.07, frame, 'satin', shape='box', t2=0.07, radius=0.02),   # handle post
         box([0.26, 0.07, 0.3], [0, 0.72, -0.12], '#2A2C30', 'fabric', radius=0.03),                     # saddle (0.75)
         rod([-0.22, 0.98, 0.32], [0.22, 0.98, 0.32], 0.035, '#2A2C30', 'satin'),
         rod([-0.22, 0.98, 0.32], [-0.22, 1.0, 0.18], 0.035, '#2A2C30', 'satin'),
         rod([0.22, 0.98, 0.32], [0.22, 1.0, 0.18], 0.035, '#2A2C30', 'satin'),
         box([0.2, 0.12, 0.04], [0, 1.05, 0.36], '#22252A', 'satin', radius=0.01, rotate=[-30, 0, 0]),     # console
         box([0.16, 0.08, 0.004], [0, 1.05, 0.38], accent, 'gloss', radius=0.004, rotate=[-30, 0, 0]),
         box([0.03, 0.2, 0.05], [0.12, 0.3, 0.1], '#3A3D41', 'metal', radius=0.01),                      # cranks
         box([0.1, 0.03, 0.06], [0.16, 0.2, 0.12], '#2A2C30', 'satin', radius=0.01),
         box([0.1, 0.03, 0.06], [-0.16, 0.4, 0.08], '#2A2C30', 'satin', radius=0.01)]
    return p


add('exerciseBike', placeholder(exercise_bike('#D8D9D7', '#3C8DBF')))


def yoga_mat(color):
    p = [box([0.62, 0.012, 0.9], [0, 0.006, 0.0], color, 'fabric', radius=0.004),
         cyl([0.14, 0.62, 0.14], [0, 0.07, -0.38], color, 'fabric', rotate=[0, 0, 90]),       # rolled end
         box([0.24, 0.12, 0.14], [0.36, 0.06, -0.3], '#E8E2D6', 'fabric', radius=0.04),        # block
         cyl([0.07, 0.24, 0.07], [-0.36, 0.12, -0.3], '#9DC6CF', 'gloss')]                     # bottle
    return p


add('yogaMat', placeholder(yoga_mat('#7FA38A')))
add('yogaMat@modern', placeholder(yoga_mat('#8E979E')))
add('yogaMat@cozy', placeholder(yoga_mat('#C98F6B')))


def dumbbells(frame):
    p = []
    for s in (-1, 1):
        p += [rod([s * 0.36, 0.0, -0.1], [s * 0.36, 0.7, -0.2], 0.05, frame, 'metal', shape='box', t2=0.05, radius=0.01),
              box([0.06, 0.04, 0.4], [s * 0.36, 0.02, -0.15], frame, 'metal', radius=0.01)]
    for y, z in ((0.32, -0.08), (0.58, -0.18)):
        p.append(box([0.78, 0.03, 0.14], [0, y, z], frame, 'metal', radius=0.006, rotate=[-18, 0, 0]))
        for i in range(4):
            x = -0.27 + i * 0.18
            w = 0.1 + i * 0.012
            p += [cyl([0.025, 0.14, 0.025], [x, y + 0.06, z], CHROME, 'chrome', rotate=[0, 0, 90]),
                  cyl([w, 0.04, w], [x - 0.06, y + 0.06, z], '#2F3236', 'satin', rotate=[0, 0, 90], segments=6),
                  cyl([w, 0.04, w], [x + 0.06, y + 0.06, z], '#2F3236', 'satin', rotate=[0, 0, 90], segments=6)]
    p.append(box([0.6, 0.01, 0.4], [0, 0.005, 0.2], '#3A3D41', 'matte', radius=0.004))        # floor mat
    return p


add('dumbbells', placeholder(dumbbells('#5C6168')))

# =============================================================================================
# Decor: wall art, ornaments, rugs and curtains
# =============================================================================================
def frame_on_wall(w, h, y0, frame, ffin, depth, mount=None, border=0.025, mount_w=0.06):
    """A picture frame hanging from y0; returns its parts and the picture's (centre y, width, height, face z)."""
    p = [box([w, h, depth], [0, y0 + h / 2, WALL + depth / 2], frame, ffin, radius=min(0.008, depth * 0.4))]
    zf, iw, ih = WALL + depth + 0.002, w - 2 * border, h - 2 * border
    if mount:
        p.append(box([iw, ih, 0.004], [0, y0 + h / 2, zf], mount, 'matte', radius=0.002))
        zf, iw, ih = zf + 0.003, iw - 2 * mount_w, ih - 2 * mount_w
    return p, y0 + h / 2, iw, ih, zf


# Botanical print: a white frame and mount, a leafy stem on cream paper.
p, yc, iw, ih, zf = frame_on_wall(0.5, 0.71, 1.15, '#F2F0EA', 'satin', 0.03, mount='#FBFAF6', mount_w=0.055)
p += [box([iw, ih, 0.004], [0, yc, zf], '#F1EBDD', 'matte', radius=0.002),
      box([0.008, ih * 0.75, 0.004], [0.0, yc - 0.02, zf + 0.003], OLIVE, 'matte', radius=0.002, rotate=[0, 0, 4])]
for i, (dy, s) in enumerate(((-0.14, -1), (-0.07, 1), (0.0, -1), (0.07, 1), (0.14, -1), (0.2, 1))):
    p.append(sph([0.1 - i * 0.008, 0.045 - i * 0.003, 0.004], [s * 0.045, yc + dy, zf + 0.004 + i * 0.0005], [SAGE, OLIVE][i % 2], 'matte',
                 segments=12, rotate=[0, 0, s * 35]))
addp('printFramed', p)

# Photo print: a thin black frame, a mountain lake.
p, yc, iw, ih, zf = frame_on_wall(0.75, 0.5, 1.25, '#2A2C30', 'satin', 0.033, mount='#FBFAF6', mount_w=0.05)
y0 = yc - ih / 2
p += [box([iw, ih, 0.004], [0, yc, zf], '#A9C8DD', 'matte', radius=0.002),                                   # sky
      cone([0.36, 0.2, 0.004], [-0.1, y0 + 0.17, zf + 0.003], '#6E7E8A', 'matte', segments=3),                 # mountains
      cone([0.3, 0.16, 0.004], [0.1, y0 + 0.15, zf + 0.004], '#56656F', 'matte', segments=3),
      cone([0.09, 0.05, 0.004], [-0.1, y0 + 0.245, zf + 0.0045], '#F4F4F2', 'matte', segments=3),
      box([iw, 0.08, 0.004], [0, y0 + 0.07, zf + 0.005], '#6E95AE', 'matte', radius=0.001),                   # lake
      box([iw, 0.035, 0.004], [0, y0 + 0.0175, zf + 0.006], '#4E6B4A', 'matte', radius=0.001),
      sph([0.05, 0.05, 0.004], [0.2, y0 + ih - 0.06, zf + 0.003], '#F6E7B0', 'matte', segments=12)]
addp('photoPrint', p)

# Classic painting: a gilt frame round a pastoral oil painting.
p, yc, iw, ih, zf = frame_on_wall(0.84, 0.65, 1.15, BRASS, 'metal', 0.05, border=0.07)
p.append(box([iw + 0.04, ih + 0.04, 0.012], [0, yc, WALL + 0.055], '#8E6E3A', 'metal', radius=0.004))         # inner bead
y0 = yc - ih / 2
p += [box([iw, ih, 0.004], [0, yc, zf + 0.012], '#C9D3C8', 'matte', radius=0.002),                           # sky
      sph([0.22, 0.06, 0.004], [-0.12, yc + 0.14, zf + 0.015], '#EDEBE2', 'matte', segments=12),
      sph([0.16, 0.05, 0.004], [0.16, yc + 0.17, zf + 0.015], '#EDEBE2', 'matte', segments=12),
      sph([iw * 1.1, 0.2, 0.004], [-0.05, y0 + 0.2, zf + 0.016], '#8E9A7A', 'matte', segments=20),             # hills
      box([iw, 0.16, 0.004], [0, y0 + 0.08, zf + 0.017], '#B59B5E', 'matte', radius=0.002),                    # field
      box([0.02, 0.1, 0.004], [-0.18, y0 + 0.18, zf + 0.018], '#4A3A2C', 'matte', radius=0.002),
      sph([0.16, 0.16, 0.004], [-0.18, y0 + 0.28, zf + 0.019], '#4E5E3A', 'matte', segments=14),               # tree
      box([0.08, 0.05, 0.004], [0.14, y0 + 0.17, zf + 0.018], '#E7E0D4', 'matte', radius=0.001),               # cottage
      cone([0.11, 0.04, 0.004], [0.14, y0 + 0.215, zf + 0.019], '#8E3F2E', 'matte', segments=3)]
addp('paintingClassic', p)


def canvas(colors, w=1.5, h=0.9):
    y = 1.05
    z = -0.43
    p = [box([w, h, 0.04], [0, y + h / 2, z + 0.02], '#F4F2EE', 'matte', radius=0.004)]       # stretched canvas
    a, b, c, d = colors
    p += [box([w * 0.5, h * 0.7, 0.004], [-w * 0.18, y + h * 0.45, z + 0.042], a, 'matte', radius=0.004),
          sph([h * 0.5, h * 0.5, 0.006], [w * 0.2, y + h * 0.58, z + 0.044], b, 'matte', segments=20),
          box([w * 0.3, h * 0.18, 0.004], [w * 0.22, y + h * 0.2, z + 0.046], c, 'matte', radius=0.004),
          box([0.02, h * 0.7, 0.004], [-w * 0.02, y + h * 0.5, z + 0.048], d, 'matte', radius=0.002)]
    return placeholder(p)


add('canvasAbstract', canvas((SAGE, MUSTARD, RUST, '#2A2C30')), (2, 1))
add('canvasAbstract@modern', canvas(('#9AA4AB', COGNAC, '#F2F2F0', '#2A2C30')), (2, 1))
add('canvasAbstract@cozy', canvas((CLAY, '#E9C46A', OLIVE, WALNUT)), (2, 1))
add('canvasAbstract@minimal', canvas(('#E6E2DA', '#D9C7A7', '#F2F2F0', '#B9BCBE')), (2, 1))
def wall_clock(rim, rfin, face):
    y, z = 1.9, WALL
    p = [cyl([0.36, 0.05, 0.36], [0, y, z + 0.025], rim, rfin, rotate=[90, 0, 0], segments=32),
         cyl([0.32, 0.004, 0.32], [0, y, z + 0.051], face, 'satin', rotate=[90, 0, 0], segments=32)]
    for k in range(12):
        a = math.radians(k * 30)
        long_ = k % 3 == 0
        p.append(box([0.012, 0.04 if long_ else 0.022, 0.003], [math.sin(a) * (0.125 if long_ else 0.132), y + math.cos(a) * (0.125 if long_ else 0.132), z + 0.055],
                     '#2A2C30', 'satin', radius=0.001, rotate=[0, 0, -k * 30]))
    for length, w, ang, c, dz in ((0.08, 0.014, -60, '#2A2C30', 0.057), (0.12, 0.01, 60, '#2A2C30', 0.059), (0.13, 0.004, 160, '#D9483B', 0.061)):
        a = math.radians(ang)
        p.append(box([w, length, 0.003], [math.sin(a) * length / 2 * 0.8, y + math.cos(a) * length / 2 * 0.8, z + dz], c, 'satin', radius=0.001, rotate=[0, 0, -ang]))
    p.append(cyl([0.02, 0.008, 0.02], [0, y, z + 0.064], '#2A2C30', 'satin', rotate=[90, 0, 0], segments=10))
    return p


addp('wallClock', wall_clock(STEEL, 'metal', '#FBFAF6'))

# Grandfather clock: a walnut case with a glazed trunk door (pendulum behind), a dial in the hood.
GF = '#6B4630'
z = -0.26
p = [box([0.58, 0.42, 0.4], [0, 0.21, z], GF, 'wood', radius=0.01),
     box([0.6, 0.05, 0.42], [0, 0.025, z], '#4E3222', 'wood', radius=0.008),
     box([0.6, 0.04, 0.42], [0, 0.44, z], GF, 'wood', radius=0.008),
     box([0.46, 1.08, 0.32], [0, 0.98, z - 0.02], GF, 'wood', radius=0.008),                                 # trunk
     box([0.36, 0.86, 0.02], [0, 0.98, z + 0.15], GF, 'wood', radius=0.008),
     box([0.3, 0.78, 0.004], [0, 0.98, z + 0.161], '#2A1C14', 'wood', radius=0.004),
     cyl([0.012, 0.52, 0.012], [0, 1.08, z + 0.166], BRASS, 'metal', segments=6),                            # pendulum
     cyl([0.13, 0.012, 0.13], [0, 0.8, z + 0.168], BRASS, 'metal', rotate=[90, 0, 0], segments=20),
     box([0.29, 0.77, 0.006], [0, 0.98, z + 0.176], '#DCE6E8', 'glass', radius=0.004),
     box([0.6, 0.04, 0.4], [0, 1.53, z], GF, 'wood', radius=0.008),
     box([0.54, 0.46, 0.36], [0, 1.77, z - 0.005], GF, 'wood', radius=0.01),                                 # hood
     box([0.6, 0.05, 0.4], [0, 2.0, z], '#4E3222', 'wood', radius=0.01)]
p += [sph([0.06, 0.06, 0.06], [x, 2.05, z], BRASS, 'metal', segments=10) for x in (-0.25, 0.0, 0.25)]
dial = [0, 1.77, z + 0.18]
p += [cyl([0.34, 0.01, 0.34], dial, '#F4EBD8', 'gloss', rotate=[90, 0, 0], segments=28),
      torus([0.36, 0.02, 0.36], [0, 1.77, z + 0.181], BRASS, 'metal', rotate=[90, 0, 0], segments=28)]
for k in range(12):
    a = math.radians(k * 30)
    p.append(box([0.01, 0.03, 0.003], [math.sin(a) * 0.135, 1.77 + math.cos(a) * 0.135, z + 0.187], '#2A2C30', 'metal', radius=0.001, rotate=[0, 0, -k * 30]))
p += [box([0.012, 0.08, 0.003], [-0.03, 1.79, z + 0.19], '#2A2C30', 'metal', radius=0.001, rotate=[0, 0, 40]),
      box([0.008, 0.12, 0.003], [0.03, 1.81, z + 0.192], '#2A2C30', 'metal', radius=0.001, rotate=[0, 0, -30])]
addp('grandfatherClock', p)


def plinth(h, color, fin='matte', w=0.36):
    return [box([w, h, w], [0, h / 2, 0], color, fin, radius=0.01), box([w + 0.04, 0.03, w + 0.04], [0, h - 0.015, 0], color, fin, radius=0.008),
            box([w + 0.04, 0.04, w + 0.04], [0, 0.02, 0], color, fin, radius=0.008)]


TALL_VASE = [[0, -0.5], [0.36, -0.5], [0.44, -0.3], [0.5, 0.0], [0.42, 0.3], [0.3, 0.45], [0.33, 0.5], [0.27, 0.5], [0.24, 0.42], [0, 0.4]]
p = [lathe([0.18, 0.62, 0.18], [0, 0.31, 0], '#E3DDD2', 'satin', TALL_VASE, segments=24),
     rod([0, 0.6, 0], [-0.12, 1.15, 0.04], 0.012, '#8E6E52', 'wood'), rod([0, 0.6, 0], [0.1, 1.2, -0.02], 0.012, '#8E6E52', 'wood'),
     rod([0, 0.6, 0], [0.02, 1.25, 0.08], 0.012, '#8E6E52', 'wood'),
     sph([0.12, 0.08, 0.1], [-0.11, 1.12, 0.04], '#D9C7A7', 'foliage', segments=6),
     sph([0.12, 0.08, 0.1], [0.1, 1.16, -0.02], '#D9C7A7', 'foliage', segments=6),
     sph([0.1, 0.08, 0.1], [0.02, 1.22, 0.08], '#D9C7A7', 'foliage', segments=6)]
addp('vaseTall', p)

# Porcelain ginger jar: white glaze with blue bands and sprigs, a lidded top.
BLUE = '#2E4E8E'
p = [lathe([0.36, 0.5, 0.36], [0, 0.25, 0], '#F4F4F0', 'gloss',
           [[0, -0.5], [0.3, -0.5], [0.36, -0.45], [0.48, -0.2], [0.5, 0.05], [0.44, 0.3], [0.3, 0.42], [0.28, 0.5], [0, 0.5]], segments=28),
     lathe([0.21, 0.09, 0.21], [0, 0.545, 0], '#F4F4F0', 'gloss', [[0, -0.5], [0.5, -0.5], [0.46, 0.0], [0.25, 0.35], [0.1, 0.5], [0, 0.5]], segments=24),
     sph([0.05, 0.05, 0.05], [0, 0.6, 0], BLUE, 'gloss', segments=10),
     torus([0.27, 0.014, 0.27], [0, 0.03, 0], BLUE, 'gloss', segments=28),
     torus([0.33, 0.014, 0.33], [0, 0.4, 0], BLUE, 'gloss', segments=28),
     torus([0.215, 0.012, 0.215], [0, 0.505, 0], BLUE, 'gloss', segments=24)]
for k in range(5):
    a = math.radians(k * 72 + 20)
    p += [sph([0.09, 0.09, 0.02], [math.sin(a) * 0.176, 0.25, math.cos(a) * 0.176], BLUE, 'gloss', segments=10, rotate=[0, k * 72 + 20, 0]),
          sph([0.05, 0.05, 0.02], [math.sin(a + 0.4) * 0.17, 0.33, math.cos(a + 0.4) * 0.17], BLUE, 'gloss', segments=8, rotate=[0, k * 72 + 43, 0])]
addp('urnPorcelain', p)


def vase_on_plinth(stone, vase, vfin, stems, blooms):
    y = 0.86
    p = plinth(y, stone) + [lathe([0.24, 0.34, 0.24], [0, y + 0.17, 0], vase, vfin, VASE, segments=24)]
    for x, z, top in ((-0.08, 0.02, 1.48), (0.06, -0.04, 1.54), (0.02, 0.07, 1.42)):
        p += [rod([0, y + 0.3, 0], [x, top, z], 0.01, stems, 'foliage', segments=6),
              part('blob', [0.1, 0.08, 0.1], [x, top + 0.02, z], blooms, 'foliage', noise=0.25)]
    return p


addp('vasePedestal', vase_on_plinth('#E7E3DC', '#3E6E9A', 'gloss', '#6E8B4E', '#F4EFE6'))
addp('vasePedestal@modern', vase_on_plinth('#3A3D41', '#F2F0EA', 'matte', '#B9A47E', '#D9C7A7'))

# Marble bust on a plinth.
MARBLE, y = '#EDEBE6', 1.0
p = plinth(y, '#E7E3DC', w=0.4) + [
    lathe([0.18, 0.08, 0.18], [0, y + 0.04, 0], MARBLE, 'satin', [[0, -0.5], [0.5, -0.5], [0.5, -0.2], [0.3, 0.2], [0.25, 0.5], [0, 0.5]], segments=20),
    sph([0.3, 0.2, 0.18], [0, y + 0.17, 0], MARBLE, 'satin', segments=16),
    cyl([0.08, 0.12, 0.08], [0, y + 0.3, 0.0], MARBLE, 'satin', segments=12),
    sph([0.15, 0.19, 0.17], [0, y + 0.42, 0.01], MARBLE, 'satin', segments=16),
    part('blob', [0.16, 0.11, 0.17], [0, y + 0.49, -0.01], '#E4E1DA', 'satin', noise=0.15),
    box([0.025, 0.045, 0.03], [0, y + 0.42, 0.095], MARBLE, 'satin', radius=0.01, rotate=[-15, 0, 0]),
    sph([0.03, 0.05, 0.03], [-0.075, y + 0.42, 0.0], MARBLE, 'satin', segments=8),
    sph([0.03, 0.05, 0.03], [0.075, y + 0.42, 0.0], MARBLE, 'satin', segments=8)]
addp('bustMarble', p)

# Concrete cat, sitting.
CAT = '#9A9893'
p = [sph([0.28, 0.24, 0.3], [0, 0.13, -0.05], CAT, 'matte', segments=16),
     sph([0.2, 0.3, 0.18], [0, 0.25, 0.05], CAT, 'matte', segments=16),
     sph([0.19, 0.17, 0.17], [0, 0.45, 0.07], CAT, 'matte', segments=16),
     sph([0.08, 0.05, 0.05], [0, 0.42, 0.15], '#A7A5A0', 'matte', segments=10),
     sph([0.025, 0.02, 0.01], [-0.04, 0.47, 0.152], '#6E6C68', 'matte', segments=6),
     sph([0.025, 0.02, 0.01], [0.04, 0.47, 0.152], '#6E6C68', 'matte', segments=6)]
p += [cone([0.07, 0.08, 0.035], [s * 0.055, 0.56, 0.06], CAT, 'matte', segments=3, rotate=[0, 0, -s * 12]) for s in (-1, 1)]
p += [cyl([0.05, 0.22, 0.05], [s * 0.045, 0.11, 0.12], CAT, 'matte', segments=10) for s in (-1, 1)]
p += [sph([0.06, 0.035, 0.07], [s * 0.045, 0.017, 0.15], CAT, 'matte', segments=8) for s in (-1, 1)]
p += [rod([0.1, 0.03, -0.16], [0.15, 0.03, 0.03], 0.045, CAT, 'matte', segments=8),
      rod([0.15, 0.03, 0.03], [0.07, 0.03, 0.19], 0.04, CAT, 'matte', segments=8)]
addp('statueCat', p)

# Bronze horse on an oak plinth.
BRONZE, y = '#6E4F35', 0.92
p = plinth(y, OAK, 'wood', w=0.4) + [
    box([0.34, 0.02, 0.16], [0, y + 0.01, 0], BRONZE, 'metal', radius=0.006),
    part('capsule', [0.12, 0.3, 0.11], [0, y + 0.27, 0], BRONZE, 'metal', rotate=[0, 0, 90]),
    rod([0.1, y + 0.28, 0], [0.15, y + 0.41, 0], 0.07, BRONZE, 'metal', segments=10),
    rod([0.14, y + 0.42, 0], [0.21, y + 0.36, 0], 0.06, BRONZE, 'metal', segments=10),
    rod([0.08, y + 0.33, 0], [0.13, y + 0.45, 0], 0.025, '#5A3F2A', 'metal', shape='box', t2=0.015),
    rod([-0.14, y + 0.3, 0], [-0.19, y + 0.13, 0], 0.03, '#5A3F2A', 'metal', segments=6)]
p += [cone([0.025, 0.04, 0.02], [0.14 + s * 0.0, y + 0.46, s * 0.02], BRONZE, 'metal', segments=4) for s in (-1, 1)]
p += [rod([x, y + 0.24, z], [x + dx, y + 0.02, z], 0.028, BRONZE, 'metal', segments=8)
      for x, dx in ((0.1, 0.02), (-0.1, -0.01)) for z in (-0.035, 0.035)]
addp('statueHorse', p)

# Rugs: flat layers (each a little higher than the one under it), lying from 0.004 to ~0.02.
def rug_h(k):
    top = 0.018 + 0.0015 * k
    return top - 0.004, (top + 0.004) / 2


def rbox(sx, sz, x, z, c, k, ry=0):
    h, y = rug_h(k)
    return box([sx, h, sz], [x, y, z], c, 'fabric', radius=0.003, rotate=[0, ry, 0])


def rdisc(d, c, k, x=0.0, z=0.0):
    h, y = rug_h(k)
    return cyl([d, h, d], [x, y, z], c, 'fabric', segments=48)


def rring(d, w, c, k):
    h, y = rug_h(k)
    r0 = 0.5 - w / d
    return lathe([d, h, d], [0, y, 0], c, 'fabric', [[r0, -0.5], [0.5, -0.5], [0.5, 0.5], [r0, 0.5]], segments=48)


def rframe(sx, sz, t, c, k):
    """A border line `t` wide round a sx x sz rectangle."""
    return [rbox(sx, t, 0, s * (sz - t) / 2, c, k) for s in (-1, 1)] + [rbox(t, sz - 2 * t, s * (sx - t) / 2, 0, c, k) for s in (-1, 1)]


def braid(d, colors):
    """A braided round rug: concentric ridged coils."""
    p, n = [], 9
    for i in range(n):
        r0, r1 = 0.5 * i / n, 0.5 * (i + 1) / n
        m = (r0 + r1) / 2
        p.append(lathe([d, 0.02, d], [0, 0.012, 0], colors[i % len(colors)], 'fabric',
                       [[r1, -0.5], [r1 - 0.004, 0.25], [m, 0.5], [r0 + 0.004, 0.25], [r0, -0.5]], segments=48))
    return p


def fringe(sx, z_ends, c):
    """Tassels along the short ends (at z = ±z_ends) of a rug sx wide."""
    n = int(sx / 0.08)
    return [rbox(0.025, 0.06, -sx / 2 + 0.04 + i * (sx - 0.08) / (n - 1), s * (z_ends + 0.03), c, 0) for s in (-1, 1) for i in range(n)]


def rug_rect(sx, sz, a, b, style):
    if style == 'plain':                                                       # classic: a darker border
        return [rbox(sx, sz, 0, 0, b, 0), rbox(sx - 0.24, sz - 0.24, 0, 0, a, 1)] + rframe(sx - 0.44, sz - 0.44, 0.03, b, 2)
    if style == 'stripes':                                                     # modern: bands across
        p = [rbox(sx, sz, 0, 0, a, 0)]
        long_x = sx >= sz
        span = sx if long_x else sz
        for f, w, c in ((-0.32, 0.18, b), (-0.18, 0.04, '#3A3D41'), (0.2, 0.3, b), (0.4, 0.04, '#F2F2F0')):
            p.append(rbox(w * span / 2.85, sz, f * span, 0, c, 1) if long_x else rbox(sx, w * span / 2.85, 0, f * span, c, 1))
        return p
    if style == 'medallion':                                                   # cozy: border, diamonds, fringe
        p = [rbox(sx, sz, 0, 0, b, 0), rbox(sx - 0.2, sz - 0.2, 0, 0, a, 1)] + rframe(sx - 0.3, sz - 0.3, 0.035, MUSTARD, 2)
        long_x = sx >= sz
        ds = min(sx, sz) * 0.42
        spots = [(0, 0)] + ([(s * sx * 0.3, 0) for s in (-1, 1)] if long_x else [(0, s * sz * 0.3) for s in (-1, 1)])
        for i, (x, z) in enumerate(spots):
            d = ds if i == 0 else ds * 0.55
            p += [rbox(d, d, x, z, MUSTARD, 3, 45), rbox(d * 0.55, d * 0.55, x, z, CREAM, 4, 45)]
        if long_x:
            p += [rbox(0.06, 0.025, s * (sx / 2 + 0.03), -sz / 2 + 0.06 + i * (sz - 0.12) / 11, CREAM, 0) for s in (-1, 1) for i in range(12)]
        else:
            p += fringe(sx, sz / 2, CREAM)
        return p
    return [rbox(sx, sz, 0, 0, a, 0)] + rframe(sx - 0.16, sz - 0.16, 0.02, b, 1)      # minimal: a thin line


def rug_round(d, a, b, style):
    if style == 'plain':
        return [rdisc(d, b, 0), rdisc(d - 0.24, a, 1), rring(d - 0.44, 0.03, b, 2)]
    if style == 'stripes':
        return [rdisc(d, a, 0), rdisc(d * 0.42, b, 1, x=0.22, z=-0.16), rring(d - 0.2, 0.025, '#3A3D41', 2), rdisc(d * 0.16, '#F2F2F0', 3, x=-0.3, z=0.28)]
    if style == 'medallion':
        return braid(d, (a, b, CLAY, a, '#A0603E', MUSTARD))
    return [rdisc(d, a, 0), rring(d - 0.16, 0.02, b, 1)]


RUGS = {'': (SAGE, '#7E9271', 'plain'), '@modern': ('#C9CBCC', '#8E979E', 'stripes'), '@cozy': (RUST, '#8F4F33', 'medallion'),
        '@minimal': ('#F2F0EA', '#D9CFBF', 'line')}
for sfx, (a, b, style) in RUGS.items():
    addp('rugRound' + sfx, rug_round(1.9, a, b, style), (2, 2))
    addp('rugLarge' + sfx, rug_rect(2.85, 1.85, a, b, style), (3, 2))
    addp('rugRunner' + sfx, rug_rect(0.8, 2.8, a, b, style), (1, 3))


def rug_pattern(ground, border, motif, accent):
    y = 0.02
    p = [box([2.86, 0.016, 1.86], [0, y, 0], border, 'fabric', radius=0.004),
         box([2.6, 0.017, 1.6], [0, y + 0.001, 0], ground, 'fabric', radius=0.004),
         box([2.4, 0.0175, 0.04], [0, y + 0.0015, -0.7], accent, 'fabric', radius=0.002),
         box([2.4, 0.0175, 0.04], [0, y + 0.0015, 0.7], accent, 'fabric', radius=0.002),
         box([0.04, 0.0175, 1.4], [-1.2, y + 0.0015, 0], accent, 'fabric', radius=0.002),
         box([0.04, 0.0175, 1.4], [1.2, y + 0.0015, 0], accent, 'fabric', radius=0.002),
         box([0.7, 0.018, 0.7], [0, y + 0.002, 0], motif, 'fabric', radius=0.004, rotate=[0, 45, 0]),
         box([0.4, 0.0185, 0.4], [0, y + 0.0025, 0], accent, 'fabric', radius=0.004, rotate=[0, 45, 0])]
    for x in (-0.85, 0.85):
        p.append(box([0.36, 0.018, 0.36], [x, y + 0.002, 0], motif, 'fabric', radius=0.004, rotate=[0, 45, 0]))
    for x in (-1.35, -1.2, -1.05, 1.05, 1.2, 1.35):
        p.append(box([0.03, 0.006, 0.1], [x * 1.04, 0.013, 0.96], border, 'fabric', radius=0.002))   # fringe hint
    return [dict(q, at=[q['at'][0], q['at'][1] - 0.008, q['at'][2]]) for q in p]


add('rugPattern', placeholder(rug_pattern('#B5653F', '#3E5470', '#E9C46A', '#F2E6CC')), (3, 2))
add('rugPattern@modern', placeholder(rug_pattern('#9AA4AB', '#3A3D41', '#E8E6E1', '#C08A52')), (3, 2))
add('rugPattern@minimal', placeholder(rug_pattern('#ECE7DD', '#D9C7A7', '#F2F0EA', '#B9A48C')), (3, 2))


def curtains(color, rod_col, w=0.86, top=2.45, bottom=1.0):
    z = -0.43
    h = top - bottom
    p = [cyl([0.025, w + 0.04, 0.025], [0, top + 0.05, z + 0.08], rod_col, 'metal', rotate=[0, 0, 90]),
         sph([0.05, 0.05, 0.05], [-(w / 2 + 0.03), top + 0.05, z + 0.08], rod_col, 'metal', segments=8),
         sph([0.05, 0.05, 0.05], [w / 2 + 0.03, top + 0.05, z + 0.08], rod_col, 'metal', segments=8)]
    for s in (-1, 1):                                                                      # two gathered panels, pleated
        for i in range(4):
            x = s * (w / 2 - 0.04 - i * 0.055)
            p.append(cyl([0.07, h, 0.06], [x, bottom + h / 2, z + 0.08 + (i % 2) * 0.01], color, 'fabric', segments=10))
    p += [box([0.24, 0.04, 0.03], [s * (w / 2 - 0.12), bottom + h * 0.42, z + 0.12], color, 'fabric', radius=0.015) for s in (-1, 1)]
    return placeholder(p)


add('curtains', curtains(LINEN, BLACKMETAL))
add('curtains@modern', curtains('#B9BCBE', BLACKMETAL))
add('curtains@cozy', curtains('#C98F6B', BRASS))
add('curtains@minimal', curtains('#FAF8F3', '#E3CFA8'))
add('curtainsLong', curtains(LINEN, BLACKMETAL, w=1.8, top=2.55, bottom=0.04), (2, 1))
add('curtainsLong@modern', curtains('#8E979E', BLACKMETAL, w=1.8, top=2.55, bottom=0.04), (2, 1))
add('curtainsLong@cozy', curtains('#8F4F33', BRASS, w=1.8, top=2.55, bottom=0.04), (2, 1))
add('curtainsLong@minimal', curtains('#FAF8F3', '#E3CFA8', w=1.8, top=2.55, bottom=0.04), (2, 1))


def wall_shelf(wood, fin='wood'):
    z = -0.43
    p = [box([0.9, 0.04, 0.24], [0, 1.4, z + 0.12], wood, fin, radius=0.006),
         box([0.9, 0.04, 0.24], [0, 1.78, z + 0.12], wood, fin, radius=0.006)]
    p += books(-0.4, 0.05, 1.42, z + 0.11, 0.18, 2)
    p += [cyl([0.12, 0.12, 0.12], [0.25, 1.48, z + 0.12], '#E8E6E1', 'gloss', taper=0.8),
          sph([0.2, 0.16, 0.18], [0.25, 1.6, z + 0.12], FOLIAGE, 'foliage', segments=8),
          box([0.18, 0.24, 0.02], [-0.25, 1.92, z + 0.06], '#F2F0EA', 'satin', radius=0.006),
          box([0.14, 0.2, 0.004], [-0.25, 1.92, z + 0.072], CLAY, 'matte', radius=0.004),
          cyl([0.1, 0.16, 0.1], [0.05, 1.88, z + 0.12], CREAM, 'gloss', taper=0.7),
          sph([0.12, 0.12, 0.12], [0.3, 1.86, z + 0.12], '#C9B79C', 'matte', segments=10)]
    return placeholder(p)


add('wallShelf', wall_shelf(OAK))
add('wallShelf@modern', wall_shelf('#2F3236', 'satin'))
add('wallShelf@cozy', wall_shelf(WALNUT))
add('wallShelf@minimal', wall_shelf(BIRCH))
# Standing coat rack: a turned oak pole on four feet, brass hooks, a coat, a scarf and a hat.
p = [cyl([0.05, 1.68, 0.05], [0, 0.86, 0], OAK, 'wood', taper=0.75, segments=12),
     sph([0.075, 0.075, 0.075], [0, 1.74, 0], OAK, 'wood', segments=12),
     cyl([0.08, 0.06, 0.08], [0, 0.2, 0], OAK, 'wood', segments=12)]
for k in range(4):
    a = math.radians(45 + k * 90)
    p += [rod([0, 0.2, 0], [math.cos(a) * 0.22, 0.02, math.sin(a) * 0.22], 0.035, OAK, 'wood', segments=8),
          sph([0.05, 0.03, 0.05], [math.cos(a) * 0.22, 0.015, math.sin(a) * 0.22], OAKD, 'wood', segments=8)]
for k, y in enumerate((1.6, 1.6, 1.6, 1.6, 1.42, 1.42)):
    a = math.radians(k * 90 + (45 if k > 3 else 0) + (180 if k == 5 else 0))
    tip = [math.cos(a) * 0.13, y + 0.08, math.sin(a) * 0.13]
    p += [rod([0, y, 0], tip, 0.014, BRASS, 'metal', segments=6), sph([0.026, 0.026, 0.026], tip, BRASS, 'metal', segments=8)]
p += [box([0.3, 0.62, 0.12], [0.1, 1.28, 0.06], '#3E5470', 'fabric', radius=0.05, rotate=[0, 30, 0]),        # coat
      box([0.08, 0.6, 0.02], [-0.11, 1.22, -0.06], RUST, 'fabric', radius=0.01, rotate=[0, -40, 4]),         # scarf
      cyl([0.24, 0.012, 0.24], [-0.1, 1.67, -0.06], '#C9B79C', 'fabric', segments=18),                        # hat
      cyl([0.15, 0.1, 0.15], [-0.1, 1.72, -0.06], '#C9B79C', 'fabric', taper=0.85, segments=16)]
addp('coatRack', p)


def hanging_plant(pot, cord):
    return placeholder([
        cyl([0.012, 1.0, 0.012], [0, 2.3, 0], cord, 'matte'),
        rod([0, 1.8, 0], [-0.13, 1.42, 0], 0.01, cord, 'matte'), rod([0, 1.8, 0], [0.13, 1.42, 0], 0.01, cord, 'matte'),
        rod([0, 1.8, 0], [0, 1.42, 0.13], 0.01, cord, 'matte'),
        part('lathe', [0.32, 0.24, 0.32], [0, 1.32, 0], pot, 'satin', profile=[[0, -0.5], [0.36, -0.5], [0.5, 0.2], [0.5, 0.5], [0.44, 0.5], [0, 0.0]]),
        part('blob', [0.5, 0.36, 0.5], [0, 1.5, 0], FOLIAGE, 'foliage', noise=0.3),
        part('blob', [0.18, 0.5, 0.2], [0.18, 1.18, 0.08], '#6E9A55', 'foliage', noise=0.35),        # trailing vines
        part('blob', [0.16, 0.42, 0.18], [-0.16, 1.2, -0.06], '#6E9A55', 'foliage', noise=0.35),
    ])


add('hangingPlant', hanging_plant('#F2F0EA', '#C9B79C'))
add('hangingPlant@modern', hanging_plant('#3A3D41', BLACKMETAL))
add('hangingPlant@cozy', hanging_plant(CLAY, '#C9B79C'))

# =============================================================================================
# Lighting
# =============================================================================================
DOME = [[0.5, -0.5], [0.47, 0.0], [0.36, 0.3], [0.14, 0.47], [0, 0.5], [0, 0.42], [0.13, 0.4], [0.33, 0.24], [0.44, -0.06], [0.47, -0.5]]


def pendant_dome(shade, sfin, metal, y0, h, d, bulb=BULB, bulb_size=0.08, inner=None):
    """A dome shade hanging on a cord from the ceiling, its rim at y0."""
    p = [cyl([0.12, 0.03, 0.12], [0, CEILING - 0.015, 0], metal, 'metal', segments=16),
         cyl([0.01, CEILING - (y0 + h) - 0.02, 0.01], [0, (CEILING + y0 + h) / 2 - 0.01, 0], '#2A2C30', 'metal', segments=6),
         cyl([0.06, 0.05, 0.06], [0, y0 + h + 0.01, 0], metal, 'metal', segments=12),
         lathe([d, h, d], [0, y0 + h / 2, 0], shade, sfin, DOME, segments=28),
         sph([bulb_size, bulb_size * 1.2, bulb_size], [0, y0 + 0.02, 0], bulb, 'gloss', segments=12)]
    if inner:
        p.append(lathe([d * 0.9, h * 0.85, d * 0.9], [0, y0 + h * 0.45, 0], inner, 'satin', DOME, segments=28))
    return p


addp('ceilingPendant', pendant_dome('#F2F0EA', 'satin', BRASS, 1.86, 0.24, 0.43))


def ceiling_box(frame, ffin, shade):
    """A square fabric lantern on a rod (bulb at 2.3)."""
    y0, h, w = 2.21, 0.3, 0.34
    p = [box([0.12, 0.03, 0.12], [0, CEILING - 0.015, 0], frame, ffin, radius=0.006),
         cyl([0.02, CEILING - y0 - h - 0.01, 0.02], [0, (CEILING + y0 + h) / 2, 0], frame, ffin, segments=8),
         box([w, 0.012, w], [0, y0 + h - 0.006, 0], frame, ffin, radius=0.003),
         sph([0.1, 0.12, 0.1], [0, 2.32, 0], BULB, 'gloss', segments=12),
         box([w - 0.02, 0.004, w - 0.02], [0, y0 + 0.004, 0], BULB, 'gloss', radius=0.002)]
    p += [box([w, h, 0.008], [0, y0 + h / 2, s * (w / 2 - 0.004)], shade, 'satin', radius=0.002) for s in (-1, 1)]
    p += [box([0.008, h, w], [s * (w / 2 - 0.004), y0 + h / 2, 0], shade, 'satin', radius=0.002) for s in (-1, 1)]
    p += [box([0.016, h + 0.01, 0.016], [sx * w / 2, y0 + h / 2, sz * w / 2], frame, ffin, radius=0.004) for sx in (-1, 1) for sz in (-1, 1)]
    return p


addp('ceilingLight', ceiling_box(BLACKMETAL, 'metal', '#FBF4E4'))
addp('ceilingLight@minimal', ceiling_box(BIRCH, 'wood', '#FBF4E4'))


def ring_chandelier(metal):
    y = 2.05
    p = [cyl([0.16, 0.03, 0.16], [0, CEILING - 0.015, 0], metal, 'metal'),
         cyl([0.012, CEILING - y - 0.03, 0.012], [0, (CEILING + y) / 2, 0], metal, 'metal'),
         torus([0.9, 0.03, 0.9], [0, y, 0], metal, 'metal'),
         torus([0.56, 0.025, 0.56], [0, y + 0.16, 0], metal, 'metal')]
    for i in range(3):
        a = math.radians(i * 120 + 30)
        p.append(rod([0, y + 0.3, 0], [math.cos(a) * 0.44, y, math.sin(a) * 0.44], 0.008, metal, 'metal'))
    for i in range(10):
        a = math.radians(i * 36)
        p.append(sph([0.07, 0.07, 0.07], [math.cos(a) * 0.43, y + 0.05, math.sin(a) * 0.43], BULB, 'gloss', segments=8))
    for i in range(6):
        a = math.radians(i * 60 + 15)
        p.append(sph([0.06, 0.06, 0.06], [math.cos(a) * 0.27, y + 0.2, math.sin(a) * 0.27], BULB, 'gloss', segments=8))
    return placeholder(p)


add('chandelier', ring_chandelier(BRASS))
add('chandelier@modern', ring_chandelier(BLACKMETAL))
addp('pendantIndustrial', pendant_dome('#2F4A3A', 'metal', '#2A2C30', 1.95, 0.24, 0.38, bulb='#FFD99A', bulb_size=0.09, inner='#F2F0EA'))


def ceiling_fan(body, bfin, blade):
    """Five pitched blades round a motor, a frosted light bowl under it (hangs 2.44..2.8)."""
    y = 2.56
    p = [cyl([0.16, 0.05, 0.16], [0, CEILING - 0.025, 0], body, bfin, segments=18),
         cyl([0.03, 0.14, 0.03], [0, 2.69, 0], body, bfin, segments=8),
         cyl([0.26, 0.12, 0.26], [0, y, 0], body, bfin, taper=0.75, segments=24),
         cyl([0.2, 0.04, 0.2], [0, 2.48, 0], body, bfin, segments=20),
         sph([0.18, 0.09, 0.18], [0, 2.46, 0], BULB, 'gloss', segments=16)]
    for k in range(5):
        a = k * 72
        r = math.radians(a)
        p += [box([0.46, 0.012, 0.13], [math.cos(r) * 0.36, y - 0.02, -math.sin(r) * 0.36], blade, 'wood', radius=0.004, rotate=[10, a, 0]),
              box([0.14, 0.015, 0.035], [math.cos(r) * 0.15, y - 0.012, -math.sin(r) * 0.15], body, bfin, radius=0.005, rotate=[0, a, 0])]
    return p


addp('ceilingFan', ceiling_fan('#F2F2F0', 'satin', OAK))
addp('ceilingFan@modern', ceiling_fan(BLACKMETAL, 'metal', OAKD))
def sconce(metal, shade):
    z = -0.43
    return placeholder([
        cyl([0.12, 0.02, 0.12], [0, 1.7, z + 0.01], metal, 'metal', rotate=[90, 0, 0]),          # wall plate
        rod([0, 1.7, z + 0.02], [0, 1.74, z + 0.16], 0.02, metal, 'metal'),
        cyl([0.2, 0.18, 0.2], [0, 1.8, z + 0.17], shade, 'fabric', taper=0.75, segments=20),       # shade
        sph([0.06, 0.06, 0.06], [0, 1.76, z + 0.17], BULB, 'gloss', segments=8)])


add('wallSconce', sconce(BRASS, '#F4EBD8'))
add('wallSconce@modern', sconce(BLACKMETAL, '#FAF8F3'))
# Swing-arm wall light: a wall plate, a jointed arm, a black cone shade tipped down.
z = WALL
p = [cyl([0.11, 0.02, 0.11], [0, 1.55, z + 0.01], BLACKMETAL, 'metal', rotate=[90, 0, 0], segments=16),
     rod([0, 1.55, z + 0.02], [0, 1.6, z + 0.18], 0.018, BLACKMETAL, 'metal', segments=8),
     sph([0.035, 0.035, 0.035], [0, 1.6, z + 0.18], BRASS, 'metal', segments=8),
     rod([0, 1.6, z + 0.18], [0, 1.78, z + 0.25], 0.016, BLACKMETAL, 'metal', segments=8),
     lathe([0.2, 0.16, 0.2], [0, 1.72, z + 0.28], BLACKMETAL, 'metal',
           [[0.5, -0.5], [0.2, 0.4], [0.1, 0.5], [0, 0.5], [0, 0.42], [0.09, 0.42], [0.46, -0.5]], segments=20, rotate=[-35, 0, 0]),
     sph([0.07, 0.08, 0.07], [0, 1.68, z + 0.3], BULB, 'gloss', segments=10)]
addp('wallLampArm', p)

# Outdoor bulkhead: an oval frosted glass in a black cage under a hood.
p = [box([0.2, 0.3, 0.03], [0, 2.01, z + 0.015], '#2A2C30', 'metal', radius=0.012),
     cyl([0.16, 0.07, 0.26], [0, 2.01, z + 0.065], '#F4EEDC', 'glass', rotate=[90, 0, 0], segments=24),
     sph([0.08, 0.1, 0.06], [0, 2.01, z + 0.06], BULB, 'gloss', segments=10),
     box([0.22, 0.025, 0.11], [0, 2.175, z + 0.055], '#2A2C30', 'metal', radius=0.008),
     box([0.012, 0.27, 0.012], [0, 2.01, z + 0.104], '#2A2C30', 'metal', radius=0.004)]
p += [box([0.17, 0.012, 0.012], [0, y, z + 0.102], '#2A2C30', 'metal', radius=0.004) for y in (1.94, 2.01, 2.08)]
addp('wallLightOutdoor', p)


def floor_lamp(style):
    """A floor lamp with its bulb at ~1.5: `square` (black stand, column shade), `tripod` (wooden legs, drum),
    `paper` (birch pole, tall paper cylinder)."""
    if style == 'square':
        return [box([0.28, 0.03, 0.28], [0, 0.015, 0], BLACKMETAL, 'metal', radius=0.008),
                cyl([0.022, 1.02, 0.022], [0, 0.53, 0], BLACKMETAL, 'metal', segments=8),
                cyl([0.38, 0.74, 0.38], [0, 1.23, 0], '#F4EBD8', 'fabric', taper=1.0, segments=4, rotate=[0, 45, 0]),
                sph([0.09, 0.11, 0.09], [0, 1.45, 0], BULB, 'gloss', segments=10),
                cyl([0.36, 0.004, 0.36], [0, 0.865, 0], BULB, 'gloss', segments=4, rotate=[0, 45, 0])]
    if style == 'tripod':
        p = [rod([math.cos(math.radians(a)) * 0.16, 0, math.sin(math.radians(a)) * 0.16], [0, 1.2, 0], 0.03, HONEY, 'wood', segments=8)
             for a in (90, 210, 330)]
        return p + [cyl([0.03, 0.08, 0.03], [0, 1.23, 0], BRASS, 'metal', segments=8),
                    cyl([0.4, 0.3, 0.4], [0, 1.42, 0], '#EFD9B4', 'fabric', segments=24),
                    cyl([0.37, 0.004, 0.37], [0, 1.275, 0], BULB, 'gloss', segments=20),
                    sph([0.09, 0.11, 0.09], [0, 1.4, 0], BULB, 'gloss', segments=10)]
    return [cyl([0.28, 0.04, 0.28], [0, 0.02, 0], BIRCH, 'wood', segments=24),
            cyl([0.03, 1.0, 0.03], [0, 0.52, 0], BIRCH, 'wood', segments=10),
            cyl([0.3, 0.6, 0.3], [0, 1.3, 0], '#FAF8F3', 'fabric', segments=24),
            cyl([0.27, 0.004, 0.27], [0, 1.003, 0], BULB, 'gloss', segments=20)]


addp('floorLampSimple', floor_lamp('square'))
addp('floorLampSimple@cozy', floor_lamp('tripod'))
addp('floorLampSimple@minimal', floor_lamp('paper'))


def lamp_table(wood, fin, base, bfin, stem, sfin, shade, style):
    """A small side table (top at 0.5, 0.66 x 0.42) with a lamp on it: `classic` / `turned` tables with a round
    lamp and drum shade, `modern` / `minimal` ones with a square lamp."""
    p = [box([0.66, 0.03, 0.42], [0, 0.485, 0], wood, fin, radius=0.01)]
    if style == 'modern':
        p += [box([0.025, 0.47, 0.025], [x, 0.235, z], stem, sfin, radius=0.006) for x in (-0.3, 0.3) for z in (-0.18, 0.18)]
        p += [box([0.6, 0.02, 0.025], [0, 0.12, s * 0.18], stem, sfin, radius=0.006) for s in (-1, 1)]
    elif style == 'minimal':
        p += [cyl([0.035, 0.47, 0.035], [x, 0.235, z], wood, fin, segments=10) for x in (-0.29, 0.29) for z in (-0.17, 0.17)]
        p += [box([0.6, 0.02, 0.36], [0, 0.2, 0], wood, fin, radius=0.006)]
    else:
        p += [cyl([0.045, 0.47, 0.045], [x, 0.235, z], wood, fin, taper=0.65 if style == 'classic' else 0.85, segments=12)
              for x in (-0.28, 0.28) for z in (-0.16, 0.16)]
        p += [box([0.58, 0.02, 0.34], [0, 0.14, 0], wood, fin, radius=0.006)]
        if style == 'turned':
            p += [sph([0.07, 0.05, 0.07], [x, 0.3, z], wood, fin, segments=10) for x in (-0.28, 0.28) for z in (-0.16, 0.16)]
    x = -0.12
    square = style in ('modern', 'minimal')
    p += [box([0.13, 0.18, 0.13], [x, 0.59, 0], base, bfin, radius=0.012) if square else lathe([0.17, 0.2, 0.17], [x, 0.6, 0], base, bfin, VASE, segments=18),
          cyl([0.016, 0.1, 0.016], [x, 0.74, 0], stem, sfin, segments=8),
          sph([0.07, 0.08, 0.07], [x, 0.81, 0], BULB, 'gloss', segments=10)]
    if square:
        p.append(cyl([0.32, 0.2, 0.32], [x, 0.82, 0], shade, 'fabric', taper=0.8, segments=4, rotate=[0, 45, 0]))
    else:
        p += [cyl([0.3, 0.2, 0.3], [x, 0.82, 0], shade, 'fabric', taper=0.85, segments=24),
              cyl([0.27, 0.004, 0.27], [x, 0.723, 0], BULB, 'gloss', segments=20)]
    return p


addp('tableLamp', lamp_table(OAK, 'wood', '#E8E2D6', 'gloss', BRASS, 'metal', '#F4EBD8', 'classic'))
addp('tableLamp@modern', lamp_table('#F1F0EC', 'gloss', '#F4F4F2', 'gloss', BLACKMETAL, 'metal', '#FAF8F3', 'modern'))
addp('tableLamp@cozy', lamp_table(WALNUT, 'wood', CLAY, 'gloss', BRASS, 'metal', '#EFD9B4', 'turned'))
addp('tableLamp@minimal', lamp_table(BIRCH, 'wood', BIRCH, 'wood', '#F2F2F0', 'satin', '#FAF8F3', 'minimal'))

# =============================================================================================
# Outdoor
# =============================================================================================
def grill(body):
    p = [part('lathe', [0.6, 0.36, 0.6], [0, 0.82, 0], body, 'gloss',
              profile=[[0, -0.5], [0.3, -0.45], [0.48, -0.1], [0.5, 0.5], [0, 0.5]], segments=24),       # bowl
         part('lathe', [0.6, 0.3, 0.6], [0, 1.1, -0.06], body, 'gloss',
              profile=[[0.5, -0.5], [0.46, 0.1], [0.3, 0.42], [0, 0.5], [0, -0.5]], segments=24, rotate=[-35, 0, 0]),  # lid, open
         cyl([0.56, 0.01, 0.56], [0, 0.995, 0], '#3A3D41', 'metal', segments=24),
         box([0.12, 0.03, 0.06], [0, 1.0, 0.25], '#2A2C30', 'satin', radius=0.01)]
    for a in (90, 210, 330):
        r = math.radians(a)
        p.append(rod([math.cos(r) * 0.12, 0.66, math.sin(r) * 0.12], [math.cos(r) * 0.32, 0.0, math.sin(r) * 0.32], 0.04, '#3A3D41', 'metal'))
    p += [cyl([0.16, 0.06, 0.16], [0.18, 0.08, 0.18], '#2A2C30', 'satin', rotate=[90, 0, 0]),              # wheels
          sph([0.1, 0.06, 0.07], [-0.1, 1.02, 0.05], '#8E3F2E', 'satin', segments=8),                       # sausages / burgers
          sph([0.1, 0.06, 0.07], [0.08, 1.02, -0.05], '#8E3F2E', 'satin', segments=8)]
    return p


add('grill', placeholder(grill('#2A2C30')))
add('grill@cozy', placeholder(grill('#8E2A2A')))


def grill_station(steel, wood):
    p = [box([1.6, 0.86, 0.6], [0, 0.45, -0.14], steel, 'metal', radius=0.015),
         box([1.66, 0.04, 0.66], [0, 0.9, -0.12], '#3A3D41', 'satin', radius=0.01),
         box([0.9, 0.3, 0.5], [0.0, 1.06, -0.14], steel, 'metal', radius=0.04),                    # grill hood (closed)
         box([0.7, 0.03, 0.03], [0, 1.0, 0.13], CHROME, 'chrome', radius=0.01),
         box([0.12, 0.08, 0.04], [0, 1.2, 0.12], '#9AA4AB', 'gloss', radius=0.02),                  # thermometer
         box([0.36, 0.02, 0.4], [-0.62, 0.93, -0.12], wood, 'wood', radius=0.006),                  # side boards
         box([0.36, 0.02, 0.4], [0.62, 0.93, -0.12], wood, 'wood', radius=0.006)]
    for x in (-0.4, 0.4):
        p.append(box([0.64, 0.6, 0.02], [x, 0.42, 0.165], steel, 'metal', radius=0.01))
    for x in (-0.3, -0.1, 0.1, 0.3):
        p.append(cyl([0.05, 0.03, 0.05], [x, 0.82, 0.18], '#2A2C30', 'satin', rotate=[90, 0, 0]))  # knobs
    p += [box([0.24, 0.12, 0.16], [-0.62, 1.0, -0.12], '#E9E6E0', 'gloss', radius=0.02),            # tray of food
          cyl([0.14, 0.24, 0.14], [0.62, 1.06, -0.15], '#6FA6D9', 'gloss')]
    return p


add('grillStation', placeholder(grill_station(STEEL, OAK)), (2, 1))
def folding_chair(slat, frame):
    """A slatted folding garden chair facing +Z, seat centred on the origin (seat top 0.44)."""
    p = [box([0.42, 0.02, 0.085], [0, 0.43, z], slat, 'wood', radius=0.006) for z in (-0.14, -0.045, 0.05, 0.145)]
    p += [box([0.4, 0.07, 0.02], [0, y, -0.2 - (y - 0.6) * 0.18], slat, 'wood', radius=0.008, rotate=[-10, 0, 0]) for y in (0.62, 0.74, 0.86)]
    for x in (-0.21, 0.21):
        p += [rod([x, 0, 0.2], [x, 0.92, -0.25], 0.024, frame, 'metal', segments=8),                        # front leg into back post
              rod([x, 0, -0.2], [x, 0.42, 0.14], 0.024, frame, 'metal', segments=8),
              box([0.022, 0.022, 0.36], [x, 0.415, -0.0], frame, 'metal', radius=0.006)]
    p.append(box([0.42, 0.02, 0.02], [0, 0.12, 0.13], frame, 'metal', radius=0.006))
    return p


def shifted(parts, dz):
    return [dict(q, at=[q['at'][0], q['at'][1], round(q['at'][2] + dz, 4)]) for q in parts]


# Patio set (1x2): a slatted folding table (top 0.72 at z 0.3), the seat at z -0.62 facing it, a chair across.
TK, FR = '#B5784A', '#2A2C30'
p = [box([0.6, 0.02, 0.135], [0, 0.71, 0.3 + z], TK, 'wood', radius=0.006) for z in (-0.225, -0.075, 0.075, 0.225)]
p += [box([0.58, 0.03, 0.03], [0, 0.69, 0.3 + s * 0.27], FR, 'metal', radius=0.006) for s in (-1, 1)]
for z in (0.07, 0.53):
    p += [rod([-0.26, 0, z], [0.26, 0.69, z], 0.025, FR, 'metal', segments=8), rod([0.26, 0, z], [-0.26, 0.69, z], 0.025, FR, 'metal', segments=8)]
p += [cyl([0.08, 0.09, 0.08], [0.12, 0.765, 0.36], '#F2F1EE', 'gloss', segments=14),
      cyl([0.1, 0.12, 0.1], [-0.14, 0.78, 0.22], '#9DC6CF', 'gloss', segments=14, taper=0.7)]
p += shifted(folding_chair(TK, FR), -0.62) + shifted(facing_back(folding_chair(TK, FR)), 0.72)
addp('patioSet', p, (1, 2))

# Picnic table (3x2): a planked top at 0.63 between two benches (0.4), A-frame legs; a cloth and a basket.
PW, PWD = '#9C7A58', '#7E5F42'
p = [box([2.5, 0.04, 0.135], [0, 0.61, z], PW, 'wood', radius=0.008) for z in (-0.29, -0.145, 0.0, 0.145, 0.29)]
for s in (-1, 1):
    p += [box([2.5, 0.04, 0.13], [0, 0.38, s * z], PW, 'wood', radius=0.008) for z in (0.6, 0.745)]
for x in (-0.95, 0.95):
    p += [box([0.07, 0.05, 0.72], [x, 0.565, 0], PWD, 'wood', radius=0.008),
          box([0.07, 0.06, 1.62], [x, 0.33, 0], PWD, 'wood', radius=0.008)]
    p += [rod([x, 0.025, s * 0.62], [x, 0.59, s * 0.14], 0.07, PWD, 'wood', shape='box', t2=0.05, radius=0.01) for s in (-1, 1)]
p += [rod([-0.92, 0.3, 0], [0, 0.585, 0], 0.05, PWD, 'wood', shape='box', radius=0.008),
      rod([0.92, 0.3, 0], [0, 0.585, 0], 0.05, PWD, 'wood', shape='box', radius=0.008),
      box([0.7, 0.004, 0.6], [-0.3, 0.632, 0], '#D9483B', 'fabric', radius=0.002)]
p += [box([0.175, 0.005, 0.15], [-0.3 + dx, 0.6345, dz], '#F4EFE6', 'fabric', radius=0.001)
      for dx, dz in ((-0.2625, -0.225), (-0.0875, -0.075), (0.0875, -0.225), (0.2625, -0.075), (-0.2625, 0.075), (-0.0875, 0.225), (0.0875, 0.075), (0.2625, 0.225))]
p += [box([0.4, 0.2, 0.28], [0.55, 0.73, 0.02], '#C9A979', 'fabric', radius=0.03),                           # basket
      box([0.42, 0.03, 0.3], [0.55, 0.845, 0.02], '#B8955F', 'fabric', radius=0.01),
      torus([0.3, 0.02, 0.24], [0.55, 0.85, 0.02], '#8E6E52', 'wood', rotate=[90, 0, 0], segments=16),
      cyl([0.08, 0.1, 0.08], [-0.45, 0.68, 0.12], '#F2F1EE', 'gloss', segments=12),
      cyl([0.08, 0.1, 0.08], [-0.15, 0.68, -0.1], '#9DC6CF', 'gloss', segments=12)]
addp('picnicTable', p, (3, 2))


def sun_lounger(frame, pad, ffin='wood'):
    p = [box([0.62, 0.06, 1.3], [0, 0.26, 0.28], frame, ffin, radius=0.01),
         box([0.56, 0.06, 1.24], [0, 0.32, 0.28], pad, 'fabric', radius=0.03)]                       # pad (top 0.35)
    p += [box([0.62, 0.06, 0.66], [0, 0.46, -0.6], frame, ffin, radius=0.01, rotate=[35, 0, 0]),
          box([0.56, 0.06, 0.62], [0, 0.51, -0.58], pad, 'fabric', radius=0.03, rotate=[35, 0, 0]),
          cushion([0.44, 0.1, 0.2], [0, 0.68, -0.8], '#FFFFFF', rotate=[35, 0, 0])]
    for x in (-0.28, 0.28):
        for z in (-0.3, 0.86):
            p.append(box([0.05, 0.24, 0.05], [x, 0.12, z], frame, ffin, radius=0.008))
    p += [box([0.4, 0.02, 0.3], [0, 0.36, 0.7], '#E9853B', 'fabric', radius=0.01)]                     # folded towel
    return p


add('sunLounger', placeholder(sun_lounger(OAK, '#E8E2D6')), (1, 2))
add('sunLounger@modern', placeholder(sun_lounger('#3A3D41', '#C9CBCC', 'metal')), (1, 2))
add('sunLounger@cozy', placeholder(sun_lounger(HONEY, '#C98F6B')), (1, 2))


def hammock_stand(frame, cloth, stripe):
    """A hammock on its own stand, lying along Z (head at -Z); the sling sags to ~0.62 m."""
    p = [box([0.1, 0.06, 1.86], [0, 0.03, 0], frame, 'wood', radius=0.01)]
    for s in (-1, 1):
        p += [rod([0, 0.03, s * 0.9], [0, 1.15, s * 0.86], 0.08, frame, 'wood', shape='box', t2=0.08, radius=0.015),
              box([0.7, 0.04, 0.08], [0, 0.02, s * 0.86], frame, 'wood', radius=0.01)]
    n = 6
    for i in range(n):
        z0, z1 = -0.7 + i * 1.4 / n, -0.7 + (i + 1) * 1.4 / n
        y0 = 0.62 + 0.25 * ((z0 / 0.7) ** 2)
        y1 = 0.62 + 0.25 * ((z1 / 0.7) ** 2)
        ang = -math.degrees(math.atan2(y1 - y0, z1 - z0))
        p.append(box([0.62, 0.03, math.hypot(z1 - z0, y1 - y0) + 0.01], [0, (y0 + y1) / 2, (z0 + z1) / 2], cloth if i % 2 else stripe,
                     'fabric', radius=0.01, rotate=[ang, 0, 0]))
    for s in (-1, 1):
        p.append(rod([0, 0.87, s * 0.7], [0, 1.1, s * 0.86], 0.015, '#E8E2D6', 'matte'))
    p.append(cushion([0.4, 0.08, 0.3], [0, 0.8, -0.56], '#FFFFFF'))
    return p


add('hammock', placeholder(hammock_stand('#B08E66', '#E8E2D6', '#4E7A7F')), (1, 2))
add('hammock@cozy', placeholder(hammock_stand('#8E6E52', '#EFE5D3', RUST)), (1, 2))


def hot_tub(wood, water='#6FC3CF'):
    p = [cyl([1.84, 0.82, 1.84], [0, 0.41, 0], wood, 'wood', segments=32),
         torus([1.86, 0.08, 1.86], [0, 0.82, 0], wood, 'wood', segments=32),
         cyl([1.62, 0.02, 1.62], [0, 0.74, 0], water, 'glass', segments=32),
         cyl([1.6, 0.02, 1.6], [0, 0.68, 0], '#9ED6DE', 'gloss', segments=32)]
    for y in (0.2, 0.6):
        p.append(torus([1.88, 0.03, 1.88], [0, y, 0], '#3A3D41', 'metal', segments=32))           # hoops
    p += [box([0.6, 0.2, 0.3], [0.0, 0.1, 0.8], wood, 'wood', radius=0.01),                       # step
          box([0.4, 0.08, 0.3], [0.62, 0.86, 0.5], '#FFFFFF', 'fabric', radius=0.03),             # towels
          cyl([0.08, 0.16, 0.08], [-0.6, 0.9, 0.5], '#E9C46A', 'glass')]
    return p


add('hotTubCedar', placeholder(hot_tub('#A9744E')), (2, 2))


def swing_set(frame, seat):
    p = []
    for s in (-1, 1):
        p += [rod([s * 0.92, 0.0, -0.42], [s * 0.92, 1.9, 0], 0.07, frame, 'wood'),
              rod([s * 0.92, 0.0, 0.42], [s * 0.92, 1.9, 0], 0.07, frame, 'wood')]
    p.append(cyl([0.09, 1.92, 0.09], [0, 1.92, 0], frame, 'wood', rotate=[0, 0, 90]))
    for x in (-0.48, 0.48):
        p += [cyl([0.012, 1.4, 0.012], [x - 0.17, 1.2, 0], '#C9CBCC', 'metal'),
              cyl([0.012, 1.4, 0.012], [x + 0.17, 1.2, 0], '#C9CBCC', 'metal'),
              box([0.42, 0.04, 0.2], [x, 0.48, 0], seat, 'satin', radius=0.02)]                      # seat (0.5)
    return p


add('swingSet', placeholder(swing_set('#B08E66', '#D9483B')), (2, 1))


def sandbox():
    p = []
    for z, w, d in ((-0.92, 1.88, 0.08), (0.92, 1.88, 0.08)):
        p.append(box([w, 0.26, d], [0, 0.13, z], '#C9A27A', 'wood', radius=0.01))
    for x in (-0.92, 0.92):
        p.append(box([0.08, 0.26, 1.88], [x, 0.13, 0], '#C9A27A', 'wood', radius=0.01))
    p += [box([1.78, 0.18, 1.78], [0, 0.09, 0], '#E9D5A8', 'matte', radius=0.02),
          part('blob', [0.5, 0.2, 0.4], [-0.3, 0.2, 0.2], '#E3CC9A', 'matte', noise=0.15),
          lathe([0.22, 0.18, 0.22], [0.3, 0.27, -0.2], '#D9483B', 'satin', [[0, -0.5], [0.36, -0.5], [0.5, 0.5], [0, 0.5]]),     # bucket
          box([0.1, 0.02, 0.3], [0.45, 0.2, 0.2], '#3E8EDE', 'satin', radius=0.01, rotate=[0, 30, 0]),
          lathe([0.2, 0.18, 0.2], [-0.32, 0.36, 0.2], '#E3CC9A', 'matte', [[0, -0.5], [0.5, -0.5], [0.42, 0.5], [0, 0.5]])]     # sandcastle
    return p


add('sandbox', placeholder(sandbox()), (2, 2))


def bike_rack(frame, bike):
    p = [box([1.8, 0.05, 0.12], [0, 0.025, -0.3], frame, 'metal', radius=0.01)]
    for x in (-0.6, -0.2, 0.2, 0.6):
        p.append(torus([0.4, 0.04, 0.02], [x, 0.24, -0.3], frame, 'metal', rotate=[90, 0, 0], segments=16))
    # A bicycle leaning in the rack, along x.
    for x in (-0.55, 0.45):
        p += [torus([0.62, 0.04, 0.62], [x, 0.33, 0.05], '#2A2C30', 'satin', rotate=[90, 0, 0], segments=24),
              cyl([0.04, 0.04, 0.04], [x, 0.33, 0.05], '#C9CBCC', 'metal', rotate=[90, 0, 0])]
    p += [rod([-0.55, 0.33, 0.05], [-0.1, 0.33, 0.05], 0.03, bike, 'gloss'),
          rod([-0.1, 0.33, 0.05], [0.25, 0.68, 0.05], 0.035, bike, 'gloss'),
          rod([-0.1, 0.33, 0.05], [-0.25, 0.7, 0.05], 0.03, bike, 'gloss'),
          rod([-0.25, 0.7, 0.05], [0.25, 0.68, 0.05], 0.035, bike, 'gloss'),
          rod([-0.55, 0.33, 0.05], [-0.25, 0.7, 0.05], 0.025, bike, 'gloss'),
          rod([0.45, 0.33, 0.05], [0.3, 0.78, 0.05], 0.03, bike, 'gloss'),
          box([0.22, 0.05, 0.1], [-0.27, 0.76, 0.05], '#2A2C30', 'fabric', radius=0.02),
          rod([0.3, 0.84, -0.12], [0.3, 0.84, 0.22], 0.025, '#2A2C30', 'satin')]
    return p


add('bikeRack', placeholder(bike_rack('#5C6168', '#3E8EDE')), (2, 1))


def fire_pit(stone):
    p = [cyl([0.86, 0.3, 0.86], [0, 0.15, 0], stone, 'matte', segments=20),
         cyl([0.66, 0.02, 0.66], [0, 0.305, 0], '#2A2624', 'matte', segments=20)]
    for i in range(10):
        a = math.radians(i * 36)
        p.append(box([0.2, 0.08, 0.14], [math.cos(a) * 0.38, 0.34, math.sin(a) * 0.38], stone, 'matte', radius=0.03,
                     rotate=[0, -i * 36, 0]))
    for r in (0, 60, 120):
        p.append(cyl([0.08, 0.5, 0.08], [0, 0.36, 0], '#6B4A33', 'bark', rotate=[0, r, 80]))
    p += [part('blob', [0.36, 0.3, 0.36], [0, 0.48, 0], '#FF9A3C', 'gloss', noise=0.3),
          part('blob', [0.2, 0.36, 0.2], [0.03, 0.56, 0.0], '#FFD27A', 'gloss', noise=0.3)]
    return p


add('firePit', placeholder(fire_pit('#8E8A84')))


def garden_chair(wood):
    p = []
    for x in (-0.24, -0.12, 0.0, 0.12, 0.24):                                                  # slatted back, reclined
        p.append(box([0.1, 0.8, 0.025], [x, 0.66, -0.28], wood, 'satin', radius=0.01, rotate=[-22, 0, 0]))
    for z in (-0.12, 0.0, 0.12, 0.24):
        p.append(box([0.56, 0.025, 0.1], [0, 0.4 - z * 0.15, z], wood, 'satin', radius=0.008))   # seat slats (~0.4)
    for s in (-1, 1):
        p += [box([0.12, 0.025, 0.66], [s * 0.36, 0.62, -0.02], wood, 'satin', radius=0.01),       # wide arms
              box([0.06, 0.62, 0.06], [s * 0.36, 0.31, 0.26], wood, 'satin', radius=0.01),
              box([0.06, 0.4, 0.06], [s * 0.32, 0.2, -0.24], wood, 'satin', radius=0.01)]
    p.append(cyl([0.12, 0.1, 0.12], [0.36, 0.68, 0.18], '#E8E2D6', 'gloss'))                   # cup on the arm
    return p


add('gardenChair', placeholder(garden_chair('#F2F0EA')))
add('gardenChair@modern', placeholder(garden_chair('#5C6168')))
add('gardenChair@cozy', placeholder(garden_chair('#4E7A7F')))


def lamp_post(metal):
    return placeholder([
        cyl([0.3, 0.12, 0.3], [0, 0.06, 0], metal, 'metal', taper=0.7),
        cyl([0.07, 2.0, 0.07], [0, 1.1, 0], metal, 'metal'),
        cyl([0.12, 0.06, 0.12], [0, 2.1, 0], metal, 'metal'),
        box([0.26, 0.34, 0.26], [0, 2.3, 0], BULB, 'glass', radius=0.01),
        sph([0.12, 0.12, 0.12], [0, 2.28, 0], BULB, 'gloss', segments=10),
        cone([0.38, 0.14, 0.38], [0, 2.53, 0], metal, 'metal', taper=0.2, segments=4, rotate=[0, 45, 0]),
    ])


add('lampPost', lamp_post('#2F3236'))


def string_lights(post):
    p = []
    for s in (-1, 1):
        p += [box([0.08, 2.4, 0.08], [s * 0.88, 1.2, 0], post, 'wood', radius=0.01),
              cyl([0.24, 0.3, 0.24], [s * 0.88, 0.15, 0], '#8E8A84', 'matte', taper=0.8)]           # weighted planters
    n = 8
    for i in range(n):
        x0, x1 = -0.84 + i * 1.68 / n, -0.84 + (i + 1) * 1.68 / n
        y0 = 2.25 - 0.3 * (1 - (x0 / 0.84) ** 2)
        y1 = 2.25 - 0.3 * (1 - (x1 / 0.84) ** 2)
        p.append(rod([x0, y0, 0], [x1, y1, 0], 0.008, '#2A2C30', 'matte'))
        xm = (x0 + x1) / 2
        p.append(sph([0.07, 0.09, 0.07], [xm, (y0 + y1) / 2 - 0.07, 0], BULB, 'gloss', segments=8))
    return p


add('stringLights', placeholder(string_lights('#8E6E52')), (2, 1))

# =============================================================================================
# Validation (footprint bounds, finishes) and output
# =============================================================================================
# Things on a wall or the ceiling may reach past their tiles (a ceiling fan's blades).
MOUNTED = {'ceilingFan', 'chandelier', 'ceilingPendant', 'ceilingLight', 'pendantIndustrial', 'hangingPlant'}


def main(path):
    bad = 0
    for k, e in E.items():
        fp = FOOT[k]
        issues = [] if 'url' not in e else ['a downloaded model (url): draw it from parts']
        parts = e['placeholder']
        hung = k.split('@')[0].split('.', 1)[1] in MOUNTED
        for p in parts:
            q = dict(p, shape='cylinder') if p['shape'] in ('lathe', 'torus') else p
            issues += [i for i in OX.validate(k, [q], fp) if 'finishes (max 4)' not in i and 'exceeds' not in i and 'below floor' not in i]
            lo, hi = OX.bounds(q)
            if lo[1] < -0.015: issues.append(f'part {p["shape"]} at {p["at"]} below the floor')
            if not hung and (lo[0] < -fp[0] / 2 - 0.02 or hi[0] > fp[0] / 2 + 0.02 or lo[2] < -fp[1] / 2 - 0.02 or hi[2] > fp[1] / 2 + 0.02):
                issues.append(f'part {p["shape"]} at {p["at"]} leaves the {fp} footprint')
        fins = {p.get('material') for p in parts}
        if k in STRICT and len(fins) > 4:
            issues.append(f'{len(fins)} finishes (max 4): {sorted(f.split(".")[-1] for f in fins)}')
        for s in issues: print(f'  ! {k}: {s}')
        bad += len(issues)
    lines = []
    for k, e in E.items():
        lines.append(f'    {json.dumps(k)}: {json.dumps(e, separators=(",", ":"))}')
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write('{\n  "$comment": "Furniture, lighting, decor, kids\' and garden things of the 0.20 catalogue (content/furniture.json), '
                'written by tools/art/catalog.py. Listed in the base manifest\'s packs.",\n  "version": 1,\n  "entries": {\n')
        f.write(',\n'.join(lines) + '\n  }\n}\n')
    json.load(open(path))
    print(f'{len(E)} entries ({len({k.split("@")[0] for k in E})} models) -> {path}; {bad} issue(s)')
    return bad


if __name__ == '__main__':
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'web', 'public', 'assets', 'catalog', 'manifest.json')
    sys.exit(1 if main(os.path.normpath(sys.argv[1] if len(sys.argv) > 1 else out)) else 0)
