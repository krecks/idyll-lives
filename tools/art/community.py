# Model entries for the town's community things (`model.<id>`, web/public/content/community.json): street
# furniture, the playground, food carts and machines, shop fittings, plus the street dressing props
# (`model.street.*`) that render/babylon/street.ts draws as thin instances along the kerbs.
#
# All primitive parts drawn here (original, CC0), at most four finishes per model (one draw call each).
# Same conventions as catalog.py: 1 unit = 1 m, origin at the footprint centre on the ground, front +Z.
# Seated residents are spread across the width per slot ((slot - (n-1)/2) * width / n), face +Z and sit
# at the content's `seat` height and forward offset, so seats here are straight rows along X.
#
# Run: python3 -I -B community.py   (writes web/public/assets/community/manifest.json, a manifest listed in
# the base manifest's `packs`; one entry per line)
import json, math, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import objects_extra as OX                                          # noqa: E402
from objects_extra import box, cyl, cone, sph, rod, on              # noqa: E402
from furniture import placeholder, torus, WHITE, LGREY, OAK, HONEY, WALNUT, CREAM, SAGE, TEAL, RUST, MUSTARD  # noqa: E402

E = {}
FOOT = {}

IRON = '#2B302E'           # cast iron, dark green-black
STONE, GRANITE, CORK = '#C9C2B4', '#8E8C88', '#B88A5A'
RED, GREEN, BLUE, YELLOW = '#B8322B', '#3C6B3A', '#2E6FA8', '#F2C14E'
WATER, GLASS = '#5B93A8', '#C9DDE3'
BULB = '#FFF1CF'
BRONZE = '#5E7F6E'         # weathered bronze
BENCH_WOOD = '#9A6B45'
FOLIAGE = '#5E8F4E'


def add(key, parts, fp=(1, 1)):
    E['model.' + key] = placeholder(parts)
    FOOT['model.' + key] = fp


def ring(n, r, y, fn):
    """`fn(x, z, angle)` for n points on a circle of radius r."""
    return [fn(r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), 2 * math.pi * i / n) for i in range(n)]


# =============================================================================================
# Street and square
# =============================================================================================
def street_lamp():
    return [
        cyl([0.36, 0.16, 0.36], [0, 0.08, 0], IRON, 'metal', taper=0.75, segments=12),
        cyl([0.2, 0.5, 0.2], [0, 0.41, 0], IRON, 'metal', taper=0.55, segments=12),
        cyl([0.09, 2.85, 0.09], [0, 2.08, 0], IRON, 'metal', segments=10),
        cyl([0.18, 0.08, 0.18], [0, 3.5, 0], IRON, 'metal', segments=10),
        box([0.3, 0.42, 0.3], [0, 3.75, 0], BULB, 'glass', radius=0.01),
        sph([0.16, 0.2, 0.16], [0, 3.74, 0], BULB, 'gloss', segments=10),
        cone([0.66, 0.22, 0.66], [0, 4.07, 0], IRON, 'metal', taper=0.15, segments=4, rotate=[0, 45, 0]),   # 0.47 square
        sph([0.07, 0.07, 0.07], [0, 4.2, 0], IRON, 'metal', segments=8),
        rod([-0.29, 3.12, 0], [0.29, 3.12, 0], 0.032, IRON, 'metal'),                                       # ladder bar
        sph([0.06, 0.06, 0.06], [-0.3, 3.12, 0], IRON, 'metal', segments=8),
        sph([0.06, 0.06, 0.06], [0.3, 3.12, 0], IRON, 'metal', segments=8),
    ]


add('streetLamp', street_lamp())


def bench(width, legs, wood=BENCH_WOOD, frame=IRON, depth=0.5, z0=0.0):
    """A slatted bench along X (seat ~0.45 m, centred on z0), cast-iron ends and middle supports."""
    p = []
    for x in legs:
        p += [box([0.06, 0.42, depth], [x, 0.21, z0], frame, 'metal', radius=0.01),
              rod([x, 0.44, z0 - depth / 2 + 0.04], [x, 0.9, z0 - depth / 2 - 0.02], 0.055, frame, 'metal', shape='box')]
    for x in (legs[0], legs[-1]):
        p.append(box([0.06, 0.04, depth - 0.06], [x, 0.66, z0 + 0.02], frame, 'metal', radius=0.015))     # armrests
        p.append(box([0.05, 0.22, 0.05], [x, 0.55, z0 + depth / 2 - 0.06], frame, 'metal', radius=0.01))
    for z in (-0.16, -0.055, 0.05, 0.155):
        p.append(box([width, 0.035, 0.09], [0, 0.45, z0 + z * depth / 0.5], wood, 'wood', radius=0.01))
    for i, y in enumerate((0.6, 0.73, 0.86)):
        p.append(box([width, 0.085, 0.03], [0, y, z0 - depth / 2 + 0.06 - i * 0.045 - 0.02], wood, 'wood', radius=0.01, rotate=[-12, 0, 0]))
    return p


add('parkBench', bench(2.86, (-1.36, 0.0, 1.36)), (3, 1))


def drinking_fountain():
    return [
        box([0.56, 0.08, 0.56], [0, 0.04, 0], GRANITE, 'matte', radius=0.02),
        box([0.34, 0.82, 0.34], [0, 0.49, -0.04], STONE, 'matte', radius=0.03),
        cyl([0.42, 0.12, 0.42], [0, 0.94, 0.02], STONE, 'matte', taper=1.12, segments=16),
        cyl([0.36, 0.01, 0.36], [0, 1.0, 0.02], WATER, 'gloss', segments=16),
        cyl([0.05, 0.14, 0.05], [0, 1.06, -0.1], '#C8CCD0', 'chrome', segments=8),
        rod([0, 1.13, -0.1], [0, 1.13, -0.02], 0.035, '#C8CCD0', 'chrome'),
        box([0.18, 0.06, 0.04], [0.0, 0.5, 0.15], '#C8CCD0', 'chrome', radius=0.01),                     # foot pedal
    ]


add('drinkingFountain', drinking_fountain())


def litter_bin(colour=GREEN):
    p = [box([0.07, 0.95, 0.07], [0, 0.475, -0.28], IRON, 'metal', radius=0.01),
         cyl([0.42, 0.6, 0.42], [0, 0.48, 0.02], colour, 'metal', segments=14),
         cyl([0.46, 0.05, 0.46], [0, 0.8, 0.02], IRON, 'metal', segments=14),
         cyl([0.36, 0.01, 0.36], [0, 0.826, 0.02], '#1E2120', 'matte', segments=14),
         box([0.2, 0.05, 0.08], [0, 0.6, 0.235], '#E8E6E0', 'matte', radius=0.01)]                       # label
    for a in range(6):                                                                                    # slots
        t = math.radians(a * 60 + 30)
        p.append(box([0.04, 0.4, 0.012], [0.212 * math.sin(t), 0.48, 0.02 + 0.212 * math.cos(t)], '#24402A', 'matte', rotate=[0, a * 60 + 30, 0]))
    return p


add('litterBin', litter_bin())


def bus_shelter():
    p = []
    for x in (-1.42, 1.42):
        for z in (-0.4, 0.38):
            p.append(box([0.07, 2.36, 0.07], [x, 1.18, z], '#5C6168', 'metal', radius=0.01))
    p += [box([3.0, 0.09, 1.0], [0, 2.42, 0.0], '#5C6168', 'metal', radius=0.02, rotate=[4, 0, 0]),
          box([2.78, 1.9, 0.02], [0, 1.25, -0.42], GLASS, 'glass'),
          box([0.02, 1.9, 0.72], [1.42, 1.25, -0.03], GLASS, 'glass'),
          box([2.78, 0.06, 0.06], [0, 0.32, -0.42], '#5C6168', 'metal'),
          # bench
          box([2.5, 0.04, 0.34], [0, 0.45, -0.18], OAK, 'wood', radius=0.01),
          box([0.05, 0.43, 0.3], [-1.0, 0.215, -0.2], '#5C6168', 'metal'),
          box([0.05, 0.43, 0.3], [1.0, 0.215, -0.2], '#5C6168', 'metal'),
          # timetable and the stop's flag
          box([0.5, 0.7, 0.03], [-0.95, 1.45, -0.4], WHITE, 'gloss', radius=0.01),
          box([0.4, 0.06, 0.035], [-0.95, 1.73, -0.398], BLUE, 'gloss'),
          box([0.6, 0.36, 0.04], [1.2, 2.72, 0.44], BLUE, 'gloss', radius=0.02),
          box([0.44, 0.22, 0.045], [1.2, 2.72, 0.44], WHITE, 'gloss', radius=0.02),
          box([0.05, 0.3, 0.05], [1.2, 2.5, 0.44], '#5C6168', 'metal')]
    return p


