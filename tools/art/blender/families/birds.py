# The garden birds the game flies about by code (`model.bird.*`): the blue tit, chaffinch, willow warbler,
# blackbird and collared dove, at their real sizes and in their own plumage, perched (standing on a twig or the
# lawn), each in five rigid parts the renderer moves (birds.md):
#
#   body     the torso with the wings folded on it and the legs; origin between the feet, on the perch
#   head     the head, bill and eyes; origin at the neck joint
#   tail     origin at its base, under the rump
#   wing.L   the wings spread for flight, level, origin at the shoulder, reaching out to the bird's own left
#   wing.R   (game -X, Blender +X) and right (game +X); the renderer flaps them about the body's forward axis
#            and hides them (scale 0) while the bird is perched. At rest they are spread, at scale 1.
#
# No clips: the game moves the parts. A bird is a handful of pixels from the game's camera (6-95 m away, mostly
# 15-30), so each is a few hundred triangles, lofted rings coloured by plumage zone, every material plain
# (matte feathers, satin bill and legs, a gloss eye): one draw call for the lot.
#
# Run:  cargo art birds [--only blueTit] [--preview]
#       (--preview also renders each from the side, the front and above, and at 6 and 20 m beside a resident
#       1.77 m tall: previews/birds/<id>.<view>.png)
import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

# The plumage zones, one material each; every species colours them its own way (`palette`). Feathers are
# matte, the bill and legs satin, the eye gloss: all plain, so a bird is one draw call.
BASE = dict(
    back=('#8A9A5A', 'matte'),      # the mantle and the back of the body
    cap=('#3F7FC8', 'matte'),       # the crown
    brow=('#F2F2EE', 'matte'),      # over the eye (a tit's white band, a warbler's pale eyebrow)
    mask=('#1E2430', 'matte'),      # the stripe through the eye
    face=('#F4F4F0', 'matte'),      # the cheek
    throat=('#1E2430', 'matte'),    # chin and throat
    nape=('#3F6FB0', 'matte'),      # the back of the head
    collar=('#1A1A1A', 'matte'),    # a dove's black half-collar
    breast=('#F2D23A', 'matte'),
    belly=('#E8D050', 'matte'),
    wing=('#4A82C8', 'matte'),      # the coverts, folded and spread
    bar=('#F2F2EE', 'matte'),       # wing bars and the chaffinch's shoulder patch
    flight=('#4F6E9A', 'matte'),    # the flight feathers
    tip=('#3E5578', 'matte'),       # the primaries' ends (a dove's dark wingtips)
    under=('#C8CCC8', 'matte'),     # under the spread wing
    tail=('#3F6FB0', 'matte'),
    tailtip=('#EEEAE4', 'matte'),   # a dove's pale tail end
    bill=('#2A2A2A', 'satin'),
    eye=('#121212', 'gloss'),
    ring=('#F2B828', 'satin'),      # a blackbird's eye ring
    leg=('#6A7A90', 'satin'),
)
M = G.Palette(**BASE)


def palette(**colours):
    """The family's palette in one species' colours (the finishes stay the family's)."""
    return G.Palette(**{**BASE, **{k: (v, BASE[k][1]) for k, v in colours.items()}})


# ---- the build: a bird of `S` (a species' numbers, metres and degrees) ----------------------------------
# Built in a side view: f forward (Blender -y), u up (z), x to the bird's left (Blender +x, the game's -x).

def P(x, f, u):
    return Vector((x, -f, u))


def _mesh(name, verts, faces):
    """A closed mesh from `verts` and `faces` [(indices, material)], its normals outwards, smooth."""
    mats = list(dict.fromkeys(m for _, m in faces))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in verts], [], [f for f, _ in faces])
    for m in mats:
        mesh.materials.append(m)
    for poly, (_, m) in zip(mesh.polygons, faces):
        poly.material_index = mats.index(m)
        poly.use_smooth = True
    mesh.validate()
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    return obj


