# Model entries for the second catalogue (`model.<id>`): the separate-piece furniture, tabletop things,
# lamps and garden things in web/public/content/catalog2/*.json. Every model is drawn here from primitive
# parts (box, cylinder, cone, sphere, capsule, blob, lathe, torus; original, CC0); no downloads.
#
# Conventions as in catalog.py: 1 unit = 1 m, origin at the footprint centre on the floor, front +Z (a seat's
# sitter faces +Z, so a chair whose front tile holds a table faces it), back against the wall at -Z.
# Things on a wall hang with their back on the wall face (z = -0.43); ceiling things hang from 2.8 m.
# Things on the `surface` layer (vases, books, lamps on a table) have their origin at their base, centred:
# the renderer lifts them to their host's top.
#
# Besides the manifest this module records, per model, what the content needs to agree with the drawing:
# TOPS (the rectangle and height of a top that things stand on: content `surface`), SEATS (seat height
# and how far forward the hips sit: content `seat`), LIGHTS (bulb height: content `light.height`) and the
# radius of things on a surface (content `radius`). `surface_of`, `seat_of`, `radius_of` and `light_of`
# turn them into content values.
#
# Run: python3 -B catalog2.py   (writes web/public/assets/catalog2/manifest.json, listed in the base
# manifest's `packs`; one entry per line). Validates footprints, finishes (at most 4 per model) and sizes.
import json, math, os, random, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import objects_extra as OX                                          # noqa: E402
from objects_extra import box, cyl, cone, sph, rod, part            # noqa: E402
from furniture import (cushion, lathe, torus, WHITE, OFFWHITE, LGREY, MGREY, OAK, OAKD, BIRCH, HONEY,  # noqa: E402
                       WALNUT, BLACKMETAL, STEEL, CHROME, BRASS, TEAL, COGNAC, MUSTARD, SAGE, RUST, CREAM, SCREEN)

E, FOOT, KIND = {}, {}, {}
TOPS, SEATS, LIGHTS = {}, {}, {}
CEILING = 2.8
WALL = -0.43                    # a wall thing's back (the wall face)
LINEN, SAND, CLAY, NAVY, BLUSH, OLIVE = '#E7E0D4', '#D9C7A7', '#C98F6B', '#3E5470', '#E4B8AE', '#8C9468'
BULB, FOLIAGE, LEAF2, SOIL = '#FFF1CF', '#5E8F4E', '#7FA35A', '#4A3A2C'
PLINTH, CAB, WORKTOP, PORCELAIN = '#2F3236', '#EDEBE6', '#E8E6E1', '#F5F3EE'
RED, BLUE, YELLOW, GREEN = '#D9483B', '#3E8EDE', '#F2C14E', '#5BAE6A'
TEAK, CHARCOAL, IRON = '#9A6B45', '#3A3D41', '#2A2C30'
CT, BT, TT, DT = 0.95, 1.06, 0.75, 0.74      # counter, bar, dining table and desk tops


def add(key, parts, fp=(1, 1), kind='floor', top=None, seat=None, light=None):
    """A model. `top` = (x0, x1, z0, z1, height): the rectangle things stand on; `seat` = (height, forward)."""
    E['model.' + key] = {'type': 'model', 'placeholder': parts}
    FOOT[key], KIND[key] = fp, kind
    if top: TOPS[key] = top
    if seat: SEATS[key] = seat
    if light is not None: LIGHTS[key] = light


def surface_of(key, kind):
    """Content `surface` for a model with a top: the drawn top's height, size and middle (`at`, off the
    footprint's middle); `size`/`at` are left out where the top covers the whole footprint."""
    x0, x1, z0, z1, h = TOPS[key]
    out = {'height': round(h, 3), 'kind': kind}
    size, at = [round(x1 - x0, 3), round(z1 - z0, 3)], [round((x0 + x1) / 2, 3), round((z0 + z1) / 2, 3)]
    if size != [float(v) for v in FOOT[key]] or at != [0, 0]:
        out['size'] = size
        if at != [0, 0]: out['at'] = at
    return out


def seat_of(key):
    h, f = SEATS[key]
    return {'height': round(h, 3), 'forward': round(f, 3)}


def radius_of(key):
    """Things on a surface: the radius of the disc they take (their drawn extent, at least 6 cm)."""
    r = 0.0
    for p in E['model.' + key]['placeholder']:
        q = dict(p, shape='cylinder') if p['shape'] in ('lathe', 'torus') else p
        lo, hi = OX.bounds(q)
        r = max(r, math.hypot(max(abs(lo[0]), abs(hi[0])), max(abs(lo[2]), abs(hi[2]))) * 0.92)
    return round(min(max(r + 0.01, 0.06), 0.3), 2)


def light_of(key):
    return LIGHTS[key]


# =============================================================================================
# Shared shapes
# =============================================================================================
def legs4(xs, zs, h, t, color, fin, y0=0.0, round_=False, taper=None):
    out = []
    for x in xs:
        for z in zs:
            if round_:
                kw = {'taper': taper} if taper else {}
                out.append(cyl([t, h, t], [x, y0 + h / 2, z], color, fin, segments=10, **kw))
            else:
                out.append(box([t, h, t], [x, y0 + h / 2, z], color, fin, radius=min(0.01, t * 0.2)))
    return out


def splayed_leg(x, z, h, t, color, fin, out=0.04, y0=0.0):
    """A round leg whose foot splays outwards from (x, z) at the top by `out`."""
    fx = x + math.copysign(out, x) if x else x
    fz = z + math.copysign(out, z) if z else z
    return rod([fx, y0, fz], [x, y0 + h, z], t, color, fin, segments=10)


def flare(size, at, color, finish, t, **kw):
    """A pot or cup `t` times wider at the top than at the bottom, `size` measured at the bottom. Drawn as
    a lathe: tapered cylinders flaring upwards come out dark-sided."""
    return lathe([size[0] * t, size[1], size[2] * t], at, color, finish, [[0, -0.5], [0.5 / t, -0.5], [0.5, 0.5], [0, 0.5]], **kw)


def case(w, h, d, x, y, z, color, fin, back=None):
    """An open box (back, sides, top and bottom) `w` x `h` x `d` centred on (x, z), bottom at y."""
    t = 0.022
    return [box([w, h, t], [x, y + h / 2, z - d / 2 + t / 2], back or color, fin, radius=0.003),
            box([t, h, d], [x - w / 2 + t / 2, y + h / 2, z], color, fin, radius=0.003),
            box([t, h, d], [x + w / 2 - t / 2, y + h / 2, z], color, fin, radius=0.003),
            box([w, t, d], [x, y + h - t / 2, z], color, fin, radius=0.003),
            box([w, t, d], [x, y + t / 2, z], color, fin, radius=0.003)]


def glazed_door(x, y, w, h, z, color, fin, bar=0.045):
    """A door frame (stiles and rails) with the glass left out, so what is behind it shows."""
    return [box([bar, h, 0.02], [x - w / 2 + bar / 2, y + h / 2, z], color, fin, radius=0.004),
            box([bar, h, 0.02], [x + w / 2 - bar / 2, y + h / 2, z], color, fin, radius=0.004),
            box([w, bar, 0.02], [x, y + bar / 2, z], color, fin, radius=0.004),
            box([w, bar, 0.02], [x, y + h - bar / 2, z], color, fin, radius=0.004)]


def handle_v(x, y, z, color=CHROME, fin='chrome', h=0.14):
    return box([0.014, h, 0.022], [x, y, z], color, fin, radius=0.005)


def handle_h(x, y, z, color=CHROME, fin='chrome', w=0.16):
    return box([w, 0.014, 0.022], [x, y, z], color, fin, radius=0.005)


def books_row(x0, x1, y, z, depth=0.18, seed=1, colors=(TEAL, RUST, MUSTARD, SAGE, CREAM, NAVY, CLAY), hmax=0.24, fin='matte', lean=True):
    """Books standing on a shelf from x0 to x1 (bottom at y), spines facing +Z."""
    p, x, k = [], x0, seed
    while x < x1 - 0.03:
        k = (k * 7 + 3) % 11
        w = 0.022 + (k % 4) * 0.008
        h = hmax * (0.7 + (k % 5) * 0.07)
        if x + w > x1: break
        p.append(box([w, h, depth], [x + w / 2, y + h / 2, z], colors[k % len(colors)], fin, radius=0.003))
        x += w + 0.003
        if lean and k % 7 == 0 and x + 0.12 < x1:
            p.append(box([0.03, h * 0.9, depth], [x + 0.06, y + h * 0.42, z], colors[(k + 2) % len(colors)], fin, radius=0.003, rotate=[0, 0, -22]))
            x += 0.11
    return p


def plant_pot(x, z, y, pot, d=0.2, h=0.18, leaves=FOLIAGE, pot_fin='gloss', tall=0.25, soil=True):
    """A pot with a leafy plant (foliage blobs); `soil=False` saves a finish."""
    p = [flare([d, h, d], [x, y + h / 2, z], pot, pot_fin, 1.18, segments=16),
         part('blob', [d * 1.3, tall, d * 1.3], [x, y + h + tall * 0.42, z], leaves, 'foliage', noise=0.32),
         part('blob', [d * 0.8, tall * 0.7, d * 0.8], [x + d * 0.2, y + h + tall * 0.7, z - d * 0.1], LEAF2, 'foliage', noise=0.3)]
    if soil: p.insert(1, cyl([d * 1.08, 0.012, d * 1.08], [x, y + h - 0.005, z], SOIL, 'matte', segments=16))
    return p


VASE = [[0, -0.5], [0.32, -0.5], [0.46, -0.3], [0.5, -0.05], [0.4, 0.25], [0.24, 0.42], [0.26, 0.5], [0.2, 0.5], [0.18, 0.4], [0, 0.38]]
BOWL = [[0, -0.5], [0.3, -0.5], [0.42, -0.2], [0.5, 0.5], [0.45, 0.5], [0.36, -0.15], [0.25, -0.35], [0, -0.35]]
JAR = [[0, -0.5], [0.46, -0.5], [0.5, -0.42], [0.5, 0.3], [0.38, 0.42], [0.38, 0.5], [0, 0.5]]
CUP = [[0, -0.5], [0.38, -0.5], [0.5, 0.5], [0.44, 0.5], [0.34, -0.4], [0, -0.4]]
BOTTLE = [[0, -0.5], [0.5, -0.5], [0.5, 0.15], [0.2, 0.32], [0.17, 0.5], [0, 0.5]]


def mug(x, z, y, color, d=0.08, h=0.09, rot=0):
    a = math.radians(rot)
    return [cyl([d, h, d], [x, y + h / 2, z], color, 'gloss', segments=14),
            cyl([d * 0.84, 0.006, d * 0.84], [x, y + h - 0.004, z], '#5A3A26', 'gloss', segments=12),
            torus([h * 0.6, 0.014, h * 0.6], [x + math.cos(a) * d * 0.55, y + h / 2, z - math.sin(a) * d * 0.55], color, 'gloss', rotate=[90, rot, 0], segments=12)]


def plate(x, z, y, color=PORCELAIN, d=0.24, rim=None):
    p = [cyl([d, 0.014, d], [x, y + 0.007, z], color, 'gloss', segments=20)]
    if rim: p.append(torus([d * 0.96, 0.008, d * 0.96], [x, y + 0.014, z], rim, 'gloss', segments=20))
    return p


def wine_glass(x, z, y, h=0.17, wine=None):
    p = [cyl([0.06, 0.004, 0.06], [x, y + 0.002, z], '#DCE6EA', 'glass', segments=10),
         cyl([0.008, h * 0.45, 0.008], [x, y + h * 0.225, z], '#DCE6EA', 'glass', segments=6),
         lathe([0.075, h * 0.55, 0.075], [x, y + h * 0.72, z], '#DCE6EA', 'glass', [[0, -0.5], [0.4, -0.35], [0.5, 0.1], [0.42, 0.5], [0.38, 0.5], [0.44, 0.1], [0, -0.42]], segments=12)]
    if wine: p.append(cyl([0.05, 0.03, 0.05], [x, y + h * 0.58, z], wine, 'gloss', segments=10))
    return p


# =============================================================================================
# Kitchen: counters, islands, bars, appliances, wall units
# =============================================================================================
def counter_body(w=1.0, x=0.0, body=CAB, fin='satin', top=WORKTOP, tfin='gloss'):
    """A base unit: carcass y 0.1..0.9 with its front at z = 0.24, a plinth and a worktop to 0.95 (front edge z = 0.31)."""
    return [box([w - 0.004, 0.8, 0.7], [x, 0.5, -0.11], body, fin, radius=0.01),
            box([w - 0.004, 0.1, 0.64], [x, 0.05, -0.14], PLINTH, 'satin', radius=0.004),
            box([w, 0.05, 0.79], [x, 0.925, -0.085], top, tfin, radius=0.008)]


def counter_top(key, w=1.0, x=0.0):
    return (x - w / 2, x + w / 2, -0.48, 0.31, CT)


def doors(x, w, color, fin, handle=CHROME, hfin='chrome', y0=0.12, y1=0.86, z=0.25, n=2, knob=False):
    p, dw = [], (w - 0.02) / n
    for i in range(n):
        cx = x - w / 2 + 0.01 + dw * (i + 0.5)
        p.append(box([dw - 0.012, y1 - y0, 0.02], [cx, (y0 + y1) / 2, z], color, fin, radius=0.008))
        hx = cx + (dw / 2 - 0.05) * (1 if i % 2 == 0 and n > 1 else -1)
        if knob:
            p.append(sph([0.03, 0.03, 0.025], [hx, y1 - 0.12, z + 0.02], handle, hfin, segments=8))
        else:
            p.append(handle_v(hx, y1 - 0.12, z + 0.02, handle, hfin))
    return p


def drawers(x, w, color, fin, n=3, handle=CHROME, hfin='chrome', y0=0.12, y1=0.86, z=0.25):
    p, dh = [], (y1 - y0) / n
    for i in range(n):
        cy = y0 + dh * (i + 0.5)
        p += [box([w - 0.03, dh - 0.012, 0.02], [x, cy, z], color, fin, radius=0.008),
              handle_h(x, cy + dh * 0.25, z + 0.02, handle, hfin, w=min(0.3, w * 0.4))]
    return p


def jar(x, z, y, color, d=0.09, h=0.13, lid=WALNUT, lid_fin='wood'):
    return [lathe([d, h, d], [x, y + h / 2, z], color, 'glass', JAR, segments=14),
            cyl([d * 0.8, 0.025, d * 0.8], [x, y + h + 0.01, z], lid, lid_fin, segments=14)]


# Counter end: doors at the front and open quarter shelves on its +X end.
p = counter_body(0.84, -0.08) + doors(-0.08, 0.84, CAB, 'satin', n=1)
p += [box([0.16, 0.8, 0.7], [0.42, 0.5, -0.11], OAK, 'wood', radius=0.008),
      box([0.16, 0.05, 0.79], [0.42, 0.925, -0.085], WORKTOP, 'gloss', radius=0.008)]
for y in (0.38, 0.66):
    p.append(box([0.17, 0.025, 0.66], [0.42, y, -0.1], WORKTOP, 'gloss', radius=0.004))
p += [flare([0.14, 0.12, 0.14], [0.42, 0.45, 0.05], '#9DC6CF', 'gloss', 1.15), cyl([0.12, 0.1, 0.12], [0.42, 0.72, -0.15], CREAM, 'gloss'),
      cyl([0.13, 0.08, 0.13], [0.42, 0.18, -0.05], RUST, 'gloss')]
add('counterEnd', p, top=counter_top('counterEnd'))

# Outer corner: fronts on +Z and +X, a worktop over both.
p = [box([0.74, 0.8, 0.7], [-0.13, 0.5, -0.11], CAB, 'satin', radius=0.01),
     box([0.7, 0.1, 0.64], [-0.15, 0.05, -0.14], PLINTH, 'satin', radius=0.004),
     box([0.81, 0.05, 0.79], [-0.095, 0.925, -0.085], WORKTOP, 'gloss', radius=0.008)]
p += doors(-0.13, 0.74, CAB, 'satin', n=1)
p += [box([0.02, 0.74, 0.66], [0.25, 0.49, -0.11], CAB, 'satin', radius=0.008), handle_v(0.27, 0.74, 0.12)]
p += [cyl([0.24, 0.02, 0.24], [0.12, 0.96, 0.08], PORCELAIN, 'gloss'), sph([0.08, 0.07, 0.08], [0.1, 1.0, 0.07], YELLOW, 'satin', segments=8)]
add('counterCornerOuter', p, top=(-0.5, 0.31, -0.48, 0.31, CT))

add('counterDrawers', counter_body() + drawers(0, 1.0, CAB, 'satin', n=4), top=counter_top('counterDrawers'))

# Open shelves: two shelves of bowls, jars and a stack of plates.
p = counter_body(body=OAK, fin='wood')
p[0] = box([0.996, 0.8, 0.7], [0, 0.5, -0.11], OAK, 'wood', radius=0.01)
p += [box([0.92, 0.72, 0.03], [0, 0.5, 0.245], '#3B2E24', 'matte', radius=0.004)]       # dark open interior
for y in (0.16, 0.5):
    p.append(box([0.94, 0.025, 0.1], [0, y, 0.24], OAK, 'wood', radius=0.004))
for i, x in enumerate((-0.32, -0.1, 0.14, 0.34)):
    p.append(cyl([0.16 - i % 2 * 0.03, 0.025 + i % 2 * 0.06, 0.16 - i % 2 * 0.03], [x, 0.2 + (0.025 + i % 2 * 0.06) / 2, 0.22], [PORCELAIN, CREAM][i % 2], 'gloss', segments=14))
p += [cyl([0.14, 0.15, 0.14], [-0.3, 0.6, 0.2], '#B9C9C4', 'gloss', taper=0.9), cyl([0.12, 0.2, 0.12], [-0.1, 0.62, 0.2], TEAL, 'gloss'),
      cyl([0.2, 0.05, 0.2], [0.25, 0.54, 0.2], PORCELAIN, 'gloss'), cyl([0.2, 0.05, 0.2], [0.25, 0.59, 0.2], PORCELAIN, 'gloss')]
add('counterOpenShelf', p, top=counter_top('counterOpenShelf'))

# Butcher block: a thick end-grain top on turned legs, a slatted shelf and a basket below.
p = [box([0.9, 0.1, 0.8], [0, 0.88, 0], '#B98A5A', 'wood', radius=0.012),
     box([0.84, 0.03, 0.74], [0, 0.93, 0], '#C99A68', 'wood', radius=0.004)]
p += legs4((-0.41, 0.41), (-0.36, 0.36), 0.83, 0.07, '#9C6B43', 'wood')
for z in (-0.24, -0.08, 0.08, 0.24):
    p.append(box([0.8, 0.025, 0.12], [0, 0.2, z], '#9C6B43', 'wood', radius=0.004))
p += [box([0.4, 0.2, 0.3], [-0.16, 0.32, 0.0], '#C9A979', 'fabric', radius=0.03), box([0.2, 0.16, 0.2], [0.24, 0.29, 0.06], PORCELAIN, 'gloss', radius=0.03),
      handle_h(0, 0.72, 0.4, BLACKMETAL, 'metal', w=0.6)]
add('butcherBlock', p, top=(-0.45, 0.45, -0.4, 0.4, 0.93))

# Marble counter: navy doors, a white marble top with a soft grey vein.
p = counter_body(body=NAVY, top='#F4F3F0', tfin='gloss') + doors(0, 1.0, NAVY, 'satin', handle=BRASS, hfin='metal')
p += [box([0.6, 0.004, 0.03], [-0.1, 0.951, -0.05], '#C9CBCC', 'gloss', radius=0.001, rotate=[0, 28, 0]),
      box([0.35, 0.004, 0.02], [0.22, 0.951, 0.08], '#C9CBCC', 'gloss', radius=0.001, rotate=[0, -15, 0])]
add('counterMarble', p, top=counter_top('counterMarble'))

# Double sink: two basins in a 2-wide unit, a gooseneck tap over each.
p = counter_body(2.0) + doors(-0.5, 0.98, CAB, 'satin') + doors(0.5, 0.98, CAB, 'satin')
for x in (-0.42, 0.42):
    p += [box([0.66, 0.03, 0.48], [x, 0.94, -0.06], '#C3C7CB', 'metal', radius=0.03),
          box([0.6, 0.012, 0.42], [x, 0.952, -0.06], '#8E959B', 'metal', radius=0.03),
          cyl([0.04, 0.3, 0.04], [x, 1.1, -0.38], CHROME, 'chrome'),
          rod([x, 1.25, -0.38], [x, 1.22, -0.18], 0.03, CHROME, 'chrome')]
p += [box([0.2, 0.04, 0.14], [0, 0.97, -0.34], '#E8E6E1', 'gloss', radius=0.02), cyl([0.06, 0.16, 0.06], [0.04, 1.03, -0.36], SAGE, 'gloss', taper=0.8)]
add('counterSinkDouble', p, (2, 1))


def bar(top_x0=-0.5, top_x1=0.5, end=False, body=WALNUT, top=OAK, top_fin='wood', rail=BRASS):
    """A bar: slatted front, a top at 1.06 overhanging on the stool side (+Z), a foot rail."""
    bw = (top_x1 - top_x0) - (0.08 if end else 0.0)
    bx = top_x0 + bw / 2
    p = [box([bw, 0.98, 0.5], [bx, 0.49, -0.12], body, 'wood', radius=0.01),
         box([top_x1 - top_x0, 0.06, 0.88], [(top_x0 + top_x1) / 2, BT - 0.03, -0.04], top, top_fin, radius=0.012),
         box([bw, 0.04, 0.26], [bx, 0.92, -0.36], top, top_fin, radius=0.006)]             # bartender's ledge
    n = int(bw / 0.1)
    for i in range(n):
        p.append(box([bw / n - 0.012, 0.92, 0.02], [top_x0 + bw / n * (i + 0.5), 0.47, 0.135], body, 'wood', radius=0.004))
    p.append(cyl([0.04, bw, 0.04], [bx, 0.2, 0.26], rail, 'metal', rotate=[0, 0, 90]))
    for x in (top_x0 + 0.12, top_x0 + bw - 0.12):
        p.append(box([0.03, 0.06, 0.12], [x, 0.2, 0.2], rail, 'metal', radius=0.006))
    if end:
        p += [box([0.06, BT - 0.06, 0.88], [top_x1 - 0.03, (BT - 0.06) / 2, -0.04], top, top_fin, radius=0.008)]   # waterfall end
    return p


add('barSegment', bar(), top=(-0.5, 0.5, -0.48, 0.4, BT))
add('barEnd', bar(end=True), top=(-0.5, 0.5, -0.48, 0.4, BT))


def island_unit(w, body, top, tfin, handle, fp=(1, 1)):
    """Freestanding island: cupboards on the back (-Z), an overhang for stools on the front (+Z)."""
    p = [box([w - 0.16, 0.82, 0.58], [0, 0.51, -0.16], body, 'satin', radius=0.012),
         box([w - 0.2, 0.1, 0.52], [0, 0.05, -0.16], PLINTH, 'satin', radius=0.004),
         box([w - 0.04, 0.05, 0.94], [0, CT - 0.025, 0], top, tfin, radius=0.012)]
    n = max(1, round((w - 0.16) / 0.46))
    dw = (w - 0.18) / n
    for i in range(n):
        x = -(w - 0.18) / 2 + dw * (i + 0.5)
        p += [box([dw - 0.02, 0.72, 0.02], [x, 0.5, -0.455], body, 'satin', radius=0.008),
              handle_h(x, 0.78, -0.47, handle, 'metal', w=0.14)]
    for x in (-(w - 0.16) / 2, (w - 0.16) / 2):                                             # end panels
        p.append(box([0.04, 0.92, 0.6], [x, 0.46, -0.15], body, 'satin', radius=0.008))
    return p


p = island_unit(1.0, SAGE, '#F2F1EE', 'gloss', BRASS)
p += [cyl([0.26, 0.02, 0.26], [-0.18, 0.96, -0.12], PORCELAIN, 'gloss')]
add('islandSmall', p, top=(-0.48, 0.48, -0.47, 0.47, CT))
p = island_unit(3.0, '#F1F0EC', '#C9CBCC', 'satin', BLACKMETAL, (3, 1))
p += [box([0.42, 0.03, 0.28], [0.9, 0.965, -0.24], OAK, 'wood', radius=0.01)]
add('islandLong', p, (3, 1), top=(-1.48, 1.48, -0.47, 0.47, CT))

# Tall larder: two doors over two drawers, a cornice.
p = [box([0.92, 2.06, 0.66], [0, 1.08, -0.12], CAB, 'satin', radius=0.012),
     box([0.9, 0.1, 0.6], [0, 0.05, -0.14], PLINTH, 'satin', radius=0.004),
     box([0.96, 0.05, 0.7], [0, 2.135, -0.11], CAB, 'satin', radius=0.01)]
