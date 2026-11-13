# Forms: the shapes the primitives can't make, each one mesh object (Blender axes, metres), deterministic, with its
# material, as shapes.py's builders are. A profile swept along a path (`sweep`: rails, piping, bent tube, a
# cabriole leg, a branch), rings lofted into a body (`loft`, `ring`: a bottle, a lamp base, a rounded cabinet),
# an outline with holes extruded into a slab (`extrude`, `rounded_rect`, `smooth_outline`: a shaped apron, a
# headboard, a door with a window), a soft box (`soft_box`: cushions, mattresses, upholstery, with piping), and
# what is done to a part after (`mirrored`, `cut`, `jitter`).
#
# Smooth forms are shaded smooth with their edges over `angle` degrees kept sharp, so a cap or a square
# profile stays crisp without a bevel.
import math

import bmesh
import bpy
from mathutils import Matrix, Vector
from mathutils import noise as vnoise

from . import uv
from .far import is_low
from .shapes import SHARP, _recalc_normals, bevel_edges, round_angle, shade, sides


# ---- profiles and outlines (2D) ------------------------------------------------------------------------

def circle(r, verts=12):
    """A round profile of radius `r`, `verts` points."""
    return [(r * math.cos(math.tau * i / verts), r * math.sin(math.tau * i / verts)) for i in range(verts)]


def rounded_rect(w, h, r, corner=4):
    """A w × h rectangle with corners of radius `r` (`corner` segments each), centred, anticlockwise."""
    r = min(r, w / 2, h / 2)
    out = []
    for cx, cy, a0 in ((w / 2 - r, h / 2 - r, 0), (-w / 2 + r, h / 2 - r, 90), (-w / 2 + r, -h / 2 + r, 180),
                       (w / 2 - r, -h / 2 + r, 270)):
        for i in range(corner + 1):
            a = math.radians(a0 + 90 * i / corner)
            out.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return _dedupe(out)


def smooth_outline(points, passes=2, closed=True):
    """Chaikin's corner cutting: each pass replaces every corner by two points a quarter in, rounding it."""
    pts = [tuple(p) for p in points]
    for _ in range(passes):
        out = [] if closed else [pts[0]]
        n = len(pts)
        for i in range(n if closed else n - 1):
            a, b = Vector(pts[i]), Vector(pts[(i + 1) % n])
            out += [tuple(a * 0.75 + b * 0.25), tuple(a * 0.25 + b * 0.75)]
        if not closed:
            out.append(pts[-1])
        pts = out
    return pts


def _inside(p, loop):
    """Whether 2D point `p` is inside the closed `loop` (even-odd)."""
    x, y = p[0], p[1]
    inside = False
    for (x0, y0), (x1, y1) in zip(loop, loop[1:] + loop[:1]):
        if (y0 > y) != (y1 > y) and x < x0 + (y - y0) * (x1 - x0) / (y1 - y0):
            inside = not inside
    return inside


def _dedupe(pts, eps=1e-7):
    out = []
    for p in pts:
        if not out or (Vector(p) - Vector(out[-1])).length > eps:
            out.append(p)
    if len(out) > 2 and (Vector(out[0]) - Vector(out[-1])).length <= eps:
        out.pop()
    return out


# ---- paths -----------------------------------------------------------------------------------------------

def _catmull(points, resolution, closed):
    """The points, with `resolution` - 1 more on a Catmull-Rom curve between each pair (1: as they are). Returns
    (points, the fraction of the way along the given points at each, 0..1)."""
    pts = [Vector(p) for p in points]
    n = len(pts)
    spans = n if closed else n - 1
    if resolution <= 1 or n < 3:
        return pts, [i / spans for i in range(n)]

    def at(i):
        return pts[i % n] if closed else pts[max(0, min(n - 1, i))]
    out, where = [], []
    for i in range(spans):
        p0, p1, p2, p3 = at(i - 1), at(i), at(i + 1), at(i + 2)
        if not closed and i == 0:
            p0 = p1 * 2 - p2
        if not closed and i == spans - 1:
            p3 = p2 * 2 - p1
        for k in range(resolution):
            t = k / resolution
            t2, t3 = t * t, t * t * t
            out.append(0.5 * ((2 * p1) + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2 + (-p0 + 3 * p1 - 3 * p2 + p3) * t3))
            where.append((i + t) / spans)
    if not closed:
        out.append(pts[-1])
        where.append(1.0)
    return out, where