def loft(name, rings, zone, start=None, end=None):
    """A closed tube through `rings` (each the same number of points, round the same way), closed at each end by a
    point (`start`, `end`: a fan) or flat. `zone(i, j)` is the material of the face between ring i and i + 1 at
    j (from point j to j + 1); the end faces take zone(-1, j) and zone(len(rings) - 1, j)."""
    n = len(rings[0])
    verts = [v for r in rings for v in r]
    faces = []
    for i in range(len(rings) - 1):
        for j in range(n):
            k = (j + 1) % n
            faces.append(((i * n + j, i * n + k, (i + 1) * n + k, (i + 1) * n + j), zone(i, j)))
    last = (len(rings) - 1) * n
    if start is not None:
        verts.append(start)
        s = len(verts) - 1
        faces += [((s, k, j), zone(-1, j)) for j, k in ((j, (j + 1) % n) for j in range(n))]
    else:
        faces.append((tuple(range(n))[::-1], zone(-1, 0)))
    if end is not None:
        verts.append(end)
        e = len(verts) - 1
        faces += [((e, last + j, last + (j + 1) % n), zone(len(rings) - 1, j)) for j in range(n)]
    else:
        faces.append((tuple(range(last, last + n)), zone(len(rings) - 1, 0)))
    return _mesh(name, verts, faces)


def octahedron(name, at, r, material, squash=1.0):
    """A small eight-sided gem (an eye), `r` across its middle, flattened along x by `squash`."""
    c = Vector(at)
    v = [c + Vector(d) for d in ((r * squash, 0, 0), (-r * squash, 0, 0), (0, r, 0), (0, -r, 0), (0, 0, r), (0, 0, -r))]
    faces = [((a, b, c_), material) for a, b, c_ in
             ((0, 2, 4), (2, 1, 4), (1, 3, 4), (3, 0, 4), (2, 0, 5), (1, 2, 5), (3, 1, 5), (0, 3, 5))]
    return _mesh(name, v, faces)


# Where a body's ring points are, round from the top (degrees; 90 = the bird's left): faces centred at 0, 45 ... 315.
BODY_RING = [22.5 + 45 * j for j in range(8)]
# A head's: a top (the crown), an upper side, a narrow band through the eye, the cheek and the throat.
HEAD_RING = [35, 62, 100, 150, 210, 260, 298, 325]
# The body's stations from rump (0) to neck (1) and how full it is at each.
BODY_T = [0.0, 0.08, 0.2, 0.38, 0.58, 0.76, 0.9, 1.0]
BODY_K = [0.32, 0.64, 0.87, 1.0, 1.0, 0.9, 0.7, 0.52]


def _mid(a, b):
    return (a + b) / 2 % 360 if abs(a - b) < 180 else ((a + b + 360) / 2) % 360


def _side(deg):
    """How far round a face centred at `deg` is from the top: 0 (top) to 180 (underneath)."""
    return 180 - abs(180 - deg % 360)