p += doors(0, 0.92, CAB, 'satin', y0=0.62, y1=2.06, z=0.22, handle=BRASS, hfin='metal')
p += drawers(0, 0.92, CAB, 'satin', n=2, y0=0.12, y1=0.6, z=0.22, handle=BRASS, hfin='metal')
p += [box([0.3, 0.22, 0.26], [-0.22, 2.27, -0.16], '#C9A979', 'fabric', radius=0.04)]       # basket on top
add('pantryTall', p)

# Oven tower: two ovens one above the other in a tall unit.
p = [box([0.7, 2.06, 0.64], [0, 1.08, -0.13], CAB, 'satin', radius=0.012),
     box([0.68, 0.1, 0.58], [0, 0.05, -0.15], PLINTH, 'satin', radius=0.004)]
for y in (0.98, 1.56):
    p += [box([0.6, 0.5, 0.03], [0, y, 0.2], '#2B2C2E', 'gloss', radius=0.01),
          box([0.46, 0.26, 0.01], [0, y - 0.04, 0.218], '#4A3B30', 'gloss', radius=0.02),       # warm window
          handle_h(0, y + 0.19, 0.235, CHROME, 'chrome', w=0.46),
          box([0.6, 0.05, 0.02], [0, y + 0.235, 0.205], '#3A3D41', 'gloss', radius=0.004)]
p += doors(0, 0.7, CAB, 'satin', y0=0.12, y1=0.68, z=0.2, n=1) + doors(0, 0.7, CAB, 'satin', y0=1.86, y1=2.06, z=0.2, n=1)
add('wallOven', p)

# Range cooker: six burners, two ovens, chrome rail; cream enamel.
ENAMEL = '#EFE5D3'
p = [box([1.56, 0.86, 0.66], [0, 0.5, -0.13], ENAMEL, 'gloss', radius=0.02),
     box([1.5, 0.07, 0.6], [0, 0.035, -0.13], PLINTH, 'satin', radius=0.006),
     box([1.58, 0.04, 0.68], [0, 0.95, -0.13], '#2B2C2E', 'satin', radius=0.01),
     box([1.58, 0.16, 0.06], [0, 1.04, -0.45], ENAMEL, 'gloss', radius=0.02)]                # back splash
for i, x in enumerate((-0.52, -0.02, 0.48)):
    for z in (-0.3, 0.0):
        r = 0.2 if (i + (z == 0)) % 2 else 0.16
        p += [cyl([r, 0.02, r], [x, 0.98, z], '#1E1F22', 'satin', segments=16), cyl([r * 0.5, 0.025, r * 0.5], [x, 0.98, z], '#3A3D41', 'satin', segments=12)]
for x, w in ((-0.38, 0.72), (0.38, 0.72)):
    p += [box([w - 0.04, 0.56, 0.03], [x, 0.44, 0.205], ENAMEL, 'gloss', radius=0.02),
          box([w - 0.22, 0.26, 0.01], [x, 0.42, 0.222], '#2B2C2E', 'glass', radius=0.02)]
p += [cyl([0.03, 1.5, 0.03], [0, 0.8, 0.26], CHROME, 'chrome', rotate=[0, 0, 90])]
for x in (-0.7, 0.7):
    p.append(box([0.03, 0.03, 0.06], [x, 0.8, 0.23], CHROME, 'chrome', radius=0.005))
for i in range(7):
    p.append(cyl([0.045, 0.03, 0.045], [-0.6 + i * 0.2, 0.86, 0.215], CHROME, 'chrome', rotate=[90, 0, 0], segments=10))
p += [cyl([0.26, 0.14, 0.26], [-0.52, 1.06, -0.3], '#B9BDC1', 'chrome', segments=16)]          # stock pot
add('rangeCooker', p, (2, 1))

# Hob counter: a black glass induction hob in a worktop, drawers under it.
p = counter_body() + drawers(0, 1.0, CAB, 'satin', n=2)
p += [box([0.62, 0.008, 0.5], [0, 0.954, -0.1], '#16181B', 'gloss', radius=0.01)]
for x, z, r in ((-0.15, -0.22, 0.2), (0.15, -0.22, 0.16), (-0.15, 0.04, 0.16), (0.15, 0.04, 0.2)):
    p.append(torus([r, 0.006, r], [x, 0.959, z], '#5A5E63', 'gloss', segments=20))
add('cooktopCounter', p, top=counter_top('cooktopCounter'))


def fridge_french(steel=STEEL, fin='metal'):
    p = [box([0.9, 1.8, 0.7], [0, 0.92, -0.12], steel, fin, radius=0.02),
         box([0.86, 0.04, 0.66], [0, 0.02, -0.12], PLINTH, 'satin', radius=0.004)]
    for x in (-0.222, 0.222):
        p.append(box([0.436, 1.1, 0.03], [x, 1.25, 0.24], steel, fin, radius=0.015))
    p += [box([0.88, 0.58, 0.03], [0, 0.38, 0.24], steel, fin, radius=0.015),
          handle_h(0, 0.6, 0.27, CHROME, 'chrome', w=0.6)]
    for x in (-0.03, 0.03):
        p.append(box([0.022, 0.7, 0.03], [x, 1.25, 0.275], CHROME, 'chrome', radius=0.008))
    p += [box([0.16, 0.26, 0.012], [-0.25, 1.3, 0.258], '#2B2C2E', 'gloss', radius=0.02)]
    return p


add('fridgeFrenchDoor', fridge_french())

# Wine fridge under a worktop: a dark glass door with bottles, a drawer stack beside it.
p = counter_body()
p += [box([0.58, 0.76, 0.03], [-0.2, 0.5, 0.25], '#2B2C2E', 'satin', radius=0.01),
      box([0.48, 0.64, 0.01], [-0.2, 0.5, 0.268], '#3A2A2E', 'glass', radius=0.01),
      handle_v(0.06, 0.56, 0.275, CHROME, 'chrome', h=0.4)]
for row in range(5):
    for col in range(3):
        p.append(cyl([0.07, 0.3, 0.07], [-0.36 + col * 0.15, 0.2 + row * 0.13, 0.08], ['#5A1E2A', '#E3D9A0', '#5A1E2A'][(row + col) % 3], 'gloss', rotate=[90, 0, 0], segments=8))
p += drawers(0.3, 0.4, CAB, 'satin', n=3)
add('wineFridge', p, top=counter_top('wineFridge'))

# Pedal bin and recycling bins.
p = [cyl([0.3, 0.6, 0.3], [0, 0.32, -0.2], STEEL, 'metal', segments=20),
     lathe([0.31, 0.06, 0.31], [0, 0.65, -0.2], STEEL, 'metal', [[0, 0.5], [0.5, -0.5]], segments=20),
     cyl([0.3, 0.02, 0.3], [0, 0.01, -0.2], '#2B2C2E', 'satin', segments=20),
     box([0.1, 0.03, 0.08], [0, 0.03, -0.03], '#2B2C2E', 'satin', radius=0.01)]
add('trashCan', p)
p = []
for i, c in enumerate(('#3E7CC2', '#4E9A5A', '#E3B23C')):
    x = -0.31 + i * 0.31
    p += [box([0.29, 0.62, 0.42], [x, 0.31, -0.22], c, 'satin', radius=0.03),
          box([0.3, 0.04, 0.44], [x, 0.64, -0.22], c, 'satin', radius=0.015),
          box([0.14, 0.03, 0.03], [x, 0.6, 0.0], '#2B2C2E', 'satin', radius=0.01)]
add('recyclingBins', p)


def wall_cab(w=0.96, y=1.45, h=0.72, d=0.36, body=CAB, fin='satin'):
    return [box([w, h, d], [0, y + h / 2, WALL + d / 2], body, fin, radius=0.01)]


p = case(0.96, 0.72, 0.36, 0, 1.45, WALL + 0.18, CAB, 'satin', back='#C9D6DA')
for x in (-0.24, 0.24):
    p += glazed_door(x, 1.46, 0.46, 0.7, WALL + 0.37, CAB, 'satin')
for y in (1.47, 1.84):
    p.append(box([0.9, 0.02, 0.3], [0, y, WALL + 0.17], CAB, 'satin', radius=0.004))
for x in (-0.36, -0.2, -0.04):
    p.append(cyl([0.07, 0.12, 0.07], [x, 1.54, WALL + 0.17], '#DCE6EA', 'glass', segments=10))
for i in range(5):
    p.append(cyl([0.2, 0.012, 0.2], [0.24, 1.86 + i * 0.016, WALL + 0.17], PORCELAIN, 'gloss', segments=16))
p += [handle_v(-0.04, 1.6, WALL + 0.395), handle_v(0.04, 1.6, WALL + 0.395)]
add('wallCabinetGlass', p, kind='wall')

p = wall_cab(0.6, body=CAB) + [box([0.58, 0.7, 0.02], [-0.18, 1.81, WALL + 0.37], CAB, 'satin', radius=0.008)]
p[0]['at'][0] = -0.18
p += [handle_v(0.06, 1.6, WALL + 0.395)]
for y in (1.46, 1.81, 2.15):
    p.append(box([0.36, 0.025, 0.34], [0.3, y, WALL + 0.17], OAK, 'wood', radius=0.006))      # open end shelves
p += [box([0.03, 0.72, 0.36], [0.46, 1.81, WALL + 0.18], OAK, 'wood', radius=0.006)]
p += [cyl([0.1, 0.14, 0.1], [0.24, 1.53, WALL + 0.16], SAGE, 'gloss', taper=0.8), cyl([0.14, 0.05, 0.14], [0.3, 1.86, WALL + 0.14], PORCELAIN, 'gloss')]
add('wallCabinetCorner', p, kind='wall')

p = [box([0.52, 0.44, 0.02], [0, 1.52, WALL + 0.01], OAK, 'wood', radius=0.006)]
for y in (1.33, 1.47, 1.61):
    p += [box([0.5, 0.015, 0.09], [0, y, WALL + 0.055], OAK, 'wood', radius=0.003),
          box([0.5, 0.012, 0.012], [0, y + 0.06, WALL + 0.1], BRASS, 'metal', radius=0.003)]
    for i in range(6):
        p += jar(-0.2 + i * 0.08, WALL + 0.055, y + 0.008, ['#C98F4B', '#7A8C3A', '#B5452E', '#E3C66A', '#5E4B3A', '#9C6B43'][(i + int(y * 10)) % 6], d=0.05, h=0.08, lid=BLACKMETAL, lid_fin='metal')
add('spiceRack', p, kind='wall')

p = [cyl([0.025, 0.9, 0.025], [0, 1.6, WALL + 0.08], BLACKMETAL, 'metal', rotate=[0, 0, 90])]
for x in (-0.44, 0.44):
    p.append(box([0.03, 0.03, 0.1], [x, 1.6, WALL + 0.04], BLACKMETAL, 'metal', radius=0.006))
for x, kind in ((-0.32, 'pan'), (-0.12, 'ladle'), (0.06, 'pot'), (0.28, 'pan'), (0.4, 'ladle')):
    p.append(torus([0.04, 0.008, 0.04], [x, 1.58, WALL + 0.08], BLACKMETAL, 'metal', rotate=[90, 90, 0], segments=8))
    if kind == 'pan':
        p += [cyl([0.03, 0.22, 0.012], [x, 1.43, WALL + 0.08], BLACKMETAL, 'metal'),
              cyl([0.26, 0.04, 0.26], [x, 1.2, WALL + 0.06], '#B87333', 'metal', rotate=[90, 0, 0], segments=18)]
    elif kind == 'pot':
        p += [cyl([0.02, 0.1, 0.02], [x, 1.5, WALL + 0.08], BLACKMETAL, 'metal'),
              cyl([0.2, 0.16, 0.2], [x, 1.37, WALL + 0.12], '#B87333', 'metal', segments=16)]
    else:
        p += [cyl([0.014, 0.3, 0.014], [x, 1.42, WALL + 0.08], STEEL, 'metal'),
              sph([0.07, 0.04, 0.07], [x, 1.27, WALL + 0.09], STEEL, 'metal', segments=8)]
add('potRail', p, kind='wall')

p = [box([0.9, 0.03, 0.24], [0, y, WALL + 0.12], OAK, 'wood', radius=0.004) for y in (1.4, 1.75)]
p += [box([0.03, 0.12, 0.2], [x, 1.35, WALL + 0.1], OAKD, 'wood', radius=0.004) for x in (-0.36, 0.36)]
p += jar(-0.32, WALL + 0.12, 1.415, '#E3C66A') + jar(-0.2, WALL + 0.12, 1.415, '#F2F0EA', h=0.16) + jar(-0.08, WALL + 0.12, 1.415, '#C98F4B', d=0.08)
p += [cyl([0.22, 0.012, 0.22], [0.22, 1.42 + i * 0.014, WALL + 0.12], PORCELAIN, 'gloss', segments=16) for i in range(4)]
p += [cyl([0.1, 0.1, 0.1], [-0.25, 1.815, WALL + 0.12], TEAL, 'gloss'), cyl([0.1, 0.1, 0.1], [-0.12, 1.815, WALL + 0.12], CREAM, 'gloss')]
p += plant_pot(0.25, WALL + 0.12, 1.765, PORCELAIN, d=0.12, h=0.1, tall=0.18, soil=False)
add('kitchenShelfWall', p, kind='wall')

# Retro fridge: rounded mint body, chrome lever handle.
p = [box([0.72, 1.56, 0.66], [0, 0.86, -0.13], '#BFE0D2', 'gloss', radius=0.09),
     box([0.66, 0.08, 0.6], [0, 0.04, -0.13], '#C9CBCC', 'chrome', radius=0.01),
     box([0.68, 0.012, 0.02], [0, 1.18, 0.205], '#A9CFC0', 'gloss', radius=0.004),
     box([0.06, 0.24, 0.05], [0.27, 1.0, 0.23], CHROME, 'chrome', radius=0.02),
     box([0.18, 0.05, 0.012], [0, 1.5, 0.205], CHROME, 'chrome', radius=0.01)]
add('fridgeRetro', p)

# Compact dishwasher: a stainless door under the worktop.
p = counter_body()
p += [box([0.92, 0.66, 0.03], [0, 0.46, 0.25], STEEL, 'metal', radius=0.01),
      box([0.92, 0.1, 0.03], [0, 0.84, 0.25], '#2B2C2E', 'satin', radius=0.008),
      handle_h(0, 0.74, 0.275, CHROME, 'chrome', w=0.5),
      sph([0.02, 0.02, 0.01], [0.36, 0.84, 0.268], '#5EC28B', 'satin', segments=6)]
add('dishwasherCompact', p, top=counter_top('dishwasherCompact'))


# =============================================================================================
# Dining: chair-less tables, chairs, benches, stools, sideboards
# =============================================================================================
def table_rect(w, d, top, legc, top_fin='wood', leg_fin='wood', h=TT, t=0.045, leg=0.06, inset=0.06, round_legs=False, taper=None, apron=True):
    p = [box([w, t, d], [0, h - t / 2, 0], top, top_fin, radius=0.012)]
    lx, lz = w / 2 - inset - leg / 2, d / 2 - inset - leg / 2
    if apron:
        p += [box([w - 2 * inset - 0.02, 0.08, 0.025], [0, h - t - 0.04, s * lz], legc, leg_fin, radius=0.005) for s in (-1, 1)]
        p += [box([0.025, 0.08, d - 2 * inset - 0.02], [s * lx, h - t - 0.04, 0], legc, leg_fin, radius=0.005) for s in (-1, 1)]
    p += legs4((-lx, lx), (-lz, lz), h - t, leg, legc, leg_fin, round_=round_legs, taper=taper)
    if w > 2.6:                                                                                     # middle legs on long tables
        p += legs4((0,), (-lz, lz), h - t, leg, legc, leg_fin, round_=round_legs, taper=taper)
    return p


def table_round(dia, top, base, top_fin='wood', base_fin='metal', h=TT, foot=0.5, t=0.04):
    return [cyl([dia, t, dia], [0, h - t / 2, 0], top, top_fin, segments=32),
            cyl([dia * 0.3, 0.04, dia * 0.3], [0, h - t - 0.02, 0], base, base_fin, segments=16),
            cyl([0.08, h - t - 0.06, 0.08], [0, (h - t) / 2, 0], base, base_fin, segments=14),
            lathe([foot, 0.07, foot], [0, 0.035, 0], base, base_fin, [[0, -0.5], [0.5, -0.5], [0.5, -0.2], [0.15, 0.5], [0, 0.5]], segments=24)]


def round_top(key, dia, h, pad=0.0):
    r = dia / 2 / math.sqrt(2) - pad
    return (-r, r, -r, r, h)


add('tableSquare2', table_rect(0.82, 0.82, OAK, OAKD, round_legs=True, taper=0.75), top=(-0.41, 0.41, -0.41, 0.41, TT))
add('tableBistroRound', table_round(0.72, '#F1F0EC', BLACKMETAL, 'gloss'), top=round_top('tableBistroRound', 0.72, TT))
add('tableDining4', table_rect(1.84, 0.88, OAK, OAK, leg=0.065), (2, 1), top=(-0.92, 0.92, -0.44, 0.44, TT))
add('tableDining6', table_rect(2.84, 0.92, WALNUT, WALNUT, leg=0.07, round_legs=True, taper=0.7), (3, 1), top=(-1.42, 1.42, -0.46, 0.46, TT))
add('tableBanquet8', table_rect(3.84, 0.94, '#8A5A36', '#6B4A33', leg=0.08), (4, 1), top=(-1.92, 1.92, -0.47, 0.47, TT))
p = table_round(1.52, BIRCH, BIRCH, 'wood', 'wood', foot=0.6)
p[2] = cyl([0.22, TT - 0.12, 0.22], [0, (TT - 0.08) / 2 + 0.03, 0], BIRCH, 'wood', taper=0.7, segments=18)
add('tableRound4', p, (2, 2), top=round_top('tableRound4', 1.52, TT))
p = table_rect(1.86, 1.86, '#F1F0EC', BLACKMETAL, 'satin', 'metal', leg=0.06, inset=0.1, apron=False)
p += [box([1.5, 0.03, 0.03], [0, 0.66, s * 0.8], BLACKMETAL, 'metal', radius=0.006) for s in (-1, 1)]
p += [box([0.03, 0.03, 1.5], [s * 0.8, 0.66, 0], BLACKMETAL, 'metal', radius=0.006) for s in (-1, 1)]
add('tableSquare8', p, (2, 2), top=(-0.93, 0.93, -0.93, 0.93, TT))

# Farmhouse table: a thick planked top on trestle legs with a stretcher.
FARM, FARMD = '#B48A5E', '#8E6A48'
p = [box([2.86, 0.06, 0.94], [0, TT - 0.03, 0], FARM, 'wood', radius=0.01)]
for z in (-0.31, 0.0, 0.31):
    p.append(box([2.86, 0.004, 0.006], [0, TT + 0.001, z + 0.155], '#7A5A3A', 'wood', radius=0.001))
for x in (-1.15, 1.15):
    p += [box([0.1, 0.6, 0.1], [x, 0.37, 0], FARMD, 'wood', radius=0.012),
          box([0.1, 0.07, 0.8], [x, 0.035, 0], FARMD, 'wood', radius=0.012),
          box([0.1, 0.06, 0.78], [x, TT - 0.09, 0], FARMD, 'wood', radius=0.012)]
p.append(box([2.3, 0.08, 0.06], [0, 0.32, 0], FARMD, 'wood', radius=0.01))
add('tableFarmhouse6', p, (3, 1), top=(-1.43, 1.43, -0.47, 0.47, TT))

# Bar-height café table (for bar stools).
p = table_round(0.7, '#2B2C2E', BLACKMETAL, 'satin', 'metal', h=BT, foot=0.5)
p.append(torus([0.4, 0.025, 0.4], [0, 0.3, 0], BLACKMETAL, 'metal', segments=20))
add('tableBarHigh', p, top=round_top('tableBarHigh', 0.7, BT))

SZ = 0.05        # chairs: the seat's centre sits a little forward, towards the table


def chair_frame(wood, fin, seat_h=0.46, w=0.44, d=0.42, leg=0.035, round_=False, taper=None, back_h=0.9):
    """Four legs, the back legs rising into the back's posts (back at -Z)."""
    z0, z1, x = SZ - d / 2 + 0.03, SZ + d / 2 - 0.03, w / 2 - 0.03
    kw = {'taper': taper} if taper else {}
    if round_:
        p = [cyl([leg, seat_h - 0.03, leg], [s * x, (seat_h - 0.03) / 2, z1], wood, fin, segments=10, **kw) for s in (-1, 1)]
        p += [rod([s * x, 0, z0 - 0.03], [s * x, back_h, z0 + 0.01], leg, wood, fin, segments=10) for s in (-1, 1)]
    else:
        p = [box([leg, seat_h - 0.03, leg], [s * x, (seat_h - 0.03) / 2, z1], wood, fin, radius=0.006) for s in (-1, 1)]
        p += [box([leg, back_h, leg], [s * x, back_h / 2, z0], wood, fin, radius=0.006, rotate=[-4, 0, 0]) for s in (-1, 1)]
    return p


def chair_wood(wood=HONEY, seat=None):
    p = chair_frame(wood, 'wood')
    p += [box([0.44, 0.04, 0.42], [0, 0.45, SZ], wood, 'wood', radius=0.014),
          box([0.4, 0.1, 0.03], [0, 0.84, SZ - 0.18], wood, 'wood', radius=0.012, rotate=[-4, 0, 0]),
          box([0.4, 0.05, 0.025], [0, 0.6, SZ - 0.18], wood, 'wood', radius=0.01, rotate=[-4, 0, 0])]
    for x in (-0.09, 0.0, 0.09):
        p.append(box([0.03, 0.22, 0.02], [x, 0.71, SZ - 0.186], wood, 'wood', radius=0.006, rotate=[-4, 0, 0]))
    p += [box([0.36, 0.025, 0.025], [0, 0.18, SZ + s * 0.18], wood, 'wood', radius=0.005) for s in (-1, 1)]
    if seat: p.append(cushion([0.38, 0.035, 0.36], [0, 0.485, SZ + 0.01], seat, radius=0.015))
    return p


def chair_upholstered(fabric, legs):
    p = [cushion([0.46, 0.09, 0.44], [0, 0.44, SZ + 0.01], fabric, radius=0.03),
         cushion([0.44, 0.5, 0.07], [0, 0.74, SZ - 0.19], fabric, radius=0.03, rotate=[-7, 0, 0])]
    for x in (-0.19, 0.19):
        p += [cyl([0.04, 0.4, 0.04], [x, 0.2, SZ + 0.17], legs, 'wood', taper=0.6, segments=10),
              rod([x, 0, SZ - 0.24], [x, 0.4, SZ - 0.17], 0.04, legs, 'wood', segments=10)]
    return p


def chair_modern(shell, legs):
    p = [box([0.46, 0.04, 0.44], [0, 0.45, SZ + 0.01], shell, 'gloss', radius=0.02),
         box([0.46, 0.36, 0.04], [0, 0.66, SZ - 0.2], shell, 'gloss', radius=0.02, rotate=[-10, 0, 0]),
         box([0.04, 0.2, 0.36], [-0.22, 0.52, SZ - 0.02], shell, 'gloss', radius=0.02),
         box([0.04, 0.2, 0.36], [0.22, 0.52, SZ - 0.02], shell, 'gloss', radius=0.02)]
    for x in (-0.17, 0.17):                                                                       # sled frame
        p += [box([0.025, 0.43, 0.025], [x, 0.215, SZ + 0.17], legs, 'metal', radius=0.008),
              box([0.025, 0.43, 0.025], [x, 0.215, SZ - 0.17], legs, 'metal', radius=0.008),
              box([0.025, 0.025, 0.36], [x, 0.012, SZ], legs, 'metal', radius=0.008)]
    return p


def chair_windsor(wood):
    p = [lathe([0.46, 0.05, 0.44], [0, 0.44, SZ], wood, 'wood', [[0, -0.5], [0.5, -0.5], [0.5, 0.3], [0.44, 0.5], [0, 0.4]], segments=20)]
    for sx in (-1, 1):
        for sz in (-1, 1):
            p.append(rod([sx * 0.22, 0, SZ + sz * 0.22], [sx * 0.15, 0.42, SZ + sz * 0.14], 0.035, wood, 'wood', segments=10))
    p.append(box([0.3, 0.025, 0.025], [0, 0.17, SZ], wood, 'wood', radius=0.008, rotate=[0, 90, 0]))
    for i in range(7):
        x = -0.15 + i * 0.05
        p.append(rod([x, 0.45, SZ - 0.15], [x * 1.25, 0.94, SZ - 0.2], 0.018, wood, 'wood', segments=8))
    p.append(torus([0.46, 0.04, 0.16], [0, 0.95, SZ - 0.18], wood, 'wood', segments=24))
    return p