def _frames(pts, closed, up):
    """A (tangent, normal, binormal) frame at each point, carried along without twisting (parallel transport),
    the first normal as near `up` as the tangent allows."""
    n = len(pts)
    tangents = []
    for i in range(n):
        a = pts[(i - 1) % n] if closed or i > 0 else pts[i]
        b = pts[(i + 1) % n] if closed or i < n - 1 else pts[i]
        t = b - a
        tangents.append(t.normalized() if t.length > 1e-9 else Vector((0, 0, 1)))
    up = Vector(up)
    normal = up - tangents[0] * up.dot(tangents[0])
    if normal.length < 1e-6:
        normal = tangents[0].orthogonal()
    normal.normalize()
    frames = []
    for i, t in enumerate(tangents):
        if i:
            normal = tangents[i - 1].rotation_difference(t) @ normal
            normal = (normal - t * normal.dot(t)).normalized()
        frames.append((t, normal, t.cross(normal)))
    return frames


def sweep(name, points, radius=0.02, profile=None, material=None, verts=None, scale=1.0, resolution=1, closed=False,
          caps=True, up=(0, 0, 1), smooth=True, angle=None, least=12):
    """
    A profile swept along a path through `points` (Blender, metres): a tube of `radius` (`verts` round: by default
    `least`, 12, and more for one over 3 cm in radius at its thickest, as many as keep its outline within a
    millimetre of the circle, `shapes.sides`), or any
    closed `profile` [(u, v), ...] about the path, u along the frame's normal (as near `up` as it goes), v across.
    `resolution` > 1 curves the path through the points (Catmull-Rom), that many steps between each pair.
    `scale` thickens it along the way: a number, a list (one per point, linear between) or a function of how far
    along (0..1), so a branch or a cabriole leg tapers. `closed` joins the path's ends (a ring, piping round a
    cushion); otherwise the ends are capped flat (`caps`). Smooth, edges turning more than `angle` degrees stay
    sharp: by default `SHARP` for a `profile`, and for a round tube enough that its sides stay round (`round_angle`).
    """
    pts, where = _catmull(points, resolution, closed)
    if callable(scale):
        scales = [scale(w) for w in where]
    elif isinstance(scale, (list, tuple)):
        last = len(scale) - 1
        scales = []
        for w in where:
            f = w * last
            i = min(int(f), last - 1) if last else 0
            scales.append(scale[i] + (scale[min(i + 1, last)] - scale[i]) * (f - i) if last else scale[0])
    else:
        scales = [scale] * len(pts)
    if verts is None and not profile:
        verts = sides(radius * max(abs(k) for k in scales), least)
    prof = profile or circle(radius, verts)
    frames = _frames(pts, closed, up)
    vs, faces = [], []
    m = len(prof)
    for p, (_, nrm, bi), s in zip(pts, frames, scales):
        for u, v in prof:
            vs.append(p + (nrm * u + bi * v) * s)
    rings = len(pts)
    for i in range(rings if closed else rings - 1):
        a, b = i * m, ((i + 1) % rings) * m
        for k in range(m):
            k2 = (k + 1) % m
            faces.append((a + k, a + k2, b + k2, b + k))
    if caps and not closed:
        faces += [tuple(range(m - 1, -1, -1)), tuple(range((rings - 1) * m, rings * m))]
    if angle is None:
        angle = SHARP if profile else round_angle(verts)
    obj = _object(name, vs, faces, material, smooth, angle)
    return uv.along(obj, rings, prof, scales, pts, closed)


# ---- lofts -----------------------------------------------------------------------------------------------

def ring(rx, ry, z=0.0, verts=24, square=2.0, at=(0.0, 0.0)):
    """A loop of `verts` points round (at[0], at[1]) at height `z`: an ellipse of radii rx, ry (`square` 2), squarer
    towards a rounded rectangle as `square` grows (4: a soft square, 10: nearly sharp corners)."""
    out = []
    e = 2.0 / square
    for i in range(verts):
        a = math.tau * i / verts
        c, s = math.cos(a), math.sin(a)
        out.append((at[0] + rx * math.copysign(abs(c) ** e, c), at[1] + ry * math.copysign(abs(s) ** e, s), z))
    return out