class Bird:
    """One bird built from its numbers, part by part, keeping the points the parts meet at."""

    def __init__(self, S, P_):
        self.S, self.M = S, P_
        p = math.radians(S['pitch'])
        self.a = Vector((0, math.cos(p), math.sin(p)))         # (x, f, u): the body's axis, rump to breast
        self.n = Vector((0, -math.sin(p), math.cos(p)))        # square to it, up
        self.c = Vector((0, S['bodyAt'], S['leg'] + S['depth'] * 0.47))   # the body's middle over the feet

    # (x, f, u) -> Blender
    @staticmethod
    def b(v):
        return P(v.x, v.y, v.z)

    def body_point(self, t, deg, out=0.0):
        """A point on the body's surface at `t` (rump 0 .. neck 1) and `deg` round from the top, `out` metres off it."""
        S = self.S
        k = _interp(BODY_T, BODY_K, t)
        th = math.radians(deg)
        centre = self.c + self.a * (S['body'] * (t - 0.5))
        hw, hh = S['width'] / 2 * k, S['depth'] / 2 * k
        sag = -self.n * hh * 0.1 * math.sin(math.pi * min(1, max(0, t)))      # a fuller breast and belly
        normal = (self.n * (math.cos(th) / max(hh, 1e-6)) + Vector((1, 0, 0)) * (math.sin(th) / max(hw, 1e-6))).normalized()
        return centre + sag + self.n * (hh * math.cos(th)) + Vector((hw * math.sin(th), 0, 0)) + normal * out

    def torso(self):
        S, Z = self.S, self.S['zones']
        rings = [[self.b(self.body_point(t, d)) for d in BODY_RING] for t in BODY_T]

        def zone(i, j):
            t = (BODY_T[max(0, min(i, 6))] + BODY_T[max(0, min(i + 1, 7))]) / 2
            deg = _side(_mid(BODY_RING[j], BODY_RING[(j + 1) % 8]))
            return getattr(self.M, Z['body'](t, deg))
        rump = self.b(self.c + self.a * (-S['body'] * 0.52))
        return loft('torso', rings, zone, start=rump)

    def folded_wing(self, side):
        """A wing folded on the body's upper side, from the shoulder back over the rump to its tip over the tail."""
        S, Z = self.S, self.S['zones']
        st = [(0.76, 60, 0.14), (0.62, 63, 0.27), (0.5, 62, 0.29), (0.4, 58, 0.27), (0.16, 44, 0.17)]
        rings = []
        for t, deg, w in st:
            centre = self.body_point(t, deg, S['depth'] * 0.03)
            outward = (self.body_point(t, deg, 1.0) - self.body_point(t, deg)).normalized()
            along = (self.body_point(t, deg + 8) - self.body_point(t, deg - 8)).normalized()
            half, thick = S['depth'] * w, S['depth'] * 0.06
            ring = [centre - along * half, centre + outward * thick, centre + along * half, centre - outward * thick * 0.5]
            rings.append([self.b(Vector((v.x * side, v.y, v.z))) for v in ring])
        tip = self.tail_base + self.tail_dir * (S['tailLen'] * S['tipOver']) + self.tail_up * S['width'] * 0.09
        tip = self.b(Vector((side * S['width'] * 0.1, tip.y, tip.z)))
        bands = Z['folded']                                        # a zone for each length: shoulder .. tip
        return loft(f'folded{side}', rings, lambda i, j: getattr(self.M, bands[max(0, i)]), end=tip)

    def legs(self):
        S = self.S
        parts = []
        for side in (-1, 1):
            x = side * S['width'] * 0.17
            r = S['legR']
            top = self.b(Vector((x, 0.0, S['leg'] + S['depth'] * 0.25)))
            foot = self.b(Vector((x, 0.0, r)))
            parts.append(_tube(f'leg{side}', [top, foot], r, r, self.M.leg))
            toe = S['toe']
            parts.append(_tube(f'toes{side}', [self.b(Vector((x, -toe * 0.45, r))), self.b(Vector((x, toe, r)))], r, r * 0.7,
                               self.M.leg))
        return parts

    def head(self):
        """The head (a lofted ball), its bill and its eyes; returns them and the neck joint (Blender)."""
        S, Z = self.S, self.S['zones']
        R = S['head']
        neck = self.c + self.a * (S['body'] * 0.5)
        up = math.radians(S['neckUp'])
        hc = neck + Vector((0, math.cos(up), math.sin(up))) * R * S['neck']
        self.head_centre = hc
        ss = [-0.72, -0.25, 0.25, 0.68]

        def at(s, deg, out=0.0):
            q = math.sqrt(max(0.04, 1 - s * s))
            th = math.radians(deg)
            hw, hh = R * 0.9 * q, R * q
            return hc + Vector((hw * math.sin(th) * (1 + out), s * R, hh * math.cos(th) * (1 + out)))
        rings = [[self.b(at(s, d)) for d in HEAD_RING] for s in ss]
        back = self.b(hc + Vector((0, -R, 0.05 * R)))
        front = self.b(hc + Vector((0, R * 0.96, -0.1 * R)))
        zs = [(-1.0 + ss[0]) / 2] + [(ss[i] + ss[i + 1]) / 2 for i in range(3)] + [(ss[-1] + 1) / 2]

        def zone(i, j):
            deg = _side(_mid(HEAD_RING[j], HEAD_RING[(j + 1) % 8]))
            return getattr(self.M, Z['head'](zs[i + 1], deg))
        parts = [loft('skull', rings, zone, start=back, end=front)]
        # The bill: a four-sided point from the face, tipped down a little.
        base = hc + Vector((0, R * 0.78, -R * S['billLow']))
        drop = math.radians(S['billDroop'])
        tipv = base + Vector((0, math.cos(drop), -math.sin(drop))) * (S['bill'] + R * 0.2)
        bw, bh = S['billW'] / 2, S['billH'] / 2
        ring = [base + Vector((bw, 0, 0)), base + Vector((0, 0, bh)), base + Vector((-bw, 0, 0)), base + Vector((0, 0, -bh))]
        parts.append(loft('bill', [[self.b(v) for v in ring]], lambda i, j: self.M.bill, end=self.b(tipv)))
        self.bill_tip = tipv
        # The eyes, on the band through them (a blackbird's in a yellow ring).
        for side in (-1, 1):
            e = at(S['eyeAt'], 88)
            e = Vector((side * e.x, e.y, e.z))
            out = Vector((side, 0.25, 0.1)).normalized()
            r = R * S['eye']
            if S.get('ring'):
                parts.append(octahedron(f'ring{side}', self.b(e - out * r * 0.55), r * 1.5, self.M.ring, squash=0.6))
            parts.append(octahedron(f'eye{side}', self.b(e - out * r * 0.25), r, self.M.eye, squash=0.6))
        return parts, self.b(neck)

    def tail(self):
        """The tail from under the rump, back and down; returns it and its base (Blender)."""
        S, Z = self.S, self.S['zones']
        d, up = self.tail_dir, self.tail_up
        base = self.tail_base
        T = S['tailLen']
        W = S['width']
        # Narrow and deep at the base (the coverts over and under it), spreading and thinning to its end: `tail`
        # [(along it, half its width, its thickness), in body widths].
        rings = []
        for f, w, th in S['tail']:
            c = base + d * (T * f)
            rings.append([self.b(c + Vector((W * w, 0, 0))), self.b(c + up * W * th), self.b(c - Vector((W * w, 0, 0))),
                          self.b(c - up * W * th * 0.6)])
        bands = Z['tail']
        return loft('tail', rings, lambda i, j: getattr(self.M, bands[max(0, min(i, 2))])), self.b(base)

    def wing(self, side):
        """A wing spread for flight, level, from the shoulder out to `side` (1: the bird's left, Blender +x)."""
        S, Z = self.S, self.S['zones']
        sh = self.shoulder
        span = S['wingspan'] / 2 - abs(sh.x)
        C = S['chord']
        rings = []
        for u, lead, trail in S['planform']:
            x = abs(sh.x) + span * u
            th = C * (0.11 * (1 - u) + 0.022)
            l, t_ = lead * C, trail * C
            pts = [(l, 0.0), (l + (t_ - l) / 3, th * 0.7), (l + 2 * (t_ - l) / 3, th * 0.45), (t_, 0.0),
                   (l + 2 * (t_ - l) / 3, -th * 0.2), (l + (t_ - l) / 3, -th * 0.3)]
            rings.append([self.b(Vector((side * x, sh.y + f, sh.z + z))) for f, z in pts])
        bands = Z['wing']                                          # (top front, top middle, top back) per segment

        def zone(i, j):
            i = max(0, min(i, len(bands) - 1))
            return getattr(self.M, bands[i][j] if j < 3 else ('tip' if bands[i][2] == 'tip' else 'under'))
        return loft(f'wing{side}', rings, zone)

    def build(self):
        S = self.S
        drop = math.radians(S['tailDown'])
        self.tail_dir = Vector((0, -math.cos(drop), -math.sin(drop)))
        self.tail_up = Vector((0, -math.sin(drop), math.cos(drop)))
        self.tail_base = self.body_point(0.07, 180, -S['depth'] * 0.12) + self.n * S['depth'] * 0.12
        self.tail_base.x = 0.0
        self.shoulder = self.body_point(0.72, 62)
        with G.new_parts() as head:
            _, neck = self.head()
        # The tail reaches as far back as makes the bird its length, bill tip to tail tip.
        S['tailLen'] = _reach(self.bill_tip, self.tail_base, self.tail_dir, S['length'])
        with G.new_parts() as body:
            self.torso()
            self.folded_wing(1)
            self.folded_wing(-1)
            self.legs()
        body = G.join(body, 'body', origin=(0, 0, 0))
        head = G.join(head, 'head', origin=neck)
        with G.new_parts() as tail:
            _, base = self.tail()
        tail = G.join(tail, 'tail', origin=base)
        wings = []
        for side, name in ((1, 'wing.L'), (-1, 'wing.R')):
            w = self.wing(side)
            wings.append(G.join([w], name, origin=self.b(Vector((side * abs(self.shoulder.x), self.shoulder.y, self.shoulder.z)))))
        print(f'[gamekit] {S["name"]}: tail {S["tailLen"] * 100:.1f} cm, body middle {self.c.z * 100:.1f} cm up, '
              f'head {self.head_centre.z * 100:.1f} cm up')
        return [body, head, tail, *wings]