def chair_bentwood(wood, seat):
    p = [cyl([0.42, 0.04, 0.42], [0, 0.45, SZ], wood, 'wood', segments=24),
         cyl([0.36, 0.012, 0.36], [0, 0.474, SZ], seat, 'fabric', segments=20),
         torus([0.34, 0.025, 0.34], [0, 0.2, SZ], wood, 'wood', segments=20)]
    for sx, sz in ((-1, 1), (1, 1)):
        p.append(rod([sx * 0.18, 0, SZ + sz * 0.2], [sx * 0.14, 0.44, SZ + sz * 0.13], 0.032, wood, 'wood', segments=10))
    for sx in (-1, 1):
        p.append(rod([sx * 0.17, 0, SZ - 0.2], [sx * 0.15, 0.92, SZ - 0.17], 0.03, wood, 'wood', segments=10))
    p += [torus([0.32, 0.03, 0.44], [0, 0.73, SZ - 0.17], wood, 'wood', rotate=[90, 0, 0], segments=24),
          torus([0.18, 0.02, 0.24], [0, 0.66, SZ - 0.17], wood, 'wood', rotate=[90, 0, 0], segments=20)]
    return p


def chair_folding(frame, seat):
    p = [box([0.42, 0.03, 0.38], [0, 0.45, SZ + 0.02], seat, 'satin', radius=0.012),
         box([0.4, 0.2, 0.025], [0, 0.78, SZ - 0.19], seat, 'satin', radius=0.012, rotate=[-8, 0, 0])]
    for x in (-0.2, 0.2):
        p += [rod([x, 0, SZ + 0.22], [x, 0.88, SZ - 0.2], 0.022, frame, 'metal', segments=8),
              rod([x, 0, SZ - 0.2], [x, 0.44, SZ + 0.12], 0.022, frame, 'metal', segments=8),
              box([0.03, 0.02, 0.06], [x, 0.01, SZ + 0.22], '#2B2C2E', 'satin', radius=0.006),
              box([0.03, 0.02, 0.06], [x, 0.01, SZ - 0.2], '#2B2C2E', 'satin', radius=0.006)]
    p.append(cyl([0.02, 0.4, 0.02], [0, 0.1, SZ + 0.15], frame, 'metal', rotate=[0, 0, 90], segments=8))
    return p


def chair_arm(wood, seat):
    p = chair_frame(wood, 'wood', w=0.54, back_h=0.92)
    p += [box([0.54, 0.05, 0.44], [0, 0.44, SZ], wood, 'wood', radius=0.012),
          cushion([0.46, 0.06, 0.4], [0, 0.49, SZ + 0.01], seat, radius=0.02),
          cushion([0.44, 0.32, 0.05], [0, 0.72, SZ - 0.17], seat, radius=0.02, rotate=[-6, 0, 0]),
          box([0.5, 0.06, 0.035], [0, 0.89, SZ - 0.19], wood, 'wood', radius=0.012, rotate=[-4, 0, 0])]
    for x in (-0.255, 0.255):
        p += [box([0.06, 0.03, 0.46], [x, 0.66, SZ - 0.02], wood, 'wood', radius=0.01),
              box([0.03, 0.2, 0.03], [x, 0.55, SZ + 0.18], wood, 'wood', radius=0.006)]
    return p


add('chairWood', chair_wood(HONEY), seat=(0.47, SZ))
add('chairUpholstered', chair_upholstered('#B7A48A', WALNUT), seat=(0.48, SZ))
add('chairModern', chair_modern('#E8E4DC', BLACKMETAL), seat=(0.47, SZ))
add('chairWindsor', chair_windsor('#6B4A33'), seat=(0.47, SZ))
add('chairBentwood', chair_bentwood('#3B2A20', '#8E2A2A'), seat=(0.48, SZ))
add('chairFolding', chair_folding(STEEL, '#4E7A7F'), seat=(0.47, SZ))
add('chairArm', chair_arm(OAK, SAGE), seat=(0.5, SZ))

# Dining bench (2x1, two seats): a plank on splayed legs.
p = [box([1.6, 0.05, 0.36], [0, 0.44, SZ], OAK, 'wood', radius=0.012),
     box([1.4, 0.06, 0.03], [0, 0.39, SZ], OAKD, 'wood', radius=0.008)]
for x in (-0.68, 0.68):
    for z in (-0.12, 0.12):
        p.append(splayed_leg(x, SZ + z, 0.42, 0.045, OAKD, 'wood', out=0.04))
add('benchDining', p, (2, 1), seat=(0.47, SZ))


def stool(seat_h, top, legs, back=False, fin='wood'):
    p = [cyl([0.38, 0.05, 0.38], [0, seat_h - 0.025, SZ], legs, fin, segments=20),
         cushion([0.36, 0.05, 0.36], [0, seat_h + 0.01, SZ], top, radius=0.03) if back else cyl([0.36, 0.04, 0.36], [0, seat_h + 0.005, SZ], top, 'fabric', segments=20)]
    for a in (45, 135, 225, 315):
        r = math.radians(a)
        p.append(rod([math.cos(r) * 0.22, 0, SZ + math.sin(r) * 0.22], [math.cos(r) * 0.13, seat_h - 0.04, SZ + math.sin(r) * 0.13], 0.035, legs, fin, segments=8))
    p.append(torus([0.36, 0.025, 0.36], [0, seat_h * 0.36, SZ], legs, fin, segments=20))
    if back:
        p += [box([0.36, 0.14, 0.03], [0, seat_h + 0.24, SZ - 0.17], legs, fin, radius=0.02, rotate=[-8, 0, 0]),
              rod([-0.13, seat_h, SZ - 0.15], [-0.15, seat_h + 0.2, SZ - 0.17], 0.025, legs, fin, segments=8),
              rod([0.13, seat_h, SZ - 0.15], [0.15, seat_h + 0.2, SZ - 0.17], 0.025, legs, fin, segments=8)]
    return p


add('stoolCounter', stool(0.66, SAGE, OAK), seat=(0.68, SZ))
add('stoolBarHigh', stool(0.77, COGNAC, BLACKMETAL, back=True, fin='metal'), seat=(0.8, SZ))

# High chair: splayed legs, a tall seat with arms and a footrest.
p = [box([0.38, 0.05, 0.36], [0, 0.58, SZ], WHITE, 'satin', radius=0.02),
     cushion([0.32, 0.04, 0.3], [0, 0.62, SZ + 0.01], '#9DC6CF', radius=0.015),
     box([0.36, 0.32, 0.04], [0, 0.78, SZ - 0.17], WHITE, 'satin', radius=0.03, rotate=[-6, 0, 0]),
     box([0.32, 0.03, 0.12], [0, 0.32, SZ + 0.18], WHITE, 'satin', radius=0.01)]
for x in (-0.17, 0.17):
    p += [box([0.04, 0.16, 0.32], [x, 0.7, SZ], WHITE, 'satin', radius=0.015)]
for sx in (-1, 1):
    for sz in (-1, 1):
        p.append(rod([sx * 0.26, 0, SZ + sz * 0.26], [sx * 0.15, 0.56, SZ + sz * 0.14], 0.035, BIRCH, 'wood', segments=10))
add('highChair', p, seat=(0.62, SZ))


def sideboard(w, h, body, fin, fronts, legs, top_col=None, handle=BRASS, d=0.62):
    """Back against the wall (z = -0.47): a long cabinet on legs."""
    z = -0.47 + d / 2
    leg_h = 0.16
    p = [box([w, h - leg_h, d], [0, leg_h + (h - leg_h) / 2, z], body, fin, radius=0.012),
         box([w + 0.02, 0.03, d + 0.02], [0, h - 0.015, z], top_col or body, fin, radius=0.008)]
    p += legs4((-w / 2 + 0.08, w / 2 - 0.08), (z - d / 2 + 0.06, z + d / 2 - 0.06), leg_h, 0.04, legs, 'wood', round_=True, taper=0.7)
    p += fronts(z + d / 2 + 0.005, leg_h + 0.02, h - 0.05, handle)
    return p


def slatted(n, w):
    def f(zf, y0, y1, handle):
        p, dw = [], (w - 0.04) / n
        for i in range(n):
            x = -w / 2 + 0.02 + dw * (i + 0.5)
            p.append(box([dw - 0.012, y1 - y0, 0.02], [x, (y0 + y1) / 2, zf], WALNUT, 'wood', radius=0.006))
            for k in range(-3, 4):
                p.append(box([0.012, y1 - y0 - 0.06, 0.008], [x + k * dw / 8, (y0 + y1) / 2, zf + 0.012], '#5E3F28', 'wood', radius=0.002))
            p.append(sph([0.03, 0.03, 0.02], [x + (dw / 2 - 0.05) * (1 if i % 2 else -1), (y0 + y1) / 2, zf + 0.02], handle, 'metal', segments=8))
        return p
    return f


p = sideboard(1.8, 0.8, WALNUT, 'wood', slatted(4, 1.8), WALNUT)
p += [cyl([0.14, 0.3, 0.14], [-0.6, 0.95, -0.3], CREAM, 'gloss', taper=0.6)]
add('sideboard', p, (2, 1), top=(-0.91, 0.91, -0.48, 0.16, 0.8))


def buffet_front(zf, y0, y1, handle):
    p, ym = [], y1 - 0.16
    for x in (-0.6, 0.0, 0.6):
        p += [box([0.56, 0.13, 0.02], [x, ym + 0.075, zf], '#8FA27E', 'satin', radius=0.006),
              handle_h(x, ym + 0.075, zf + 0.02, BRASS, 'metal', w=0.14)]
        p += doors(x, 0.58, '#8FA27E', 'satin', handle=BRASS, hfin='metal', y0=y0, y1=ym - 0.01, z=zf, n=1 if x == 0 else 2, knob=True)
    return p


p = sideboard(1.84, 0.9, '#9DAF8C', 'satin', buffet_front, OAK, top_col=OAK)
p[1] = box([1.88, 0.035, 0.66], [0, 0.885, -0.16], OAK, 'wood', radius=0.008)
add('buffet', p, (2, 1), top=(-0.94, 0.94, -0.49, 0.17, 0.9))

# China cabinet: a glazed hutch with plates on show over a two-door base.
p = [box([0.94, 0.86, 0.48], [0, 0.47, -0.22], '#F1EDE4', 'satin', radius=0.012),
     box([0.98, 0.04, 0.52], [0, 0.92, -0.21], '#F1EDE4', 'satin', radius=0.008),
     box([0.98, 0.06, 0.42], [0, 2.07, -0.27], '#F1EDE4', 'satin', radius=0.01)]
p += case(0.9, 1.1, 0.36, 0, 0.94, -0.28, '#F1EDE4', 'satin', back='#9DB7C9')
p += [
     box([0.94, 0.04, 0.48], [0, 0.02, -0.22], '#B9A890', 'satin', radius=0.006)]
p += doors(0, 0.94, '#F1EDE4', 'satin', handle=BRASS, hfin='metal', y0=0.08, y1=0.82, z=0.03, knob=True)
for x in (-0.22, 0.22):
    p += glazed_door(x, 0.96, 0.43, 1.06, -0.09, '#F1EDE4', 'satin')
for y in (1.17, 1.52, 1.86):
    p.append(box([0.84, 0.02, 0.3], [0, y - 0.13, -0.29], '#F1EDE4', 'satin', radius=0.004))
    for x in (-0.3, -0.1, 0.1, 0.3):
        p.append(cyl([0.18, 0.015, 0.18], [x, y - 0.02, -0.34], [PORCELAIN, '#9DC6CF'][int(x * 10 + y * 10) % 2], 'gloss', rotate=[78, 0, 0], segments=16))
add('chinaCabinet', p)

# Bar cart: two brass-framed glass tiers on castors, bottles and glasses.
p = []
for y in (0.3, 0.76):
    p += [box([0.72, 0.012, 0.42], [0, y, 0], '#DCE6EA', 'glass', radius=0.004),
          torus([0.76, 0.02, 0.46], [0, y + 0.02, 0], BRASS, 'metal', segments=28)]
for x in (-0.34, 0.34):
    for z in (-0.18, 0.18):
        p += [cyl([0.022, 0.78, 0.022], [x, 0.43, z], BRASS, 'metal', segments=8),
              sph([0.05, 0.05, 0.05], [x, 0.025, z], '#2B2C2E', 'satin', segments=8)]
p += [rod([-0.36, 0.84, -0.2], [-0.36, 0.84, 0.2], 0.024, BRASS, 'metal')]
for i, (x, c) in enumerate(((-0.2, '#2F5A3A'), (-0.08, '#7A2E2E'), (0.04, '#C9A64A'))):
    p.append(lathe([0.08, 0.3, 0.08], [x, 0.92, -0.08], c, 'glass', BOTTLE, segments=12))
p += wine_glass(0.18, 0.08, 0.77) + wine_glass(0.26, 0.0, 0.77)
p += [lathe([0.14, 0.16, 0.14], [0.0, 0.39, 0.0], '#DCE6EA', 'glass', VASE, segments=14), cyl([0.18, 0.1, 0.18], [0.2, 0.36, 0.05], '#C9A979', 'fabric')]
add('barCart', p)


# =============================================================================================
# Living: sectional modules, sofas, chairs, tables, shelves, fireplace
# =============================================================================================
SOFA_H = 0.44


def sofa_run(w, fabric, legs, arms=(), backs=('back',), depth=0.88, x=0.0, z=-0.04, cushions=None, leg_fin='wood', pillows=()):
    """A sofa (or a module of one) `w` wide centred on x: a base, seat and back cushions, arms on the
    `arms` sides (-1 = -X, 1 = +X) and back rests along -Z ('back') and/or a side ('left' = -X, 'right' = +X)."""
    z0, z1 = z - depth / 2, z + depth / 2
    p = [box([w, 0.22, depth], [x, 0.21, z], fabric, 'fabric', radius=0.04)]
    sx0, sx1, sz0 = x - w / 2, x + w / 2, z0
    if 'back' in backs:
        p.append(box([w, 0.5, 0.2], [x, 0.57, z0 + 0.1], fabric, 'fabric', radius=0.06))
        sz0 = z0 + 0.2
    for side in backs:
        if side in ('left', 'right'):
            s = -1 if side == 'left' else 1
            p.append(box([0.2, 0.5, depth], [x + s * (w / 2 - 0.1), 0.57, z], fabric, 'fabric', radius=0.06))
            if s < 0: sx0 += 0.2
            else: sx1 -= 0.2
    for s in arms:
        p.append(box([0.17, 0.3, depth], [x + s * (w / 2 - 0.085), 0.47, z], fabric, 'fabric', radius=0.07))
        if s < 0: sx0 += 0.17
        else: sx1 -= 0.17
    n = cushions or max(1, round((sx1 - sx0) / 0.9))
    cw = (sx1 - sx0) / n
    for i in range(n):
        cx = sx0 + cw * (i + 0.5)
        p.append(cushion([cw - 0.02, 0.13, z1 - sz0 - 0.02], [cx, 0.385, (sz0 + z1) / 2 + 0.005], fabric))
        if 'back' in backs:
            p.append(cushion([cw - 0.03, 0.4, 0.16], [cx, 0.62, sz0 + 0.07], fabric, rotate=[-9, 0, 0]))
    for side in backs:
        if side in ('left', 'right'):
            s = -1 if side == 'left' else 1
            p.append(cushion([0.16, 0.4, depth - 0.24], [x + s * (w / 2 - 0.27), 0.62, z + 0.1], fabric, rotate=[0, 0, s * 9]))
    for px, pz, c, ry in pillows:
        p.append(cushion([0.4, 0.36, 0.12], [px, 0.66, pz], c, rotate=[-12, ry, 0]))
    for lx in (x - w / 2 + 0.06, x + w / 2 - 0.06):
        for lz in (z0 + 0.06, z1 - 0.06):
            p.append(cyl([0.045, 0.1, 0.045], [lx, 0.05, lz], legs, leg_fin, taper=0.8, segments=8))
    return p


SECT, SECT_LEG = '#A9A49A', '#3B302A'
add('sectionalMiddle', sofa_run(1.0, SECT, SECT_LEG), seat=(SOFA_H, 0.08))
add('sectionalArmLeft', sofa_run(1.0, SECT, SECT_LEG, arms=(1,)), seat=(SOFA_H, 0.08))
add('sectionalArmRight', sofa_run(1.0, SECT, SECT_LEG, arms=(-1,)), seat=(SOFA_H, 0.08))
add('sectionalCorner', sofa_run(1.0, SECT, SECT_LEG, backs=('back', 'right'), depth=0.96, z=-0.02,
                                pillows=[(0.18, -0.24, MUSTARD, -35)]), seat=(SOFA_H, 0.1))
# Chaise (1x2): a long seat with a back at -Z and an arm along +X.
p = [box([0.96, 0.22, 1.9], [0, 0.21, 0.0], SECT, 'fabric', radius=0.04),
     box([0.96, 0.5, 0.2], [0, 0.57, -0.85], SECT, 'fabric', radius=0.06),
     box([0.17, 0.3, 1.9], [0.395, 0.47, 0.0], SECT, 'fabric', radius=0.07),
     cushion([0.76, 0.13, 1.66], [-0.08, 0.385, 0.1], SECT),
     cushion([0.74, 0.4, 0.16], [-0.08, 0.62, -0.68], SECT, rotate=[-9, 0, 0]),
     cushion([0.5, 0.06, 0.6], [-0.06, 0.475, 0.55], '#D9CFC0', radius=0.03)]                     # throw
for lx in (-0.42, 0.42):
    for lz in (-0.88, 0.88):
        p.append(cyl([0.045, 0.1, 0.045], [lx, 0.05, lz], SECT_LEG, 'wood', taper=0.8, segments=8))
add('chaise', p, (1, 2), seat=(SOFA_H, -0.3))

# Sofa bed: a boxy sofa with a visible fold line.
p = sofa_run(1.86, '#6F7F8C', '#2B2C2E', arms=(-1, 1), cushions=1, leg_fin='metal', pillows=[(-0.55, -0.22, '#E4B8AE', 10), (0.55, -0.22, CREAM, -10)])
p.append(box([1.5, 0.006, 0.02], [0, 0.453, 0.12], '#5D6A75', 'fabric', radius=0.002))
add('sofaBed', p, (2, 1), seat=(SOFA_H, 0.08))
add('sofaThreeSeat', sofa_run(2.86, '#7A8C6E', OAK, arms=(-1, 1), cushions=3, pillows=[(-1.0, -0.22, CREAM, 14), (1.0, -0.22, MUSTARD, -14)]),
    (3, 1), seat=(SOFA_H, 0.08))

# Recliner: a chunky armchair with its footrest up.
REC = '#7A5A44'
p = sofa_run(0.92, REC, '#2B2C2E', arms=(-1, 1), depth=0.74, z=-0.12, leg_fin='satin')
p += [cushion([0.54, 0.13, 0.24], [0, 0.36, 0.35], REC, rotate=[16, 0, 0]),
      box([0.4, 0.04, 0.16], [0, 0.27, 0.27], '#2B2C2E', 'satin', radius=0.01, rotate=[40, 0, 0])]
add('recliner', p, seat=(SOFA_H, 0.0))

# Rocking chair: spindle back, curved rockers.
RK = '#8A5A36'
p = [box([0.5, 0.04, 0.46], [0, 0.44, 0.04], RK, 'wood', radius=0.012),
     cushion([0.44, 0.05, 0.4], [0, 0.48, 0.05], '#C9B79C', radius=0.02)]
for x in (-0.22, 0.22):
    R = 1.0
    for i in range(6):
        a0, a1 = math.radians(-26 + i * 52 / 6), math.radians(-26 + (i + 1) * 52 / 6)
        p.append(rod([x, R + 0.025 - R * math.cos(a0), 0.05 + R * math.sin(a0)], [x, R + 0.025 - R * math.cos(a1), 0.05 + R * math.sin(a1)], 0.05, RK, 'wood', shape='box', t2=0.03, radius=0.008))
    p += [rod([x, 0.06, 0.26], [x, 0.44, 0.22], 0.035, RK, 'wood', segments=8),
          rod([x, 0.06, -0.18], [x * 1.05, 1.02, -0.3], 0.035, RK, 'wood', segments=8),
          box([0.05, 0.03, 0.42], [x * 1.08, 0.66, 0.02], RK, 'wood', radius=0.01),
          rod([x * 1.08, 0.44, 0.2], [x * 1.08, 0.66, 0.2], 0.03, RK, 'wood', segments=8)]
for i in range(6):
    xx = -0.15 + i * 0.06
    p.append(rod([xx, 0.46, -0.17], [xx, 0.98, -0.28], 0.02, RK, 'wood', segments=8))
p.append(box([0.48, 0.07, 0.04], [0, 1.0, -0.29], RK, 'wood', radius=0.015, rotate=[-12, 0, 0]))
add('rockingChair', p, seat=(0.48, 0.05))

p = [sph([0.84, 0.46, 0.84], [0, 0.23, 0.02], '#C25B3E', 'fabric', segments=20),
     sph([0.72, 0.56, 0.4], [0, 0.42, -0.2], '#C25B3E', 'fabric', segments=20, rotate=[-12, 0, 0]),
     sph([0.5, 0.12, 0.4], [0, 0.42, 0.05], '#B5523A', 'fabric', segments=16)]
add('beanBag', p, seat=(0.3, 0.05))
p = [cyl([0.62, 0.36, 0.62], [0, 0.22, 0], '#3E5470', 'fabric', segments=24),
     cyl([0.6, 0.06, 0.6], [0, 0.41, 0], '#3E5470', 'fabric', segments=24)]
for a in range(0, 360, 60):
    r = math.radians(a)
    p.append(sph([0.025, 0.02, 0.025], [math.cos(r) * 0.15, 0.442, math.sin(r) * 0.15], '#2E3F56', 'fabric', segments=6))
p += [cyl([0.05, 0.04, 0.05], [x, 0.02, z], WALNUT, 'wood') for x, z in ((-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2))]
add('ottoman', p, seat=(0.45, 0.0))
p = [cyl([0.62, 0.12, 0.62], [0, 0.06, 0], '#D6A64A', 'fabric', segments=24),
     torus([0.62, 0.05, 0.62], [0, 0.1, 0], '#C9963D', 'fabric', segments=24),
     sph([0.04, 0.03, 0.04], [0, 0.13, 0], '#B5853A', 'fabric', segments=8)]
add('floorCushion', p, seat=(0.14, 0.0))

# Wingback armchair.
p = sofa_run(0.84, '#2F5A4E', WALNUT, arms=(-1, 1), depth=0.8, z=-0.06)
p += [box([0.16, 0.4, 0.3], [s * 0.34, 0.86, -0.3], '#2F5A4E', 'fabric', radius=0.06) for s in (-1, 1)]
p.append(box([0.84, 0.3, 0.18], [0, 0.95, -0.37], '#2F5A4E', 'fabric', radius=0.07))
add('armchairWingback', p, seat=(SOFA_H, 0.04))

# Coffee and side tables (tops things stand on).
p = [cyl([0.84, 0.04, 0.84], [0, 0.4, 0], '#E8E2D6', 'gloss', segments=32),
     cyl([0.6, 0.36, 0.6], [0, 0.19, 0], OAK, 'wood', taper=0.85, segments=24),
     cyl([0.7, 0.02, 0.7], [0, 0.01, 0], OAK, 'wood', segments=24)]
add('coffeeTableRound', p, top=round_top('coffeeTableRound', 0.84, 0.42))
p = [box([1.66, 0.04, 0.72], [0, 0.4, 0], WALNUT, 'wood', radius=0.012),
     box([1.5, 0.025, 0.58], [0, 0.14, 0], WALNUT, 'wood', radius=0.008)]
p += legs4((-0.76, 0.76), (-0.3, 0.3), 0.38, 0.05, '#2B2C2E', 'metal')
p += [box([0.32, 0.05, 0.24], [0.4, 0.178, 0.0], '#C9B79C', 'matte', radius=0.008), box([0.28, 0.04, 0.2], [0.4, 0.22, 0.0], TEAL, 'matte', radius=0.008)]
add('coffeeTableLong', p, (2, 1), top=(-0.83, 0.83, -0.36, 0.36, 0.42))
p = [box([0.56, 0.025, 0.46], [0, 0.5, -0.02], '#F1F0EC', 'gloss', radius=0.008),
     box([0.42, 0.025, 0.36], [0.08, 0.38, 0.12], OAK, 'wood', radius=0.008)]
for x in (-0.26, 0.26):
    p += [box([0.02, 0.5, 0.02], [x, 0.25, -0.23], BLACKMETAL, 'metal', radius=0.004), box([0.02, 0.5, 0.02], [x, 0.25, 0.19], BLACKMETAL, 'metal', radius=0.004)]
for x in (-0.11, 0.27):
    p += [box([0.018, 0.37, 0.018], [x, 0.185, -0.04], BLACKMETAL, 'metal', radius=0.004), box([0.018, 0.37, 0.018], [x, 0.185, 0.28], BLACKMETAL, 'metal', radius=0.004)]
add('nestingTables', p, top=(-0.28, 0.28, -0.25, 0.21, 0.51))
p = [cyl([0.52, 0.03, 0.52], [0, 0.545, 0], '#E7E3DC', 'gloss', segments=28),
     cyl([0.06, 0.5, 0.06], [0, 0.28, 0], BRASS, 'metal', segments=12),
     cyl([0.36, 0.03, 0.36], [0, 0.015, 0], BRASS, 'metal', segments=24)]