def loft(name, sections, material=None, caps=True, smooth=True, angle=SHARP, at=(0, 0, 0), rot=(0, 0, 0)):
    """A body through `sections`, each a closed loop of Blender points with the same count (`ring`), bottom
    first: a bottle, a vase, a lamp's base, a rounded fridge. The first and last loops are capped flat (`caps`);
    a loop shrunk to a point closes the body there instead."""
    m = len(sections[0])
    if any(len(s) != m for s in sections):
        raise ValueError(f'{name}: every section needs the same number of points ({m})')
    vs = [Vector(p) for s in sections for p in s]
    faces = []
    for i in range(len(sections) - 1):
        a, b = i * m, (i + 1) * m
        for k in range(m):
            k2 = (k + 1) % m
            faces.append((a + k, a + k2, b + k2, b + k))
    if caps:
        faces += [tuple(range(m - 1, -1, -1)), tuple(range((len(sections) - 1) * m, len(sections) * m))]
    obj = _object(name, vs, faces, material, smooth, angle, merge=True)
    if all(max(p[2] for p in sec) - min(p[2] for p in sec) < 1e-6 for sec in sections):
        obj['uv'] = uv.ROUND   # level rings up its z (a drum, a vase): textured round it (gamekit/uv.py)
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


# ---- slabs -----------------------------------------------------------------------------------------------

# Where an outline is drawn: (Blender axis of its u, of its v, of the thickness).
PLANES = {'XZ': (0, 2, 1), 'XY': (0, 1, 2), 'YZ': (1, 2, 0)}