add('busStop', bus_shelter(), (3, 1))


def post_box():
    return [
        cyl([0.48, 0.08, 0.48], [0, 0.04, 0], '#1E2120', 'matte', segments=18),
        cyl([0.42, 1.24, 0.42], [0, 0.7, 0], RED, 'gloss', segments=18),
        cyl([0.5, 0.08, 0.5], [0, 1.36, 0], RED, 'gloss', segments=18),
        sph([0.46, 0.22, 0.46], [0, 1.42, 0], RED, 'gloss', segments=14),
        box([0.22, 0.03, 0.02], [0, 1.12, 0.205], '#1A1A1A', 'matte'),                                    # slot
        box([0.15, 0.12, 0.01], [0, 0.86, 0.21], '#F2F0EA', 'matte'),                                      # collection times
        box([0.08, 0.05, 0.01], [0, 1.24, 0.21], '#D6A64A', 'matte'),                                      # crest
    ]


add('postBox', post_box())


def notice_board():
    p = [box([0.08, 1.95, 0.08], [x, 0.975, -0.05], WALNUT, 'wood', radius=0.01) for x in (-0.42, 0.42)]
    p += [box([0.84, 0.72, 0.05], [0, 1.36, -0.05], CORK, 'matte', radius=0.005),
          box([0.9, 0.05, 0.08], [0, 1.74, -0.04], WALNUT, 'wood', radius=0.01),
          box([0.9, 0.05, 0.08], [0, 0.98, -0.04], WALNUT, 'wood', radius=0.01),
          box([1.0, 0.03, 0.2], [0, 2.0, 0.03], '#5E4535', 'wood', radius=0.005, rotate=[28, 0, 0]),
          box([1.0, 0.03, 0.2], [0, 2.0, -0.13], '#5E4535', 'wood', radius=0.005, rotate=[-28, 0, 0])]
    notes = [(-0.27, 1.55, 0.18, 0.22, '#F4F1E6'), (-0.05, 1.5, 0.16, 0.2, '#F2D16B'), (0.2, 1.56, 0.2, 0.16, '#E9A6A6'),
             (-0.25, 1.2, 0.16, 0.22, '#9FC5E8'), (0.0, 1.22, 0.2, 0.18, '#F4F1E6'), (0.25, 1.24, 0.16, 0.2, '#B6D7A8')]
    for x, y, w, h, c in notes:
        p.append(box([w, h, 0.006], [x, y, -0.022], c, 'satin'))
        p.append(sph([0.025, 0.025, 0.012], [x, y + h / 2 - 0.02, -0.016], RED, 'satin', segments=6))
    return p


add('noticeBoard', notice_board())


def phone_box():
    p = [box([0.9, 0.12, 0.9], [0, 0.06, 0], RED, 'gloss', radius=0.01),
         box([0.86, 2.14, 0.86], [0, 1.19, 0], RED, 'gloss', radius=0.015),
         box([0.92, 0.22, 0.92], [0, 2.36, 0], RED, 'gloss', radius=0.02),
         box([0.84, 0.08, 0.84], [0, 2.51, 0], RED, 'gloss', radius=0.03),
         box([0.6, 0.08, 0.6], [0, 2.58, 0], RED, 'gloss', radius=0.03),
         sph([0.1, 0.08, 0.1], [0, 2.65, 0], RED, 'gloss', segments=8)]
    for k in range(4):                                                                                    # four faces
        yaw = 90 * k
        def at(x, y, z): return on([0, 0, 0], [0, yaw, 0], [x, y, z])
        p.append(box([0.6, 0.1, 0.012], at(0, 2.36, 0.462), '#F2F0EA', 'matte', rotate=[0, yaw, 0]))       # TELEPHONE
        p.append(box([0.6, 1.5, 0.01], at(0, 1.3, 0.432), GLASS, 'glass', rotate=[0, yaw, 0]))
        for x in (-0.1, 0.1):
            p.append(box([0.025, 1.5, 0.014], at(x, 1.3, 0.438), RED, 'gloss', rotate=[0, yaw, 0]))
        for y in (0.8, 1.05, 1.3, 1.55, 1.8):
            p.append(box([0.6, 0.025, 0.014], at(0, y, 0.438), RED, 'gloss', rotate=[0, yaw, 0]))
    p.append(box([0.16, 0.24, 0.1], [0.0, 1.35, -0.36], '#1E2120', 'matte', radius=0.01))                 # the phone
    return p


add('phoneBooth', phone_box())


def town_fountain():
    """A square stone basin (straight rims to sit on) with a round two-tier fountain in the middle."""
    p = [box([2.9, 0.06, 2.9], [0, 0.03, 0], GRANITE, 'matte', radius=0.01)]
    for s in (-1, 1):
        p.append(box([2.9, 0.44, 0.24], [0, 0.25, s * 1.33], STONE, 'matte', radius=0.02))
        p.append(box([0.24, 0.44, 2.42], [s * 1.33, 0.25, 0], STONE, 'matte', radius=0.02))
        p.append(box([2.96, 0.06, 0.3], [0, 0.48, s * 1.32], STONE, 'matte', radius=0.02))
        p.append(box([0.3, 0.06, 2.36], [s * 1.32, 0.48, 0], STONE, 'matte', radius=0.02))
    p += [box([2.42, 0.02, 2.42], [0, 0.36, 0], WATER, 'gloss'),
          cyl([0.7, 0.3, 0.7], [0, 0.5, 0], STONE, 'matte', taper=0.6, segments=16),
          cyl([0.26, 0.7, 0.26], [0, 0.95, 0], STONE, 'matte', segments=14),
          cyl([0.7, 0.2, 0.7], [0, 1.36, 0], STONE, 'matte', taper=1.75, segments=20),
          cyl([1.16, 0.02, 1.16], [0, 1.465, 0], WATER, 'gloss', segments=20),
          cyl([0.14, 0.45, 0.14], [0, 1.68, 0], STONE, 'matte', segments=12),
          cyl([0.3, 0.12, 0.3], [0, 1.94, 0], STONE, 'matte', taper=1.6, segments=14),
          sph([0.14, 0.16, 0.14], [0, 2.06, 0], STONE, 'matte', segments=10),
          # water: a jet on top, spilling from the bowl into the basin
          cyl([0.05, 0.4, 0.05], [0, 2.28, 0], '#DDEEF2', 'glass', taper=0.4, segments=8)]
    p += ring(8, 0.56, 0, lambda x, z, a: cyl([0.05, 1.0, 0.05], [x, 0.93, z], '#CFE6EE', 'glass', taper=0.6, segments=6))
    p += ring(8, 0.62, 0, lambda x, z, a: sph([0.16, 0.06, 0.16], [x, 0.38, z], '#E6F2F5', 'glass', segments=8))
    return p


add('fountainTown', town_fountain(), (3, 3))


def statue():
    b = BRONZE
    p = [box([1.8, 0.2, 1.8], [0, 0.1, 0], GRANITE, 'matte', radius=0.02),
         box([1.2, 1.1, 1.2], [0, 0.75, 0], GRANITE, 'matte', radius=0.02),
         box([1.36, 0.12, 1.36], [0, 1.36, 0], GRANITE, 'matte', radius=0.02),
         box([0.5, 0.28, 0.02], [0, 0.85, 0.605], '#8A6A3A', 'metal', radius=0.01)]                         # plaque
    y = 1.42
    p += [box([0.14, 0.08, 0.26], [-0.1, y + 0.04, 0.04], b, 'metal', radius=0.03),
          box([0.14, 0.08, 0.26], [0.12, y + 0.04, 0.0], b, 'metal', radius=0.03),
          cone([0.62, 1.15, 0.5], [0, y + 0.62, 0], b, 'metal', taper=0.55, segments=14),                    # long coat
          cyl([0.36, 0.42, 0.3], [0, y + 1.38, 0], b, 'metal', taper=0.9, segments=12),
          sph([0.5, 0.18, 0.3], [0, y + 1.56, 0], b, 'metal', segments=12),                                   # shoulders
          cyl([0.1, 0.12, 0.1], [0, y + 1.68, 0], b, 'metal', segments=8),
          sph([0.23, 0.28, 0.25], [0, y + 1.84, 0.01], b, 'metal', segments=12),
          sph([0.15, 0.12, 0.14], [0, y + 1.96, -0.1], b, 'metal', segments=8),                              # bun
          rod([0.22, y + 1.55, 0], [0.62, y + 1.78, 0.3], 0.09, b, 'metal'),                                  # pointing
          sph([0.09, 0.09, 0.1], [0.66, y + 1.8, 0.32], b, 'metal', segments=8),
          rod([-0.22, y + 1.55, 0], [-0.28, y + 1.08, 0.06], 0.09, b, 'metal'),
          box([0.06, 0.28, 0.2], [-0.32, y + 1.1, 0.08], b, 'metal', radius=0.01)]                            # a book
    return p


