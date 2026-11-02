# Far versions built by hand (`Model(far=G.far_of(build))`): the model's own build run again at low detail,
# so the far version has the same shape, colours and size as the model, standing where it stands. The build
# asks `G.low(full, reduced)` for each detail (segments, rings, counts, bevels) and `G.is_low()` for what
# to leave out (a flower's centre, the moving parts kept apart: a far version is all one still part).
#
# Better than `far=0.25` (decimating to a quarter) for small, separate parts (flowers, leaves, lumps, a bale's
# twine): decimating shrinks thin parts into spikes and pulls the lowest vertices off the floor. Draw every
# random number the full build draws, in the same order, so the far version's plants stand where the
# model's do (`low` chooses only how finely each is built).

# Whether a far version is being built: a list, so a family that imports it shares the one flag.
LOW = [False]


def low(full, reduced):
    """`full` detail, or `reduced` while a far version is being built."""
    return reduced if LOW[0] else full


def is_low():
    """Whether a far version is being built (`far_of`)."""
    return LOW[0]


def far_of(build):
    """A model's far version (`Model(far=G.far_of(build))`): the same build at low detail (`low`)."""
    def far():
        LOW[0] = True
        try:
            return build()
        finally:
            LOW[0] = False
    far.__name__ = f'{getattr(build, "__name__", "build")}_far'
    return far


def decimated(obj, ratio, feet=0.01, edge=0.003):
    """Decimates `obj` (a far version, joined, at rest) to `ratio` of its triangles, in place, and applies it:
    - the vertices sitting on one another merged first (a can's turned ends meeting on its axis in many: decimated
      unmerged, they tore a drinks machine's cabinet into spikes 26 cm below the floor);
    - its feet (within `feet` of its bottom) and its outermost faces (within `edge` of its box) kept whole, so it
      stands and measures as the model does (decimating them lifted a tractor's wheels 1.7 cm off the ground);
    - each face the collapse turned round turned back to face the way the model's surface there does, so it isn't
      lit from behind (the gate's "normals turned against their winding").
    Returns `obj`."""
    import bmesh
    import bpy
    from mathutils.bvhtree import BVHTree
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    bm.to_mesh(obj.data)
    bm.free()
    mesh = obj.data
    reference = BVHTree.FromPolygons([v.co.copy() for v in mesh.vertices], [list(p.vertices) for p in mesh.polygons],
                                     all_triangles=False)
    normals = [p.normal.copy() for p in mesh.polygons]
    lo = [min(v.co[i] for v in mesh.vertices) for i in range(3)]
    hi = [max(v.co[i] for v in mesh.vertices) for i in range(3)]

    def free(v):
        return v.co.z > lo[2] + feet and all(lo[i] + edge < v.co[i] < hi[i] - edge for i in range(3))
    keep = obj.vertex_groups.new(name='decimate')
    keep.add([v.index for v in mesh.vertices if free(v)], 1.0, 'REPLACE')
    mod = obj.modifiers.new('far', 'DECIMATE')
    mod.ratio = ratio
    mod.vertex_group = 'decimate'
    depsgraph = bpy.context.evaluated_depsgraph_get()
    done = bpy.data.meshes.new_from_object(obj.evaluated_get(depsgraph))
    obj.modifiers.remove(mod)
    obj.vertex_groups.remove(keep)
    old, obj.data = obj.data, done
    bpy.data.meshes.remove(old)
    bm = bmesh.new()
    bm.from_mesh(done)
    turned = []
    for f in bm.faces:
        if f.calc_area() < 1e-8:
            continue
        hit = reference.find_nearest(f.calc_center_median())
        if hit[2] is not None and f.normal.dot(normals[hit[2]]) < 0:
            turned.append(f)
    if turned:
        bmesh.ops.reverse_faces(bm, faces=turned)
    bm.to_mesh(done)
    bm.free()
    print(f'[gamekit] {obj.name}: {len(done.polygons)} faces, {len(turned)} turned back')
    return obj