def extrude(name, outline, depth, at=(0, 0, 0), material=None, holes=(), plane='XZ', rot=(0, 0, 0), bevel=0.0,
            bevel_segments=2, smooth=False, angle=SHARP):
    """
    A slab `depth` thick from a closed 2D `outline` [(u, v), ...] and any `holes` (outlines inside it): the
    outline's (0, 0) goes on `at`, and the slab is centred on it through its thickness. `plane` is where the outline is drawn: 'XZ' faces the front (u across, v up, thick front to back: an apron,
    a headboard, a chair's back), 'XY' lies flat (thick upwards: a shaped top, a tray), 'YZ' faces the side (u
    back to front, v up: a chair's side, a rocker). `bevel` rounds its edges, as the primitives' do.
    """
    bm = bmesh.new()
    edges = []
    for loop in [outline, *holes]:
        vs = [bm.verts.new((u, v, 0.0)) for u, v in _dedupe(loop)]
        edges += [bm.edges.new((vs[i], vs[(i + 1) % len(vs)])) for i in range(len(vs))]
    bmesh.ops.triangle_fill(bm, use_beauty=True, use_dissolve=False, edges=edges, normal=(0, 0, 1))
    # the fill covers the holes too: take out what lies inside one
    filled = [f for f in bm.faces if any(_inside(f.calc_center_median(), h) for h in holes)]
    bmesh.ops.delete(bm, geom=filled, context='FACES_ONLY')
    faces = [f for f in bm.faces]
    for f in faces:
        if f.normal.z < 0:
            f.normal_flip()
    ext = bmesh.ops.extrude_face_region(bm, geom=faces)
    moved = [g for g in ext['geom'] if isinstance(g, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, vec=(0, 0, depth), verts=moved)
    bmesh.ops.translate(bm, vec=(0, 0, -depth / 2), verts=bm.verts[:])
    iu, iv, iw = PLANES[plane]
    for v in bm.verts:
        p = [0.0, 0.0, 0.0]
        p[iu], p[iv], p[iw] = v.co.x, v.co.y, v.co.z
        v.co = p
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = _link(name, mesh, material)
    _recalc_normals(obj)
    shade(obj, smooth, angle)
    if bevel:
        bevel_edges(obj, bevel, bevel_segments, angle=30)
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


# ---- soft things -----------------------------------------------------------------------------------------

SOFT_FACET = 0.0015  # metres: how far a soft thing's rounded edges may stand in from their curve (`cargo review facets`)
CROWN_FACET = 0.002  # and its crowned faces from theirs: a face's middle isn't an outline, so it may be coarser


def _steps(h, r, round_segments, flat):
    """Positions from -h to h: `round_segments` closing in on each end over the rounding `r`, placed so the
    rounding turns by the same angle from each to the next (its eighth of a turn on this face in equal parts),
    `flat` steps across the middle."""
    edge = [-(h - r) - r * math.tan(math.pi / 4 * (1 - i / round_segments)) for i in range(round_segments)]
    inner = [-(h - r) + 2 * (h - r) * i / flat for i in range(flat + 1)] if h > r + 1e-6 else [0.0]
    return edge + inner + [-x for x in reversed(edge)]


def soft_box(name, size, at=(0, 0, 0), material=None, rot=(0, 0, 0), round=0.04, puff=None, round_segments=None,
             flat=None, piping=None, piping_material=None, pinch=None, waist=0.0, sag=0.0, overhang=0.0, tufts=None,
             tuft=0.012, diamond=False, button=None, rumple=0.0, rumple_size=0.24, seed=0.0, calm=None,
             resting=False):
    """
    A soft box `size` (x, y, z) across: its edges rounded by `round`, its faces puffed out by `puff` metres in the
    middle (a number for its two largest faces, the ones across its thinnest side; or (x, y, z), each face pair's
    own), falling to nothing at its edges, as a stuffed cushion is. For seat, back and arm cushions, pillows,
    mattresses, duvets, upholstered arms and backs. What stuffing and seams do to it, each a few centimetres at most
    and none of it past the box (`size` stays its size):
    - `pinch`: its corners pulled in and thinned where the seams meet, so its sides bow out between them, as a
      boxed cushion's do (metres at the corner; by default 0.6 of its deepest puff, 0 for none and for a box whose
      sides are crossed in one step). Its corners are then inside the box: give 0 to one turned so that a corner
      is the model's outermost point (a folded towel askew on a stack), or the model is that much smaller;
    - `waist`: its sides drawn in between its corners, which stand out as ears, as a pillow sewn flat and stuffed
      does (metres at the middle of a side);
    - `sag`: its top (+z) sinking over its front third to this much lower at its front edge (-y), as a seat
      cushion's does where legs go over it (or a vector (x, y, z) towards the edge it sinks to, its length how far);
      its middle and back, where the sitter's hips rest, keep their height, and its back corners are pinched a
      third as much as its front ones (they are held under the back cushions);
    - `overhang`: its sides drawn in by this much between its two seams, so the top and the bottom stand over them;
    - `tufts`: (across, along) rows of dents `tuft` deep in the face across its thin side that looks up or to the
      front (both faces of one thin across x), in a square grid or staggered (`diamond`), each with a button in
      the material `button`;
    - `rumple`: its top sunk by up to this much in soft folds about `rumple_size` apart, lying askew, and what
      hangs down its sides drawn in in flutes, deeper towards the hem, which is square, not rounded under
      (`seed`: which folds), as bedding shaken out lies; only downwards and inwards; `calm` (from, to) along its
      own y: where something lies across it (a runner) and it is smooth;
    - `piping` (a radius): a cord along both seams of its thin side, following them, in `piping_material` (its own
      material by default).
    `flat` is how many steps cross a puffed face and `round_segments` how many turn each rounded edge on each of
    the two faces it joins: by default as many as keep the rounded edges within 1.5 mm of their curve and the
    crowned faces within 2 mm of theirs (and what tufts and a rumple need). `resting`: it lies on its underside (-z) or, thin from front to back, leans on its back
    (+y), which nobody sees: that face's middle is one flat piece, for a third fewer triangles.
    """
    hx, hy, hz = (s / 2 for s in size)
    h = (hx, hy, hz)
    r = min(round, hx, hy, hz)
    thin = min(range(3), key=lambda i: h[i])
    plan = [i for i in range(3) if i != thin]
    if puff is None:
        puffs = (0.0, 0.0, 0.0)
    elif isinstance(puff, (int, float)):
        puffs = tuple(puff if i == thin else 0.0 for i in range(3))
    else:
        puffs = tuple(puff)
    low = is_low()
    own_pinch = pinch is not None
    pinch = min(pinch if own_pinch else 0.6 * max(puffs), 0.45 * min(h[i] for i in plan))
    if isinstance(sag, (int, float)):
        sag = (0.0, -sag, 0.0)
    sag = Vector(sag)
    if low:
        tufts, rumple = None, 0.0
    if round_segments is None:
        # the rounding's quarter turn in 2 x round_segments equal parts: each stands r (1 - cos(pi / 8n)) in
        round_segments = 1 if r < 0.008 or low else 2 if r * (1 - math.cos(math.pi / 16)) <= SOFT_FACET else 3
    inner = [h[i] - r for i in range(3)]
    count = []
    for i in range(3):   # how many steps cross each axis between its rounded ends
        across = max(puffs[a] for a in range(3) if a != i)   # the deepest puff on a face this axis crosses
        n = 1
        if across:
            # a crown `p` deep in n steps stands p / n² in from its curve
            n = flat if flat is not None else 2 if low else max(2, min(5, math.ceil(math.sqrt(across / CROWN_FACET))))
        if flat is None and not low and sag.length and abs(sag[i]) > 1e-9:
            n = max(n, 3)
        if overhang and i == thin:
            n = max(n, 2)
        if rumple and i != thin:
            n = max(n, math.ceil(2 * inner[i] / (rumple_size / 2.5)))
        if tufts and i != thin:
            k = 4 * tufts[plan.index(i)]     # a vertex on every tuft and three between two: a dimple, not a pyramid
            n = k * math.ceil(n / k)
        count.append(n)
    if not own_pinch and min(count[i] for i in plan) < 2:
        pinch = 0.0   # a side crossed in one step has no middle to stay put: pinching its corners would narrow it
    steps = [_steps(h[i], r, round_segments, count[i]) for i in range(3)]
    shape = _Soft(h, inner, r, puffs, thin, plan, pinch, waist, sag, overhang, tufts, tuft, diamond, rumple, rumple_size,
                  seed, calm)
    index, vs, faces = {}, [], []

    def vert(p):
        key = tuple(round_(c) for c in p)
        if key not in index:
            index[key] = len(vs)
            vs.append(shape(p))
        return index[key]
    hidden = {2: -1, 1: 1}.get(thin) if resting else None   # the side of the thin axis it rests on
    for a in range(3):                      # the face pairs across axis a
        b, c = (a + 1) % 3, (a + 2) % 3
        for side in (-1, 1):
            plain = a == thin and side == hidden and count[b] > 1 and count[c] > 1
            if plain:                       # the middle of the face it rests on, in one piece
                i0, i1, j0, j1 = round_segments, round_segments + count[b], round_segments, round_segments + count[c]
                ring = ([(i, j0) for i in range(i0, i1)] + [(i1, j) for j in range(j0, j1)]
                        + [(i, j1) for i in range(i1, i0, -1)] + [(i0, j) for j in range(j1, j0, -1)])
                loop = []
                for i, j in ring:
                    p = [0.0, 0.0, 0.0]
                    p[a], p[b], p[c] = side * h[a], steps[b][i], steps[c][j]
                    loop.append(vert(p))
                faces.append(tuple(loop if side > 0 else reversed(loop)))
            for i in range(len(steps[b]) - 1):
                for j in range(len(steps[c]) - 1):
                    if plain and i0 <= i < i1 and j0 <= j < j1:
                        continue
                    quad = []
                    for di, dj in ((0, 0), (1, 0), (1, 1), (0, 1)):
                        p = [0.0, 0.0, 0.0]
                        p[a], p[b], p[c] = side * h[a], steps[b][i + di], steps[c][j + dj]
                        quad.append(vert(p))
                    faces.append(tuple(quad if side > 0 else reversed(quad)))
    obj = _object(name, vs, faces, material, True, 75)
    made = [obj]
    if piping:
        made += _piping(name, h, steps, thin, plan, shape, piping, piping_material or material,
                        [side for side in (-1, 1) if side != hidden])
    if tufts and button:
        made += _buttons(name, shape, button)
    if len(made) > 1:
        bpy.ops.object.select_all(action='DESELECT')
        for o in made:
            o.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.join()
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


def round_(c):
    return int(math.floor(c * 1e6 + 0.5))


class _Soft:
    """A soft box's shape: a point of the box's surface moved onto the rounded, puffed, pinched one (`soft_box`)."""

    def __init__(self, h, inner, r, puffs, thin, plan, pinch, waist, sag, overhang, tufts, tuft, diamond, rumple, size,
                 seed, calm=None):
        self.calm = calm
        self.h, self.inner, self.r, self.puffs, self.thin, self.plan = h, inner, r, puffs, thin, plan
        self.pinch, self.sag, self.overhang, self.rumple, self.size, self.seed = pinch, sag, overhang, rumple, size, seed
        self.waist = min(waist, 0.4 * min(h[i] for i in plan))
        self.tuft = tuft
        # where the seams run: half way round the rounding of the thin side's edges
        self.seam = max(1e-6, h[thin] - r * (1 - math.sqrt(0.5)))
        # the tufted faces: the one that looks up (thin across z) or to the front (-y); both of one thin across x
        self.sides = {2: (1,), 1: (-1,), 0: (-1, 1)}[thin]
        self.tufts = []
        if tufts:
            b, c = plan
            nu, nv = tufts[0], tufts[1]
            pu, pv = 2 * inner[b] / nu, 2 * inner[c] / nv
            for j in range(nv):
                odd = diamond and j % 2 == 1
                for i in range(nu - 1 if odd else nu):
                    self.tufts.append((-inner[b] + (i + (1.0 if odd else 0.5)) * pu, -inner[c] + (j + 0.5) * pv))
            self.reach = 0.22 * min(pu, pv)

    def __call__(self, p):
        h, r, a = self.h, self.r, self.thin
        b, c = self.plan
        centre = Vector([max(-self.inner[i], min(self.inner[i], p[i])) for i in range(3)])
        d = Vector(p) - centre
        base = centre + d.normalized() * r if d.length > 1e-9 else Vector(p)
        if self.rumple and p[a] < -self.inner[a]:
            # a cover hangs to a hem: below its sides its edge isn't rounded under, only round its corners
            across = math.hypot(d[b], d[c])
            base = Vector(p)
            if across > r:
                base[b], base[c] = centre[b] + d[b] * r / across, centre[c] + d[c] * r / across
        q = base.copy()
        for k in range(3):
            if self.puffs[k]:
                m, n = (k + 1) % 3, (k + 2) % 3
                fall = max(0.0, 1 - (base[m] / h[m]) ** 2) * max(0.0, 1 - (base[n] / h[n]) ** 2)
                q[k] += math.copysign(self.puffs[k] * fall * (abs(base[k]) / h[k]) ** 4, base[k])
        u, v, w = base[b] / h[b], base[c] / h[c], base[a] / h[a]
        face = abs(w) ** 4            # 1 on the faces across the thin side, falling away round their edges
        if self.tufts and (1 if w > 0 else -1) in self.sides:
            dent = sum(math.exp(-((base[b] - x) ** 2 + (base[c] - y) ** 2) / self.reach ** 2) for x, y in self.tufts)
            q[a] -= math.copysign(self.tuft * min(1.0, dent) * face, w)
        if self.rumple:
            s = self.size
            x, y = base[b] * 0.83 + base[c] * 0.55, base[c] * 0.83 - base[b] * 0.55   # the folds lie askew
            fold = vnoise.noise(Vector((x / s + self.seed, y / (2.4 * s) - self.seed, self.seed * 0.5)))
            fine = vnoise.noise(Vector((x / (0.5 * s) - self.seed, y / (0.8 * s) + 7.3, self.seed)))
            n = max(0.0, min(1.0, 0.5 + 1.5 * fold + 0.3 * fine))
            still = 1.0
            if self.calm:     # smooth under what lies across it, the folds coming back over 8 cm either side
                off = max(self.calm[0] - base[1], base[1] - self.calm[1], 0.0) / 0.08
                still = min(1.0, off) ** 2
            if w > 0:
                q[a] -= self.rumple * n * n * (3 - 2 * n) * w * w * still
            # what hangs down its sides, drawn in in flutes that deepen towards the hem
            out = math.hypot(d[b], d[c]) / r if r > 1e-9 else 0.0
            hang = max(0.0, min(1.0, (1 - w) / 2)) * out * out
            if hang:
                along = base[b] - base[c] if abs(d[b]) < abs(d[c]) else base[b] + base[c]   # the way along the edge
                flute = vnoise.noise(Vector((along / (0.55 * s) + 3.1 + self.seed, self.seed * 1.7, base[a] / (4 * s))))
                n2 = max(0.0, min(1.0, 0.5 + 1.6 * flute)) * (1 - abs(u * v) ** 3)   # (not at its corners)
                for k in (b, c):
                    q[k] -= d[k] / r * self.rumple * 0.6 * n2 * hang * still
        if self.sag.length and w > 0:
            to = self.sag.normalized()
            reach = sum(abs(to[i]) * h[i] for i in range(3)) or 1.0
            # (its front third: where the hips rest, and 5 cm round it, stays at its height)
            t = max(0.0, min(1.0, (0.5 + 0.5 * base.dot(to) / reach - 0.68) / 0.32))
            q[a] -= self.sag.length * t * t * w
        if self.pinch:
            k = abs(u) ** 3 * abs(v) ** 3
            if self.sag.length:   # a seat cushion's back corners, held under the back, are pinched a third as much
                to = self.sag.normalized()
                reach = sum(abs(to[i]) * h[i] for i in range(3)) or 1.0
                k *= 0.3 + 0.7 * max(0.0, min(1.0, 0.5 + 0.5 * base.dot(to) / reach))
            q[b] -= math.copysign(self.pinch * k, u)
            q[c] -= math.copysign(self.pinch * k, v)
            q[a] -= min(0.5 * self.pinch, 0.45 * h[a]) * k * w
        if self.waist:
            # (nothing where a side ends and its corner's rounding begins: its ends are the box's outermost points)
            along_b = min(1.0, abs(base[b]) / self.inner[b]) if self.inner[b] > 1e-9 else 1.0
            along_c = min(1.0, abs(base[c]) / self.inner[c]) if self.inner[c] > 1e-9 else 1.0
            q[b] -= math.copysign(self.waist * abs(u) ** 3 * (1 - along_c ** 2), u)
            q[c] -= math.copysign(self.waist * abs(v) ** 3 * (1 - along_b ** 2), v)
        if self.overhang and r > 1e-9:
            between = max(0.0, 1 - (base[a] / self.seam) ** 2)
            out = math.hypot(d[b], d[c]) / r
            for k in (b, c):
                q[k] -= d[k] / r * self.overhang * between * out
        return q


def _thinned(points, tolerance=0.0025):
    """A closed path with the points left out that lie within `tolerance` of the line between their neighbours kept
    (a straight run's, a gentle bow's): fewer rings for a cord swept along it."""
    pts = [Vector(p) for p in points]
    keep = [True] * len(pts)
    n = len(pts)
    last = 0
    for i in range(1, n):
        nxt = pts[(i + 1) % n]
        a, line = pts[last], nxt - pts[last]
        if line.length > 1e-9 and all(((pts[k] - a).cross(line)).length / line.length < tolerance for k in range(last + 1, i + 1)):
            keep[i] = False
        else:
            last = i
    return [p for p, k in zip(pts, keep) if k]


def _piping(name, h, steps, thin, plan, shape, radius, material, sides=(-1, 1)):
    """Cords along the seams of a soft box's thin side (`sides`: which), where its faces turn into the boxing:
    through the shape's own seam, so they follow its pinched corners and bowed sides, standing a fifth of their
    radius proud of it, so a cord cut straight across a gentle bow never sinks into the cushion. They reach as far
    out as a cord of `radius` centred on the seam would (a model whose outermost point is a welt keeps its size),
    so they are that much slimmer."""
    b, c = plan
    made = []
    radius /= 1.2
    loop = ([(x, -h[c]) for x in steps[b][:-1]] + [(h[b], y) for y in steps[c][:-1]]
            + [(x, h[c]) for x in reversed(steps[b][1:])] + [(-h[b], y) for y in reversed(steps[c][1:])])
    for side in sides:
        pts = []
        for x, y in loop:
            p = [0.0, 0.0, 0.0]
            p[b], p[c], p[thin] = x, y, side * h[thin]
            out = Vector([p[i] - max(-shape.inner[i], min(shape.inner[i], p[i])) for i in range(3)])
            pts.append(shape(p) + out.normalized() * radius * 0.2)
        # (four sides stand 1.5 mm in from a 5 mm cord's round, five from one of 8 mm)
        made.append(sweep(f'{name}_piping{side}', _thinned(pts), radius, material=material,
                          verts=4 if radius <= 0.0043 else 5, closed=True))
    return made


def _buttons(name, shape, material):
    """A covered button in each of a soft box's tufts: a low dome sitting in the dent, wider for a deeper one."""
    radius = max(0.009, min(0.014, 0.75 * shape.tuft))
    a = shape.thin
    b, c = shape.plan
    made = []
    for side in shape.sides:
        for i, (x, y) in enumerate(shape.tufts):
            p = [0.0, 0.0, 0.0]
            p[b], p[c], p[a] = x, y, side * shape.h[a]
            q = shape(p)
            q[a] += side * radius * 0.45     # proud of the dent's floor, so its rim shows
            scale = [1.0, 1.0, 1.0]
            scale[a] = 0.55
            bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=4, radius=radius)
            ball = bpy.context.object
            ball.data.transform(Matrix.Diagonal((*scale, 1.0)))
            ball.data.transform(Matrix.Translation(q))
            ball.name = f'{name}_button{side}{i}'
            if material:
                ball.data.materials.append(material)
            shade(ball, True, 180)
            made.append(ball)
    return made