add('sideTableRound', p, top=round_top('sideTableRound', 0.52, 0.56))
p = [box([1.8, 0.035, 0.56], [0, 0.78, -0.18], OAK, 'wood', radius=0.01),
     box([1.7, 0.025, 0.48], [0, 0.22, -0.18], OAK, 'wood', radius=0.006)]
p += legs4((-0.86, 0.86), (-0.42, 0.06), 0.77, 0.04, BLACKMETAL, 'metal')
p += [box([0.36, 0.14, 0.3], [-0.5, 0.31, -0.18], '#C9A979', 'fabric', radius=0.03), box([0.36, 0.14, 0.3], [0.0, 0.31, -0.18], '#C9A979', 'fabric', radius=0.03)]
add('consoleTable', p, (2, 1), top=(-0.9, 0.9, -0.46, 0.1, 0.8))

# Low TV bench with a flat TV on it.
p = [box([1.8, 0.4, 0.46], [0, 0.26, -0.22], '#F1F0EC', 'satin', radius=0.012),
     box([1.82, 0.03, 0.48], [0, 0.475, -0.22], OAK, 'wood', radius=0.008)]
p += legs4((-0.84, 0.84), (-0.4, -0.04), 0.06, 0.04, OAK, 'wood')
for x in (-0.6, 0.0, 0.6):
    p += [box([0.58, 0.34, 0.02], [x, 0.26, 0.015], '#F1F0EC', 'satin', radius=0.006), box([0.1, 0.012, 0.02], [x, 0.37, 0.03], OAK, 'wood', radius=0.004)]
p += [box([1.3, 0.76, 0.05], [0, 0.95, -0.3], '#1E1F22', 'satin', radius=0.01),
      box([1.26, 0.72, 0.01], [0, 0.95, -0.272], SCREEN, 'gloss', radius=0.006),
      box([0.3, 0.02, 0.16], [0, 0.5, -0.3], '#1E1F22', 'satin', radius=0.006),
      box([0.06, 0.08, 0.04], [0, 0.55, -0.31], '#1E1F22', 'satin', radius=0.006)]
add('tvStandLow', p, (2, 1))


def bookcase(w, h, d, wood, fin, shelves, z=None, seed=1, books=True, back=None):
    z = -0.47 + d / 2 if z is None else z
    p = [box([0.025, h, d], [s * (w / 2 - 0.0125), h / 2, z], wood, fin, radius=0.004) for s in (-1, 1)]
    p += [box([w, 0.03, d], [0, h - 0.015, z], wood, fin, radius=0.004), box([w - 0.05, 0.06, d], [0, 0.03, z], wood, fin, radius=0.004)]
    if back: p.append(box([w - 0.05, h - 0.06, 0.01], [0, h / 2, z - d / 2 + 0.005], back, 'matte', radius=0.002))
    gap = (h - 0.09) / shelves
    for i in range(shelves):
        y = 0.06 + gap * i
        if i: p.append(box([w - 0.05, 0.022, d - 0.01], [0, y, z], wood, fin, radius=0.003))
        if books and (i + seed) % 3 != 2:
            p += books_row(-w / 2 + 0.04, w / 2 - 0.04 - (0.2 if i % 2 else 0), y + 0.011, z + 0.02, depth=d * 0.7, seed=seed + i * 3, hmax=min(0.26, gap - 0.04))
        elif books:
            p += [cyl([0.12, 0.16, 0.12], [-w / 4, y + 0.09, z], CREAM, 'gloss', taper=0.8), box([0.2, 0.08, 0.16], [w / 5, y + 0.05, z], RUST, 'matte', radius=0.01)]
    return p


add('bookcaseTall', bookcase(0.9, 2.0, 0.34, WALNUT, 'wood', 6, seed=2))
p = bookcase(0.6, 1.9, 0.3, '#F1F0EC', 'satin', 5, z=-0.33, seed=4, back=TEAL)
p = [dict(q, at=[q['at'][0] - 0.12, q['at'][1], q['at'][2] - 0.02]) for q in p]
for y in (0.06, 0.43, 0.8, 1.17, 1.54, 1.87):                                                        # open end shelves on the +X side
    p.append(box([0.38, 0.022, 0.4], [0.3, y, -0.27], '#F1F0EC', 'satin', radius=0.006))
p += [box([0.025, 1.9, 0.4], [0.47, 0.95, -0.27], '#F1F0EC', 'satin', radius=0.004)]
p += plant_pot(0.3, -0.27, 0.81, CLAY, d=0.14, h=0.12) + [cyl([0.12, 0.2, 0.12], [0.3, 1.28, -0.27], '#9DC6CF', 'gloss', taper=0.6)]
add('bookcaseCorner', p)

# Cube shelf: 2x2 open cubes with fabric bins; things stand on top.
CUBE = '#F1F0EC'
p = case(0.8, 0.8, 0.4, 0, 0, -0.27, CUBE, 'satin', back='#D9D6CF')
p += [box([0.78, 0.022, 0.38], [0, 0.4, -0.27], CUBE, 'satin', radius=0.003), box([0.022, 0.78, 0.38], [0, 0.4, -0.27], CUBE, 'satin', radius=0.003)]
p += [box([0.32, 0.3, 0.32], [-0.19, 0.18, -0.25], '#9DAF8C', 'fabric', radius=0.03), box([0.32, 0.3, 0.32], [0.19, 0.57, -0.25], '#C98F6B', 'fabric', radius=0.03)]
p += books_row(-0.37, -0.03, 0.411, -0.27, depth=0.26, seed=3, hmax=0.3) + [box([0.2, 0.12, 0.2], [0.18, 0.08, -0.27], CREAM, 'gloss', radius=0.03)]
add('cubeShelf', p, top=(-0.4, 0.4, -0.47, -0.07, 0.8))

p = []
for x in (-0.3, 0.3):
    p.append(rod([x, 0, -0.1], [x, 1.8, -0.42], 0.04, OAK, 'wood', shape='box', t2=0.03, radius=0.006))
for i, y in enumerate((0.36, 0.76, 1.14, 1.5)):
    zf = -0.1 - (y / 1.8) * 0.32
    d = 0.36 - i * 0.06
    p.append(box([0.62, 0.025, d], [0, y, zf - d / 2 + 0.06], OAK, 'wood', radius=0.004))
p += plant_pot(-0.12, -0.12, 0.373, PORCELAIN, d=0.14, h=0.12, soil=False) + books_row(0.02, 0.26, 0.773, -0.22, depth=0.18, seed=5, hmax=0.2)
p += [cyl([0.1, 0.18, 0.1], [-0.15, 1.24, -0.31], CLAY, 'gloss', taper=0.7), box([0.16, 0.2, 0.03], [0.14, 1.25, -0.33], '#F2F0EA', 'gloss', radius=0.006, rotate=[-8, 0, 0])]
add('ladderShelf', p)

# Display cabinet: a glazed walnut vitrine with ornaments on glass shelves.
p = [box([0.78, 1.7, 0.02], [0, 1.0, -0.45], '#E8E2D6', 'wood', radius=0.004),
     box([0.84, 0.06, 0.46], [0, 1.88, -0.25], WALNUT, 'wood', radius=0.01),
     box([0.84, 0.12, 0.46], [0, 0.12, -0.25], WALNUT, 'wood', radius=0.01)]
p += legs4((-0.38, 0.38), (-0.44, -0.06), 0.06, 0.04, WALNUT, 'wood')
for x in (-0.4, 0.4):
    for z in (-0.46, -0.04):
        p.append(box([0.03, 1.7, 0.03], [x, 1.0, z], WALNUT, 'wood', radius=0.004))
for y in (0.6, 1.05, 1.5):
    p.append(box([0.74, 0.012, 0.38], [0, y, -0.25], '#DCE6EA', 'glass', radius=0.002))
p += [lathe([0.14, 0.24, 0.14], [-0.2, 0.31, -0.25], '#3E5470', 'gloss', VASE, segments=14),
      sph([0.16, 0.16, 0.16], [0.18, 0.27, -0.25], BRASS, 'metal', segments=12),
      cyl([0.12, 0.12, 0.12], [-0.18, 0.67, -0.25], CREAM, 'gloss', taper=0.7), box([0.16, 0.2, 0.03], [0.16, 0.71, -0.3], BRASS, 'metal', radius=0.006),
      lathe([0.1, 0.2, 0.1], [0.0, 1.16, -0.25], '#C25B3E', 'gloss', VASE, segments=14), sph([0.1, 0.1, 0.1], [0.2, 1.11, -0.25], '#9DC6CF', 'gloss', segments=10),
      cyl([0.2, 0.012, 0.2], [-0.15, 1.6, -0.36], PORCELAIN, 'gloss', rotate=[78, 0, 0], segments=16)]
add('displayCabinet', p)

# Electric fireplace: a slim white surround with a glowing firebox.
p = [box([1.6, 0.98, 0.32], [0, 0.49, -0.31], '#F4F3EF', 'satin', radius=0.012),
     box([1.76, 0.06, 0.4], [0, 1.0, -0.29], OAK, 'wood', radius=0.01),
     box([0.96, 0.52, 0.04], [0, 0.42, -0.14], '#16181B', 'gloss', radius=0.01),
     box([0.86, 0.06, 0.02], [0, 0.22, -0.12], '#4A3B30', 'matte', radius=0.01)]
for i, x in enumerate((-0.3, -0.12, 0.06, 0.24)):
    p.append(part('blob', [0.16, 0.24 + (i % 2) * 0.08, 0.04], [x, 0.36 + (i % 2) * 0.04, -0.118], '#FF9A3C', 'gloss', noise=0.25))
p += [part('blob', [0.5, 0.12, 0.03], [0, 0.28, -0.116], '#FFD27A', 'gloss', noise=0.2),
      cyl([0.12, 0.26, 0.12], [-0.6, 1.16, -0.32], CREAM, 'gloss', taper=0.6), box([0.22, 0.28, 0.03], [0.55, 1.17, -0.36], '#2B2C2E', 'satin', radius=0.006)]
add('fireplaceElectric', p, (2, 1), light=0.45)


# =============================================================================================
# Bedroom
# =============================================================================================
def bed(w, l, frame, ffin, head, sheet, duvet, pillow, head_h=1.0, head_fin=None, fp=(2, 2)):
    """A bed `w` wide and `l` long, head at -Z, along +Z."""
    z0 = -fp[1] / 2 + 0.04
    zc = z0 + l / 2
    p = [box([w, 0.24, l - 0.06], [0, 0.2, zc + 0.03], frame, ffin, radius=0.02),
         box([w - 0.06, 0.2, l - 0.14], [0, 0.42, zc + 0.05], sheet, 'fabric', radius=0.05),
         box([w + 0.02, 0.08, l * 0.64], [0, 0.5, zc + l * 0.17], duvet, 'fabric', radius=0.035),
         box([w + 0.04, 0.3, 0.06], [0, 0.4, zc + l * 0.17 + l * 0.32 - 0.02], duvet, 'fabric', radius=0.03),
         box([w, head_h, 0.08], [0, head_h / 2, z0 + 0.04], head, head_fin or ffin, radius=0.03)]
    n = 2 if w > 1.2 else 1
    for i in range(n):
        x = 0 if n == 1 else (i - 0.5) * w / 2
        p.append(cushion([w / n - 0.12, 0.14, 0.34], [x, 0.58, z0 + 0.28], pillow, rotate=[-10, 0, 0]))
    p += legs4((-w / 2 + 0.06, w / 2 - 0.06), (z0 + 0.12, z0 + l - 0.06), 0.1, 0.06, frame, ffin)
    return p


p = bed(1.62, 1.92, OAK, 'wood', OAK, '#F7F7F5', '#9DB7C9', '#F2F2F0', head_h=0.95)
for x in (-0.6, -0.2, 0.2, 0.6):
    p.append(box([0.36, 0.5, 0.012], [x, 0.66, -0.875], OAKD, 'wood', radius=0.004))
add('bedDouble', p, (2, 2))
p = bed(1.72, 1.92, '#B9B2A6', 'fabric', '#B9B2A6', '#FFFFFF', '#E9DCC9', '#F6F3EE', head_h=1.3)
for x in (-0.64, -0.32, 0.0, 0.32, 0.64):
    p.append(cushion([0.28, 0.9, 0.06], [x, 0.85, -0.89], '#C4BDB1', radius=0.03))
p += [cushion([0.42, 0.3, 0.12], [x, 0.7, -0.62], '#8C9468', rotate=[-14, 0, 0]) for x in (-0.36, 0.36)]
add('bedQueenUpholstered', p, (2, 2))
CAN = '#4A3426'
p = bed(1.62, 1.92, CAN, 'wood', CAN, '#F7F2EA', '#B5653F', '#F2EDE4', head_h=1.0)
for x in (-0.82, 0.82):
    for z in (-0.92, 0.92):
        p.append(box([0.07, 2.1, 0.07], [x, 1.05, z], CAN, 'wood', radius=0.01))
for z in (-0.92, 0.92):
    p.append(box([1.7, 0.06, 0.06], [0, 2.08, z], CAN, 'wood', radius=0.01))
for x in (-0.82, 0.82):
    p.append(box([0.06, 0.06, 1.9], [x, 2.08, 0], CAN, 'wood', radius=0.01))
for x in (-0.82, 0.82):
    for z in (-0.86, 0.86):
        p.append(box([0.16, 1.7, 0.02], [x - math.copysign(0.05, x), 1.2, z], '#F2EBDD', 'fabric', radius=0.01))
p.append(box([1.7, 0.1, 1.9], [0, 2.02, 0], '#F2EBDD', 'fabric', radius=0.02))
add('bedCanopy', p, (2, 2))
# Daybed (1x2): low rails at both ends and a back along -X, a bolster.
DB = '#F1F0EC'
p = [box([0.96, 0.26, 1.94], [0, 0.2, 0], DB, 'satin', radius=0.02),
     box([0.9, 0.18, 1.84], [0.02, 0.42, 0], '#F7F7F5', 'fabric', radius=0.05),
     box([0.92, 0.07, 1.2], [0.02, 0.5, 0.3], SAGE, 'fabric', radius=0.03),
     box([0.06, 0.82, 1.94], [-0.45, 0.41, 0], DB, 'satin', radius=0.02)]
for z in (-0.95, 0.95):
    p.append(box([0.96, 0.72, 0.05], [0, 0.36, z], DB, 'satin', radius=0.02))
p += [cushion([0.2, 0.36, 1.2], [-0.33, 0.66, -0.1], '#E4B8AE', rotate=[0, 0, -12]), cushion([0.6, 0.14, 0.3], [0.05, 0.56, -0.74], WHITE, rotate=[-10, 0, 0])]
add('bedDaybed', p, (1, 2))


def nightstand(body, fin, top_col=None, handle=BRASS, drawers_n=1, open_shelf=True, w=0.56, h=0.56, legs_col=None):
    """A bedside table, back on the wall and deep enough (to z = 0.16) for a lamp and a book."""
    d = 0.58
    z = -0.42 + d / 2
    leg = 0.12 if legs_col else 0.0
    p = [box([w, h - leg - 0.03, d], [0, leg + (h - leg - 0.03) / 2, z], body, fin, radius=0.012),
         box([w + 0.02, 0.03, d + 0.02], [0, h - 0.015, z], top_col or body, 'wood' if top_col else fin, radius=0.008)]
    if legs_col:
        p += legs4((-w / 2 + 0.05, w / 2 - 0.05), (z - d / 2 + 0.05, z + d / 2 - 0.05), leg, 0.035, legs_col, 'wood', round_=True, taper=0.7)
    y_top = h - 0.03
    dh = 0.15
    for i in range(drawers_n):
        y = y_top - dh * (i + 0.5) - 0.01
        p += [box([w - 0.04, dh - 0.015, 0.02], [0, y, z + d / 2 + 0.005], body, fin, radius=0.006),
              handle_h(0, y, z + d / 2 + 0.02, handle, 'metal', w=0.12)]
    if open_shelf:
        p.append(box([w - 0.06, y_top - dh * drawers_n - leg - 0.06, 0.02], [0, leg + (y_top - dh * drawers_n - leg) / 2 + 0.01, z + d / 2 - 0.004], '#3B302A', 'matte', radius=0.004))
    return p


add('nightstandModern', nightstand('#F1F0EC', 'satin', top_col=OAK, handle=BLACKMETAL, legs_col=OAK), top=(-0.29, 0.29, -0.43, 0.17, 0.56))
p = [cyl([0.5, 0.5, 0.5], [0, 0.29, -0.06], '#C98F6B', 'satin', segments=28),
     cyl([0.54, 0.03, 0.54], [0, 0.555, -0.06], OAK, 'wood', segments=28),
     box([0.3, 0.12, 0.02], [0, 0.42, 0.195], '#B87D5C', 'satin', radius=0.006), sph([0.03, 0.03, 0.02], [0, 0.42, 0.212], BRASS, 'metal', segments=8),
     cyl([0.4, 0.04, 0.4], [0, 0.02, -0.06], '#B87D5C', 'satin', segments=24)]
add('nightstandRound', p, top=(-0.19, 0.19, -0.25, 0.13, 0.57))
add('nightstandDrawers', nightstand(WALNUT, 'wood', drawers_n=3, open_shelf=False, h=0.6), top=(-0.29, 0.29, -0.43, 0.17, 0.6))


def chest(w, h, body, fin, n, handle, d=0.5, cols=1, legs_col=None, top_col=None):
    z = -0.47 + d / 2
    leg = 0.1 if legs_col else 0.06
    p = [box([w, h - leg - 0.03, d], [0, leg + (h - leg - 0.03) / 2, z], body, fin, radius=0.012),
         box([w + 0.02, 0.03, d + 0.02], [0, h - 0.015, z], top_col or body, fin, radius=0.008)]
    if legs_col:
        p += legs4((-w / 2 + 0.06, w / 2 - 0.06), (z - d / 2 + 0.06, z + d / 2 - 0.06), leg, 0.04, legs_col, 'wood', round_=True, taper=0.7)
    else:
        p.append(box([w - 0.04, leg, d - 0.04], [0, leg / 2, z], PLINTH, 'satin', radius=0.004))
    dh = (h - leg - 0.06) / n
    for c in range(cols):
        cw = (w - 0.03) / cols
        x = -w / 2 + 0.015 + cw * (c + 0.5)
        for i in range(n):
            y = leg + 0.015 + dh * (i + 0.5)
            p += [box([cw - 0.015, dh - 0.014, 0.02], [x, y, z + d / 2 + 0.005], body, fin, radius=0.006),
                  handle_h(x, y + dh * 0.15, z + d / 2 + 0.02, handle, 'metal', w=min(0.16, cw * 0.3))]
    return p


add('chestOfDrawers', chest(0.86, 1.16, '#E9E2D4', 'satin', 5, BRASS, d=0.66), top=(-0.44, 0.44, -0.48, 0.2, 1.16))
add('dresserLow', chest(1.8, 0.8, OAK, 'wood', 3, BLACKMETAL, d=0.64, cols=3, legs_col=OAKD), (2, 1), top=(-0.91, 0.91, -0.48, 0.18, 0.8))
p = [box([0.9, 2.0, 0.6], [0, 1.04, -0.16], '#F1F0EC', 'satin', radius=0.012),
     box([0.88, 0.05, 0.56], [0, 0.025, -0.17], PLINTH, 'satin', radius=0.004),
     box([0.86, 1.92, 0.02], [0, 1.04, 0.15], '#F1F0EC', 'satin', radius=0.01),
     box([0.02, 0.5, 0.03], [0.36, 1.1, 0.17], OAK, 'wood', radius=0.008),
     box([0.3, 0.24, 0.3], [-0.2, 2.16, -0.2], '#C9B79C', 'matte', radius=0.02)]
add('wardrobeSingle', p)
ARM = '#6B4A33'
p = [box([1.7, 1.8, 0.58], [0, 1.08, -0.17], ARM, 'wood', radius=0.012),
     box([1.82, 0.1, 0.66], [0, 2.02, -0.16], ARM, 'wood', radius=0.014),
     box([1.86, 0.04, 0.68], [0, 2.09, -0.16], '#5A3D2A', 'wood', radius=0.01),
     box([1.74, 0.1, 0.6], [0, 0.17, -0.16], '#5A3D2A', 'wood', radius=0.01)]
p += [cyl([0.08, 0.12, 0.08], [x, 0.06, z], '#5A3D2A', 'wood', taper=0.7, segments=10) for x in (-0.8, 0.8) for z in (-0.4, 0.08)]
for x in (-0.42, 0.42):
    p += [box([0.8, 1.66, 0.025], [x, 1.08, 0.13], ARM, 'wood', radius=0.012),
          box([0.62, 0.6, 0.012], [x, 1.42, 0.146], '#7E5A40', 'wood', radius=0.02),
          box([0.62, 0.6, 0.012], [x, 0.72, 0.146], '#7E5A40', 'wood', radius=0.02),
          sph([0.035, 0.035, 0.03], [x - math.copysign(0.33, x), 1.08, 0.16], BRASS, 'metal', segments=8)]
add('armoire', p, (2, 1))
CED = '#A0603E'
p = [box([0.9, 0.42, 0.46], [0, 0.25, 0], CED, 'wood', radius=0.012),
     box([0.94, 0.05, 0.5], [0, 0.48, 0], CED, 'wood', radius=0.012),
     box([0.94, 0.04, 0.5], [0, 0.04, 0], '#8A5034', 'wood', radius=0.01)]
for x in (-0.3, 0.3):
    p += [box([0.04, 0.38, 0.012], [x, 0.25, 0.236], BLACKMETAL, 'metal', radius=0.003)]
p += [box([0.1, 0.06, 0.02], [0, 0.42, 0.24], BRASS, 'metal', radius=0.006), box([0.04, 0.05, 0.16], [-0.46, 0.32, 0], BLACKMETAL, 'metal', radius=0.008),
      box([0.04, 0.05, 0.16], [0.46, 0.32, 0], BLACKMETAL, 'metal', radius=0.008)]
add('cedarChest', p, top=(-0.47, 0.47, -0.25, 0.25, 0.505))
p = [cushion([1.6, 0.12, 0.42], [0, 0.42, 0], '#8C9468', radius=0.04),
     box([1.56, 0.04, 0.38], [0, 0.34, 0], BRASS, 'metal', radius=0.01)]
for x in (-0.74, 0.74):
    for z in (-0.16, 0.16):
        p.append(cyl([0.03, 0.34, 0.03], [x, 0.17, z], BRASS, 'metal', segments=8))
for x in range(-5, 6):
    p.append(sph([0.025, 0.02, 0.025], [x * 0.13, 0.482, 0], '#737B54', 'fabric', segments=6))
add('benchBedEnd', p, (2, 1), seat=(0.47, 0.0))
p = [box([0.56, 1.66, 0.04], [0, 0.95, -0.06], '#C9A06A', 'metal', radius=0.03, rotate=[-8, 0, 0]),
     box([0.5, 1.58, 0.01], [0, 0.95, -0.035], '#D7E2E6', 'gloss', radius=0.025, rotate=[-8, 0, 0], mirror='front'),
     rod([0, 0, -0.38], [0, 1.4, -0.12], 0.03, '#C9A06A', 'metal', segments=8)]
add('mirrorFloor', p)
p = [flare([0.42, 0.56, 0.42], [0, 0.28, -0.2], '#C9A979', 'fabric', 1.1, segments=20),
     cyl([0.47, 0.05, 0.47], [0, 0.585, -0.2], '#B8955F', 'fabric', segments=20),
     sph([0.06, 0.04, 0.06], [0, 0.62, -0.2], '#B8955F', 'fabric', segments=8),
     torus([0.44, 0.03, 0.44], [0, 0.4, -0.2], '#B8955F', 'fabric', segments=20)]
add('laundryHamper', p)


# =============================================================================================
# Bathroom
# =============================================================================================
p = [lathe([0.38, 0.4, 0.5], [0, 0.2, 0.02], PORCELAIN, 'gloss', [[0, -0.5], [0.3, -0.5], [0.34, 0.0], [0.5, 0.45], [0.48, 0.5], [0, 0.5]], segments=24),
     cyl([0.4, 0.03, 0.5], [0, 0.415, 0.02], PORCELAIN, 'gloss', segments=24),
     cyl([0.38, 0.025, 0.48], [0, 0.44, 0.02], '#8A5A36', 'wood', segments=24),
     box([0.44, 0.4, 0.2], [0, 0.62, -0.3], PORCELAIN, 'gloss', radius=0.03),
     box([0.47, 0.04, 0.22], [0, 0.835, -0.3], PORCELAIN, 'gloss', radius=0.015),
     box([0.06, 0.02, 0.03], [-0.15, 0.76, -0.19], CHROME, 'chrome', radius=0.006)]