add('townStatue', statue(), (2, 2))

# =============================================================================================
# Playground
# =============================================================================================
def slide():
    frame, chute = BLUE, YELLOW
    p = [box([0.62, 0.06, 0.42], [0, 1.4, -0.5], HONEY, 'wood', radius=0.01)]
    for x in (-0.28, 0.28):
        for z in (-0.32, -0.68):
            p.append(cyl([0.06, 1.95, 0.06], [x, 0.975, z], frame, 'metal', segments=10))
        p.append(rod([x, 1.85, -0.32], [x, 1.85, -0.68], 0.045, frame, 'metal'))                        # rails
        p.append(rod([x, 0.0, -0.94], [x, 1.43, -0.7], 0.05, frame, 'metal'))                             # ladder
    for i in range(1, 6):
        t = i / 6
        p.append(box([0.5, 0.035, 0.07], [0, 1.43 * t, -0.94 + 0.24 * t], HONEY, 'wood', radius=0.008))
    a, b = [0, 1.42, -0.3], [0, 0.32, 0.72]
    p.append(rod(a, b, 0.48, chute, 'gloss', shape='box', t2=0.04))
    for x in (-0.26, 0.26):
        p.append(rod([x, a[1] + 0.07, a[2]], [x, b[1] + 0.07, b[2]], 0.04, chute, 'gloss', shape='box', t2=0.12))
    p += [box([0.48, 0.04, 0.24], [0, 0.3, 0.84], chute, 'gloss', radius=0.01),
          box([0.06, 0.28, 0.06], [-0.18, 0.14, 0.8], frame, 'metal'),
          box([0.06, 0.28, 0.06], [0.18, 0.14, 0.8], frame, 'metal'),
          cone([1.06, 0.4, 1.06], [0, 2.12, -0.5], RED, 'gloss', taper=0.05, segments=4, rotate=[0, 45, 0])]   # roof, 0.75 square
    return p


add('slide', slide(), (1, 2))


def climbing_frame():
    p = []
    for x in (-0.85, 0.85):
        for z in (-0.85, 0.85):
            p.append(box([0.1, 2.3, 0.1], [x, 1.15, z], HONEY, 'wood', radius=0.012))
    p.append(box([1.8, 0.06, 1.8], [0, 1.1, 0], HONEY, 'wood', radius=0.01))
    for s in (-1, 1):                                                                                         # railings
        p.append(box([1.7, 0.4, 0.04], [0, 1.4, -0.85], OAK, 'wood', radius=0.01))
        p.append(box([0.04, 0.4, 1.7], [s * 0.85, 1.4, 0], OAK, 'wood', radius=0.01))
    for s in (-1, 1):                                                                                         # pitched roof
        p.append(box([1.0, 0.04, 1.9], [s * 0.43, 2.47, 0], '#B5463A', 'satin', radius=0.01, rotate=[0, 0, -s * 30]))
    rope = '#D8C9A6'
    for i in range(5):                                                                                       # net down the front
        x = -0.7 + i * 0.35
        p.append(rod([x, 1.08, 0.86], [x, 0.05, 0.86], 0.03, rope, 'fabric'))
    for j in range(1, 5):
        y = 1.08 * j / 5
        p.append(rod([-0.72, y, 0.86], [0.72, y, 0.86], 0.03, rope, 'fabric'))
    p.append(cyl([0.05, 2.9, 0.05], [0.85, 2.0, -0.85], '#C3C7CB', 'metal', segments=8))                      # flagpole
    p.append(box([0.02, 0.22, 0.32], [0.85, 3.3, -0.69], RED, 'satin'))
    return p


add('climbingFrame', climbing_frame(), (2, 2))


def seesaw():
    p = [box([0.3, 0.4, 0.42], [0, 0.2, 0], BLUE, 'metal', radius=0.02),
         cyl([0.12, 0.46, 0.12], [0, 0.44, 0], '#5C6168', 'metal', rotate=[90, 0, 0], segments=10)]
    # a 2.5 m plank, level at rest, its seats on the content's two slots (x = +-0.75 m across 3 tiles)
    pivot, r = [0, 0.46, 0], [0, 0, 0]
    p.append(box([2.5, 0.07, 0.24], pivot, YELLOW, 'gloss', radius=0.02, rotate=r))
    for s in (-1, 1):
        p.append(box([0.32, 0.06, 0.3], on(pivot, r, [s * 0.75, 0.06, 0]), RED, 'gloss', radius=0.02, rotate=r))
        a = on(pivot, r, [s * 0.5, 0.04, 0])
        b = on(pivot, r, [s * 0.5, 0.34, 0])
        p.append(rod(a, b, 0.04, '#5C6168', 'metal'))
        p.append(rod([b[0], b[1], -0.12], [b[0], b[1], 0.12], 0.04, '#5C6168', 'metal'))
        p.append(cyl([0.26, 0.12, 0.26], [s * 1.1, 0.06, 0], '#26282A', 'matte', segments=12))                # bumper tyres under the ends
    return p


add('seesaw', seesaw(), (3, 1))


def spring_rider():
    horse, mane = '#D9483B', '#F2F0EA'
    p = [cyl([0.4, 0.04, 0.4], [0, 0.02, 0], '#5C6168', 'metal', segments=14)]
    for i in range(6):
        p.append(torus([0.2, 0.03, 0.2], [0, 0.07 + i * 0.06, 0], '#C3C7CB', 'metal', segments=12))
    p += [sph([0.26, 0.3, 0.66], [0, 0.6, -0.02], horse, 'gloss', segments=14),
          rod([0, 0.68, 0.2], [0, 0.92, 0.3], 0.14, horse, 'gloss'),
          sph([0.18, 0.2, 0.3], [0, 0.96, 0.36], horse, 'gloss', segments=12),
          cone([0.05, 0.1, 0.05], [-0.05, 1.08, 0.3], horse, 'gloss', segments=6),
          cone([0.05, 0.1, 0.05], [0.05, 1.08, 0.3], horse, 'gloss', segments=6),
          box([0.05, 0.12, 0.28], [0, 0.9, 0.18], mane, 'matte', radius=0.02, rotate=[-30, 0, 0]),
          box([0.06, 0.24, 0.06], [0, 0.6, -0.36], mane, 'matte', radius=0.02, rotate=[30, 0, 0]),             # tail
          box([0.3, 0.05, 0.26], [0, 0.76, -0.04], BLUE, 'gloss', radius=0.02),                                # saddle
          rod([-0.16, 0.92, 0.24], [0.16, 0.92, 0.24], 0.03, YELLOW, 'gloss'),                                  # handles
          rod([-0.2, 0.5, 0.06], [0.2, 0.5, 0.06], 0.03, YELLOW, 'gloss'),                                      # foot pegs
          sph([0.04, 0.04, 0.03], [-0.07, 1.0, 0.48], '#1A1A1A', 'matte', segments=6),
          sph([0.04, 0.04, 0.03], [0.07, 1.0, 0.48], '#1A1A1A', 'matte', segments=6)]
    return p


add('springRider', spring_rider())