# ---- after making -----------------------------------------------------------------------------------------

def mirrored(obj, axis='X', name=None):
    """A copy of `obj` mirrored across the world's plane through the origin normal to `axis` (the model's middle):
    build one side, mirror it for the other."""
    copy = obj.copy()
    copy.data = obj.data.copy()
    copy.name = name or f'{obj.name}_mirror'
    bpy.context.collection.objects.link(copy)
    flip = Matrix.Diagonal([(-1 if a == axis else 1) for a in 'XYZ'] + [1])
    copy.matrix_world = flip @ obj.matrix_world
    bpy.context.view_layer.update()
    # a mirroring matrix turns the faces inside out: bake it into the mesh and turn them back
    mw = copy.matrix_world.copy()
    copy.data.transform(mw)
    copy.matrix_world = Matrix.Identity(4)
    copy.data.flip_normals()
    return copy


def cut(obj, cutter, keep_cutter=False):
    """Cuts `cutter`'s volume out of `obj` (a boolean difference, applied): a recess, a slot, a hole through. Both
    should be closed meshes."""
    mod = obj.modifiers.new('cut', 'BOOLEAN')
    mod.operation = 'DIFFERENCE'
    mod.solver = 'EXACT'
    mod.object = cutter
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=mod.name)
    if not keep_cutter:
        bpy.data.objects.remove(cutter, do_unlink=True)
    return obj


