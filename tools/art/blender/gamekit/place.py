# Placing parts in the game's axes. Placeholder parts (the manifests' `placeholder`, the art scripts in
# tools/art) are written as (x, height, depth) with the front at +depth; Blender is (x, y, z) with the
# front at -y and the game's x mirrored. A GameFrame takes the game's numbers as they are:
#
#   F = G.GameFrame(shift=0.094)
#   F.box('door', 0.728, 1.146, 0.04, (0, 0.643, 0.195), M.body, bevel=0.01)   # w, h, d at (x, y, z)
#
# `shift` moves every depth back by that much: placeholders that stand against the back of their tile
# (furniture_prims.BACK) are centred on the tile this way. `depths` redraws a placeholder's numbers at
# another depth: pairs (game depth, drawn depth), ascending; depths between two pairs are mapped
# linearly, beyond the ends moved as the end is. A box's back and front are mapped, so it keeps its
# place between them; anything else moves by its middle (the 0.79-deep kitchen counters drawn 0.66
# deep, as catalog2.py `shallow`).
from . import shapes


class GameFrame:
    def __init__(self, shift=0.0, depths=None):
        self.shift = shift
        self.depths = depths

    def depth(self, z):
        """A game depth as drawn (through `depths`, if given)."""
        d = self.depths
        if not d:
            return z
        if z <= d[0][0]:
            return z + d[0][1] - d[0][0]
        for (a, b), (c, e) in zip(d, d[1:]):
            if z <= c:
                return b + (z - a) * (e - b) / (c - a)
        return z + d[-1][1] - d[-1][0]

    def at(self, x, y, z):
        """A game position (x, height, depth) as a Blender position (front -Y, up +Z)."""
        return (-x, -self.depth(z) - self.shift, y)

    def box(self, name, w, h, d, at, material, **kw):
        """A box of game width, height and depth at a game position."""
        if self.depths:
            z0, z1 = self.depth(at[2] - d / 2), self.depth(at[2] + d / 2)
            d = z1 - z0
            if kw.get('bevel'):
                kw['bevel'] = min(kw['bevel'], d * 0.45)
            return shapes.cube(name, (w, d, h), at=(-at[0], -(z0 + z1) / 2 - self.shift, at[1]), material=material, **kw)
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