def merry_go_round():
    p = [cyl([0.4, 0.14, 0.4], [0, 0.07, 0], '#5C6168', 'metal', segments=14),
         cyl([1.9, 0.1, 1.9], [0, 0.2, 0], RED, 'gloss', segments=28),
         torus([1.9, 0.05, 1.9], [0, 0.22, 0], '#C3C7CB', 'metal', segments=28),
         box([0.2, 0.012, 1.7], [0, 0.256, 0], YELLOW, 'gloss', rotate=[0, 45, 0]),
         box([0.2, 0.012, 1.7], [0, 0.256, 0], YELLOW, 'gloss', rotate=[0, -45, 0]),
         cyl([0.1, 0.72, 0.1], [0, 0.6, 0], '#C3C7CB', 'metal', segments=10),
         sph([0.16, 0.12, 0.16], [0, 0.98, 0], YELLOW, 'gloss', segments=10)]
    for k in range(4):                                                                                       # bent hand rails
        t = math.radians(45 + 90 * k)
        c, s = math.sin(t), math.cos(t)
        p.append(rod([0.04 * c, 0.92, 0.04 * s], [0.7 * c, 0.78, 0.7 * s], 0.04, '#C3C7CB', 'metal'))
        p.append(rod([0.7 * c, 0.78, 0.7 * s], [0.78 * c, 0.25, 0.78 * s], 0.04, '#C3C7CB', 'metal'))
    return p


add('merryGoRound', merry_go_round(), (2, 2))


def basketball_hoop():
    return [
        box([0.6, 0.24, 0.7], [0, 0.12, -0.12], '#3A3D41', 'metal', radius=0.03),                             # ballast
        cyl([0.1, 2.9, 0.1], [0, 1.6, -0.3], '#5C6168', 'metal', segments=10),
        rod([0, 2.95, -0.3], [0, 3.05, 0.02], 0.07, '#5C6168', 'metal', shape='box'),
        box([0.95, 0.62, 0.03], [0, 3.18, 0.04], WHITE, 'gloss', radius=0.01),
        box([0.34, 0.26, 0.035], [0, 3.06, 0.045], RED, 'gloss'),
        box([0.3, 0.2, 0.04], [0, 3.06, 0.046], WHITE, 'gloss'),
        torus([0.44, 0.022, 0.44], [0, 2.94, 0.27], '#E0602A', 'gloss', segments=16),
        cyl([0.3, 0.34, 0.3], [0, 2.76, 0.27], '#F2F2F0', 'fabric', taper=1.4, segments=12),
        sph([0.24, 0.24, 0.24], [0.3, 0.12, 0.32], '#D86A2A', 'matte', segments=12),
    ]


add('basketballHoop', basketball_hoop())


def ping_pong():
    top, lines = '#2F6B4F', '#F2F2F0'
    p = [box([1.9, 0.05, 0.94], [0, 0.755, 0], top, 'satin', radius=0.01)]
    for x in (-0.6, 0.6):
        p.append(box([0.12, 0.73, 0.66], [x, 0.365, 0], '#A9ADB0', 'matte', radius=0.01))
    p += [box([1.9, 0.004, 0.025], [0, 0.782, 0.455], lines, 'satin'),
          box([1.9, 0.004, 0.025], [0, 0.782, -0.455], lines, 'satin'),
          box([0.025, 0.004, 0.94], [0.935, 0.782, 0], lines, 'satin'),
          box([0.025, 0.004, 0.94], [-0.935, 0.782, 0], lines, 'satin'),
          box([1.9, 0.004, 0.015], [0, 0.782, 0], lines, 'satin'),
          box([0.03, 0.15, 0.98], [0, 0.855, 0], '#5C6168', 'matte'),                                        # steel net
          cyl([0.16, 0.012, 0.16], [-0.55, 0.787, 0.2], RED, 'gloss', segments=12),
          box([0.03, 0.012, 0.1], [-0.55, 0.787, 0.32], HONEY, 'gloss'),
          sph([0.04, 0.04, 0.04], [0.35, 0.8, -0.1], WHITE, 'gloss', segments=8)]
    return p


add('pingPongTable', ping_pong(), (2, 1))


def park_chess():
    """A stone table in the front half and a stone bench behind it (players sit side by side, facing +Z)."""
    p = [box([0.86, 0.05, 0.3], [0, 0.44, -0.27], STONE, 'matte', radius=0.02),
         box([0.12, 0.42, 0.22], [-0.3, 0.21, -0.27], GRANITE, 'matte', radius=0.01),
         box([0.12, 0.42, 0.22], [0.3, 0.21, -0.27], GRANITE, 'matte', radius=0.01),
         cyl([0.2, 0.7, 0.2], [0, 0.35, 0.18], GRANITE, 'matte', segments=12),
         box([0.6, 0.06, 0.56], [0, 0.73, 0.18], STONE, 'matte', radius=0.02),
         box([0.4, 0.004, 0.4], [0, 0.762, 0.18], '#E8DFC8', 'satin')]
    for i in range(8):
        for j in range(8):
            if (i + j) % 2:
                p.append(box([0.05, 0.004, 0.05], [-0.175 + i * 0.05, 0.765, 0.005 + j * 0.05], '#3A3330', 'satin'))
    for x, z, c in ((-0.12, 0.08, '#F2EDE2'), (0.07, 0.13, '#F2EDE2'), (0.0, 0.28, '#2A2624'), (0.12, 0.23, '#2A2624')):
        p.append(cyl([0.03, 0.05, 0.03], [x, 0.79, z], c, 'gloss', segments=8))
        p.append(sph([0.03, 0.03, 0.03], [x, 0.825, z], c, 'gloss', segments=6))
    return p


add('parkChessTable', park_chess())

# =============================================================================================
# Carts and machines
# =============================================================================================
def wheel(x, z, r=0.17):
    return cyl([r * 2, 0.07, r * 2], [x, r, z], '#26282A', 'matte', rotate=[90, 0, 0], segments=12)


def ice_cream_cart():
    body, stripe = '#F4EFE6', '#E9A6B4'
    p = [box([1.3, 0.62, 0.62], [-0.15, 0.62, 0.0], body, 'gloss', radius=0.03),
         box([1.31, 0.1, 0.63], [-0.15, 0.5, 0.0], stripe, 'gloss', radius=0.01),
         box([1.31, 0.1, 0.63], [-0.15, 0.74, 0.0], stripe, 'gloss', radius=0.01),
         box([1.22, 0.04, 0.56], [-0.15, 0.95, 0.0], '#C3C7CB', 'metal', radius=0.01),
         wheel(-0.62, 0.27), wheel(-0.62, -0.27), wheel(0.38, 0.27), wheel(0.38, -0.27),
         rod([0.5, 0.86, -0.22], [0.82, 1.0, -0.22], 0.03, '#C3C7CB', 'metal'),                             # push bar
         rod([0.5, 0.86, 0.22], [0.82, 1.0, 0.22], 0.03, '#C3C7CB', 'metal'),
         rod([0.82, 1.0, -0.22], [0.82, 1.0, 0.22], 0.035, '#C3C7CB', 'metal'),
         cyl([0.04, 1.4, 0.04], [-0.15, 1.65, 0.0], '#C3C7CB', 'metal', segments=8)]
    p += [cone([1.5, 0.34, 0.94], [-0.15, 2.24, 0.0], WHITE, 'fabric', taper=0.0, segments=16),                   # parasol
          cone([0.96, 0.22, 0.6], [-0.15, 2.3, 0.0], stripe, 'fabric', taper=0.0, segments=16),
          torus([1.5, 0.05, 0.94], [-0.15, 2.07, 0.0], stripe, 'fabric', segments=24),
          cone([0.16, 0.26, 0.16], [0.28, 1.1, 0.0], '#D9A35F', 'matte', taper=0.0, rotate=[180, 0, 0], segments=10),   # sign: a cone
          sph([0.2, 0.18, 0.2], [0.28, 1.28, 0.0], '#F3B6C4', 'gloss', segments=10)]
    return p


add('iceCreamCart', ice_cream_cart(), (2, 1))