def _interp(xs, ys, x):
    if x <= xs[0]:
        return ys[0]
    for i in range(len(xs) - 1):
        if x <= xs[i + 1]:
            f = (x - xs[i]) / (xs[i + 1] - xs[i])
            return ys[i] + (ys[i + 1] - ys[i]) * f
    return ys[-1]


def _reach(a, base, d, length):
    """How far from `base` along `d` (unit) is `length` from `a`."""
    w = base - a
    bq = 2 * w.dot(d)
    cq = w.dot(w) - length * length
    return (-bq + math.sqrt(max(0.0, bq * bq - 4 * cq))) / 2


def _tube(name, points, r0, r1, material):
    """A three-sided rod (a leg, a toe)."""
    pts = [Vector(p) for p in points]
    t = (pts[-1] - pts[0]).normalized()
    a = t.orthogonal().normalized()
    b = t.cross(a)
    rings = []
    for i, p in enumerate(pts):
        r = r0 + (r1 - r0) * i / (len(pts) - 1)
        rings.append([p + (a * math.cos(k * math.tau / 3) + b * math.sin(k * math.tau / 3)) * r for k in range(3)])
    return loft(name, rings, lambda i, j: material)


# ---- the species ------------------------------------------------------------------------------------------
# length bill to tail tip and wingspan (m) are the field guides'; the rest shapes it: `body` the torso's length,
# `width`/`depth` across it, `bodyAt` how far the body's middle stands ahead of the feet, `leg` the leg showing,
# `pitch` the torso's rise from level (degrees), `head` its radius, `neck` how far the head's middle stands off the
# neck (head radii) at `neckUp` degrees, `bill`, `billW`, `billH`, `billDroop`, `billLow`, `eye` (head radii),
# `eyeAt` how far forward the eye is (-1 .. 1 along the head), `tailDown` (degrees below level), `tipOver` how far
# down the tail the folded wingtips reach, `chord` the spread wing's depth at the body, `planform` its outline
# [(along the span 0 .. 1, leading edge, trailing edge: chords forward of the shoulder)], `tail` its shape (SQUARE ...),
# `legR` and `toe` the legs' thickness and the toes' length, `ring` an eye ring, and `zones`: which material each face
# of the body (by `t` rump 0 .. neck 1 and degrees round from the top), the head (by `s` back -1 .. front 1 and
# degrees), the folded wing (shoulder .. tip), the tail (base .. end) and the spread wing (top front, middle, back for
# each length out from the body) takes.