add('toiletClassic', p, seat=(0.45, -0.02))
p = [box([0.36, 0.32, 0.54], [0, 0.42, -0.17], PORCELAIN, 'gloss', radius=0.12),
     cyl([0.36, 0.03, 0.46], [0, 0.6, -0.12], PORCELAIN, 'gloss', segments=24),
     box([0.6, 1.1, 0.12], [0, 0.55, -0.42], '#C9C6C0', 'matte', radius=0.01),
     box([0.22, 0.15, 0.012], [0, 1.0, -0.355], '#2B2C2E', 'gloss', radius=0.01),
     box([0.09, 0.12, 0.006], [-0.05, 1.0, -0.348], CHROME, 'chrome', radius=0.006),
     box([0.09, 0.12, 0.006], [0.05, 1.0, -0.348], CHROME, 'chrome', radius=0.006)]
add('toiletWallHung', p, seat=(0.45, -0.1))
p = [flare([1.7, 0.56, 0.78], [0, 0.28, -0.04], PORCELAIN, 'gloss', 1.06, segments=36),
     cyl([1.58, 0.02, 0.66], [0, 0.5, -0.04], '#BFD9E0', 'gloss', segments=32),
     torus([1.76, 0.05, 0.82], [0, 0.57, -0.04], PORCELAIN, 'gloss', segments=36),
     cyl([0.05, 0.9, 0.05], [0.9, 0.45, 0.3], CHROME, 'chrome', segments=10),
     rod([0.9, 0.88, 0.3], [0.78, 0.84, 0.22], 0.035, CHROME, 'chrome'),
     cyl([0.14, 0.02, 0.14], [0.9, 0.01, 0.3], CHROME, 'chrome')]
add('bathtubFreestanding', p, (2, 1), seat=(0.28, 0.0))
p = [box([0.94, 0.08, 0.94], [0, 0.04, 0], '#E8E6E1', 'matte', radius=0.01),
     box([0.94, 2.1, 0.06], [0, 1.09, -0.44], '#D7D3CC', 'matte', radius=0.01),
     box([0.06, 2.0, 0.9], [-0.44, 1.04, 0.0], '#D7D3CC', 'matte', radius=0.01),
     box([0.012, 1.9, 0.86], [0.45, 1.03, 0.0], '#DCE6EA', 'glass', radius=0.004),
     box([0.84, 1.9, 0.012], [0.02, 1.03, 0.45], '#DCE6EA', 'glass', radius=0.004),
     box([0.02, 1.96, 0.02], [0.45, 1.06, 0.45], BLACKMETAL, 'metal', radius=0.004),
     box([0.9, 0.02, 0.02], [0.0, 2.02, 0.45], BLACKMETAL, 'metal', radius=0.004),
     rod([0, 2.12, -0.4], [0, 2.18, -0.12], 0.025, BLACKMETAL, 'metal'),
     box([0.3, 0.02, 0.3], [0, 2.17, 0.0], BLACKMETAL, 'metal', radius=0.01),
     box([0.04, 0.16, 0.04], [0.3, 1.1, -0.39], BLACKMETAL, 'metal', radius=0.01)]
add('showerRain', p)
p = [box([0.8, 0.52, 0.5], [0, 0.56, -0.21], '#8A5A36', 'wood', radius=0.012),
     box([0.84, 0.04, 0.54], [0, 0.84, -0.2], '#F4F3EF', 'gloss', radius=0.008),
     lathe([0.44, 0.16, 0.36], [0, 0.94, -0.18], PORCELAIN, 'gloss', BOWL, segments=24),
     cyl([0.035, 0.32, 0.035], [0, 1.0, -0.42], BLACKMETAL, 'metal'),
     rod([0, 1.15, -0.42], [0, 1.15, -0.24], 0.03, BLACKMETAL, 'metal'),
     box([0.7, 0.025, 0.42], [0, 0.18, -0.21], '#8A5A36', 'wood', radius=0.006)]
p += [box([0.025, 0.3, 0.025], [x, 0.15, z], '#8A5A36', 'wood') for x in (-0.38, 0.38) for z in (-0.44, 0.02)]
p += [box([0.36, 0.06, 0.24], [-0.15, 0.225, -0.22], '#F2F0EA', 'fabric', radius=0.03), box([0.36, 0.06, 0.24], [0.2, 0.225, -0.22], '#9DC6CF', 'fabric', radius=0.03),
      box([0.36, 0.2, 0.02], [-0.2, 0.56, 0.05], '#7A4E30', 'wood', radius=0.006), box([0.36, 0.2, 0.02], [0.2, 0.56, 0.05], '#7A4E30', 'wood', radius=0.006)]
add('sinkVessel', p)
p = [box([0.6, 0.12, 0.46], [0, 0.82, -0.21], PORCELAIN, 'gloss', radius=0.04),
     box([0.48, 0.04, 0.32], [0, 0.865, -0.18], '#D9E2E6', 'gloss', radius=0.04),
     cyl([0.035, 0.2, 0.035], [0, 0.96, -0.4], BRASS, 'metal'), rod([0, 1.05, -0.4], [0, 1.03, -0.3], 0.025, BRASS, 'metal'),
     cyl([0.02, 0.6, 0.02], [0, 0.6, 0.0], BRASS, 'metal', rotate=[0, 0, 90])]
p += [cyl([0.03, 0.76, 0.03], [x, 0.38, 0.0], BRASS, 'metal', segments=8) for x in (-0.27, 0.27)]
p += [rod([x, 0.76, 0.0], [x, 0.8, -0.2], 0.03, BRASS, 'metal', segments=8) for x in (-0.27, 0.27)]
p += [box([0.4, 0.03, 0.04], [0, 0.6, 0.02], '#E4B8AE', 'fabric', radius=0.01), box([0.38, 0.4, 0.02], [0, 0.4, 0.03], '#E4B8AE', 'fabric', radius=0.01)]
add('sinkConsole', p)
p = [box([0.42, 1.76, 0.34], [0, 0.94, -0.28], '#F1F0EC', 'satin', radius=0.012),
     box([0.4, 0.06, 0.3], [0, 0.03, -0.29], PLINTH, 'satin', radius=0.004),
     box([0.4, 1.0, 0.02], [0, 1.28, -0.1], '#F1F0EC', 'satin', radius=0.008),
     box([0.4, 0.5, 0.02], [0, 0.36, -0.1], '#F1F0EC', 'satin', radius=0.008),
     box([0.02, 0.12, 0.02], [0.14, 0.95, -0.08], CHROME, 'chrome', radius=0.004),
     box([0.36, 0.022, 0.3], [0, 0.66, -0.27], '#F1F0EC', 'satin', radius=0.004)]
p += [box([0.3, 0.08, 0.24], [0, 0.71 + i * 0.07, -0.27], c, 'fabric', radius=0.03) for i, c in enumerate(('#9DC6CF', '#F2F0EA'))]
add('bathCabinetTall', p)
p = []
for x in (-0.24, 0.24):
    p.append(rod([x, 0, -0.1], [x, 1.7, -0.42], 0.04, BIRCH, 'wood', segments=8))
for i in range(5):
    y = 0.3 + i * 0.33
    zf = -0.1 - (y / 1.7) * 0.32
    p.append(cyl([0.025, 0.5, 0.025], [0, y, zf], BIRCH, 'wood', rotate=[0, 0, 90], segments=8))
for y, c in ((0.96, '#9DC6CF'), (1.29, '#F2F0EA'), (0.63, SAGE)):
    zf = -0.1 - (y / 1.7) * 0.32
    p += [box([0.4, 0.3, 0.025], [0, y - 0.13, zf + 0.025], c, 'fabric', radius=0.01), box([0.4, 0.3, 0.025], [0, y - 0.13, zf - 0.025], c, 'fabric', radius=0.01)]
add('towelLadder', p)
add('bathMat', [box([0.76, 0.02, 0.5], [0, 0.01, 0], '#B9CCC4', 'fabric', radius=0.009),
                box([0.68, 0.006, 0.42], [0, 0.021, 0], '#C9DAD3', 'fabric', radius=0.003)], kind='rug')
p = [torus([0.62, 0.04, 0.62], [0, 1.55, WALL + 0.03], BRASS, 'metal', rotate=[90, 0, 0], segments=32),
     cyl([0.58, 0.012, 0.58], [0, 1.55, WALL + 0.02], '#D7E2E6', 'gloss', rotate=[90, 0, 0], segments=32, mirror='front'),
     cyl([0.012, 0.3, 0.012], [0, 1.95, WALL + 0.01], BRASS, 'metal', rotate=[0, 0, 30])]
add('mirrorRound', p, kind='wall')
p = [flare([0.46, 0.34, 0.36], [0, 0.17, -0.2], '#C9A979', 'fabric', 1.12, segments=20),
     torus([0.5, 0.04, 0.4], [0, 0.34, -0.2], '#B8955F', 'fabric', segments=20)]
p += [part('blob', [0.3, 0.12, 0.24], [x, 0.36, -0.2 + z], c, 'fabric', noise=0.2) for x, z, c in ((-0.08, 0.0, '#9DC6CF'), (0.1, 0.03, '#F2F0EA'), (0.0, -0.06, '#E4B8AE'))]
add('laundryBasket', p)


# =============================================================================================
# Kids
# =============================================================================================
p = [box([0.64, 0.035, 0.64], [0, 0.4825, 0], '#F4F1EA', 'satin', radius=0.03)]
p += [cyl([0.05, 0.465, 0.05], [x, 0.2325, z], c, 'gloss', segments=12) for (x, z), c in zip(((-0.26, -0.26), (0.26, -0.26), (-0.26, 0.26), (0.26, 0.26)), (RED, BLUE, YELLOW, GREEN))]
add('kidsTable', p, top=(-0.32, 0.32, -0.32, 0.32, 0.5))
p = [box([0.32, 0.03, 0.3], [0, 0.29, SZ], '#F4F1EA', 'satin', radius=0.012),
     box([0.32, 0.18, 0.03], [0, 0.48, SZ - 0.15], BLUE, 'gloss', radius=0.03, rotate=[-6, 0, 0])]
p += [cyl([0.035, 0.28, 0.035], [x, 0.14, SZ + z], BLUE, 'gloss', segments=10) for x in (-0.13, 0.13) for z in (-0.12, 0.12)]
p += [cyl([0.03, 0.2, 0.03], [x, 0.4, SZ - 0.14], BLUE, 'gloss', segments=8) for x in (-0.13, 0.13)]
add('kidsChair', p, seat=(0.31, SZ))
p = [box([0.7, 0.9, 0.025], [s * 0.35, 0.45, -0.25], '#F4F1EA', 'satin', radius=0.01, rotate=[0, 90, 0]) for s in (-1, 1)]
p[0]['size'] = [0.025, 0.9, 0.4]; p[0]['rotate'] = [0, 0, 0]; p[0]['at'] = [-0.36, 0.45, -0.26]
p[1]['size'] = [0.025, 0.9, 0.4]; p[1]['rotate'] = [0, 0, 0]; p[1]['at'] = [0.36, 0.45, -0.26]
p.append(box([0.72, 0.9, 0.02], [0, 0.45, -0.45], '#F4F1EA', 'satin', radius=0.004))
for i, y in enumerate((0.12, 0.4, 0.68)):
    zf = -0.12 - i * 0.07
    p += [box([0.7, 0.02, 0.2], [0, y, zf - 0.08], '#F4F1EA', 'satin', radius=0.004),
          box([0.7, 0.08, 0.015], [0, y + 0.04, zf + 0.02], [RED, '#9DC6CF', YELLOW][i], 'gloss', radius=0.004)]
    for k in range(3):
        p.append(box([0.18, 0.24, 0.02], [-0.22 + k * 0.22, y + 0.13, zf - 0.02], [TEAL, MUSTARD, RUST, SAGE, BLUE, BLUSH][(i + k * 2) % 6], 'matte', radius=0.004, rotate=[-14, 0, 0]))
add('kidsBookshelf', p)
TOY = '#E9C46A'
p = [box([0.8, 0.42, 0.46], [0, 0.23, -0.18], TOY, 'satin', radius=0.03),
     box([0.82, 0.05, 0.48], [0, 0.73, -0.43], TOY, 'satin', radius=0.02, rotate=[-80, 0, 0]),
     box([0.8, 0.06, 0.02], [0, 0.32, 0.055], RED, 'satin', radius=0.01),
     box([0.04, 0.06, 0.04], [-0.42, 0.3, -0.18], BLUE, 'satin', radius=0.01), box([0.04, 0.06, 0.04], [0.42, 0.3, -0.18], BLUE, 'satin', radius=0.01),
     sph([0.18, 0.18, 0.18], [-0.18, 0.47, -0.15], RED, 'gloss', segments=12),
     box([0.12, 0.12, 0.12], [0.1, 0.46, -0.2], BLUE, 'gloss', radius=0.01, rotate=[0, 20, 10]),
     box([0.1, 0.1, 0.1], [0.22, 0.44, -0.1], GREEN, 'gloss', radius=0.01, rotate=[0, -15, 0]),
     sph([0.14, 0.16, 0.12], [0.0, 0.5, -0.06], '#C08A52', 'fabric', segments=10)]
add('toyChest', p)
TENT = '#F2EBDD'
p = [cone([0.92, 1.3, 0.92], [0, 0.65, -0.04], TENT, 'fabric', segments=6),
     box([0.32, 0.6, 0.02], [0, 0.3, 0.36], '#3B302A', 'matte', radius=0.005, rotate=[-17, 0, 0]),
     box([0.2, 0.62, 0.02], [-0.16, 0.32, 0.34], '#E4B8AE', 'fabric', radius=0.005, rotate=[-17, 0, 12]),
     box([0.2, 0.62, 0.02], [0.16, 0.32, 0.34], '#E4B8AE', 'fabric', radius=0.005, rotate=[-17, 0, -12]),
     cyl([0.7, 0.03, 0.7], [0, 0.015, 0.0], '#9DC6CF', 'fabric', segments=20)]
for a in range(0, 360, 60):
    r = math.radians(a + 30)
    p.append(rod([math.cos(r) * 0.4, 0, -0.04 + math.sin(r) * 0.4], [-math.cos(r) * 0.06, 1.55, -0.04 - math.sin(r) * 0.06], 0.025, '#C9A27A', 'wood', segments=6))
for i in range(5):
    a = math.radians(-60 + i * 30)
    p.append(cone([0.06, 0.08, 0.02], [math.sin(a) * 0.36, 0.9 - abs(i - 2) * 0.03, -0.04 + math.cos(a) * 0.2], [RED, YELLOW, BLUE, GREEN, RED][i], 'fabric', segments=3, rotate=[180, 0, 0]))
add('playTent', p)
p = []
for s in (-1, 1):
    p += [rod([-0.3, 0, s * 0.3], [-0.3, 1.2, 0], 0.035, BIRCH, 'wood', segments=8), rod([0.3, 0, s * 0.3], [0.3, 1.2, 0], 0.035, BIRCH, 'wood', segments=8),
          box([0.56, 0.62, 0.012], [0, 0.82, s * 0.11], WHITE, 'matte', radius=0.004, rotate=[s * 14, 0, 0]),
          box([0.6, 0.03, 0.1], [0, 0.48, s * 0.21], BIRCH, 'wood', radius=0.006)]
p += [part('blob', [0.18, 0.14, 0.01], [-0.1, 0.9, 0.13], RED, 'matte', noise=0.3, rotate=[14, 0, 0]), part('blob', [0.16, 0.2, 0.01], [0.12, 0.78, 0.1], BLUE, 'matte', noise=0.3, rotate=[14, 0, 0]),
      part('blob', [0.22, 0.08, 0.01], [0.0, 0.66, 0.07], GREEN, 'matte', noise=0.3, rotate=[14, 0, 0])]
p += [cyl([0.05, 0.05, 0.05], [-0.18 + i * 0.12, 0.52, 0.22], c, 'gloss', segments=10) for i, c in enumerate((RED, YELLOW, BLUE, GREEN))]
add('kidsEasel', p)
p = [box([1.76, 0.04, 0.8], [0, 0.42, 0], '#6FA36A', 'satin', radius=0.012),
     box([1.8, 0.06, 0.84], [0, 0.38, 0], '#F4F1EA', 'satin', radius=0.012)]
p += legs4((-0.82, 0.82), (-0.34, 0.34), 0.36, 0.06, '#F4F1EA', 'satin')
p += [torus([1.4, 0.04, 0.6], [0, 0.445, 0], '#8E6E52', 'wood', segments=36),
      box([0.6, 0.02, 0.24], [-0.2, 0.45, 0.0], '#9DC6CF', 'gloss', radius=0.04),
      box([0.16, 0.12, 0.16], [0.35, 0.5, 0.1], RED, 'gloss', radius=0.01), box([0.12, 0.1, 0.12], [0.35, 0.6, 0.1], RED, 'gloss', radius=0.01),
      box([0.14, 0.04, 0.03], [0.45, 0.46, 0.25], YELLOW, 'gloss', radius=0.01),
      box([0.2, 0.1, 0.1], [0.62, 0.5, 0.06], BLUE, 'gloss', radius=0.01, rotate=[0, 60, 0]), cyl([0.05, 0.08, 0.05], [0.6, 0.58, 0.02], '#2B2C2E', 'gloss'),
      box([0.16, 0.08, 0.1], [0.68, 0.49, -0.12], GREEN, 'gloss', radius=0.01, rotate=[0, 80, 0])]
p += [cone([0.1, 0.2, 0.1], [x, 0.54, z], '#3F7A4A', 'foliage', segments=8) for x, z in ((-0.7, -0.3), (-0.6, -0.32), (0.7, -0.3))]
add('trainTable', p, (2, 1))
p = [cyl([0.92, 0.36, 0.92], [0, 0.18, 0], '#9DC6CF', 'satin', segments=28),
     torus([0.94, 0.08, 0.94], [0, 0.36, 0], '#F4F1EA', 'satin', segments=28)]
_rng = random.Random(7)
for i in range(26):
    a, rr = _rng.uniform(0, 6.283), _rng.uniform(0, 0.36)
    p.append(sph([0.08, 0.08, 0.08], [math.cos(a) * rr, 0.37 + _rng.uniform(0, 0.03), math.sin(a) * rr], [RED, YELLOW, BLUE, GREEN, '#E07BC3'][i % 5], 'gloss', segments=8))
add('ballPit', p)
p = [sph([0.62, 0.36, 0.62], [0, 0.18, 0.02], '#5BB3C9', 'fabric', segments=20),
     sph([0.52, 0.42, 0.3], [0, 0.32, -0.15], '#5BB3C9', 'fabric', segments=20, rotate=[-12, 0, 0]),
     sph([0.14, 0.14, 0.03], [0, 0.3, 0.27], YELLOW, 'fabric', segments=10)]
add('beanBagKids', p, seat=(0.24, 0.05))
CARC = '#D9483B'
p = [box([0.94, 0.3, 1.94], [0, 0.22, 0], CARC, 'gloss', radius=0.12),
     box([0.84, 0.16, 1.5], [0, 0.42, -0.1], '#F7F7F5', 'fabric', radius=0.05),
     box([0.86, 0.06, 0.9], [0, 0.5, 0.14], '#3E8EDE', 'fabric', radius=0.03),
     box([0.94, 0.5, 0.3], [0, 0.42, 0.82], CARC, 'gloss', radius=0.12),
     box([0.94, 0.24, 0.16], [0, 0.48, -0.9], CARC, 'gloss', radius=0.07),
     box([0.7, 0.04, 0.12], [0, 0.72, -0.92], '#2B2C2E', 'satin', radius=0.02),
     cushion([0.5, 0.12, 0.3], [0, 0.56, -0.66], WHITE, rotate=[-10, 0, 0])]
for x in (-0.44, 0.44):
    for z in (-0.6, 0.6):
        p += [cyl([0.36, 0.08, 0.36], [x, 0.18, z], '#2B2C2E', 'satin', rotate=[0, 0, 90], segments=18),
              cyl([0.18, 0.085, 0.18], [x, 0.18, z], '#D0D3D6', 'satin', rotate=[0, 0, 90], segments=12)]
p += [sph([0.12, 0.1, 0.04], [x, 0.5, 0.97], YELLOW, 'gloss', segments=8) for x in (-0.3, 0.3)]
add('kidsBedCar', p, (1, 2))


# =============================================================================================
# Office
# =============================================================================================
p = [box([1.5, 0.035, 0.72], [0, DT - 0.0175, -0.1], OAK, 'wood', radius=0.01),
     box([0.42, 0.12, 0.6], [0.45, DT - 0.1, -0.12], '#F1F0EC', 'satin', radius=0.008),
     handle_h(0.45, DT - 0.1, 0.19, BLACKMETAL, 'metal', w=0.14)]
for x in (-0.68, 0.68):
    for z in (-0.4, 0.2):
        p.append(rod([x, 0, z], [x * 0.96, DT - 0.035, z * 0.94], 0.018, BLACKMETAL, 'metal', segments=6))
        p.append(rod([x + 0.04, 0, z], [x * 0.96 + 0.04, DT - 0.035, z * 0.94], 0.018, BLACKMETAL, 'metal', segments=6))
add('deskSimple', p, (2, 1), top=(-0.75, 0.75, -0.46, 0.26, DT))
DC = '#F1F0EC'
p = [box([1.96, 0.035, 1.36], [0, DT - 0.0175, -0.3], DC, 'satin', radius=0.008),
     box([1.36, 0.035, 0.6], [-0.3, DT - 0.0175, 0.68], DC, 'satin', radius=0.008),
     box([0.849, 0.035, 0.42], [0.539, DT - 0.018, 0.539], DC, 'satin', radius=0.004, rotate=[0, 45, 0]),
     box([0.04, DT - 0.035, 1.3], [-0.94, (DT - 0.035) / 2, 0.0], OAK, 'wood', radius=0.006),
     box([0.04, DT - 0.035, 0.6], [0.94, (DT - 0.035) / 2, -0.66], OAK, 'wood', radius=0.006),
     box([0.6, DT - 0.035, 0.04], [-0.66, (DT - 0.035) / 2, 0.94], OAK, 'wood', radius=0.006),
     box([1.8, 0.4, 0.02], [0, 0.45, -0.94], OAK, 'wood', radius=0.004),
     box([0.42, 0.6, 0.55], [-0.7, 0.3, -0.62], DC, 'satin', radius=0.01)]
p += [handle_h(-0.7, 0.5 - i * 0.18, -0.335, BLACKMETAL, 'metal', w=0.14) for i in range(3)]
add('deskCorner', p, (2, 2), top=(-0.98, 0.98, -0.98, 0.98, DT))
p = [box([0.96, 0.03, 0.66], [0, 1.035, -0.13], BIRCH, 'wood', radius=0.01),
     box([0.84, 0.05, 0.08], [0, 0.98, -0.13], '#2B2C2E', 'metal', radius=0.008)]
for x in (-0.38, 0.38):
    p += [box([0.07, 0.95, 0.06], [x, 0.5, -0.13], '#2B2C2E', 'metal', radius=0.01),
          box([0.08, 0.04, 0.62], [x, 0.02, -0.13], '#2B2C2E', 'metal', radius=0.01)]
p += [box([0.06, 0.03, 0.02], [0.4, 1.0, 0.2], '#3A3D41', 'satin', radius=0.006),
      box([0.56, 0.34, 0.03], [0, 1.37, -0.38], '#1E1F22', 'satin', radius=0.01), box([0.53, 0.31, 0.01], [0, 1.37, -0.364], SCREEN, 'gloss', radius=0.006),
      box([0.04, 0.2, 0.04], [0, 1.15, -0.41], '#2B2C2E', 'metal', radius=0.01), box([0.12, 0.03, 0.06], [0, 1.065, -0.42], '#2B2C2E', 'metal', radius=0.008)]
add('deskStanding', p, top=(-0.48, 0.48, -0.46, 0.2, 1.05))
EXE = '#5A3825'
p = [box([1.8, 0.05, 0.8], [0, DT - 0.025, -0.06], EXE, 'wood', radius=0.012),
     box([1.4, 0.004, 0.5], [0, DT + 0.001, -0.1], '#2F4A3A', 'fabric', radius=0.002),
     box([1.0, 0.5, 0.03], [0, 0.42, -0.36], EXE, 'wood', radius=0.006)]
for x in (-0.66, 0.66):
    p += [box([0.44, DT - 0.05, 0.72], [x, (DT - 0.05) / 2, -0.06], EXE, 'wood', radius=0.012)]
    p += [box([0.4, 0.2, 0.02], [x, 0.58 - i * 0.22, 0.31], '#6B4630', 'wood', radius=0.006) for i in range(3)]
    p += [handle_h(x, 0.58 - i * 0.22, 0.325, BRASS, 'metal', w=0.12) for i in range(3)]
