# Round-trips the exported garden GLBs back through Blender: re-imports each file, prints its
# bounds (metres, y up, front +Z) and renders a preview, so export problems show up.
# Run: Blender --background --python tools/art/blender/check_glb.py
import os
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G

HERE = os.path.dirname(os.path.abspath(__file__))
GLB = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'assets', 'models', 'garden'))
OUT = os.path.join(HERE, 'previews')
NAMES = ['gnome', 'birdbath', 'fountain', 'lantern', 'wheelbarrow', 'boulder']


def main():
    for name in NAMES:
        G.reset()
        bpy.ops.import_scene.gltf(filepath=os.path.join(GLB, name + '.glb'))
        ws = [o.matrix_world @ Vector(c) for o in bpy.context.scene.objects if o.type == 'MESH' for c in [v.co for v in o.data.vertices]]
        mn = Vector((min(v[i] for v in ws) for i in range(3)))
        mx = Vector((max(v[i] for v in ws) for i in range(3)))
        tris = 0
        for o in bpy.context.scene.objects:
            if o.type == 'MESH':
                o.data.calc_loop_triangles()
                tris += len(o.data.loop_triangles)
        mats = sorted({s.material.name for o in bpy.context.scene.objects if o.type == 'MESH' for s in o.material_slots if s.material})
        print(f'{name:11s} size=({mx.x - mn.x:.3f},{mx.y - mn.y:.3f},{mx.z - mn.z:.3f}) '
              f'min=({mn.x:.2f},{mn.y:.2f},{mn.z:.2f}) tris={tris} mats={mats}')
        G.preview(os.path.join(OUT, 'rt_' + name + '.png'))


main()