def jitter(obj, amount, size=0.3, seed=0.0, up=1.0):
    """Moves `obj`'s vertices by smooth noise up to `amount` metres, in features about `size` across, the same every
    build for a `seed`: bark, a lumpy stone, a hand-made look. `up` scales the vertical part (0: only sideways)."""
    bpy.context.view_layer.update()   # matrix_world as placed, not as it was before (it is read below)
    for v in obj.data.vertices:
        p = obj.matrix_world @ v.co
        q = Vector((p.x / size + seed, p.y / size - seed, p.z / size + seed * 0.5))
        # three samples of `noise` far apart, not `noise_vector`, which differs from one Blender process to the next
        n = [vnoise.noise(q + Vector(o)) for o in ((0, 0, 0), (31.4, 17.7, 5.9), (-13.3, 47.1, 29.2))]
        v.co += Vector((n[0], n[1], n[2] * up)) * amount
    obj.data.update()
    return obj


# ---- objects ----------------------------------------------------------------------------------------------

def _object(name, vs, faces, material, smooth, angle, merge=False):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in vs], [], faces)
    mesh.validate()
    obj = _link(name, mesh, material)
    if merge:
        bm = bmesh.new()
        bm.from_mesh(mesh)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
        bm.to_mesh(mesh)
        bm.free()
    _recalc_normals(obj)
    shade(obj, smooth, angle)
    return obj


def _link(name, mesh, material):
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if material:
        obj.data.materials.append(material)
    return obj