# Tails: a tit's and a finch's square-ended, a warbler's short, a blackbird's long and round-ended, a dove's long and broad.
SQUARE = [(0.0, 0.16, 0.22), (0.45, 0.27, 0.11), (0.8, 0.33, 0.06), (1.0, 0.33, 0.04)]
LONG = [(0.0, 0.15, 0.22), (0.4, 0.22, 0.1), (0.82, 0.26, 0.05), (1.0, 0.2, 0.035)]
BROAD = [(0.0, 0.17, 0.2), (0.4, 0.27, 0.08), (0.8, 0.34, 0.045), (1.0, 0.3, 0.03)]

ROUNDED = [(0.0, 0.2, -0.84), (0.2, 0.27, -0.9), (0.42, 0.27, -0.9), (0.64, 0.18, -0.8), (0.84, 0.04, -0.66),
           (1.0, -0.16, -0.46)]
POINTED = [(0.0, 0.2, -0.85), (0.2, 0.27, -0.86), (0.42, 0.27, -0.78), (0.64, 0.12, -0.62), (0.84, -0.12, -0.52),
           (1.0, -0.42, -0.5)]


def tit_body(t, deg):
    if deg < 30:
        return 'back' if t < 0.86 else 'nape'
    if deg < 70:
        return 'back' if t < 0.86 else 'face'
    if deg < 110:
        return 'breast' if t > 0.35 else 'back'
    return 'breast' if t > 0.3 else 'belly'


def tit_head(s, deg):
    if deg < 60 and s > 0.6:
        return 'brow'                                     # the white forehead
    if deg < 25:
        return 'cap' if s > -0.75 else 'nape'
    if deg < 60:
        return 'brow' if s > -0.1 else ('cap' if s > -0.75 else 'nape')
    if deg < 100:
        return 'mask'
    if deg < 150:
        return 'face' if s > -0.6 else 'mask'
    return 'throat' if s > -0.2 else 'breast'


BLUE_TIT = dict(
    name='blue tit', length=0.12, wingspan=0.185, body=0.058, width=0.036, depth=0.04, bodyAt=0.004, leg=0.011,
    pitch=34, head=0.0145, neck=0.55, neckUp=60, bill=0.0045, billW=0.0045, billH=0.0042, billDroop=4, billLow=0.1,
    eye=0.2, eyeAt=0.3, legR=0.0012, toe=0.009, tailDown=28, tipOver=0.32, chord=0.042, planform=ROUNDED, tail=SQUARE,
    zones=dict(body=tit_body, head=tit_head, folded=['wing', 'wing', 'bar', 'flight', 'flight'],
               tail=['tail', 'tail', 'tail'],
               wing=[('wing', 'bar', 'flight'), ('wing', 'bar', 'flight'), ('wing', 'flight', 'flight'),
                     ('flight', 'flight', 'flight'), ('flight', 'flight', 'flight')]),
)
BLUE_TIT_COLOURS = dict(back='#93A35C', cap='#3D7ED0', brow='#F4F4F0', mask='#1C2438', face='#F6F6F2', throat='#1C2438',
                        nape='#2C4A86', breast='#F4D234', belly='#E9D24E', wing='#4683CC', bar='#F2F2EC', flight='#4E6C98',
                        tip='#3E5578', under='#CED2CE', tail='#3D70B4', bill='#2A2A2C', eye='#101010', leg='#6A7A92')