add('deskExecutive', p, (2, 1), top=(-0.9, 0.9, -0.46, 0.34, DT))
FILE = '#8E959B'
p = [box([0.46, 1.0, 0.62], [0, 0.5, -0.16], FILE, 'metal', radius=0.01)]
for i in range(3):
    y = 0.18 + i * 0.32
    p += [box([0.42, 0.3, 0.02], [0, y, 0.155], FILE, 'metal', radius=0.008),
          handle_h(0, y + 0.06, 0.17, CHROME, 'metal', w=0.14),
          box([0.1, 0.04, 0.005], [0, y + 0.11, 0.167], '#F2F0EA', 'gloss', radius=0.002)]
p += plant_pot(0.08, -0.2, 1.0, PORCELAIN, d=0.14, h=0.12, soil=False) + [box([0.24, 0.03, 0.3], [-0.08, 1.015, -0.2], '#F2F0EA', 'gloss', radius=0.004)]
add('filingCabinet', p)


def swivel(fabric, frame, fin='fabric', back_h=0.62, arms=True, seat_y=0.47):
    p = [cushion([0.52, 0.09, 0.5], [0, seat_y - 0.04, SZ], fabric, fin, radius=0.04),
         cushion([0.5, back_h, 0.09], [0, seat_y + back_h / 2 + 0.05, SZ - 0.24], fabric, fin, rotate=[-9, 0, 0], radius=0.04),
         cyl([0.05, seat_y - 0.18, 0.05], [0, (seat_y - 0.18) / 2 + 0.1, SZ], CHROME, 'chrome'),
         cyl([0.07, 0.05, 0.07], [0, 0.12, SZ], frame, 'metal')]
    if arms:
        for x in (-0.28, 0.28):
            p += [box([0.05, 0.035, 0.3], [x, seat_y + 0.19, SZ + 0.02], frame, 'metal', radius=0.012),
                  box([0.025, 0.2, 0.025], [x, seat_y + 0.08, SZ - 0.04], frame, 'metal', radius=0.006)]
    for i in range(5):
        a = math.radians(i * 72 + 18)
        tip = [math.cos(a) * 0.27, 0.05, SZ + math.sin(a) * 0.27]
        p += [rod([0, 0.1, SZ], tip, 0.03, frame, 'metal', shape='box', t2=0.035, radius=0.008),
              sph([0.05, 0.05, 0.05], [tip[0], 0.025, tip[2]], '#1E1F22', 'satin', segments=8)]
    return p


p = swivel('#3B2A20', BLACKMETAL, 'satin', back_h=0.78)
p += [box([0.02, 0.6, 0.012], [x, 0.98, SZ - 0.2], '#2E2019', 'satin', radius=0.004, rotate=[-9, 0, 0]) for x in (-0.1, 0.1)]
add('chairExecutive', p, seat=(0.49, SZ))
p = swivel('#4E6E7A', '#2B2C2E', back_h=0.5)
p[1] = box([0.46, 0.5, 0.03], [0, 0.77, SZ - 0.24], '#3A3D41', 'fabric', radius=0.03, rotate=[-9, 0, 0])
add('chairTask', p, seat=(0.48, SZ))
p = [box([0.92, 0.62, 0.025], [0, 1.4, WALL + 0.0125], '#C9CBCC', 'metal', radius=0.01),
     box([0.88, 0.58, 0.006], [0, 1.4, WALL + 0.027], '#FBFBF9', 'gloss', radius=0.006),
     box([0.6, 0.025, 0.06], [0, 1.08, WALL + 0.03], '#C9CBCC', 'metal', radius=0.006)]
p += [box([0.012, 0.012, 0.12], [x, 1.1, WALL + 0.05], c, 'gloss', radius=0.004, rotate=[0, 90, 0]) for x, c in ((-0.2, RED), (-0.12, BLUE), (-0.04, '#2B2C2E'))]
p += [box([w, 0.01, 0.004], [x, y, WALL + 0.031], c, 'gloss', radius=0.002, rotate=[0, 0, r]) for x, y, w, c, r in
      ((-0.2, 1.55, 0.3, BLUE, 0), (-0.22, 1.5, 0.26, BLUE, 0), (-0.24, 1.45, 0.22, BLUE, 0), (0.18, 1.42, 0.3, RED, 25), (0.15, 1.36, 0.2, RED, -10), (-0.1, 1.28, 0.4, '#2B2C2E', 0))]
p.append(box([0.1, 0.1, 0.004], [0.32, 1.58, WALL + 0.031], YELLOW, 'gloss', radius=0.002))
add('whiteboard', p, kind='wall')
p = [box([0.74, 0.52, 0.03], [0, 1.45, WALL + 0.015], '#8A5A36', 'wood', radius=0.008),
     box([0.68, 0.46, 0.012], [0, 1.45, WALL + 0.03], '#C49A6C', 'matte', radius=0.004)]
for x, y, c, r in ((-0.2, 1.55, YELLOW, 6), (0.0, 1.5, '#F2F0EA', -4), (0.2, 1.56, BLUSH, 3), (-0.15, 1.35, '#9DC6CF', -6), (0.18, 1.34, '#F2F0EA', 4)):
    p += [box([0.14, 0.12, 0.004], [x, y, WALL + 0.038], c, 'matte', radius=0.002, rotate=[0, 0, r]),
          sph([0.016, 0.016, 0.016], [x, y + 0.04, WALL + 0.045], RED, 'gloss', segments=6)]
add('corkboard', p, kind='wall')


# =============================================================================================
# Hobbies
# =============================================================================================
p = [cyl([0.4, 0.04, 0.4], [0, 0.5, -0.3], '#2B2C2E', 'fabric', segments=16),
     cyl([0.05, 0.48, 0.05], [0, 0.24, -0.3], CHROME, 'chrome'), cyl([0.3, 0.02, 0.3], [0, 0.01, -0.3], CHROME, 'chrome')]
p += [cyl([0.5, 0.36, 0.5], [0, 0.27, 0.2], '#B03A2E', 'gloss', rotate=[90, 0, 0], segments=24),
      cyl([0.46, 0.37, 0.46], [0, 0.27, 0.2], '#F2F0EA', 'satin', rotate=[90, 0, 0], segments=24),
      cyl([0.26, 0.2, 0.26], [-0.14, 0.62, 0.15], '#B03A2E', 'gloss', rotate=[20, 0, 0], segments=18),
      cyl([0.24, 0.18, 0.24], [0.16, 0.62, 0.15], '#B03A2E', 'gloss', rotate=[20, 0, 0], segments=18),
      cyl([0.34, 0.14, 0.34], [-0.28, 0.6, -0.08], '#D9D6CF', 'chrome', segments=18),
      cyl([0.36, 0.26, 0.36], [0.3, 0.3, -0.05], '#B03A2E', 'gloss', segments=18)]
for x, y, z, d in ((-0.3, 1.0, 0.18, 0.34), (0.3, 1.05, 0.12, 0.36), (-0.32, 0.86, -0.16, 0.28)):
    p += [cyl([0.02, y, 0.02], [x, y / 2, z], CHROME, 'chrome', segments=6),
          cyl([d, 0.012, d], [x, y, z], '#C9A64A', 'chrome', rotate=[10, 0, 8], segments=20)]
add('drumKit', p, seat=(0.52, -0.3))
AC = '#2D3E8C'
p = [box([0.04, 1.8, 0.66], [s * 0.31, 0.9, -0.14], '#1E1F22', 'satin', radius=0.01) for s in (-1, 1)]
p += [box([0.012, 1.4, 0.4], [s * 0.331, 1.0, -0.1], AC, 'gloss', radius=0.004) for s in (-1, 1)]
p += [box([0.58, 1.8, 0.3], [0, 0.9, -0.32], '#1E1F22', 'satin', radius=0.01),
      box([0.58, 0.86, 0.46], [0, 0.43, -0.02], '#1E1F22', 'satin', radius=0.01),
      box([0.3, 0.12, 0.012], [0, 0.55, 0.215], '#3A3D41', 'satin', radius=0.004),
      box([0.66, 0.07, 0.3], [0, 0.9, 0.25], '#1E1F22', 'satin', radius=0.02, rotate=[10, 0, 0]),
      box([0.58, 0.56, 0.04], [0, 1.24, -0.06], '#16181B', 'gloss', radius=0.01, rotate=[-14, 0, 0]),
      box([0.5, 0.42, 0.01], [0, 1.24, -0.035], '#3E8EDE', 'gloss', radius=0.006, rotate=[-14, 0, 0]),
      box([0.58, 0.2, 0.04], [0, 1.66, 0.04], YELLOW, 'gloss', radius=0.01),
      box([0.66, 0.05, 0.62], [0, 1.795, -0.15], '#1E1F22', 'satin', radius=0.01)]
p += [cyl([0.03, 0.08, 0.03], [-0.15, 0.97, 0.28], '#2B2C2E', 'satin'), sph([0.06, 0.06, 0.06], [-0.15, 1.02, 0.28], RED, 'gloss', segments=10)]
p += [cyl([0.05, 0.02, 0.05], [0.05 + i * 0.08, 0.95, 0.26 + (i % 2) * 0.05], [RED, BLUE, YELLOW, GREEN][i], 'gloss', segments=10) for i in range(4)]
add('arcadeCabinet', p)
p = [cyl([0.66, 0.36, 0.66], [0, 0.18, 0.12], '#7F8C8D', 'satin', segments=24),
     lathe([0.7, 0.12, 0.7], [0, 0.42, 0.12], '#9AA3A5', 'satin', BOWL, segments=24),
     cyl([0.34, 0.03, 0.34], [0, 0.46, 0.12], '#9AA3A5', 'satin', segments=20),
     lathe([0.2, 0.2, 0.2], [0, 0.575, 0.12], '#B5653F', 'matte', VASE, segments=16),
     cyl([0.36, 0.04, 0.36], [0, 0.42, -0.3], '#8A5A36', 'wood', segments=16)]
p += [splayed_leg(x, z - 0.3, 0.4, 0.035, '#8A5A36', 'wood', out=0.03) for x, z in ((-0.12, -0.1), (0.12, -0.1), (-0.12, 0.1), (0.12, 0.1))]
add('potteryWheel', p, seat=(0.45, -0.3))
CR = '#E9E2D4'
p = [box([1.7, 0.035, 0.5], [0, 0.735, 0.2], CR, 'satin', radius=0.01)]
p += legs4((-0.8, 0.8), (0.0, 0.4), 0.72, 0.045, OAK, 'wood')
p += [box([0.4, 0.01, 0.3], [-0.4, 0.757, 0.2], '#E7D6B8', 'matte', radius=0.003, rotate=[0, 8, 0]),
      box([0.3, 0.012, 0.22], [-0.38, 0.763, 0.18], BLUSH, 'matte', radius=0.003, rotate=[0, -6, 0]),
      cyl([0.08, 0.1, 0.08], [0.2, 0.8, 0.32], TEAL, 'gloss'),
      rod([0.18, 0.82, 0.32], [0.17, 0.92, 0.3], 0.01, RED, 'gloss'), rod([0.22, 0.82, 0.32], [0.23, 0.91, 0.34], 0.01, YELLOW, 'gloss'),
      torus([0.1, 0.03, 0.1], [0.45, 0.77, 0.22], '#9DC6CF', 'matte', segments=12), torus([0.1, 0.03, 0.1], [0.56, 0.77, 0.16], MUSTARD, 'matte', segments=12),
      box([0.16, 0.04, 0.1], [0.62, 0.775, 0.32], '#C0473A', 'satin', radius=0.01),
      cyl([0.38, 0.04, 0.38], [0, 0.46, -0.25], OAK, 'wood', segments=16)]
p += [splayed_leg(x, z - 0.25, 0.44, 0.035, OAK, 'wood', out=0.03) for x, z in ((-0.12, -0.1), (0.12, -0.1), (-0.12, 0.1), (0.12, 0.1))]
add('craftTable', p, (2, 1), seat=(0.48, -0.25))
p = [box([0.86, 0.035, 0.5], [0, 0.735, 0.2], WALNUT, 'wood', radius=0.01),
     box([0.6, 0.006, 0.4], [0, 0.755, 0.2], '#5E7A9A', 'matte', radius=0.002)]
p += legs4((-0.38, 0.38), (0.0, 0.4), 0.72, 0.045, WALNUT, 'wood')
_rng = random.Random(3)
for i in range(16):
    x, z = _rng.uniform(-0.38, 0.38), _rng.uniform(0.0, 0.42)
    if abs(x) < 0.3 and 0.02 < z < 0.38: continue
    p.append(box([0.045, 0.006, 0.045], [x, 0.756, z], ['#5E7A9A', '#E3C66A', '#7FA35A', '#E4B8AE'][i % 4], 'matte', radius=0.002, rotate=[0, _rng.uniform(0, 90), 0]))
p += [box([0.2, 0.04, 0.14], [0.3, 0.775, 0.36], '#C9A979', 'matte', radius=0.006), cyl([0.36, 0.04, 0.36], [0, 0.46, -0.25], WALNUT, 'wood', segments=16)]
p += [splayed_leg(x, z - 0.25, 0.44, 0.035, WALNUT, 'wood', out=0.03) for x, z in ((-0.12, -0.1), (0.12, -0.1), (-0.12, 0.1), (0.12, 0.1))]
add('puzzleTable', p, seat=(0.48, -0.25))
GU = '#C08A52'
p = [cyl([0.36, 0.1, 0.42], [0, 0.42, 0.0], GU, 'gloss', rotate=[90, 0, 0], segments=24),
     cyl([0.28, 0.1, 0.32], [0, 0.72, 0.0], GU, 'gloss', rotate=[90, 0, 0], segments=24),
     cyl([0.09, 0.012, 0.09], [0, 0.62, 0.052], '#2B2C2E', 'matte', rotate=[90, 0, 0], segments=16),
     box([0.05, 0.5, 0.025], [0, 1.08, 0.02], '#3B2A20', 'wood', radius=0.006),
     box([0.08, 0.18, 0.025], [0, 1.4, 0.02], '#3B2A20', 'wood', radius=0.006),
     rod([-0.14, 0, 0.18], [0, 0.3, -0.02], 0.025, BLACKMETAL, 'metal', segments=6), rod([0.14, 0, 0.18], [0, 0.3, -0.02], 0.025, BLACKMETAL, 'metal', segments=6),
     rod([0, 0, -0.22], [0, 0.95, -0.06], 0.025, BLACKMETAL, 'metal', segments=6), box([0.3, 0.04, 0.06], [0, 0.32, 0.05], BLACKMETAL, 'metal', radius=0.01)]
add('guitarAcoustic', p)


# =============================================================================================
# Tabletop: small things standing on tables, counters, desks and shelves (origin at their base)
# =============================================================================================
def tt(key, parts):
    add(key, parts, kind='surface')


tt('fruitBowl', [lathe([0.28, 0.1, 0.28], [0, 0.05, 0], '#E8E2D6', 'gloss', BOWL, segments=20),
                 sph([0.08, 0.075, 0.08], [-0.05, 0.1, 0.02], RED, 'satin', segments=10), sph([0.08, 0.08, 0.08], [0.05, 0.1, -0.03], '#F29B38', 'satin', segments=10),
                 sph([0.075, 0.07, 0.075], [0.03, 0.1, 0.07], '#9BC24A', 'satin', segments=10), sph([0.075, 0.075, 0.075], [-0.04, 0.15, -0.03], '#F29B38', 'satin', segments=10),
                 part('capsule', [0.04, 0.2, 0.04], [0.0, 0.16, 0.0], YELLOW, 'satin', rotate=[0, 30, 70])])
tt('plateSet', plate(0, 0, 0, d=0.27, rim='#3E5470') + plate(0, -0.01, 0.014, d=0.19) +
   [box([0.018, 0.004, 0.18], [-0.17, 0.002, 0.0], CHROME, 'chrome', radius=0.002), box([0.02, 0.004, 0.2], [0.17, 0.002, 0.0], CHROME, 'chrome', radius=0.002),
    flare([0.07, 0.11, 0.07], [0.14, 0.055, -0.15], '#DCE6EA', 'glass', 1.1, segments=12)])
tt('bookStack', [box([0.24, 0.035, 0.17], [0, 0.0175, 0], NAVY, 'matte', radius=0.004),
                 box([0.22, 0.03, 0.16], [0.01, 0.05, 0.0], MUSTARD, 'matte', radius=0.004, rotate=[0, 12, 0]),
                 box([0.2, 0.04, 0.15], [-0.01, 0.085, 0.0], RUST, 'matte', radius=0.004, rotate=[0, -8, 0]),
                 box([0.18, 0.028, 0.13], [0.0, 0.119, 0.0], SAGE, 'matte', radius=0.004, rotate=[0, 20, 0]),
                 box([0.205, 0.012, 0.145], [-0.01, 0.06, 0.0], '#F2F0EA', 'matte', radius=0.002, rotate=[0, -8, 0])])
p = [box([0.02, 0.16, 0.12], [x, 0.08, 0], BRASS, 'metal', radius=0.004) for x in (-0.15, 0.15)]
p += [box([0.3, 0.012, 0.12], [0, 0.006, 0], BRASS, 'metal', radius=0.003)]
p += books_row(-0.135, 0.135, 0.012, 0.0, depth=0.13, seed=6, hmax=0.2, lean=False)
tt('booksUpright', p)
p = [lathe([0.13, 0.24, 0.13], [0, 0.12, 0], '#9DC6CF', 'gloss', VASE, segments=16)]
for i, (dx, dz, c) in enumerate(((0.0, 0.0, '#F2F0EA'), (-0.06, 0.03, BLUSH), (0.06, -0.02, YELLOW), (0.03, 0.06, '#E07B7B'), (-0.04, -0.05, '#F2F0EA'))):
    p += [rod([0, 0.2, 0], [dx, 0.34 + (i % 2) * 0.04, dz], 0.008, '#5E8F4E', 'foliage', segments=5),
          sph([0.07, 0.06, 0.07], [dx, 0.35 + (i % 2) * 0.04, dz], c, 'satin', segments=8)]
p.append(part('blob', [0.14, 0.08, 0.14], [0, 0.27, 0], FOLIAGE, 'foliage', noise=0.3))
tt('vaseFlowers', p)
tt('vaseSmall', [lathe([0.08, 0.16, 0.08], [0, 0.08, 0], '#C25B3E', 'gloss', VASE, segments=14),
                 rod([0, 0.13, 0], [0.02, 0.26, 0.01], 0.006, '#5E8F4E', 'foliage', segments=5), sph([0.04, 0.035, 0.04], [0.02, 0.27, 0.01], '#F2F0EA', 'satin', segments=8)])
p = [flare([0.14, 0.07, 0.14], [0, 0.035, 0], '#E8E2D6', 'matte', 1.15, segments=16), cyl([0.14, 0.01, 0.14], [0, 0.068, 0], SOIL, 'matte', segments=14)]
for i in range(8):
    a = math.radians(i * 45)
    p.append(cone([0.04, 0.07, 0.025], [math.cos(a) * 0.035, 0.1, math.sin(a) * 0.035], '#8FB59A', 'foliage', rotate=[math.sin(a) * 40, 0, -math.cos(a) * 40], segments=6))
p.append(sph([0.05, 0.05, 0.05], [0, 0.1, 0], '#A9C9A4', 'foliage', segments=8))
tt('potSucculent', p)
tt('potCactus', [flare([0.12, 0.1, 0.12], [0, 0.05, 0], '#C26E4A', 'matte', 1.2, segments=14), cyl([0.13, 0.01, 0.13], [0, 0.1, 0], '#D9C7A7', 'matte', segments=14),
                 part('capsule', [0.07, 0.22, 0.07], [0, 0.2, 0], '#4F8A55', 'foliage'), part('capsule', [0.045, 0.1, 0.045], [0.05, 0.2, 0], '#4F8A55', 'foliage', rotate=[0, 0, -40]),
                 part('capsule', [0.04, 0.08, 0.04], [-0.045, 0.17, 0.0], '#4F8A55', 'foliage', rotate=[0, 0, 45]), sph([0.035, 0.025, 0.035], [0, 0.315, 0], '#E07BC3', 'satin', segments=8)])
tt('potSmall', plant_pot(0, 0, 0, '#F2F0EA', d=0.13, h=0.11, tall=0.16))
tt('photoFrame', [box([0.18, 0.23, 0.02], [0, 0.11, 0.0], '#2B2C2E', 'satin', radius=0.004, rotate=[-10, 0, 0]),
                  box([0.14, 0.19, 0.004], [0, 0.112, 0.012], '#9DB7C9', 'matte', radius=0.002, rotate=[-10, 0, 0]),
                  sph([0.05, 0.06, 0.003], [0, 0.12, 0.015], '#E4B8AE', 'matte', segments=8, rotate=[-10, 0, 0]),
                  box([0.03, 0.18, 0.012], [0, 0.085, -0.05], '#2B2C2E', 'satin', radius=0.003, rotate=[18, 0, 0])])
p = []
for x, h in ((-0.05, 0.18), (0.05, 0.14)):
    p += [lathe([0.07, h, 0.07], [x, h / 2, 0], BRASS, 'metal', [[0, -0.5], [0.5, -0.5], [0.5, -0.42], [0.14, -0.3], [0.12, 0.3], [0.3, 0.42], [0.3, 0.5], [0, 0.5]], segments=12),
          cyl([0.022, 0.14, 0.022], [x, h + 0.07, 0], '#F4EBD8', 'satin', segments=8),
          sph([0.016, 0.03, 0.016], [x, h + 0.155, 0], '#FFD27A', 'gloss', segments=6)]
tt('candlesticks', p)
tt('kettle', [cyl([0.16, 0.2, 0.16], [0, 0.12, 0], '#F2F0EA', 'gloss', taper=0.8, segments=18), cyl([0.17, 0.02, 0.17], [0, 0.01, 0], '#2B2C2E', 'satin', segments=18),
              torus([0.15, 0.025, 0.06], [-0.09, 0.15, 0], '#2B2C2E', 'satin', rotate=[90, 90, 0], segments=12),
              rod([0.06, 0.16, 0], [0.11, 0.21, 0], 0.03, '#F2F0EA', 'gloss'), cyl([0.07, 0.02, 0.07], [0, 0.225, 0], '#2B2C2E', 'satin', segments=12)])
tt('toaster', [box([0.28, 0.18, 0.16], [0, 0.09, 0], '#E3D9C6', 'gloss', radius=0.05),
               box([0.2, 0.01, 0.025], [0, 0.18, -0.03], '#2B2C2E', 'satin', radius=0.005), box([0.2, 0.01, 0.025], [0, 0.18, 0.03], '#2B2C2E', 'satin', radius=0.005),
               box([0.12, 0.06, 0.01], [0.0, 0.2, -0.03], '#D9A55B', 'matte', radius=0.02), box([0.03, 0.02, 0.03], [0.15, 0.13, 0], '#2B2C2E', 'satin', radius=0.006)])
tt('blender', [box([0.14, 0.1, 0.14], [0, 0.05, 0], '#3A3D41', 'satin', radius=0.03),
               flare([0.12, 0.22, 0.12], [0, 0.21, 0], '#DCE6EA', 'glass', 1.2, segments=14),
               flare([0.1, 0.1, 0.1], [0, 0.15, 0], '#E9A3B5', 'gloss', 1.1, segments=12),
               cyl([0.15, 0.03, 0.15], [0, 0.335, 0], '#2B2C2E', 'satin', segments=14),
               torus([0.09, 0.02, 0.12], [0.08, 0.22, 0], '#3A3D41', 'satin', rotate=[90, 90, 0], segments=10)])
MIX = '#C0473A'
tt('standMixer', [box([0.3, 0.04, 0.17], [0.0, 0.02, 0], MIX, 'gloss', radius=0.02),
                  box([0.09, 0.22, 0.12], [-0.1, 0.15, 0], MIX, 'gloss', radius=0.04),
                  part('capsule', [0.12, 0.3, 0.12], [-0.01, 0.29, 0], MIX, 'gloss', rotate=[0, 0, 90]),
                  lathe([0.16, 0.12, 0.16], [0.06, 0.1, 0], STEEL, 'metal', BOWL, segments=16), cyl([0.015, 0.08, 0.015], [0.06, 0.2, 0], STEEL, 'metal')])
tt('espressoMachineSmall', [box([0.22, 0.3, 0.26], [0, 0.15, -0.02], STEEL, 'metal', radius=0.02),
                            box([0.2, 0.02, 0.12], [0, 0.015, 0.14], '#2B2C2E', 'satin', radius=0.006),
                            cyl([0.06, 0.03, 0.06], [0, 0.2, 0.13], '#2B2C2E', 'satin'),
                            rod([0, 0.18, 0.13], [0, 0.18, 0.24], 0.025, '#2B2C2E', 'satin'),
                            cyl([0.05, 0.05, 0.05], [0, 0.05, 0.14], '#F2F0EA', 'gloss', segments=10),
                            cyl([0.04, 0.03, 0.04], [0.07, 0.31, -0.06], '#2B2C2E', 'satin')])