def hot_dog_stand():
    body, awning = RED, '#F2F0EA'
    p = [box([1.5, 0.66, 0.66], [-0.1, 0.62, -0.05], body, 'gloss', radius=0.03),
         box([1.56, 0.05, 0.72], [-0.1, 0.975, -0.05], '#C3C7CB', 'metal', radius=0.01),
         box([0.6, 0.12, 0.4], [-0.35, 1.06, -0.1], '#C3C7CB', 'metal', radius=0.02),                      # steamer
         wheel(-0.68, 0.24, 0.2), wheel(-0.68, -0.34, 0.2), wheel(0.5, 0.24, 0.2), wheel(0.5, -0.34, 0.2),
         cyl([0.08, 0.2, 0.08], [0.3, 1.1, 0.1], '#D6A64A', 'gloss', segments=8),                           # mustard
         cyl([0.08, 0.2, 0.08], [0.42, 1.1, 0.1], RED, 'gloss', segments=8)]                                  # ketchup
    for x in (-0.82, 0.62):
        for z in (-0.4, 0.38):
            p.append(cyl([0.04, 1.3, 0.04], [x, 1.62, z], '#C3C7CB', 'metal', segments=8))
    for i in range(6):                                                                                     # striped awning
        x = -0.85 + 0.25 * i + 0.125
        p.append(box([0.25, 0.04, 0.98], [x, 2.3, 0.0], RED if i % 2 else awning, 'fabric', radius=0.005, rotate=[-8, 0, 0]))
    p += [box([0.62, 0.22, 0.04], [-0.1, 0.72, 0.29], '#F2F0EA', 'gloss', radius=0.02),                     # sign panel
          box([0.44, 0.08, 0.045], [-0.1, 0.72, 0.29], '#B36A3A', 'gloss', radius=0.03),                     # a hot dog on it
          box([0.38, 0.04, 0.05], [-0.1, 0.74, 0.29], '#9E3B2A', 'gloss', radius=0.02)]
    return p


add('hotDogStand', hot_dog_stand(), (2, 1))


def vending(body, items, panel=None):
    """A machine against the back of its tile, glass front at z ~ 0.29."""
    p = [box([0.9, 1.86, 0.72], [0, 0.95, -0.07], body, 'gloss', radius=0.02),
         box([0.88, 0.04, 0.7], [0, 0.02, -0.07], '#26282A', 'satin')]
    if panel:
        p.append(box([0.6, 1.36, 0.01], [-0.1, 1.1, 0.293], panel, 'gloss'))
    else:
        p.append(box([0.6, 1.36, 0.01], [-0.1, 1.1, 0.29], '#2A2F36', 'satin'))
        for row in range(5):
            y = 0.55 + row * 0.27
            p.append(box([0.58, 0.015, 0.01], [-0.1, y - 0.08, 0.296], '#C3C7CB', 'satin'))
            for col in range(5):
                c = items[(row * 5 + col * 3) % len(items)]
                p.append(box([0.08, 0.13, 0.008], [-0.34 + col * 0.12, y, 0.297], c, 'gloss', radius=0.005))
    p += [box([0.18, 0.5, 0.02], [0.33, 1.2, 0.3], '#2A2F36', 'satin', radius=0.01),
          box([0.12, 0.08, 0.01], [0.33, 1.36, 0.311], '#7FD1A8', 'gloss'),
          box([0.1, 0.18, 0.01], [0.33, 1.13, 0.311], '#C3C7CB', 'satin'),
          box([0.5, 0.16, 0.02], [-0.1, 0.26, 0.3], '#1E2226', 'satin', radius=0.01)]                        # pick-up flap
    return p


add('vendingSnack', vending(RED, ['#F2C14E', '#4E7A7F', '#D9483B', '#8C5A3C', '#9DAF8C', '#E9A6B4', '#3E5470']))


def drink_machine():
    p = vending(BLUE, [], panel='#EAF4F8')
    for i, c in enumerate(('#D9483B', '#F2C14E', '#5EC28B')):                                             # cans on the lit panel
        p.append(cyl([0.12, 0.26, 0.12], [-0.28 + i * 0.18, 1.2, 0.3], c, 'gloss', segments=12))
    p += [box([0.6, 0.12, 0.012], [-0.1, 1.62, 0.3], '#2E5C8A', 'gloss'),
          box([0.6, 0.07, 0.012], [-0.1, 0.7, 0.3], '#2E5C8A', 'gloss')]
    return p


add('vendingDrink', drink_machine())


def market_stall():
    awning = '#4E8A5A'
    p = [box([1.84, 0.05, 0.8], [0, 0.8, 0.02], HONEY, 'wood', radius=0.01)]
    for x in (-0.88, 0.88):
        for z in (-0.4, 0.4):
            p.append(box([0.06, 2.1 if z < 0 else 1.9, 0.06], [x, 1.05 if z < 0 else 0.95, z], WALNUT, 'wood', radius=0.01))
    for i in range(7):                                                                                     # striped awning
        x = -0.92 + 0.263 * i + 0.13
        p.append(box([0.263, 0.03, 1.0], [x, 2.03, 0.02], awning if i % 2 == 0 else WHITE, 'fabric', radius=0.005, rotate=[-12, 0, 0]))
    # crates of fruit and vegetables, tilted towards the front
    crates = [(-0.6, '#D9483B'), (-0.2, '#E8902E'), (0.2, '#8DB84A'), (0.6, '#F2C14E')]
    for x, c in crates:
        p.append(box([0.36, 0.14, 0.5], [x, 0.9, 0.05], OAK, 'wood', radius=0.01, rotate=[10, 0, 0]))
        for i in range(3):
            for j in range(2):
                p.append(sph([0.09, 0.09, 0.09], [x - 0.1 + i * 0.1, 0.99 - j * 0.03, -0.08 + j * 0.16 + i * 0.01], c, 'gloss', segments=8))
    p += [box([0.04, 0.4, 0.3], [0.9, 0.6, 0.35], '#2A2C30', 'matte', rotate=[0, 0, 10]),                     # chalk board
          box([0.24, 0.12, 0.42], [-0.6, 0.06, -0.15], OAK, 'wood', radius=0.01),                             # spare crate
          box([0.24, 0.12, 0.42], [-0.32, 0.06, -0.15], OAK, 'wood', radius=0.01)]
    return p


add('marketStall', market_stall(), (2, 1))

# =============================================================================================
# Shop and café
# =============================================================================================
def forward(parts, dz):
    """`parts` moved `dz` towards the front: a counter whose worktop reached past the wall face (-0.43)."""
    return [dict(q, at=[q['at'][0], q['at'][1], q['at'][2] + dz]) for q in parts]