def finch_body(t, deg):
    if deg < 30:
        return 'back' if t < 0.86 else 'nape'
    if deg < 70:
        return 'back' if t < 0.86 else 'face'
    if deg < 110:
        return 'breast' if t > 0.32 else 'belly'
    return 'breast' if t > 0.45 else 'belly'


def finch_head(s, deg):
    if deg < 60:
        return 'cap'
    if deg < 100:
        return 'face' if s > -0.5 else 'cap'
    if deg < 150:
        return 'face'
    return 'throat'


CHAFFINCH = dict(
    name='chaffinch', length=0.145, wingspan=0.26, body=0.064, width=0.04, depth=0.042, bodyAt=0.004, leg=0.013,
    pitch=27, head=0.0158, neck=0.55, neckUp=55, bill=0.0085, billW=0.0068, billH=0.0068, billDroop=6, billLow=0.12,
    eye=0.19, eyeAt=0.28, legR=0.0013, toe=0.011, tailDown=22, tipOver=0.3, chord=0.054, planform=ROUNDED, tail=SQUARE,
    zones=dict(body=finch_body, head=finch_head, folded=['bar', 'wing', 'bar', 'flight', 'flight'],
               tail=['tail', 'tail', 'tail'],
               wing=[('bar', 'bar', 'flight'), ('wing', 'bar', 'flight'), ('wing', 'bar', 'flight'),
                     ('flight', 'flight', 'flight'), ('flight', 'flight', 'flight')]),
)
CHAFFINCH_COLOURS = dict(back='#8C6448', cap='#6C88B0', nape='#6C88B0', face='#C98476', throat='#CB8676', breast='#CC8274',
                         belly='#EEDCD2', wing='#33302E', bar='#F2F0EA', flight='#3C3632', under='#DAD4CC', tail='#3A3634',
                         bill='#8C9CAC', eye='#101010', leg='#8C7262')


def warbler_body(t, deg):
    if deg < 70:
        return 'back'
    if deg < 110:
        return 'breast' if t > 0.4 else 'back'
    return 'breast' if t > 0.5 else 'belly'


def warbler_head(s, deg):
    if deg < 25:
        return 'cap'
    if deg < 60:
        return 'brow' if s > -0.3 else 'cap'
    if deg < 100:
        return 'mask' if s > -0.5 else 'cap'
    if deg < 150:
        return 'face'
    return 'throat'


WARBLER = dict(
    name='willow warbler', length=0.11, wingspan=0.19, body=0.05, width=0.03, depth=0.032, bodyAt=0.004, leg=0.013,
    pitch=26, head=0.0124, neck=0.55, neckUp=50, bill=0.0068, billW=0.003, billH=0.0028, billDroop=3, billLow=0.1,
    eye=0.2, eyeAt=0.3, legR=0.0011, toe=0.009, tailDown=18, tipOver=0.3, chord=0.04, planform=ROUNDED, tail=SQUARE,
    zones=dict(body=warbler_body, head=warbler_head, folded=['wing', 'wing', 'wing', 'flight', 'flight'],
               tail=['tail', 'tail', 'tail'],
               wing=[('wing', 'flight', 'flight'), ('wing', 'flight', 'flight'), ('wing', 'flight', 'flight'),
                     ('flight', 'flight', 'flight'), ('flight', 'flight', 'flight')]),
)
WARBLER_COLOURS = dict(back='#86985A', cap='#86985A', brow='#ECE6A0', mask='#646A40', face='#DCD994', throat='#F0EAAA',
                       breast='#EEE6A4', belly='#F2F0DC', wing='#7C8A58', flight='#6C764E', under='#E6E2C0', tail='#6E7852',
                       bill='#8A7A5C', eye='#101010', leg='#B89A7A')


def blackbird_body(t, deg):
    if deg < 70:
        return 'back'
    return 'breast' if t > 0.4 else 'belly'


