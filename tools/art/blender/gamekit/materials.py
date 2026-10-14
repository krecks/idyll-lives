# Materials. A model's colours are a `Palette`: one material per colour, named for what it is, each
# with the game finish it is drawn with (docs/design/objects.md, "Materials, finishes ...").
#
#   M = G.Palette(body=('#F3F3F1', 'gloss'), handle=('#D0D3D6', 'chrome'))
#   G.cube('door', (0.7, 0.04, 1.1), material=M.body)
#
# The game takes only a material's colour from the file; the finish decides how it shines. So the
# palette gives each material the finish's roughness and metalness (for the previews) and writes the
# pack's `materials` map from the same table: the finish is said once.
import json
import os

import bpy

from .scene import ASSETS


def _finishes():
    """{'matte': {'roughness', 'metallic'}, ...}: the base manifest's `material.finish.*` entries."""
    with open(os.path.join(ASSETS, 'manifest.json')) as f:
        entries = json.load(f)['entries']
    prefix = 'material.finish.'
    return {key[len(prefix):]: {'roughness': e.get('roughness', 0.6), 'metallic': e.get('metallic', 0.0)}
            for key, e in entries.items() if key.startswith(prefix)}


FINISHES = _finishes()

_MATERIALS = {}


def mat(name, color, roughness=0.6, metallic=0.0, alpha=1.0, emission=0.0):
    """
    A cached Principled material (prefer a `Palette`). `color` is sRGB, as the game's manifests write
    colours: a hex string or an (r, g, b) 0..1 tuple. Blender's colour sockets and glTF's colour factors
    are linear, so it is converted. One-sided: the meshes are closed (glTF `doubleSided` would draw
    every back face too, and hide faces wound the wrong way).
    """
    key = (name, color, roughness, metallic, alpha, emission)
    m = _MATERIALS.get(key)
    if m is not None:
        try:
            m.name  # removed by reset(): make it again
            return m
        except ReferenceError:
            pass
    m = bpy.data.materials.new(name)
    if m.name != name:
        raise ValueError(f'two materials named {name!r}: the name is the key the manifest maps')
    m.use_nodes = True
    m.use_backface_culling = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    rgb = linear(rgb_of(color))
    bsdf.inputs['Base Color'].default_value = (*rgb, 1.0)
    bsdf.inputs['Roughness'].default_value = roughness
    bsdf.inputs['Metallic'].default_value = metallic
    # The viewport colour too, so `preview` (Workbench) shows it.
    m.diffuse_color = (*rgb, alpha)
    m.roughness = roughness
    m.metallic = metallic
    if alpha < 1.0:
        bsdf.inputs['Alpha'].default_value = alpha
        m.blend_method = 'BLEND'
    if emission:
        for socket in ('Emission Color', 'Emission'):
            if socket in bsdf.inputs:
                bsdf.inputs[socket].default_value = (*rgb, 1.0)
                break
        if 'Emission Strength' in bsdf.inputs:
            bsdf.inputs['Emission Strength'].default_value = emission
    _MATERIALS[key] = m
    return m


class Palette:
    """
    A family's colours: `name=(sRGB hex, finish)`. `palette.name` is that Blender material, made on
    first use after each `reset()`; `palette.finishes(names)` is the pack entry's `materials` map.
    The finish is one of the base manifest's `material.finish.*` (matte, satin, gloss, metal, chrome,
    skin, hair, fabric, wood, bark, foliage, glass). A third says more of it (the pack entry's
    material fields):
      mirror glass, the side its face reflects to: `glass=('#DCE6EA', 'glass', 'front')` (or 'back'),
        on a flat part;
      a lamp's shade or a screen that glows at night, in a colour: `shade=('#F2E6CC', 'fabric',
        {'glow': '#FFD9A0'})`.
    """

    def __init__(self, **colours):
        for name, (color, finish, *more) in colours.items():
            if finish not in FINISHES:
                raise ValueError(f'{name}: no finish {finish!r} (one of {", ".join(sorted(FINISHES))})')
            extra = _extra(name, more)
            if 'glow' in extra:
                rgb_of(extra['glow'])
            rgb_of(color)
        self._colours = colours

    def __getattr__(self, name):
        colours = self.__dict__.get('_colours', {})
        if name not in colours:
            raise AttributeError(f'no colour {name!r} in the palette')
        color, finish = colours[name][:2]
        return mat(name, color, **FINISHES[finish])

    def finish(self, name):
        return f'material.finish.{self._colours[name][1]}'

    def finishes(self, names):
        """{material: {'finish': ...}} for the materials a model uses (the pack entry's `materials`)."""
        unknown = sorted(set(names) - set(self._colours))
        if unknown:
            raise ValueError(f'materials not in the palette: {", ".join(unknown)}')
        return {name: {'finish': self.finish(name), **_extra(name, self._colours[name][2:])} for name in sorted(names)}


def _extra(name, more):
    """A palette colour's third element as the material's extra pack fields: 'front' / 'back' (a
    mirror's side), or {'mirror': ..., 'glow': '#rrggbb'}."""
    if not more:
        return {}
    extra = {'mirror': more[0]} if isinstance(more[0], str) else dict(more[0])
    unknown = set(extra) - {'mirror', 'glow'}
    if len(more) > 1 or unknown:
        raise ValueError(f'{name}: after the finish, a mirror\'s side or {{"mirror", "glow"}}, not {more}')
    if extra.get('mirror') not in (None, 'front', 'back'):
        raise ValueError(f'{name}: a mirror reflects to the front or the back, not {extra["mirror"]!r}')
    return extra


def rgb_of(color):
    """An sRGB hex string or (r, g, b) as a 0..1 tuple."""
    if isinstance(color, str):
        c = color.lstrip('#')
        if len(c) != 6:
            raise ValueError(f'colour {color!r}: want #rrggbb')
        return tuple(int(c[i:i + 2], 16) / 255 for i in (0, 2, 4))
    return tuple(color)


def linear(rgb):
    """sRGB 0..1 -> linear 0..1 (the sRGB transfer function)."""
    return tuple(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in rgb)
