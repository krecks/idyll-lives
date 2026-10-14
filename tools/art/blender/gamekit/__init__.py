# gamekit: the toolkit the Blender-built models are made with (tools/art/blender/README.md; the standard
# is docs/design/objects.md). A family script does `import gamekit as G` and ends with `FAMILY.run(argv)`.
#
# Conventions, matching the game: 1 unit = 1 metre, origin at the footprint centre on the floor (z = 0),
# front facing -Y in Blender. The glTF exporter's +Y up turns Blender -Y into the game's +Z front, and
# Blender +X into the game's -X: Blender (x, y, z) is the game's (-x, z, -y). `GameFrame` converts.
from .export import export_glb, materials_used, stats, write_pack
from .family import Family, Model
from .materials import FINISHES, Palette, mat
from .motion import clips, grow, slide, swing
from .parts import check_size, fit, game_size, hinged, join, join_all, measure, new_parts, set_origin, settle, turn
from .place import GameFrame
from .preview import preview, preview_clips
from .scene import ASSETS, PREVIEWS, TAU, cosd, meshes, reset, sind
from .shapes import cone, cube, cylinder, revolve, rock, rod, sphere, torus

__all__ = [
    'ASSETS', 'FINISHES', 'PREVIEWS', 'TAU', 'Family', 'GameFrame', 'Model', 'Palette', 'check_size', 'clips',
    'cone', 'cosd', 'cube', 'cylinder', 'export_glb', 'fit', 'game_size', 'grow', 'hinged', 'join', 'join_all', 'mat',
    'materials_used', 'measure', 'meshes', 'new_parts', 'preview', 'preview_clips', 'reset', 'revolve', 'rock',
    'rod', 'set_origin', 'settle', 'sind', 'slide', 'sphere', 'stats', 'swing', 'torus', 'turn', 'write_pack',
]
