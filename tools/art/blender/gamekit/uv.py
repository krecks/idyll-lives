# UVs for the textured finishes (wood, fabric, bark, foliage), in metres, made for each part in its own frame before
# it is joined (`parts.join`), so a texture follows the part: the wood's grain (up its picture, along v) runs along a
# leg, a rail or a table's length, not along whichever axis of the model each face happens to face; bark runs up a
# trunk and round it; a clump's leaves aren't smeared where its faces turn from one axis to the next. The game tiles
# them by the finish's `uvScale` (as it did the box projection it makes for files without them).
#
# A part's way is how it was made (`obj['uv']`): a box per face along its own longest sides (the default), round about
# its own z (cylinders, cones, lathes: `ROUND`), or along its path (a sweep, which writes its own: `DONE`). Parts on
# plain finishes get none of their own; joined with textured parts, theirs are zero (one UV map for the mesh).
import math

LAYER = 'UVMap'
ROUND = 'round'
DONE = 'done'


def textured(obj):
    """Whether any of `obj`'s materials is on a textured finish."""
    from .materials import TEXTURES
    return any(s.material and s.material.get('finish') in TEXTURES for s in obj.material_slots)


def _layer(mesh):
    layer = mesh.uv_layers.get(LAYER)
    if layer is None:
        layer = mesh.uv_layers[0] if len(mesh.uv_layers) else mesh.uv_layers.new(name=LAYER)
        layer.name = LAYER
    for other in [u for u in mesh.uv_layers if u.name != LAYER]:
        mesh.uv_layers.remove(other)
    return layer


def ensure(objs):
    """Gives each of `objs` (parts about to be joined, or a model's meshes as exported) its UVs, once: in metres in
    its own frame if it is textured, zero if it isn't but one of the others is; none if none of them is. Returns
    whether any is textured."""
    any_textured = any(textured(o) for o in objs if o.type == 'MESH')
    if not any_textured:
        return False
    for o in objs:
        if o.type != 'MESH' or o.data.get('uv_metres'):
            continue
        mesh = o.data
        layer = _layer(mesh)
        if textured(o):
            way = o.get('uv')
            if way == DONE:
                pass
            elif way == ROUND:
                _round(mesh, layer)
            else:
                _box(mesh, layer)
        _plain_zero(o, layer)
        mesh['uv_metres'] = True
    return True


def _plain_zero(obj, layer):
    """Zeroes the UVs of `obj`'s faces on plain finishes (a cushion's piping on a fabric one, a primitive's own
    unwrap): nothing reads them, and zeros cost next to nothing compressed."""
    from .materials import TEXTURES
    slots = [s.material and s.material.get('finish') in TEXTURES for s in obj.material_slots]
    mesh = obj.data
    for p in mesh.polygons:
        if not (p.material_index < len(slots) and slots[p.material_index]):
            for li in p.loop_indices:
                layer.data[li].uv = (0.0, 0.0)


def _box(mesh, layer):
    """Per face, the two axes across its normal's largest, v along the longer of them over the whole part (the
    grain along its length), u along the other. A face keeps one projection for all its corners: no smear."""
    co = [v.co for v in mesh.vertices]
    if not co:
        return
    ext = [max(c[i] for c in co) - min(c[i] for c in co) for i in range(3)]
    longest = sorted(range(3), key=lambda i: -ext[i])
    for p in mesh.polygons:
        d = max(range(3), key=lambda i: abs(p.normal[i]))
        v_axis, u_axis = [i for i in longest if i != d]
        for li in p.loop_indices:
            c = co[mesh.loops[li].vertex_index]
            layer.data[li].uv = (c[u_axis], c[v_axis])


def _round(mesh, layer):
    """About its own z: round its sides u is the arc (metres at its mean radius), v the height; its caps (faces
    looking along z) flat. Each face is unwrapped where it crosses the seam behind."""
    co = [v.co for v in mesh.vertices]
    if not co:
        return
    cx = (max(c.x for c in co) + min(c.x for c in co)) / 2
    cy = (max(c.y for c in co) + min(c.y for c in co)) / 2
    radii = [math.hypot(c.x - cx, c.y - cy) for c in co]
    r = max(1e-4, sum(radii) / len(radii))
    for p in mesh.polygons:
        loops = list(p.loop_indices)
        corners = [co[mesh.loops[li].vertex_index] for li in loops]
        if abs(p.normal.z) >= 0.7:
            for li, c in zip(loops, corners):
                layer.data[li].uv = (c.x, c.y)
            continue
        angles = [math.atan2(c.y - cy, c.x - cx) for c in corners]
        if max(angles) - min(angles) > math.pi:
            angles = [a + math.tau if a < 0 else a for a in angles]
        for li, c, a in zip(loops, corners, angles):
            layer.data[li].uv = (a * r, c.z)


def along(obj, rings, profile, scales, points, closed):
    """A sweep's UVs (`forms.sweep`): u round its profile (metres, times the ring's scale), v along its path (metres),
    its caps flat in the profile's own plane. `rings` rings of `profile` (2D), at `points`."""
    mesh = obj.data
    layer = _layer(mesh)
    m = len(profile)
    round_ = [0.0]
    for k in range(1, m + 1):
        a, b = profile[k - 1], profile[k % m]
        round_.append(round_[-1] + math.hypot(b[0] - a[0], b[1] - a[1]))
    path = [0.0]
    for i in range(1, rings + 1 if closed else rings):
        a, b = points[i - 1], points[i % rings]
        path.append(path[-1] + (b - a).length)
    for p in mesh.polygons:
        loops = list(p.loop_indices)
        verts = [mesh.loops[li].vertex_index for li in loops]
        if len({v // m for v in verts}) == 1:
            for li, v in zip(loops, verts):        # a cap: flat in the profile's plane
                k, i = v % m, min(v // m, rings - 1)
                layer.data[li].uv = (profile[k][0] * scales[i], profile[k][1] * scales[i])
            continue
        ks = [v % m for v in verts]
        iss = [v // m for v in verts]
        wrap_k = max(ks) == m - 1 and min(ks) == 0
        wrap_i = closed and max(iss) == rings - 1 and min(iss) == 0
        for li, k, i in zip(loops, ks, iss):
            kk = m if wrap_k and k == 0 else k
            ii = rings if wrap_i and i == 0 else i
            layer.data[li].uv = (round_[kk] * scales[i % rings], path[ii])
    obj['uv'] = DONE
    return obj