tt('knifeBlock', [box([0.12, 0.2, 0.2], [0, 0.13, 0], '#9C6B43', 'wood', radius=0.012, rotate=[-18, 0, 0])] +
   [box([0.024, 0.07, 0.024], [x, 0.27, -0.06 + abs(x) * 0.6], '#2B2C2E', 'satin', radius=0.006, rotate=[-18, 0, 0]) for x in (-0.035, 0.0, 0.035)] +
   [box([0.03, 0.06, 0.028], [0.0, 0.25, 0.01], '#2B2C2E', 'satin', radius=0.006, rotate=[-18, 0, 0])])
tt('utensilCrock', [cyl([0.12, 0.15, 0.12], [0, 0.075, 0], '#E8E2D6', 'gloss', segments=14)] +
   [rod([dx * 0.3, 0.1, dz * 0.3], [dx, 0.3, dz], 0.014, c, 'wood', segments=6) for dx, dz, c in ((0.03, 0.02, '#9C6B43'), (-0.03, 0.01, '#B98A5A'), (0.0, -0.035, '#9C6B43'), (0.04, -0.02, '#2B2C2E'))] +
   [sph([0.05, 0.02, 0.04], [0.0, 0.3, -0.035], '#9C6B43', 'wood', segments=8), box([0.05, 0.012, 0.07], [0.04, 0.31, -0.02], '#2B2C2E', 'wood', radius=0.004)])
tt('cuttingBoard', [box([0.36, 0.025, 0.24], [0, 0.0125, 0], '#C99A68', 'wood', radius=0.01),
                    part('capsule', [0.08, 0.2, 0.08], [-0.05, 0.065, 0.0], '#D9A55B', 'matte', rotate=[0, 70, 90]),
                    cyl([0.07, 0.015, 0.07], [0.1, 0.033, 0.05], '#D9483B', 'gloss', segments=12), cyl([0.06, 0.015, 0.06], [0.12, 0.03, -0.04], '#D9483B', 'gloss', segments=12),
                    box([0.02, 0.006, 0.14], [0.06, 0.028, -0.02], STEEL, 'metal', radius=0.002, rotate=[0, 40, 0])])
tt('breadBin', [box([0.34, 0.12, 0.2], [0, 0.06, 0], '#E8E2D6', 'gloss', radius=0.02),
                cyl([0.2, 0.34, 0.16], [0, 0.12, -0.02], '#E8E2D6', 'gloss', rotate=[0, 0, 90], segments=18),
                box([0.12, 0.02, 0.02], [0, 0.17, 0.08], WALNUT, 'wood', radius=0.008)])
p = [box([0.3, 0.015, 0.12], [0, 0.0075, 0], '#9C6B43', 'wood', radius=0.004)]
for i, c in enumerate(('#C98F4B', '#7A8C3A', '#B5452E', '#E3C66A', '#5E4B3A')):
    p += jar(-0.11 + i * 0.055, 0.0, 0.015, c, d=0.045, h=0.08, lid=BLACKMETAL, lid_fin='metal')
tt('spiceJars', p)
tt('cookieJar', [lathe([0.16, 0.18, 0.16], [0, 0.09, 0], '#DCE6EA', 'glass', JAR, segments=16),
                 cyl([0.12, 0.012, 0.12], [0, 0.03, 0], '#C98F4B', 'matte', segments=12), cyl([0.12, 0.012, 0.12], [0, 0.05, 0.01], '#8A5A36', 'matte', segments=12),
                 cyl([0.13, 0.03, 0.13], [0, 0.195, 0], '#C25B3E', 'gloss', segments=16), sph([0.04, 0.04, 0.04], [0, 0.22, 0], '#C25B3E', 'gloss', segments=8)])
tt('cakeStand', [lathe([0.28, 0.1, 0.28], [0, 0.05, 0], '#F2F0EA', 'gloss', [[0, -0.5], [0.3, -0.5], [0.08, -0.2], [0.08, 0.35], [0.5, 0.4], [0.5, 0.5], [0, 0.5]], segments=20),
                 cyl([0.22, 0.1, 0.22], [0, 0.15, 0], '#E9C8A0', 'matte', segments=20), cyl([0.225, 0.025, 0.225], [0, 0.21, 0], '#F7E7EC', 'gloss', segments=20),
                 sph([0.03, 0.03, 0.03], [0, 0.235, 0], RED, 'gloss', segments=8)])
TP = '#F2F0EA'
tt('teapotSet', [box([0.4, 0.015, 0.26], [0, 0.0075, 0], '#9C6B43', 'wood', radius=0.01),
                 sph([0.15, 0.13, 0.15], [-0.07, 0.08, -0.02], TP, 'gloss', segments=14), cyl([0.07, 0.02, 0.07], [-0.07, 0.145, -0.02], TP, 'gloss'),
                 sph([0.025, 0.025, 0.025], [-0.07, 0.165, -0.02], '#3E5470', 'gloss', segments=6),
                 rod([0.0, 0.07, -0.02], [0.06, 0.13, -0.02], 0.025, TP, 'gloss'), torus([0.09, 0.016, 0.06], [-0.15, 0.09, -0.02], TP, 'gloss', rotate=[90, 90, 0], segments=10)] +
   plate(0.12, -0.06, 0.015, d=0.1) + plate(0.1, 0.07, 0.015, d=0.1) +
   [flare([0.06, 0.045, 0.06], [0.12, 0.052, -0.06], TP, 'gloss', 1.2, segments=12), flare([0.06, 0.045, 0.06], [0.1, 0.052, 0.07], TP, 'gloss', 1.2, segments=12)])
tt('mugs', mug(-0.06, 0.03, 0, '#3E5470', rot=30) + mug(0.06, 0.04, 0, '#E9C46A', rot=-20) + mug(0.0, -0.06, 0, '#E4B8AE', rot=100))
tt('wineGlasses', wine_glass(-0.07, 0.05, 0, wine='#6E1E2A') + wine_glass(0.07, 0.05, 0, wine='#6E1E2A') +
   [lathe([0.08, 0.3, 0.08], [0.0, 0.15, -0.07], '#2F4A33', 'glass', BOTTLE, segments=12), cyl([0.082, 0.07, 0.082], [0.0, 0.12, -0.07], '#F2F0EA', 'matte', segments=12)])
tt('tableClock', [cyl([0.16, 0.05, 0.16], [0, 0.11, 0], '#C0473A', 'gloss', rotate=[90, 0, 0], segments=20),
                  cyl([0.13, 0.006, 0.13], [0, 0.11, 0.026], '#F7F4EC', 'matte', rotate=[90, 0, 0], segments=20),
                  box([0.006, 0.05, 0.004], [0, 0.125, 0.03], '#2B2C2E', 'matte', radius=0.001), box([0.04, 0.006, 0.004], [0.015, 0.11, 0.03], '#2B2C2E', 'matte', radius=0.001),
                  sph([0.06, 0.04, 0.06], [-0.05, 0.2, 0], BRASS, 'metal', segments=8), sph([0.06, 0.04, 0.06], [0.05, 0.2, 0], BRASS, 'metal', segments=8),
                  rod([-0.05, 0.0, 0.0], [-0.04, 0.05, 0.0], 0.012, BRASS, 'metal'), rod([0.05, 0.0, 0.0], [0.04, 0.05, 0.0], 0.012, BRASS, 'metal')])
tt('globe', [cyl([0.12, 0.02, 0.12], [0, 0.01, 0], '#7A5234', 'wood', segments=16), cyl([0.02, 0.06, 0.02], [0, 0.05, 0], '#7A5234', 'wood'),
             sph([0.22, 0.22, 0.22], [0, 0.2, 0], '#4E8FB8', 'gloss', segments=16),
             part('blob', [0.07, 0.09, 0.03], [0.035, 0.23, 0.098], '#8EB46A', 'matte', noise=0.25), part('blob', [0.05, 0.07, 0.03], [-0.06, 0.17, 0.088], '#8EB46A', 'matte', noise=0.25),
             torus([0.25, 0.012, 0.25], [0, 0.2, 0], BRASS, 'metal', rotate=[90, 0, 23], segments=24)])
tt('snowGlobe', [cyl([0.12, 0.05, 0.12], [0, 0.025, 0], '#7A5234', 'wood', taper=0.9, segments=16),
                 sph([0.13, 0.13, 0.13], [0, 0.11, 0], '#DCE6EA', 'glass', segments=14),
                 cone([0.05, 0.08, 0.05], [0, 0.1, 0], '#3F7A4A', 'satin', segments=8), cyl([0.1, 0.01, 0.1], [0, 0.055, 0], '#F7F7F5', 'satin', segments=12)])
tt('trophy', [box([0.1, 0.04, 0.1], [0, 0.02, 0], '#2B2C2E', 'gloss', radius=0.006), cyl([0.02, 0.07, 0.02], [0, 0.075, 0], '#D6A64A', 'metal'),
              lathe([0.12, 0.12, 0.12], [0, 0.17, 0], '#D6A64A', 'metal', CUP, segments=16),
              torus([0.06, 0.012, 0.04], [-0.065, 0.18, 0], '#D6A64A', 'metal', rotate=[90, 90, 0], segments=10), torus([0.06, 0.012, 0.04], [0.065, 0.18, 0], '#D6A64A', 'metal', rotate=[90, 90, 0], segments=10)])
tt('sculptureSmall', [box([0.12, 0.05, 0.12], [0, 0.025, 0], '#E7E3DC', 'matte', radius=0.006),
                      torus([0.16, 0.04, 0.16], [0, 0.14, 0], '#2B2C2E', 'satin', rotate=[90, 30, 0], segments=20),
                      sph([0.06, 0.06, 0.06], [0.03, 0.11, 0.02], BRASS, 'metal', segments=10)])
tt('soapSet', [box([0.26, 0.012, 0.12], [0, 0.006, 0], '#E7E3DC', 'matte', radius=0.004),
               cyl([0.06, 0.12, 0.06], [-0.08, 0.072, 0], '#F2F0EA', 'gloss', segments=12), box([0.012, 0.03, 0.04], [-0.08, 0.145, 0.01], BLACKMETAL, 'metal', radius=0.003),
               cyl([0.06, 0.08, 0.06], [0.02, 0.052, 0], '#9DC6CF', 'gloss', segments=12),
               rod([0.02, 0.06, 0.0], [0.0, 0.16, 0.01], 0.012, '#E4B8AE', 'gloss'), rod([0.02, 0.06, 0.0], [0.035, 0.16, -0.01], 0.012, '#7FA35A', 'gloss'),
               box([0.07, 0.025, 0.05], [0.09, 0.025, 0.0], '#E9DCC4', 'matte', radius=0.012)])
tt('towelStack', [box([0.26, 0.05, 0.18], [0, 0.025 + i * 0.05, 0], c, 'fabric', radius=0.02) for i, c in enumerate(('#9DC6CF', '#F2F0EA', '#E4B8AE'))] +
   [box([0.25, 0.006, 0.04], [0, 0.05 + i * 0.05, 0.09], '#E7E0D4', 'fabric', radius=0.002) for i in range(2)])
tt('perfumeTray', [box([0.26, 0.015, 0.16], [0, 0.0075, 0], '#D7E2E6', 'gloss', radius=0.006), torus([0.27, 0.01, 0.17], [0, 0.015, 0], BRASS, 'metal', segments=24),
                   box([0.06, 0.1, 0.04], [-0.07, 0.065, 0.0], '#E9A3B5', 'glass', radius=0.008), sph([0.03, 0.03, 0.03], [-0.07, 0.13, 0], BRASS, 'metal', segments=8),
                   lathe([0.06, 0.08, 0.06], [0.02, 0.055, 0.02], '#E9C46A', 'glass', BOTTLE, segments=12),
                   cyl([0.05, 0.07, 0.05], [0.08, 0.05, -0.03], '#9DC6CF', 'glass', segments=12), cyl([0.03, 0.02, 0.03], [0.08, 0.095, -0.03], '#2B2C2E', 'gloss')])
tt('deskOrganiser', [cyl([0.07, 0.1, 0.07], [-0.09, 0.05, 0.02], '#2B2C2E', 'satin', segments=12)] +
   [rod([-0.09, 0.06, 0.02], [-0.09 + dx, 0.16, 0.02 + dz], 0.008, c, 'gloss', segments=5) for dx, dz, c in ((0.015, 0.0, RED), (-0.01, 0.01, BLUE), (0.0, -0.015, YELLOW))] +
   [box([0.16, 0.015, 0.2], [0.06, 0.0075 + i * 0.05, 0.0], '#2B2C2E', 'satin', radius=0.004) for i in range(2)] +
   [box([0.14, 0.01, 0.19], [0.06, 0.02 + i * 0.05, 0.0], '#F7F7F5', 'matte', radius=0.002) for i in range(2)] +
   [box([0.01, 0.06, 0.2], [0.13, 0.03, 0.0], '#2B2C2E', 'satin', radius=0.003)])
tt('laptopClosed', [box([0.32, 0.018, 0.22], [0, 0.009, 0], '#B9BCBF', 'metal', radius=0.008),
                    box([0.3, 0.004, 0.2], [0, 0.02, 0], '#A9ADB0', 'metal', radius=0.006),
                    sph([0.03, 0.002, 0.03], [0, 0.0222, 0.0], '#E7E7E5', 'gloss', segments=8)])
tt('monitor', [box([0.54, 0.32, 0.025], [0, 0.3, -0.06], '#1E1F22', 'satin', radius=0.008), box([0.52, 0.3, 0.004], [0, 0.3, -0.046], SCREEN, 'gloss', radius=0.004),
               box([0.05, 0.16, 0.03], [0, 0.11, -0.09], '#3A3D41', 'metal', radius=0.008), box([0.2, 0.012, 0.14], [0, 0.006, -0.08], '#3A3D41', 'metal', radius=0.01),
               box([0.38, 0.016, 0.12], [0, 0.008, 0.12], '#2B2C2E', 'satin', radius=0.006), box([0.06, 0.02, 0.09], [0.25, 0.01, 0.12], '#2B2C2E', 'satin', radius=0.02)])
BEAR = '#B98656'
tt('stuffedToy', [sph([0.15, 0.16, 0.13], [0, 0.08, 0], BEAR, 'fabric', segments=12), sph([0.13, 0.12, 0.12], [0, 0.21, 0.01], BEAR, 'fabric', segments=12),
                  sph([0.05, 0.04, 0.04], [0, 0.2, 0.065], '#E9D5B5', 'fabric', segments=8), sph([0.045, 0.045, 0.03], [-0.05, 0.27, 0], BEAR, 'fabric', segments=8),
                  sph([0.045, 0.045, 0.03], [0.05, 0.27, 0], BEAR, 'fabric', segments=8),
                  part('capsule', [0.05, 0.12, 0.05], [-0.06, 0.03, 0.06], BEAR, 'fabric', rotate=[80, 0, 10]), part('capsule', [0.05, 0.12, 0.05], [0.06, 0.03, 0.06], BEAR, 'fabric', rotate=[80, 0, -10]),
                  sph([0.04, 0.02, 0.03], [0, 0.17, 0.06], RED, 'gloss', segments=6)])
tt('toyBlocks', [box([0.07, 0.07, 0.07], [x, y, z], c, 'gloss', radius=0.008, rotate=[0, r, 0]) for x, y, z, c, r in
                 ((-0.05, 0.035, -0.02, RED, 10), (0.03, 0.035, -0.03, BLUE, -15), (-0.01, 0.105, -0.02, YELLOW, 30), (0.08, 0.035, 0.06, GREEN, 20), (-0.07, 0.035, 0.07, '#E07BC3', 0))] +
   [cone([0.07, 0.06, 0.07], [-0.01, 0.17, -0.02], GREEN, 'gloss', segments=4, rotate=[0, 75, 0])])


# =============================================================================================
# Lighting (lamps on surfaces have their origin at their base; `LIGHTS` is the bulb height above it)
# =============================================================================================
def drum_shade(y, d, h, color, taper=0.85):
    return [cyl([d, h, d], [0, y, 0], color, 'fabric', taper=taper, segments=24),
            cyl([d * 0.9, 0.004, d * 0.9], [0, y - h / 2 + 0.01, 0], BULB, 'gloss', segments=20)]


add('lampTableRound', [lathe([0.2, 0.26, 0.2], [0, 0.13, 0], '#3E6E9A', 'gloss', [[0, -0.5], [0.3, -0.5], [0.48, -0.25], [0.5, 0.05], [0.38, 0.38], [0.15, 0.45], [0.12, 0.5], [0, 0.5]], segments=20),
                       cyl([0.02, 0.1, 0.02], [0, 0.31, 0], BRASS, 'metal')] + drum_shade(0.44, 0.34, 0.22, '#F4EBD8'), kind='surface', light=0.42)
add('lampTableSquare', [box([0.12, 0.26, 0.12], [0, 0.13, 0], '#E7E3DC', 'matte', radius=0.01), cyl([0.018, 0.1, 0.018], [0, 0.31, 0], BLACKMETAL, 'metal'),
                        cyl([0.3, 0.2, 0.3], [0, 0.44, 0], '#EFE8DA', 'fabric', taper=0.8, segments=4, rotate=[0, 45, 0]),
                        box([0.18, 0.004, 0.18], [0, 0.35, 0], BULB, 'gloss', radius=0.002)], kind='surface', light=0.4)
add('lampBanker', [box([0.22, 0.02, 0.14], [0, 0.01, 0], BRASS, 'metal', radius=0.008), cyl([0.018, 0.3, 0.018], [-0.06, 0.16, 0], BRASS, 'metal'),
                   rod([-0.06, 0.3, 0], [0.0, 0.31, 0.03], 0.016, BRASS, 'metal'),
                   cyl([0.12, 0.3, 0.12], [0.0, 0.33, 0.03], '#2F6B4F', 'glass', rotate=[0, 0, 90], segments=20),
                   box([0.28, 0.01, 0.07], [0, 0.29, 0.03], '#FFF1CF', 'gloss', radius=0.003),
                   cyl([0.02, 0.06, 0.02], [0.08, 0.05, 0.03], BRASS, 'metal')], kind='surface', light=0.3)
add('lampMushroom', [cyl([0.14, 0.02, 0.14], [0, 0.01, 0], '#F2F0EA', 'gloss', segments=20), cyl([0.03, 0.2, 0.03], [0, 0.11, 0], '#F2F0EA', 'gloss', taper=0.5),
                     lathe([0.3, 0.16, 0.3], [0, 0.28, 0], '#E9A07B', 'gloss', [[0.5, -0.5], [0.46, 0.15], [0.3, 0.42], [0, 0.5], [0, 0.4], [0.28, 0.32], [0.44, 0.1], [0.47, -0.5]], segments=24),
                     sph([0.1, 0.05, 0.1], [0, 0.22, 0], BULB, 'gloss', segments=10)], kind='surface', light=0.22)
p = [box([0.16, 0.02, 0.16], [0, 0.01, 0], BLACKMETAL, 'metal', radius=0.004), box([0.14, 0.22, 0.14], [0, 0.13, 0], '#DCE6EA', 'glass', radius=0.004),
     cone([0.19, 0.08, 0.19], [0, 0.28, 0], BLACKMETAL, 'metal', segments=4, rotate=[0, 45, 0]),
     torus([0.06, 0.01, 0.06], [0, 0.34, 0], BLACKMETAL, 'metal', rotate=[90, 0, 0], segments=10),
     cyl([0.05, 0.1, 0.05], [0, 0.07, 0], '#F4EBD8', 'satin', segments=10), sph([0.025, 0.04, 0.025], [0, 0.14, 0], '#FFD27A', 'gloss', segments=6)]
p += [box([0.016, 0.22, 0.016], [x, 0.13, z], BLACKMETAL, 'metal', radius=0.003) for x in (-0.07, 0.07) for z in (-0.07, 0.07)]
add('lampLantern', p, kind='surface', light=0.15)
p = [cyl([0.3, 0.06, 0.3], [-0.3, 0.03, -0.3], '#E7E3DC', 'gloss', segments=20)]
pts = [(-0.3, 0.06, -0.3), (-0.3, 1.0, -0.3), (-0.26, 1.55, -0.24), (-0.12, 1.85, -0.1), (0.08, 1.88, 0.06), (0.22, 1.72, 0.18)]
p += [rod(list(pts[i]), list(pts[i + 1]), 0.025, '#C9A06A', 'metal', segments=8) for i in range(len(pts) - 1)]
p += [lathe([0.36, 0.2, 0.36], [0.22, 1.6, 0.18], '#C9A06A', 'metal', [[0.5, -0.5], [0.42, 0.2], [0.12, 0.5], [0, 0.5], [0, 0.4], [0.1, 0.4], [0.38, 0.15], [0.46, -0.5]], segments=24),
      sph([0.12, 0.08, 0.12], [0.22, 1.55, 0.18], BULB, 'gloss', segments=10)]
add('lampFloorArc', p, light=1.5)
p = [rod([math.cos(math.radians(a)) * 0.3, 0, math.sin(math.radians(a)) * 0.3], [0, 1.25, 0], 0.035, WALNUT, 'wood', segments=8) for a in (90, 210, 330)]
p += [cyl([0.03, 0.08, 0.03], [0, 1.28, 0], BRASS, 'metal')] + drum_shade(1.45, 0.48, 0.32, '#E7E0D4', taper=1.0)
add('lampFloorTripod', p, light=1.38)
add('lampTorchiere', [cyl([0.3, 0.04, 0.3], [0, 0.02, 0], '#2B2C2E', 'metal', segments=20), cyl([0.025, 1.68, 0.025], [0, 0.88, 0], BRASS, 'metal'),
                      lathe([0.4, 0.14, 0.4], [0, 1.78, 0], '#F2EBDD', 'glass', [[0.04, -0.5], [0.3, -0.1], [0.5, 0.5], [0.46, 0.5], [0.27, -0.06], [0, -0.4]], segments=24),
                      cyl([0.05, 0.04, 0.05], [0, 1.72, 0], BRASS, 'metal')], light=1.78)
p = [box([1.0, 0.04, 0.05], [0, CEILING - 0.02, 0], '#F2F2F0', 'satin', radius=0.01), box([0.12, 0.02, 0.12], [0, CEILING - 0.01, 0], '#F2F2F0', 'satin', radius=0.01)]
for x, r in ((-0.36, 20), (0.0, -15), (0.36, 25)):
    p += [cyl([0.02, 0.08, 0.02], [x, CEILING - 0.08, 0], '#2B2C2E', 'metal'),
          cyl([0.09, 0.16, 0.09], [x, CEILING - 0.18, 0], '#2B2C2E', 'metal', rotate=[r, 0, r * 0.6], segments=14),
          cyl([0.07, 0.01, 0.07], [x + math.sin(math.radians(r * 0.6)) * -0.08, CEILING - 0.26, math.sin(math.radians(r)) * 0.08], BULB, 'gloss', rotate=[r, 0, r * 0.6], segments=12)]
add('trackLights', p, kind='ceiling', light=2.5)
add('paperLantern', [cyl([0.08, 0.02, 0.08], [0, CEILING - 0.01, 0], '#F2F2F0', 'satin', segments=12), cyl([0.008, 0.48, 0.008], [0, CEILING - 0.26, 0], '#F2F2F0', 'satin', segments=6),
                     sph([0.56, 0.52, 0.56], [0, CEILING - 0.75, 0], '#FBF7EE', 'fabric', segments=18)] +
    [torus([0.56 * math.sqrt(1 - (k / 0.27) ** 2) + 0.01, 0.006, 0.56 * math.sqrt(1 - (k / 0.27) ** 2) + 0.01], [0, CEILING - 0.75 + k, 0], '#EDE6D6', 'fabric', segments=24)
     for k in (-0.2, -0.12, -0.04, 0.04, 0.12, 0.2)], kind='ceiling', light=2.05)


# =============================================================================================
# Outdoor
# =============================================================================================
p = table_round(0.8, '#3E6E73', '#2B2C2E', 'gloss', 'metal', h=0.72)
p += [torus([0.78, 0.03, 0.78], [0, 0.705, 0], '#2B2C2E', 'metal', segments=28), cyl([0.05, 0.004, 0.05], [0, 0.722, 0], '#1E1F22', 'satin')]
p += [cyl([0.06, 0.04, 0.06], [0.24 * math.cos(a), 0.722, 0.24 * math.sin(a)], '#E9C46A', 'gloss', segments=6) for a in (0.3, 1.5, 2.6, 3.8, 5.0)]
add('patioTableRound', p, top=round_top('patioTableRound', 0.8, 0.72))
p = [box([1.6, 0.03, 0.14], [0, 0.725, z], TEAK, 'wood', radius=0.008) for z in (-0.36, -0.2, -0.04, 0.12, 0.28)]
p += [box([1.5, 0.06, 0.04], [0, 0.68, s * 0.38], '#7E5636', 'wood', radius=0.008) for s in (-1, 1)]
p += legs4((-0.72, 0.72), (-0.36, 0.36), 0.7, 0.06, '#7E5636', 'wood')
add('patioTable4', p, (2, 1), top=(-0.8, 0.8, -0.43, 0.35, 0.74))
PC = '#2F5A4E'
p = [box([0.46, 0.03, 0.42], [0, 0.45, SZ], PC, 'metal', radius=0.01),
     cushion([0.42, 0.05, 0.38], [0, 0.485, SZ + 0.01], '#E8E2D6', radius=0.02)]