def shop_counter():
    p = [box([1.9, 0.92, 0.7], [0, 0.46, -0.08], OAK, 'wood', radius=0.015),
         box([1.96, 0.05, 0.78], [0, 0.945, -0.08], WALNUT, 'wood', radius=0.01),
         box([1.8, 0.08, 0.02], [0, 0.05, 0.28], WALNUT, 'wood')]
    for x in (-0.6, 0.0, 0.6):
        p.append(box([0.52, 0.66, 0.02], [x, 0.5, 0.275], HONEY, 'wood', radius=0.01))                     # panels
    p += [box([0.36, 0.16, 0.3], [0.5, 1.05, -0.2], '#3A3D41', 'satin', radius=0.02),                        # till
          box([0.26, 0.16, 0.03], [0.5, 1.2, -0.3], '#2A2F36', 'satin', rotate=[-20, 0, 0]),
          box([0.2, 0.1, 0.012], [0.5, 1.21, -0.28], '#7FD1A8', 'gloss', rotate=[-20, 0, 0]),
          cyl([0.18, 0.26, 0.18], [-0.2, 1.1, 0.05], '#DDEBF0', 'glass', segments=12),                       # sweet jar
          cyl([0.14, 0.04, 0.14], [-0.2, 1.25, 0.05], RED, 'gloss', segments=12)]
    for i, c in enumerate(('#D9483B', '#F2C14E', '#5EC28B', '#E9A6B4', '#4E7A7F')):
        p.append(sph([0.05, 0.05, 0.05], [-0.24 + (i % 3) * 0.04, 1.0 + (i // 3) * 0.05 + 0.02, 0.03 + (i % 2) * 0.04], c, 'gloss', segments=6))
    for i, c in enumerate(('#3E5470', '#D9483B', '#D6A64A', '#9DAF8C')):                                     # magazines
        p.append(box([0.18, 0.24, 0.015], [-0.72 + i * 0.12, 1.08, -0.25], c, 'gloss', rotate=[-15, 0, 0]))
    return p


add('shopCounter', forward(shop_counter(), 0.04), (2, 1))


def bakery_case():
    p = [box([1.9, 0.74, 0.82], [0, 0.37, -0.04], CREAM, 'satin', radius=0.015),
         box([1.94, 0.04, 0.86], [0, 0.76, -0.04], WALNUT, 'satin', radius=0.01),
         box([1.86, 0.02, 0.72], [0, 1.25, -0.04], '#DDEBF0', 'glass'),
         box([1.84, 0.3, 0.012], [0, 1.0, 0.33], '#DDEBF0', 'glass', rotate=[-25, 0, 0]),
         box([0.02, 0.48, 0.72], [-0.93, 1.02, -0.04], WALNUT, 'satin'),
         box([0.02, 0.48, 0.72], [0.93, 1.02, -0.04], WALNUT, 'satin'),
         box([1.9, 0.03, 0.78], [0, 1.275, -0.04], WALNUT, 'satin', radius=0.01),
         box([1.8, 0.1, 0.02], [0, 0.6, 0.375], '#C98F6B', 'satin')]
    goods = []
    for i in range(6):
        x = -0.75 + i * 0.3
        goods.append(box([0.24, 0.015, 0.2], [x, 0.785, 0.1], '#E8E2D6', 'gloss'))                            # trays
        goods.append(box([0.24, 0.015, 0.2], [x, 0.785, -0.2], '#E8E2D6', 'gloss'))
        k = i % 3
        if k == 0:
            for dx in (-0.06, 0.06):
                goods.append(sph([0.1, 0.07, 0.1], [x + dx, 0.82, 0.1], '#C98F4E', 'satin', segments=10))   # buns
        elif k == 1:
            goods.append(cyl([0.16, 0.04, 0.16], [x, 0.81, 0.1], '#D9A35F', 'satin', segments=12))           # tart
            goods.append(cyl([0.13, 0.012, 0.13], [x, 0.833, 0.1], '#C0392B', 'gloss', segments=12))
        else:
            goods.append(box([0.2, 0.07, 0.1], [x, 0.83, 0.1], '#B97A40', 'satin', radius=0.03))             # loaf
        goods.append(box([0.2, 0.08, 0.12], [x, 0.83, -0.2], '#E3B97A' if k else '#F2E1C1', 'satin', radius=0.035))
    return p + goods


add('bakeryCase', forward(bakery_case(), 0.04), (2, 1))


def grocery_shelf():
    frame = '#D3D4D2'
    p = [box([1.92, 1.72, 0.04], [0, 0.86, -0.4], frame, 'metal'),
         box([0.04, 1.72, 0.46], [-0.94, 0.86, -0.2], frame, 'metal'),
         box([0.04, 1.72, 0.46], [0.94, 0.86, -0.2], frame, 'metal'),
         box([1.88, 0.1, 0.44], [0, 0.05, -0.2], '#5C6168', 'metal')]
    colours = ['#D9483B', '#F2C14E', '#4E7A7F', '#9DAF8C', '#E8902E', '#3E5470', '#F2F0EA', '#8C5A3C', '#E9A6B4']
    k = 1
    for row, y in enumerate((0.12, 0.52, 0.92, 1.32)):
        p.append(box([1.86, 0.025, 0.44], [0, y, -0.19], frame, 'metal'))
        x = -0.88
        while x < 0.84:
            k = (k * 7 + 5) % 23
            c = colours[k % len(colours)]
            if row == 0:                                                                                       # sandwiches
                w = 0.16
                p.append(box([w, 0.07, 0.14], [x + w / 2, y + 0.05, 0.0], '#F2E1C1', 'matte', rotate=[0, 0, 0]))
            elif k % 3 == 0:                                                                                   # tins
                w = 0.1
                for z in (-0.1, 0.04):
                    p.append(cyl([0.09, 0.12, 0.09], [x + w / 2, y + 0.073, z], c, 'gloss', segments=10))
            else:                                                                                              # packets
                w = min(0.13 + (k % 3) * 0.03, 0.93 - x)                                                      # not through the side
                h = 0.18 + (k % 4) * 0.04
                p.append(box([w - 0.02, h, 0.26], [x + w / 2, y + 0.0125 + h / 2, -0.05], c, 'gloss', radius=0.008))
            x += w + 0.015
    return p


add('groceryShelf', grocery_shelf(), (2, 1))


def cafe_table():
    p = [cyl([0.62, 0.03, 0.62], [0, 0.73, 0], '#EDEAE4', 'gloss', segments=24),
         cyl([0.05, 0.68, 0.05], [0, 0.37, 0], IRON, 'metal', segments=8),
         cyl([0.12, 0.06, 0.12], [0, 0.7, 0], IRON, 'metal', segments=10)]
    for a in (0, 120, 240):
        t = math.radians(a)
        p.append(rod([0, 0.08, 0], [0.26 * math.sin(t), 0.02, 0.26 * math.cos(t)], 0.04, IRON, 'metal'))
    return p


add('cafeTable', cafe_table())


def cafe_chair(wood='#5A3B26', cane='#D9C7A7'):
    """A bentwood chair, the sitter facing +Z."""
    p = [cyl([0.42, 0.04, 0.42], [0, 0.44, 0.02], wood, 'wood', segments=20),
         cyl([0.36, 0.012, 0.36], [0, 0.465, 0.02], cane, 'fabric', segments=20),
         torus([0.4, 0.025, 0.4], [0, 0.23, 0.02], wood, 'wood', segments=20)]
    for x, z in ((-0.15, 0.16), (0.15, 0.16)):
        p.append(rod([x, 0.42, z], [x * 1.15, 0.0, z * 1.15], 0.032, wood, 'wood'))
    for x in (-0.15, 0.15):
        p.append(rod([x, 0.42, -0.13], [x * 1.15, 0.0, -0.17], 0.032, wood, 'wood'))
        p.append(rod([x, 0.44, -0.14], [x * 0.9, 0.86, -0.2], 0.03, wood, 'wood'))
    p.append(torus([0.34, 0.03, 0.34], [0, 0.78, -0.2], wood, 'wood', segments=20, rotate=[90, 0, 0]))
    return p


add('cafeChair', cafe_chair())

# =============================================================================================
# Street dressing (street.ts): drawn as thin instances along the kerbs, never on a lot
# =============================================================================================
# A slim bench for the road side of the sidewalk (back to the road, facing the houses).
add('street.bench', bench(1.7, (-0.8, 0.8), depth=0.44), (2, 1))


def planter(tree=True):
    p = [box([0.7, 0.42, 0.7], [0, 0.21, 0], '#9A958C', 'matte', radius=0.03),
         box([0.6, 0.02, 0.6], [0, 0.415, 0], '#4A3526', 'matte')]
    if tree:
        p += [cyl([0.07, 1.3, 0.07], [0, 1.05, 0], '#6B4F3A', 'bark', segments=8),
              sph([0.72, 0.62, 0.72], [0, 1.95, 0], FOLIAGE, 'foliage', segments=12),
              sph([0.5, 0.45, 0.5], [0.16, 2.22, 0.08], '#6E9C58', 'foliage', segments=10),
              sph([0.46, 0.4, 0.46], [-0.16, 2.1, -0.12], '#527F45', 'foliage', segments=10)]
    else:
        for x, z, c in ((-0.15, -0.12, '#D9483B'), (0.14, -0.1, '#F2C14E'), (0.0, 0.14, '#E9A6B4'), (-0.16, 0.16, '#9FC5E8'), (0.17, 0.15, '#F2F0EA')):
            p.append(sph([0.18, 0.14, 0.18], [x, 0.5, z], c, 'foliage', segments=8))
        p.append(sph([0.5, 0.18, 0.5], [0, 0.46, 0], FOLIAGE, 'foliage', segments=10))
    return p


add('street.planter', planter())
add('street.flowerTub', planter(False))


def street_sign():
    """A street-name sign on a pole: two blades crossing at the top."""
    return [cyl([0.07, 2.6, 0.07], [0, 1.3, 0], '#5C6168', 'metal', segments=8),
            box([0.9, 0.18, 0.03], [0.0, 2.45, 0.0], '#2F5D3A', 'gloss', radius=0.01),
            box([0.84, 0.12, 0.032], [0.0, 2.45, 0.0], WHITE, 'gloss', radius=0.01),
            box([0.03, 0.18, 0.9], [0.0, 2.25, 0.0], '#2F5D3A', 'gloss', radius=0.01),
            box([0.032, 0.12, 0.84], [0.0, 2.25, 0.0], WHITE, 'gloss', radius=0.01),
            sph([0.09, 0.09, 0.09], [0, 2.62, 0], '#5C6168', 'metal', segments=8)]


add('street.sign', street_sign())


def bus_sign():
    """A bus-stop flag on a pole with the timetable."""
    return [cyl([0.07, 2.5, 0.07], [0, 1.25, 0], '#5C6168', 'metal', segments=8),
            cyl([0.46, 0.04, 0.46], [0, 2.4, 0], BLUE, 'gloss', rotate=[90, 0, 0], segments=18),
            cyl([0.34, 0.045, 0.34], [0, 2.4, 0], WHITE, 'gloss', rotate=[90, 0, 0], segments=18),
            box([0.3, 0.42, 0.04], [0, 1.5, 0.05], WHITE, 'gloss', radius=0.01),
            box([0.26, 0.06, 0.045], [0, 1.66, 0.05], BLUE, 'gloss')]


add('street.busSign', bus_sign())


def pot(colour='#B5653F', plant=FOLIAGE):
    return [cyl([0.36, 0.34, 0.36], [0, 0.17, 0], colour, 'satin', taper=1.2, segments=14),
            cyl([0.4, 0.04, 0.4], [0, 0.35, 0], colour, 'satin', segments=14),
            sph([0.42, 0.42, 0.42], [0, 0.55, 0], plant, 'foliage', segments=10),
            sph([0.26, 0.24, 0.26], [0.08, 0.76, 0.04], '#6E9C58', 'foliage', segments=8)]


add('street.pot', pot())


def path_light():
    return [cyl([0.12, 0.42, 0.12], [0, 0.21, 0], '#2B2C2E', 'metal', segments=10),
            cyl([0.14, 0.1, 0.14], [0, 0.47, 0], BULB, 'glass', segments=10),
            cyl([0.18, 0.04, 0.18], [0, 0.54, 0], '#2B2C2E', 'metal', taper=0.5, segments=10)]


add('street.pathLight', path_light())

# =============================================================================================
# Farm (farm lots in farm worlds, game/town.ts `farmstead`)
# =============================================================================================
BARN_RED, BARN_DARK, TRIM = '#A8322D', '#842621', '#F1EEE6'
ROOF_GREY = '#4B4F55'
HAY, HAY_DARK = '#D9B95E', '#B8964A'
GALV = '#A9AFB3'


def slope(a, b, depth, colour, finish, t=0.14):
    """A flat roof plane from (x, y) `a` to `b`, running `depth` along z."""
    dx, dy = b[0] - a[0], b[1] - a[1]
    return box([math.hypot(dx, dy) + 0.06, t, depth], [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2, 0], colour, finish, rotate=[0, 0, math.degrees(math.atan2(dy, dx))])


def barn():
    """A red gambrel-roofed barn, big doors and a hayloft door at the front, white trim."""
    w, d, h = 5.6, 5.5, 3.2
    p = [box([w, h, d], [0, h / 2, 0], BARN_RED, 'matte'),
         box([w + 0.06, 0.25, d + 0.06], [0, 0.125, 0], '#8E8C88', 'matte')]                          # plinth
    # The gable: stepped to follow the roof.
    for width, y0, y1 in ((4.8, h, 3.7), (4.0, 3.7, 4.2), (3.4, 4.2, 4.6), (1.7, 4.6, 4.95)):
        p.append(box([width, y1 - y0, d], [0, (y0 + y1) / 2, 0], BARN_RED, 'matte'))
    for sx in (-1, 1):                                                                                 # gambrel roof
        p += [slope((sx * (w / 2 + 0.08), h - 0.1), (sx * 1.7, 4.6), d + 0.3, ROOF_GREY, 'satin'),
              slope((sx * 1.7, 4.6), (0, 5.3), d + 0.3, ROOF_GREY, 'satin')]
    z = d / 2 + 0.03
    p += [box([2.6, 2.7, 0.06], [0, 1.35, z], BARN_DARK, 'matte'),                                     # doors
          box([1.0, 0.95, 0.06], [0, 4.0, z], BARN_DARK, 'matte')]                                    # hayloft
    for x0, y0, x1, y1 in ((-1.3, 0.1, 1.3, 2.7), (-0.5, 3.52, 0.5, 4.47)):                            # frames, crossed
        p += [box([x1 - x0 + 0.16, 0.12, 0.08], [0, y1, z + 0.01], TRIM, 'wood'),
              box([x1 - x0 + 0.16, 0.12, 0.08], [0, y0, z + 0.01], TRIM, 'wood'),
              box([0.12, y1 - y0, 0.08], [x0, (y0 + y1) / 2, z + 0.01], TRIM, 'wood'),
              box([0.12, y1 - y0, 0.08], [x1, (y0 + y1) / 2, z + 0.01], TRIM, 'wood'),
              rod([x0, y0, z + 0.02], [x1, y1, z + 0.02], 0.09, TRIM, 'wood', shape='box', t2=0.06),
              rod([x1, y0, z + 0.02], [x0, y1, z + 0.02], 0.09, TRIM, 'wood', shape='box', t2=0.06)]
    for x in (-w / 2, w / 2):                                                                          # corner boards
        for zz in (-d / 2, d / 2):
            p.append(box([0.14, h - 0.02, 0.14], [x * 0.99, h / 2 - 0.01, zz * 0.99], TRIM, 'wood'))      # ending under the wall top
    p += [cyl([0.04, 0.7, 0.04], [0, 5.65, 0], '#2B302E', 'satin', segments=6),                        # weathervane
          box([0.6, 0.03, 0.12], [0, 5.9, 0], '#2B302E', 'satin'),
          cone([0.14, 0.2, 0.14], [0.32, 5.9, 0], '#2B302E', 'satin', rotate=[0, 0, -90], segments=6)]
    return p


add('farm.barn', barn(), (6, 6))


def silo():
    """A tall corrugated grain silo with a domed cap and a ladder."""
    p = [cyl([2.7, 0.24, 2.7], [0, 0.12, 0], '#9A958C', 'matte', segments=24),
         cyl([2.4, 6.0, 2.4], [0, 3.1, 0], '#C3C8CB', 'metal', segments=24),
         sph([2.44, 1.3, 2.44], [0, 6.1, 0], '#A8322D', 'satin', segments=24),
         cyl([0.3, 0.2, 0.3], [0, 6.8, 0], '#C3C8CB', 'metal', segments=10)]
    for y in (1.1, 2.1, 3.1, 4.1, 5.1):
        p.append(cyl([2.46, 0.06, 2.46], [0, y, 0], GALV, 'metal', segments=24))
    for x in (-0.18, 0.18):                                                                            # ladder
        p.append(box([0.04, 5.8, 0.04], [x, 3.1, 1.27], '#5C6168', 'metal'))
    for i in range(12):
        p.append(box([0.4, 0.03, 0.03], [0, 0.6 + i * 0.45, 1.27], '#5C6168', 'metal'))
    return p


add('farm.silo', silo(), (3, 3))


def tractor():
    """A small green farm tractor: big wheels behind, a cab, a hood with an exhaust stack, facing +Z."""
    green, yellow, tyre = '#3E7D3A', '#F2C14E', '#26282B'
    p = [box([0.9, 0.55, 1.7], [0, 0.95, 0.75], green, 'gloss', radius=0.06),                          # hood
         box([0.92, 0.12, 0.08], [0, 0.8, 1.62], '#2B302E', 'metal'),                                  # grille
         box([1.1, 0.3, 1.2], [0, 0.85, -0.5], green, 'gloss', radius=0.04),                           # body under the cab
         box([0.5, 0.12, 0.45], [0, 1.15, -0.6], '#2B302E', 'matte'),                                  # seat
         box([0.5, 0.4, 0.08], [0, 1.4, -0.85], '#2B302E', 'matte'),
         box([1.3, 0.08, 1.2], [0, 2.4, -0.5], green, 'gloss', radius=0.03),                           # cab roof
         cyl([0.08, 0.7, 0.08], [0.3, 1.55, 1.25], '#2B302E', 'metal', segments=8),                    # exhaust
         cyl([0.3, 0.06, 0.3], [0, 1.3, -0.1], '#2B302E', 'metal', rotate=[-40, 0, 0], segments=12)]   # wheel
    for x in (-0.6, 0.6):
        for zz in (-1.05, 0.05):
            p.append(box([0.06, 1.25, 0.06], [x, 1.78, zz], '#2B302E', 'metal'))
    for sx in (-1, 1):
        p += [cyl([1.45, 0.42, 1.45], [sx * 0.76, 0.73, -0.75], tyre, 'matte', rotate=[0, 0, 90], segments=20),
              cyl([0.85, 0.44, 0.85], [sx * 0.76, 0.73, -0.75], yellow, 'gloss', rotate=[0, 0, 90], segments=16),
              cyl([0.78, 0.3, 0.78], [sx * 0.58, 0.39, 1.3], tyre, 'matte', rotate=[0, 0, 90], segments=16),
              cyl([0.44, 0.32, 0.44], [sx * 0.58, 0.39, 1.3], yellow, 'gloss', rotate=[0, 0, 90], segments=12)]
    return p


add('farm.tractor', tractor(), (2, 4))


def hay_bale():
    """A round bale lying on its side, wrapped with twine."""
    p = [cyl([0.98, 0.86, 0.98], [0, 0.49, 0], HAY, 'fabric', rotate=[0, 0, 90], segments=20),
         cyl([0.7, 0.87, 0.7], [0, 0.49, 0], HAY_DARK, 'fabric', rotate=[0, 0, 90], segments=16)]
    for x in (-0.25, 0.0, 0.25):
        p.append(cyl([1.0, 0.03, 1.0], [x, 0.49, 0], '#8C6A3A', 'matte', rotate=[0, 0, 90], segments=20))
    return p


add('farm.hayBale', hay_bale())


def hay_stack():
    """Square bales: two side by side and one on top."""
    p = []
    for x, y in ((-0.48, 0.23), (0.48, 0.23), (0.0, 0.69)):
        p.append(box([0.92, 0.44, 0.86], [x, y, 0], HAY, 'fabric', radius=0.04))
        for dz in (-0.22, 0.22):
            p.append(box([0.94, 0.45, 0.03], [x, y, dz], '#8C6A3A', 'matte'))
    return p


add('farm.hayStack', hay_stack(), (2, 1))


def scarecrow():
    """A scarecrow on a post: a plaid shirt, straw hands, a sack head and a floppy hat."""
    p = [cyl([0.07, 1.9, 0.07], [0, 0.95, -0.05], '#7A5A3A', 'wood', segments=8),
         box([0.94, 0.06, 0.07], [0, 1.45, -0.05], '#7A5A3A', 'wood'),
         box([0.44, 0.62, 0.24], [0, 1.25, 0], '#B8322B', 'fabric', radius=0.05),                      # shirt
         box([0.46, 0.06, 0.26], [0, 1.05, 0], '#2E4E7A', 'fabric'),                                   # belt of twine
         box([0.8, 0.16, 0.16], [0, 1.45, 0], '#B8322B', 'fabric', radius=0.04),                       # sleeves
         sph([0.32, 0.36, 0.3], [0, 1.78, 0], '#C9A86A', 'matte', segments=12),                        # sack head
         cyl([0.56, 0.04, 0.56], [0, 1.93, 0], '#6E5A3A', 'fabric', segments=16),                      # hat
         cyl([0.28, 0.2, 0.28], [0, 2.03, 0], '#6E5A3A', 'fabric', taper=0.8, segments=12)]
    for sx in (-1, 1):
        p.append(cone([0.14, 0.16, 0.14], [sx * 0.4, 1.45, 0], HAY, 'matte', rotate=[0, 0, sx * 90], segments=8))
    return p


add('farm.scarecrow', scarecrow())


def trough():
    """A galvanised water trough on short legs."""
    p = [box([1.8, 0.5, 0.7], [0, 0.4, 0], GALV, 'metal', radius=0.08),
         box([1.66, 0.02, 0.56], [0, 0.6, 0], '#5B93A8', 'gloss')]
    for x in (-0.7, 0.7):
        p.append(box([0.08, 0.16, 0.6], [x, 0.08, 0], '#5C6168', 'metal'))
    return p


add('farm.trough', trough(), (2, 1))


def coop():
    """A little chicken coop on legs: a wooden house with a red roof, a ramp down the front."""
    wood, roof = '#C8A46E', '#A8322D'
    p = [box([1.5, 0.9, 1.1], [0, 1.0, -0.25], wood, 'wood'),
         box([0.3, 0.36, 0.04], [0, 0.78, 0.31], '#3A2A1E', 'matte'),                                  # hatch
         box([0.26, 0.04, 0.9], [0, 0.32, 0.6], wood, 'wood', rotate=[-30, 0, 0])]                     # ramp
    for x in (-0.65, 0.65):
        for z in (-0.7, 0.2):
            p.append(box([0.08, 0.55, 0.08], [x, 0.28, z], '#7A5A3A', 'wood'))
    for sz in (-1, 1):
        p.append(box([1.7, 0.06, 0.8], [0, 1.66, -0.25 + sz * 0.33], roof, 'satin', rotate=[sz * 32, 0, 0]))
    return p


add('farm.coop', coop(), (2, 2))


# =============================================================================================
# Validation (footprint bounds, finishes) and output
# =============================================================================================
FINISHES = {'material.finish.' + f for f in 'matte satin gloss metal chrome fabric wood bark foliage skin glass'.split()}
ICONS = {'icon.category.community': {'type': 'icon', 'url': '../icons/landmark.svg'}}
# Things that reach past their tiles on purpose.
OVERHANG = set()


def _bounds(q):
    """OX.bounds, but a four-sided cone or cylinder turned 45° about y (a pyramid roof, square to the axes) by its
    corners, not by its turned box (which counts its diagonal)."""
    lo, hi = OX.bounds(q)
    if q['shape'] in ('cone', 'cylinder') and q.get('segments') == 4 and list(q.get('rotate', [])) == [0, 45, 0]:
        half = max(q['size'][0], q['size'][2]) / 2 / math.sqrt(2)
        for k in (0, 2):
            lo[k], hi[k] = q['at'][k] - half, q['at'][k] + half
    return lo, hi


def main(path):
    bad = 0
    for k, e in E.items():
        fp = FOOT[k]
        issues = []
        parts = e['placeholder']
        fins = {p.get('material') for p in parts}
        if len(fins) > 4: issues.append(f'{len(fins)} finishes (max 4): {sorted(f.split(".")[-1] for f in fins)}')
        for p in parts:
            q = dict(p, shape='cylinder') if p['shape'] in ('lathe', 'torus') else p
            issues += [i for i in OX.validate(k, [q], fp) if 'exceeds' not in i and 'below floor' not in i and 'finishes (max 4)' not in i]
            if p.get('material') not in FINISHES: issues.append(f'bad finish {p.get("material")}')
            lo, hi = _bounds(q)
            if lo[1] < -0.015: issues.append(f'part {p["shape"]} at {p["at"]} below the ground')
            if k not in OVERHANG and (lo[0] < -fp[0] / 2 - 0.02 or hi[0] > fp[0] / 2 + 0.02 or lo[2] < -fp[1] / 2 - 0.02 or hi[2] > fp[1] / 2 + 0.02):
                issues.append(f'part {p["shape"]} at {p["at"]} leaves the {fp} footprint ({lo[0]:.2f}..{hi[0]:.2f}, {lo[2]:.2f}..{hi[2]:.2f})')
        for s in issues: print(f'  ! {k}: {s}')
        bad += len(issues)
    lines = [f'    {json.dumps(k)}: {json.dumps(e, separators=(",", ":"))}' for k, e in {**ICONS, **E}.items()]
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write('{\n  "$comment": "The town\'s community things (content/community.json: street furniture, playground, carts and machines, shop '
                'fittings) and the street dressing props (model.street.*, render/babylon/street.ts), written by tools/art/community.py. '
                'Listed in the base manifest\'s packs.",\n  "version": 1,\n  "entries": {\n')
        f.write(',\n'.join(lines) + '\n  }\n}\n')
    json.load(open(path))
    print(f'{len(E)} models -> {path}; {bad} issue(s)')
    return bad


if __name__ == '__main__':
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'web', 'public', 'assets', 'community', 'manifest.json')
    sys.exit(1 if main(os.path.normpath(sys.argv[1] if len(sys.argv) > 1 else out)) else 0)
