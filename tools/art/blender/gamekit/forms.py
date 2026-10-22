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

from .shapes import _recalc_normals

SHARP = 40  # degrees: an edge turning more than this stays sharp on a smooth form


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


def sweep(name, points, radius=0.02, profile=None, material=None, verts=12, scale=1.0, resolution=1, closed=False,
          caps=True, up=(0, 0, 1), smooth=True, angle=SHARP):
    """
    A profile swept along a path through `points` (Blender, metres): a tube of `radius` (`verts` round), or any
    closed `profile` [(u, v), ...] about the path, u along the frame's normal (as near `up` as it goes), v across.
    `resolution` > 1 curves the path through the points (Catmull-Rom), that many steps between each pair.
    `scale` thickens it along the way: a number, a list (one per point, linear between) or a function of how far
    along (0..1), so a branch or a cabriole leg tapers. `closed` joins the path's ends (a ring, piping round a
    cushion); otherwise the ends are capped flat (`caps`).
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
    return _object(name, vs, faces, material, smooth, angle)


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
    _shade(obj, smooth, angle)
    if bevel:
        mod = obj.modifiers.new('bevel', 'BEVEL')
        mod.width = bevel
        mod.segments = bevel_segments
        mod.limit_method = 'ANGLE'
        mod.angle_limit = math.radians(30)
        mod.harden_normals = False
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


# ---- soft things -----------------------------------------------------------------------------------------

def _steps(h, r, round_segments, flat):
    """Positions from -h to h: `round_segments` closing in on each end over the rounding `r` (for its curve),
    `flat` steps across the middle."""
    edge = [-h + r * (1 - math.cos(math.pi / 2 * i / round_segments)) for i in range(round_segments)]
    inner = [-(h - r) + 2 * (h - r) * i / flat for i in range(flat + 1)] if h > r + 1e-6 else [0.0]
    return edge + inner + [-x for x in reversed(edge)]


def soft_box(name, size, at=(0, 0, 0), material=None, rot=(0, 0, 0), round=0.04, puff=None, round_segments=2,
             flat=3, piping=None, piping_material=None):
    """
    A soft box `size` (x, y, z) across: its edges rounded by `round`, its faces puffed out by `puff` metres in the
    middle (a number for its two largest faces, the ones across its thinnest side; or (x, y, z), each face pair's
    own), falling to nothing at its edges, as a stuffed cushion is. `flat` is how many steps cross a puffed face (more
    for a deeper puff; a flat one is crossed in one), `round_segments` how many turn each rounded edge. `piping` (a radius) runs a cord round both seams of its thin side, in `piping_material` (its
    own material by default). For seat, back and arm cushions, pillows, mattresses, upholstered arms and backs.
    """
    hx, hy, hz = (s / 2 for s in size)
    h = (hx, hy, hz)
    r = min(round, hx, hy, hz)
    thin = min(range(3), key=lambda i: h[i])
    if puff is None:
        puffs = (0.0, 0.0, 0.0)
    elif isinstance(puff, (int, float)):
        puffs = tuple(puff if i == thin else 0.0 for i in range(3))
    else:
        puffs = tuple(puff)
    # an axis is crossed in `flat` steps only where a puffed face spans it
    steps = [_steps(h[i], r, round_segments, flat if any(puffs[a] for a in range(3) if a != i) else 1) for i in range(3)]
    inner = [h[i] - r for i in range(3)]
    index, vs, faces = {}, [], []

    def vert(p):
        key = tuple(round_(c) for c in p)
        if key not in index:
            index[key] = len(vs)
            vs.append(_shape(p, h, inner, r, puffs))
        return index[key]
    for a in range(3):                      # the face pairs across axis a
        b, c = (a + 1) % 3, (a + 2) % 3
        for side in (-1, 1):
            for i in range(len(steps[b]) - 1):
                for j in range(len(steps[c]) - 1):
                    quad = []
                    for di, dj in ((0, 0), (1, 0), (1, 1), (0, 1)):
                        p = [0.0, 0.0, 0.0]
                        p[a], p[b], p[c] = side * h[a], steps[b][i + di], steps[c][j + dj]
                        quad.append(vert(p))
                    faces.append(tuple(quad if side > 0 else reversed(quad)))
    obj = _object(name, vs, faces, material, True, 75)
    made = [obj]
    if piping:
        made += _piping(name, h, r, thin, piping, piping_material or material)
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


def _shape(p, h, inner, r, puffs):
    """A point of the box's surface moved onto the rounded, puffed one."""
    c = Vector([max(-inner[i], min(inner[i], p[i])) for i in range(3)])
    d = Vector(p) - c
    q = c + d.normalized() * r if d.length > 1e-9 else Vector(p)
    for a in range(3):
        if puffs[a]:
            b, cc = (a + 1) % 3, (a + 2) % 3
            fall = max(0.0, 1 - (q[b] / h[b]) ** 2) * max(0.0, 1 - (q[cc] / h[cc]) ** 2)
            q[a] += math.copysign(puffs[a] * fall * (abs(q[a]) / h[a]) ** 4, q[a])
    return q


def _piping(name, h, r, thin, radius, material):
    """Cords round the two seams of a soft box's thin side, where its faces turn into the boxing."""
    a, b = (thin + 1) % 3, (thin + 2) % 3
    k = r * (1 - math.sqrt(0.5))
    loop = rounded_rect(2 * (h[a] - k), 2 * (h[b] - k), r * math.sqrt(0.5), corner=2)
    made = []
    for side in (-1, 1):
        pts = []
        for u, v in loop:
            p = [0.0, 0.0, 0.0]
            p[a], p[b], p[thin] = u, v, side * (h[thin] - k)
            pts.append(p)
        made.append(sweep(f'{name}_piping{side}', pts, radius, material=material, verts=5, closed=True))
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
    for v in obj.data.vertices:
        p = obj.matrix_world @ v.co
        n = vnoise.noise_vector(Vector((p.x / size + seed, p.y / size - seed, p.z / size + seed * 0.5)))
        v.co += Vector((n.x, n.y, n.z * up)) * amount
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
    _shade(obj, smooth, angle)
    return obj


def _link(name, mesh, material):
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if material:
        obj.data.materials.append(material)
    return obj


def _shade(obj, smooth, angle):
    if smooth:
        obj.data.shade_smooth()
        obj.data.set_sharp_from_angle(angle=math.radians(angle))
    else:
        obj.data.shade_flat()