for x in (-0.2, 0.2):
    p += [rod([x, 0, SZ + 0.2], [x, 0.44, SZ + 0.18], 0.025, PC, 'metal', segments=8),
          rod([x, 0, SZ - 0.22], [x * 0.95, 0.92, SZ - 0.2], 0.025, PC, 'metal', segments=8),
          rod([x, 0.44, SZ + 0.18], [x, 0.66, SZ + 0.12], 0.02, PC, 'metal', segments=8),
          rod([x, 0.66, SZ + 0.12], [x, 0.68, SZ - 0.18], 0.02, PC, 'metal', segments=8)]
for y in (0.6, 0.72, 0.84):
    p.append(box([0.4, 0.035, 0.012], [0, y, SZ - 0.205 + (y - 0.6) * -0.02], PC, 'metal', radius=0.004))
add('patioChair', p, seat=(0.49, SZ))
AD = '#3E7A6E'
p = []
for i in range(5):
    a = (i - 2) * 6
    p.append(box([0.1, 0.84, 0.025], [math.sin(math.radians(a)) * 0.42, 0.68, -0.28], AD, 'satin', radius=0.012, rotate=[-22, 0, -a]))
p.append(box([0.6, 0.06, 0.03], [0, 0.56, -0.24], AD, 'satin', radius=0.01, rotate=[-22, 0, 0]))
for z in (-0.12, 0.0, 0.12, 0.24):
    p.append(box([0.56, 0.025, 0.1], [0, 0.4 - z * 0.15, z], AD, 'satin', radius=0.008))
for s in (-1, 1):
    p += [box([0.14, 0.025, 0.7], [s * 0.37, 0.62, -0.02], AD, 'satin', radius=0.01),
          box([0.06, 0.62, 0.06], [s * 0.36, 0.31, 0.26], AD, 'satin', radius=0.01),
          box([0.06, 0.42, 0.06], [s * 0.32, 0.21, -0.24], AD, 'satin', radius=0.01),
          box([0.05, 0.08, 0.6], [s * 0.32, 0.36, 0.0], AD, 'satin', radius=0.01, rotate=[10, 0, 0])]
add('adirondackChair', p, seat=(0.42, 0.04))
BEN = '#A9744E'
p = []
for z in (-0.08, 0.04, 0.16):
    p.append(box([2.7, 0.03, 0.1], [0, 0.45, z], BEN, 'wood', radius=0.008))
for y in (0.6, 0.72, 0.84):
    p.append(box([2.7, 0.09, 0.025], [0, y, -0.2 - (y - 0.6) * 0.15], BEN, 'wood', radius=0.008, rotate=[-12, 0, 0]))
for x in (-1.3, 0.0, 1.3):
    p += [box([0.06, 0.45, 0.06], [x, 0.225, 0.18], IRON, 'metal', radius=0.01),
          rod([x, 0, -0.18], [x, 0.9, -0.3], 0.06, IRON, 'metal', shape='box', t2=0.05, radius=0.01),
          box([0.06, 0.05, 0.46], [x, 0.42, 0.0], IRON, 'metal', radius=0.01)]
for x in (-1.3, 1.3):
    p += [box([0.08, 0.04, 0.5], [x, 0.66, 0.02], IRON, 'metal', radius=0.015), box([0.04, 0.24, 0.04], [x, 0.54, 0.22], IRON, 'metal', radius=0.01)]
add('benchGarden3', p, (3, 1), seat=(0.47, 0.02))
PSW = '#EDE6D8'
p = []
for x in (-0.92, 0.92):
    p += [rod([x, 0, -0.42], [x, 2.0, 0], 0.08, '#8E6E52', 'wood', shape='box', radius=0.01),
          rod([x, 0, 0.42], [x, 2.0, 0], 0.08, '#8E6E52', 'wood', shape='box', radius=0.01)]
p.append(box([1.92, 0.1, 0.1], [0, 2.0, 0], '#8E6E52', 'wood', radius=0.01))
p += [box([1.4, 0.04, 0.46], [0, 0.48, 0.04], PSW, 'satin', radius=0.01), box([1.4, 0.4, 0.04], [0, 0.72, -0.2], PSW, 'satin', radius=0.01, rotate=[-12, 0, 0]),
      cushion([1.3, 0.07, 0.42], [0, 0.53, 0.05], '#7FA3B8'), box([0.05, 0.22, 0.46], [-0.68, 0.6, 0.02], PSW, 'satin', radius=0.01),
      box([0.05, 0.22, 0.46], [0.68, 0.6, 0.02], PSW, 'satin', radius=0.01)]
for x in (-0.66, 0.66):
    for z in (-0.18, 0.24):
        p.append(rod([x, 0.5, z], [x * 1.02, 1.95, 0.0], 0.012, '#8E959B', 'metal', segments=6))
add('porchSwing', p, (2, 1), seat=(0.55, 0.04))
add('parasol', [cyl([0.4, 0.08, 0.4], [0, 0.04, 0], '#3A3D41', 'satin', segments=20), cyl([0.04, 2.3, 0.04], [0, 1.19, 0], '#E7E3DC', 'satin', segments=10),
                cone([2.2, 0.42, 2.2], [0, 2.18, 0], '#E7DCC4', 'fabric', segments=8, taper=0.04),
                cyl([2.2, 0.12, 2.2], [0, 1.93, 0], '#C9A979', 'fabric', segments=8, taper=1.0),
                sph([0.07, 0.08, 0.07], [0, 2.42, 0], '#E7E3DC', 'satin', segments=8)])
PB = '#8A6A4E'
p = [box([0.9, 0.44, 0.42], [0, 0.22, -0.1], PB, 'wood', radius=0.01), box([0.94, 0.04, 0.46], [0, 0.45, -0.1], '#7A5A40', 'wood', radius=0.01),
     box([0.84, 0.02, 0.36], [0, 0.44, -0.1], SOIL, 'matte', radius=0.005)]
for i, x in enumerate((-0.32, -0.16, 0.0, 0.16, 0.32)):
    p += [part('blob', [0.2, 0.2, 0.22], [x, 0.54, -0.1], FOLIAGE if i % 2 else LEAF2, 'foliage', noise=0.3),
          sph([0.07, 0.06, 0.07], [x + 0.03, 0.64, -0.06], ['#E07BC3', '#F2C14E', '#F2F0EA', '#D9483B', '#9B7FD1'][i], 'satin', segments=8),
          sph([0.06, 0.05, 0.06], [x - 0.04, 0.6, -0.16], ['#F2C14E', '#E07BC3', '#D9483B', '#F2F0EA', '#F2C14E'][i], 'satin', segments=8)]
add('planterBox', p)
RB = '#9A7452'
p = [box([1.84, 0.34, 0.86], [0, 0.17, 0], RB, 'wood', radius=0.012), box([1.76, 0.02, 0.78], [0, 0.33, 0], SOIL, 'matte', radius=0.005)]
p += [box([0.08, 0.38, 0.08], [x, 0.19, z], '#7A5A40', 'wood', radius=0.01) for x in (-0.88, 0.88) for z in (-0.39, 0.39)]
for row, z in enumerate((-0.24, 0.0, 0.24)):
    for i in range(6):
        x = -0.7 + i * 0.28
        if row == 0:
            p.append(part('blob', [0.18, 0.12, 0.18], [x, 0.4, z], '#7FB35A', 'foliage', noise=0.35))
        elif row == 1:
            p += [cone([0.04, 0.14, 0.04], [x + d, 0.42, z], '#5E8F4E', 'foliage', segments=5) for d in (-0.03, 0.03)]
            p.append(cone([0.05, 0.06, 0.05], [x, 0.33, z], '#E07A2E', 'satin', segments=8, rotate=[180, 0, 0]))
        else:
            p += [part('blob', [0.2, 0.18, 0.2], [x, 0.43, z], '#4F7A3E', 'foliage', noise=0.3), sph([0.05, 0.05, 0.05], [x + 0.05, 0.46, z + 0.05], RED, 'satin', segments=8)]
add('raisedBed', p, (2, 1))
SH, SHR = '#8FA79A', '#5A4A3E'
p = [box([1.8, 1.7, 1.6], [0, 0.85, -0.1], SH, 'wood', radius=0.01),
     box([1.86, 0.08, 1.66], [0, 0.04, -0.1], '#6E6A64', 'matte', radius=0.01)]
for s in (-1, 1):
    p.append(box([1.1, 0.06, 1.84], [s * 0.47, 2.0, -0.1], SHR, 'matte', radius=0.01, rotate=[0, 0, -s * 33]))
p += [box([0.2, 0.5, 1.6], [0, 1.82, -0.1], SH, 'wood', radius=0.01, rotate=[0, 0, 45]),
      box([0.76, 1.44, 0.04], [-0.3, 0.8, 0.71], '#F2F0EA', 'satin', radius=0.01),
      box([0.6, 1.3, 0.02], [-0.3, 0.8, 0.735], '#7E9A8D', 'wood', radius=0.006),
      sph([0.04, 0.04, 0.04], [-0.04, 0.8, 0.75], BLACKMETAL, 'satin', segments=8),
      box([0.5, 0.4, 0.04], [0.5, 1.2, 0.71], '#F2F0EA', 'satin', radius=0.01),
      box([0.42, 0.32, 0.02], [0.5, 1.2, 0.735], '#3A4A52', 'satin', radius=0.004),
      box([0.6, 0.12, 0.2], [0.5, 0.9, 0.8], '#8E6E52', 'wood', radius=0.01)]
p += [part('blob', [0.14, 0.12, 0.14], [0.36 + i * 0.14, 1.0, 0.8], ['#E07BC3', '#F2C14E', '#D9483B', '#9B7FD1'][i], 'foliage', noise=0.3) for i in range(4)]
add('shedSmall', p, (2, 2))
GH, GHF = '#DCEAE4', '#F2F2F0'
p = [box([1.8, 0.4, 2.8], [0, 0.2, 0], '#B5A48E', 'satin', radius=0.01),
     box([1.76, 1.4, 2.76], [0, 1.1, 0], GH, 'glass', radius=0.004)]
for s in (-1, 1):
    p.append(box([1.06, 0.02, 2.8], [s * 0.45, 2.05, 0], GH, 'glass', radius=0.004, rotate=[0, 0, -s * 35]))
for z in (-1.38, -0.69, 0.0, 0.69, 1.38):
    for x in (-0.89, 0.89):
        p.append(box([0.04, 1.4, 0.04], [x, 1.1, z], GHF, 'satin', radius=0.004))
    for s in (-1, 1):
        p.append(box([1.06, 0.04, 0.04], [s * 0.45, 2.07, z], GHF, 'satin', radius=0.004, rotate=[0, 0, -s * 35]))
for x in (-0.89, 0.89):
    p.append(box([0.04, 0.04, 2.8], [x, 1.8, 0], GHF, 'satin', radius=0.004))
p += [box([0.06, 0.06, 2.84], [0, 2.36, 0], GHF, 'satin', radius=0.004), box([0.62, 1.7, 0.03], [0, 0.95, 1.4], GHF, 'satin', radius=0.006),
      box([0.54, 1.6, 0.01], [0, 0.95, 1.41], GH, 'glass', radius=0.004)]
for x in (-0.6, 0.6):
    p.append(box([0.5, 0.04, 2.3], [x, 0.78, -0.1], '#9C7A58', 'wood', radius=0.006))
    for z in (-1.0, -0.6, -0.2, 0.2, 0.6):
        p += [flare([0.14, 0.12, 0.14], [x, 0.86, z], '#C26E4A', 'satin', 1.15, segments=10), part('blob', [0.2, 0.2, 0.2], [x, 1.0, z], FOLIAGE, 'foliage', noise=0.3)]
add('greenhouse', p, (2, 3))
p = [box([0.8, 0.7, 0.8], [0, 0.35, -0.08], '#3A3D41', 'satin', radius=0.02)]
p[0] = box([0.06, 0.72, 0.8], [-0.37, 0.36, -0.08], '#8E6E52', 'wood', radius=0.008)
p += [box([0.06, 0.72, 0.8], [0.37, 0.36, -0.08], '#8E6E52', 'wood', radius=0.008)]
for y in (0.08, 0.26, 0.44, 0.62):
    p += [box([0.8, 0.12, 0.04], [0, y, s], '#8E6E52', 'wood', radius=0.006) for s in (-0.46, 0.3)]
p += [part('blob', [0.66, 0.36, 0.66], [0, 0.5, -0.08], '#4A3A2C', 'matte', noise=0.25), part('blob', [0.2, 0.08, 0.16], [0.1, 0.7, 0.0], '#7FA35A', 'foliage', noise=0.4),
      part('blob', [0.12, 0.06, 0.1], [-0.15, 0.69, -0.12], '#E07A2E', 'foliage', noise=0.4)]
add('compostBin', p)
p = [cyl([0.56, 0.86, 0.56], [0, 0.48, -0.15], '#3F6E4A', 'satin', taper=0.92, segments=22),
     cyl([0.6, 0.06, 0.6], [0, 0.03, -0.15], '#3F6E4A', 'satin', segments=22),
     cyl([0.54, 0.04, 0.54], [0, 0.92, -0.15], '#35603F', 'satin', segments=22),
     torus([0.58, 0.03, 0.58], [0, 0.3, -0.15], '#35603F', 'satin', segments=22), torus([0.55, 0.03, 0.55], [0, 0.66, -0.15], '#35603F', 'satin', segments=22),
     rod([0, 0.18, 0.13], [0, 0.18, 0.22], 0.03, BRASS, 'metal'), cyl([0.05, 0.03, 0.03], [0, 0.22, 0.2], BRASS, 'metal'),
     cyl([0.08, 1.2, 0.08], [0.0, 1.5, -0.4], '#8E959B', 'metal', segments=10), rod([0.0, 1.0, -0.4], [0.0, 0.94, -0.22], 0.08, '#8E959B', 'metal', segments=10)]
add('rainBarrel', p)
CLP = '#B9BCBE'
p = []
for x in (-1.32, 1.32):
    p += [cyl([0.06, 1.8, 0.06], [x, 0.9, 0], CLP, 'metal', segments=10), box([0.06, 0.05, 0.7], [x, 1.78, 0], CLP, 'metal', radius=0.01)]
for z in (-0.26, 0.0, 0.26):
    p.append(cyl([0.008, 2.64, 0.008], [0, 1.76, z], '#F2F2F0', 'matte', rotate=[0, 0, 90], segments=5))
for x, z, w, h, c in ((-0.9, -0.26, 0.5, 0.6, '#9DC6CF'), (-0.3, -0.26, 0.4, 0.5, '#F2F0EA'), (0.4, -0.26, 0.6, 0.4, '#E4B8AE'),
                      (-0.6, 0.0, 0.36, 0.7, '#3E5470'), (0.2, 0.0, 0.5, 0.36, YELLOW), (0.9, 0.0, 0.3, 0.56, '#F2F0EA'),
                      (-1.0, 0.26, 0.3, 0.3, RED), (0.0, 0.26, 0.6, 0.6, '#9DB7C9'), (0.85, 0.26, 0.4, 0.46, SAGE)):
    p += [box([w, h, 0.01], [x, 1.76 - h / 2, z], c, 'fabric', radius=0.004),
          box([0.02, 0.05, 0.02], [x - w / 2 + 0.05, 1.76, z], '#C08A52', 'matte', radius=0.004), box([0.02, 0.05, 0.02], [x + w / 2 - 0.05, 1.76, z], '#C08A52', 'matte', radius=0.004)]
p.append(box([0.5, 0.26, 0.34], [-1.0, 0.13, 0.22], '#C9A979', 'fabric', radius=0.04))
add('clothesline', p, (3, 1))
add('outdoorBar', bar(-1.0, 1.0, body=TEAK, top='#C9A06A', rail='#8E959B'), (2, 1), top=(-1.0, 1.0, -0.48, 0.4, BT))
p = [box([0.06, 1.5, 0.06], [0, 0.75, 0], '#8E6E52', 'wood', radius=0.008),
     box([0.24, 0.26, 0.22], [0, 1.63, 0], '#E9DCC4', 'satin', radius=0.01),
     cyl([0.07, 0.01, 0.07], [0, 1.66, 0.111], '#2B2C2E', 'matte', rotate=[90, 0, 0], segments=12),
     cyl([0.015, 0.06, 0.015], [0, 1.58, 0.13], '#8E6E52', 'wood', rotate=[90, 0, 0])]
for s in (-1, 1):
    p.append(box([0.2, 0.025, 0.3], [s * 0.08, 1.82, 0], '#C0473A', 'satin', radius=0.006, rotate=[0, 0, -s * 40]))
add('birdhouse', p)
p = [cyl([0.03, 1.9, 0.03], [0, 0.95, -0.2], BLACKMETAL, 'metal', segments=8)]
for s in (-1, 1):
    p += [rod([0, 1.75, -0.2], [s * 0.28, 1.86, -0.2], 0.02, BLACKMETAL, 'metal'), rod([s * 0.28, 1.86, -0.2], [s * 0.32, 1.78, -0.2], 0.02, BLACKMETAL, 'metal'),
          cyl([0.005, 0.1, 0.005], [s * 0.32, 1.72, -0.2], BLACKMETAL, 'metal', segments=4),
          cyl([0.12, 0.2, 0.12], [s * 0.32, 1.56, -0.2], '#DCE6EA', 'glass', segments=12), cyl([0.1, 0.12, 0.1], [s * 0.32, 1.52, -0.2], '#C9A979', 'matte', segments=10),
          cone([0.16, 0.06, 0.16], [s * 0.32, 1.69, -0.2], '#2F5A4E', 'satin', segments=12), cyl([0.18, 0.02, 0.18], [s * 0.32, 1.45, -0.2], '#2F5A4E', 'satin', segments=12)]
p.append(sph([0.08, 0.06, 0.1], [0.3, 1.5, -0.08], '#D9483B', 'satin', segments=8))
add('birdFeederPole', p)
DH = '#B5653F'
p = [box([0.8, 0.6, 0.86], [0, 0.32, -0.04], DH, 'wood', radius=0.01), box([0.84, 0.04, 0.9], [0, 0.02, -0.04], '#6E6A64', 'matte', radius=0.008)]
for s in (-1, 1):
    p.append(box([0.56, 0.04, 0.96], [s * 0.22, 0.79, -0.04], '#5A4A3E', 'matte', radius=0.008, rotate=[0, 0, -s * 38]))
p += [box([0.4, 0.4, 0.86], [0, 0.65, -0.04], DH, 'wood', radius=0.01, rotate=[0, 0, 45]),
      box([0.32, 0.36, 0.02], [0, 0.22, 0.39], '#2B2C2E', 'matte', radius=0.005), cyl([0.32, 0.02, 0.18], [0, 0.4, 0.39], '#2B2C2E', 'matte', rotate=[90, 0, 0], segments=16),
      box([0.24, 0.06, 0.02], [0, 0.58, 0.395], '#F2F0EA', 'satin', radius=0.01),
      cyl([0.16, 0.05, 0.16], [0.28, 0.025, 0.42], '#3E8EDE', 'gloss', segments=14)]
add('dogHouse', p)
add('sundial', [lathe([0.36, 0.86, 0.36], [0, 0.43, 0], '#C9C2B4', 'matte', [[0, -0.5], [0.5, -0.5], [0.5, -0.42], [0.26, -0.36], [0.2, 0.3], [0.36, 0.42], [0.46, 0.5], [0, 0.5]], segments=20),
                cyl([0.32, 0.015, 0.32], [0, 0.868, 0], '#B87333', 'metal', segments=24),
                box([0.01, 0.12, 0.14], [0, 0.92, 0.0], '#B87333', 'metal', radius=0.002, rotate=[-38, 0, 0])] +
   [box([0.012, 0.004, 0.03], [math.sin(math.radians(a)) * 0.13, 0.877, math.cos(math.radians(a)) * 0.13], '#6E4A2A', 'metal', radius=0.001, rotate=[0, a, 0]) for a in range(0, 360, 30)])
p = [box([0.9, 0.04, 0.36], [0, 0.06, -0.24], BLACKMETAL, 'metal', radius=0.006)]
p += [box([0.04, 1.0, 0.04], [x, 0.5, z], BLACKMETAL, 'metal', radius=0.006) for x in (-0.44, 0.44) for z in (-0.4, -0.08)]
p += [box([0.92, 0.04, 0.04], [0, 1.0, z], BLACKMETAL, 'metal', radius=0.006) for z in (-0.4, -0.08)]
_rng = random.Random(11)
for row in range(6):
    for col in range(6 - (row % 2)):
        x = -0.36 + col * 0.145 + (row % 2) * 0.07
        p.append(cyl([0.13, 0.32, 0.13], [x, 0.14 + row * 0.12, -0.24], ['#8E6A4A', '#7A5A3E', '#9C7A58'][(row + col) % 3], 'bark', rotate=[90, 0, _rng.uniform(0, 40)], segments=7))
p += [cyl([0.12, 0.003, 0.12], [-0.36 + c * 0.145, 0.14 + 5 * 0.12, -0.079], '#D9C3A0', 'matte', rotate=[90, 0, 0], segments=7) for c in range(5)]
add('firewoodStack', p)
add('steppingStones', [cyl([d, 0.04, d * 0.86], [x, 0.02, z], c, 'matte', segments=14, rotate=[0, r, 0]) for x, z, d, c, r in
                       ((0.05, -1.05, 0.52, '#A9A49A', 10), (-0.08, -0.35, 0.48, '#B5AFA4', 40), (0.06, 0.35, 0.54, '#9E998F', 70), (-0.04, 1.05, 0.5, '#B0AA9E', 15))], (1, 3), kind='rug')


# =============================================================================================
# Validation (footprint bounds, finishes) and output
# =============================================================================================
# Things that may reach past their tiles: things on a wall or ceiling, a parasol's canopy (well above head height).
OVERHANG = {'parasol'}
FINISHES = {'material.finish.' + f for f in 'matte satin gloss metal chrome fabric wood bark foliage skin glass'.split()}


def main(path):
    bad = 0
    for k, e in E.items():
        key = k.split('.', 1)[1]
        fp, kind = FOOT[key], KIND[key]
        parts = e['placeholder']
        issues = []
        hung = kind in ('wall', 'ceiling') or key in OVERHANG
        fins = {p.get('material') for p in parts}
        if len(fins) > 4: issues.append(f'{len(fins)} finishes (max 4): {sorted(f.split(".")[-1] for f in fins)}')
        for p in parts:
            q = dict(p, shape='cylinder') if p['shape'] in ('lathe', 'torus') else p
            issues += [i for i in OX.validate(k, [q], fp) if 'finishes (max 4)' not in i and 'exceeds' not in i and 'below floor' not in i]
            lo, hi = OX.bounds(q)
            if lo[1] < -0.015: issues.append(f'part {p["shape"]} at {p["at"]} below the floor')
            if not hung and (lo[0] < -fp[0] / 2 - 0.02 or hi[0] > fp[0] / 2 + 0.02 or lo[2] < -fp[1] / 2 - 0.02 or hi[2] > fp[1] / 2 + 0.02):
                issues.append(f'part {p["shape"]} at {p["at"]} leaves the {fp} footprint')
            if kind == 'wall' and lo[2] < WALL - 0.01: issues.append(f'part {p["shape"]} at {p["at"]} is inside the wall')
            if kind == 'ceiling' and hi[1] > CEILING + 0.01: issues.append(f'part {p["shape"]} at {p["at"]} is above the ceiling')
        if kind == 'surface':
            r = radius_of(key)
            if r > 0.3: issues.append(f'radius {r} > 0.3')
        if key in TOPS:
            x0, x1, z0, z1, h = TOPS[key]
            if x1 - x0 < 0.24 or z1 - z0 < 0.24: issues.append(f'top {TOPS[key]} is too small for a small thing')
            if x0 < -fp[0] / 2 - 0.01 or x1 > fp[0] / 2 + 0.01 or z0 < -fp[1] / 2 - 0.01 or z1 > fp[1] / 2 + 0.01:
                issues.append(f'top {TOPS[key]} leaves the footprint')
        for s in issues: print(f'  ! {k}: {s}')
        bad += len(issues)
    lines = [f'    {json.dumps(k)}: {json.dumps(e, separators=(",", ":"))}' for k, e in E.items()]
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write('{\n  "$comment": "Separate-piece furniture, tabletop things, lamps and garden things of the second catalogue (content/catalog2/*.json), '
                'drawn from primitive parts by tools/art/catalog2.py. Listed in the base manifest\'s packs.",\n  "version": 1,\n  "entries": {\n')
        f.write(',\n'.join(lines) + '\n  }\n}\n')
    json.load(open(path))
    print(f'{len(E)} models -> {path}; {bad} issue(s)')
    return bad


if __name__ == '__main__':
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'web', 'public', 'assets', 'catalog2', 'manifest.json')
    sys.exit(1 if main(os.path.normpath(sys.argv[1] if len(sys.argv) > 1 else out)) else 0)
