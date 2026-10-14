# Placing parts in the game's axes. Placeholder parts (the manifests' `placeholder`, the art scripts in
# tools/art) are written as (x, height, depth) with the front at +depth; Blender is (x, y, z) with the
# front at -y and the game's x mirrored. A GameFrame takes the game's numbers as they are:
#
#   F = G.GameFrame(shift=0.094)
#   F.box('door', 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel=0.01)   # w, h, d at (x, y, z)
#
# `shift` moves every depth back by that much: placeholders that stand against the back of their tile
# (furniture_prims.BACK) are centred on the tile this way.
from . import shapes


class GameFrame:
    def __init__(self, shift=0.0):
        self.shift = shift

    def at(self, x, y, z):
        """A game position (x, height, depth) as a Blender position (front -Y, up +Z)."""
        return (-x, -z - self.shift, y)

    def box(self, name, w, h, d, at, material, **kw):
        """A box of game width, height and depth at a game position."""
        return shapes.cube(name, (w, d, h), at=self.at(*at), material=material, **kw)

    def cyl(self, name, r, h, at, material, **kw):
        """An upright cylinder of radius and height at a game position."""
        return shapes.cylinder(name, r, h, at=self.at(*at), material=material, **kw)

    def cone(self, name, r1, r2, h, at, material, **kw):
        """An upright cone, radius `r1` at the bottom and `r2` at the top, at a game position."""
        return shapes.cone(name, r1, r2, h, at=self.at(*at), material=material, **kw)

    def sphere(self, name, r, at, material, **kw):
        """A sphere at a game position."""
        return shapes.sphere(name, r, at=self.at(*at), material=material, **kw)