def blackbird_head(s, deg):
    return 'cap' if deg < 110 else 'throat'


BLACKBIRD = dict(
    name='blackbird', length=0.25, wingspan=0.36, body=0.112, width=0.066, depth=0.07, bodyAt=0.008, leg=0.03,
    pitch=20, head=0.0235, neck=0.62, neckUp=45, bill=0.019, billW=0.008, billH=0.0085, billDroop=6, billLow=0.1,
    eye=0.17, eyeAt=0.28, ring=True, legR=0.0022, toe=0.02, tailDown=14, tipOver=0.3, chord=0.085, planform=ROUNDED, tail=LONG,
    zones=dict(body=blackbird_body, head=blackbird_head, folded=['wing', 'wing', 'wing', 'flight', 'flight'],
               tail=['tail', 'tail', 'tail'],
               wing=[('wing', 'flight', 'flight'), ('wing', 'flight', 'flight'), ('wing', 'flight', 'flight'),
                     ('flight', 'flight', 'flight'), ('flight', 'flight', 'flight')]),
)
BLACKBIRD_COLOURS = dict(back='#1C1C1E', cap='#1C1C1E', throat='#202022', breast='#1F1F21', belly='#232324', wing='#202020',
                         flight='#28251F', under='#2C2A27', tail='#1C1C1C', bill='#F2A21E', eye='#0E0E0E', ring='#F4BC2A',
                         leg='#3C3228')


def dove_body(t, deg):
    if 0.86 < t < 0.97 and 20 < deg < 110:
        return 'collar'                                    # the half-collar: round the back and sides of the neck
    if deg < 70:
        return 'back' if t < 0.9 else 'cap'
    if deg < 110:
        return 'breast' if t > 0.4 else 'belly'
    return 'breast' if t > 0.45 else 'belly'


def dove_head(s, deg):
    if deg < 100:
        return 'cap'
    return 'throat'


COLLARED_DOVE = dict(
    name='collared dove', length=0.32, wingspan=0.51, body=0.145, width=0.088, depth=0.088, bodyAt=0.012, leg=0.018,
    pitch=16, head=0.024, neck=0.62, neckUp=58, bill=0.012, billW=0.006, billH=0.0055, billDroop=12, billLow=0.15,
    eye=0.17, eyeAt=0.25, legR=0.0022, toe=0.02, tailDown=10, tipOver=0.42, chord=0.115, planform=POINTED, tail=BROAD,
    zones=dict(body=dove_body, head=dove_head, folded=['wing', 'wing', 'wing', 'flight', 'tip'],
               tail=['tail', 'tail', 'tailtip'],
               wing=[('wing', 'wing', 'flight'), ('wing', 'wing', 'flight'), ('wing', 'flight', 'flight'),
                     ('tip', 'tip', 'tip'), ('tip', 'tip', 'tip')]),
)
COLLARED_DOVE_COLOURS = dict(back='#BBA88E', cap='#CFC2B6', throat='#D6C8BC', collar='#1C1A1A', breast='#D8C2B8',
                             belly='#DCD4CC', wing='#C2B29C', flight='#A8A098', tip='#3E3A38', under='#DADAD8', tail='#AA9E8E',
                             tailtip='#EEEAE4', bill='#2C2A2A', eye='#6E1E1E', leg='#B65C5C')


def bird(S, colours):
    def build():
        objs = Bird(dict(S), palette(**colours)).build()
        return G.settle(objs, centre=False, floor=False)
    build.__name__ = S['name'].replace(' ', '_')
    return build


def facts(S):
    """The entry's `bird`: what the renderer needs of its size (`perchHeight`: the body's middle over the feet)."""
    return {'length': S['length'], 'wingspan': S['wingspan'], 'perchHeight': round(S['leg'] + S['depth'] * 0.47, 3), 'pitch': S['pitch']}


# (wingspan, height, length over the ground: bill to tail tip as it sits) as measured; `centre` its box's middle
# (the feet are on the origin, the tail reaches back further than the bill forward); `sink` how far the tail
# hangs below the perch.
FAMILY = G.Family(
    'birds', M,
    [G.Model('blueTit', bird(BLUE_TIT, BLUE_TIT_COLOURS), size=(0.185, 0.072, 0.106), centre=(0, -0.0021), key='model.bird.blueTit', code=True,
             floor=False, sink=0.01, bird=facts(BLUE_TIT)),
     G.Model('chaffinch', bird(CHAFFINCH, CHAFFINCH_COLOURS), size=(0.26, 0.072, 0.135), centre=(0, -0.0059), key='model.bird.chaffinch', code=True,
             floor=False, sink=0.01, bird=facts(CHAFFINCH)),
     G.Model('warbler', bird(WARBLER, WARBLER_COLOURS), size=(0.19, 0.054, 0.103), centre=(0, -0.0019), key='model.bird.warbler', code=True,
             floor=False, bird=facts(WARBLER)),
     G.Model('blackbird', bird(BLACKBIRD, BLACKBIRD_COLOURS), size=(0.36, 0.11, 0.241), centre=(0, -0.0077), key='model.bird.blackbird', code=True,
             floor=False, bird=facts(BLACKBIRD)),
     G.Model('collaredDove', bird(COLLARED_DOVE, COLLARED_DOVE_COLOURS), size=(0.51, 0.11, 0.312), centre=(0, -0.0313), key='model.bird.collaredDove',
             code=True, floor=False, bird=facts(COLLARED_DOVE)),
     ],
    comment='Blender-built birds (tools/art/blender/families/birds.py): the blue tit, chaffinch, willow warbler, '
            'blackbird and collared dove the game flies about by code, perched, in five rigid parts the renderer moves '
            '(`body`, `head`, `tail`, `wing.L`, `wing.R`: the wings spread, which it flaps, and hides while perched). '
            '`bird`: length (bill to tail tip) and wingspan, the body\'s middle over the feet (`perchHeight`) and the '
            'perched torso\'s rise from level (`pitch`, degrees), in metres.',
)

def views(argv):
    """`--preview`'s other pictures: each bird perched from the side, the front and the front three-quarter, spread from above (previews/birds/<id>.<view>.png),
    and on the lawn beside a resident 1.77 m tall seen as the game's camera sees it (its 0.8 rad field of view,
    38 degrees down) from 6 and 20 m (<id>.at6m.png, <id>.at20m.png)."""
    args = argv[argv.index('--') + 1:] if '--' in argv else []
    only = set(args[args.index('--only') + 1].split(',')) if '--only' in args else None
    for model in FAMILY.models:
        if only and model.id not in only:
            continue
        FAMILY.make(model)
        stem = os.path.join(G.PREVIEWS, FAMILY.name, model.id)
        G.preview(f'{stem}.top.png', 'top')
        wings = [o for o in G.meshes() if o.name.startswith('wing.')]
        for o in wings:                                            # perched, as the game draws it: no spread wings
            o.scale = (1e-4, 1e-4, 1e-4)
        for view in ('side', 'front', 'three_quarter'):
            G.preview(f'{stem}.{view}.png', view)
        # A resident beside it (a plain figure: legs, body, head), on a patch of lawn.
        grey, lawn = G.mat('figure', '#8C8C96', 0.6), G.mat('lawn', '#6E9A48', 0.8)
        G.cube('lawn', (8, 8, 0.01), at=(0, 0, -0.005), material=lawn)
        G.cylinder('figure_legs', 0.13, 0.85, at=(-0.6, 0.2, 0.425), material=grey, verts=12)
        G.cylinder('figure_body', 0.19, 0.62, at=(-0.6, 0.2, 1.16), material=grey, verts=12)
        G.sphere('figure_head', 0.12, at=(-0.6, 0.2, 1.65), material=grey, segments=12, rings=8)
        scene = bpy.context.scene
        scene.render.resolution_x, scene.render.resolution_y = 960, 540
        for d in (6, 20):
            cam = bpy.data.cameras.new('game')
            cam.sensor_fit = 'VERTICAL'
            cam.angle_y = 0.8
            obj = bpy.data.objects.new('game', cam)
            bpy.context.collection.objects.link(obj)
            target = Vector((-0.3, 0.1, 0.4))
            e = math.radians(38)
            obj.location = target + Vector((d * math.cos(e) * 0.5, -d * math.cos(e) * 0.87, d * math.sin(e)))
            obj.rotation_euler = (target - obj.location).to_track_quat('-Z', 'Y').to_euler()
            scene.camera = obj
            scene.render.filepath = f'{stem}.at{d}m.png'
            bpy.ops.render.render(write_still=True)
            bpy.data.objects.remove(obj, do_unlink=True)
            bpy.data.cameras.remove(cam)
        scene.render.resolution_x = scene.render.resolution_y = 640
        print(f'[gamekit] views of {model.id} -> {stem}.side, .front, .three_quarter (perched), .top (spread), .at6m, .at20m .png')


if __name__ == '__main__':
    FAMILY.run(sys.argv)
    if '--preview' in sys.argv:
        views(sys.argv)
