# The trees' leaf-card atlas: the one texture every broadleaved tree's leaf cards are cut from (the `leafcards`
# finish; families/plants.py `Cards`, `clad`), rendered here from leaves this script generates (families/plants.py
# `cut_leaf`): nothing downloaded, nothing baked per model.
#
#   web/public/assets/textures/leafcards/albedo.png   512 x 512 RGBA, four 256 px tiles:
#     top left, top right, bottom left   a cluster of leaves on twigs, cut out (alpha), seen from above
#     bottom right                       a dense wall of leaves, opaque: the dark core inside a crown; it fills the
#                                        middle of its tile (`DENSE` of it), a clear gutter round it, so no mipmap of a
#                                        cluster's edge takes its leaves in
#
# Pale and nearly grey, as the tiling foliage albedo is: the part's colour tints it. The colour of the leaves is bled
# out into the clear texels round them, so the mipmaps and the filtering never mix in a dark fringe. The game keeps
# the clusters from thinning in the smaller mipmaps (web/src/render/pc/shaders/cutout.ts).
#
# Run:  cargo art --leafcards   (the trees needn't be rebuilt for it: their cards only say which tile)
import math
import os
import random
import sys

import bpy
import numpy as np
from mathutils import Vector

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, 'families'))
import gamekit as G  # noqa: E402
from plants import cut_leaf, outline, twig  # noqa: E402

SIZE = 512                  # the atlas, pixels: a card is 110 px across from the nearest camera, a tile 256
TILE = 0.8                  # a tile, metres as modelled (a card is whatever size its plant makes it)
DENSE = 0.8                 # the dense wall's share of its tile, across and up
OUT = os.path.join(G.scene.ASSETS, 'textures', 'leafcards', 'albedo.png')


def _material(name, value, warm):
    """A matte leaf colour: nearly grey (`value`), a little yellow (`warm` > 0) or blue-green (< 0)."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes['Principled BSDF']
    rgb = (value * (1 + 0.10 * warm), value * 1.0, value * (1 - 0.16 * warm - 0.04))
    bsdf.inputs['Base Color'].default_value = (*[min(1.0, c) for c in rgb], 1)
    bsdf.inputs['Roughness'].default_value = 0.7
    return m


def _leaf(name, rng, at, heading, length, mats, depth=0.0):
    """A broad leaf with a wavy edge, lying nearly flat, heading `heading` (radians) from `at`; `depth` 0..1: how far
    down in its cluster it lies (darker)."""
    width = length * rng.uniform(0.5, 0.62)
    edge = outline('oval', width, n=14, wave=rng.uniform(0.08, 0.16), waves=rng.choice((5, 7)))
    shade = min(len(mats) - 1, int(depth * len(mats)))
    return cut_leaf(name, at, 90 - math.degrees(heading), rng.uniform(-6, 22), length, edge, mats[shade][rng.randrange(len(mats[shade]))],
                    droop=rng.uniform(0.05, 0.3), fold=rng.uniform(0.25, 0.5), thick=0.004, ribs=5,
                    twist=rng.uniform(-25, 25))


def _fits(p, heading, length, centre, reach):
    """The longest a leaf from `p` heading `heading` may be (up to `length`) and stay within `reach` of `centre`
    across and up the tile."""
    dx, dy = math.cos(heading), math.sin(heading)
    for _ in range(12):
        if abs(p[0] + dx * length - centre[0]) < reach and abs(p[1] + dy * length - centre[1]) < reach:
            return length
        length *= 0.88
    return 0.0


def cluster(name, rng, centre, mats, wood, twigs=6, per=6, fill=10, reach=0.37, turn=0.0):
    """A cluster in a tile round `centre`: `fill` leaves lying under it, then twigs fanning from the middle with `per`
    leaves along each, heading out and to the sides, the later ones laid over the earlier; everything within `reach`
    of the middle across and up the tile, so no leaf crosses into the next."""
    cx, cy = centre
    n, total = 0, fill + twigs * per

    def add(p, heading, size):
        nonlocal n
        size = _fits(p, heading, size, centre, reach)
        if size >= 0.07:
            _leaf(f'{name}_leaf{n}', rng, (p[0], p[1], 0.012 + 0.003 * n), heading, size, mats, depth=1 - n / total)
        n += 1
    for i in range(fill):
        a, r = rng.uniform(0, math.tau), reach * rng.uniform(0.25, 0.8)
        add((cx + r * math.cos(a), cy + r * math.sin(a)), a + rng.uniform(-1.2, 1.2), rng.uniform(0.15, 0.2))
    for t in range(twigs):
        a = turn + math.tau * (t + rng.uniform(-0.25, 0.25)) / twigs
        length = reach * rng.uniform(0.6, 0.85)
        bow = rng.uniform(-0.3, 0.3)
        pts = [Vector((cx + 0.02 * math.cos(a), cy + 0.02 * math.sin(a), 0.0)),
               Vector((cx + length * 0.5 * math.cos(a + bow), cy + length * 0.5 * math.sin(a + bow), 0.01)),
               Vector((cx + length * math.cos(a), cy + length * math.sin(a), 0.015))]
        twig(f'{name}_twig{t}', pts, 0.006, 0.003, wood, verts=4)
        for i in range(per):
            f = (i + 0.6) / per
            p = pts[0].lerp(pts[1], f * 2) if f < 0.5 else pts[1].lerp(pts[2], f * 2 - 1)
            side = (-1 if i % 2 else 1) * rng.uniform(0.5, 1.25) * (1 - 0.75 * f)
            add((p.x, p.y), a + side + rng.uniform(-0.2, 0.2), rng.uniform(0.15, 0.21))


def dense(name, rng, lo, hi, mats, leaves=170):
    """A wall of leaves over the tile from `lo` to `hi` (and a little past it), at every angle, a few deep."""
    for i in range(leaves):
        at = (rng.uniform(lo[0] - 0.05, hi[0] + 0.05), rng.uniform(lo[1] - 0.05, hi[1] + 0.05), 0.002 * i)
        _leaf(f'{name}{i}', rng, at, rng.uniform(0, math.tau), rng.uniform(0.13, 0.18), mats, depth=1 - i / leaves)


def setup():
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_EEVEE'
    scene.eevee.taa_render_samples = 64
    scene.render.film_transparent = True
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.view_settings.view_transform = 'Standard'
    world = bpy.data.worlds.new('atlas')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (1, 1, 1, 1)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.3
    scene.world = world
    sun = bpy.data.objects.new('sun', bpy.data.lights.new('sun', 'SUN'))
    sun.data.energy = 1.9
    sun.data.angle = math.radians(8)
    sun.rotation_euler = Vector((0.75, 0.4, 1.0)).to_track_quat('Z', 'Y').to_euler()
    cam = bpy.data.objects.new('cam', bpy.data.cameras.new('cam'))
    cam.data.type = 'ORTHO'
    for o in (sun, cam):
        bpy.context.collection.objects.link(o)
    scene.camera = cam
    return cam


def render(cam, path, centre, across, size, show):
    """Renders the objects whose names start with one of `show`, `across` metres round `centre`, to `path` (`size` px)."""
    scene = bpy.context.scene
    for o in G.meshes():
        o.hide_render = not o.name.startswith(show)
    cam.data.ortho_scale = across
    cam.location = (centre[0], centre[1], 5)
    scene.render.resolution_x = scene.render.resolution_y = size
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def pixels(path, size):
    image = bpy.data.images.load(path)
    px = np.empty(size * size * 4, dtype=np.float32)
    image.pixels.foreach_get(px)
    return image, px.reshape(size, size, 4)             # rows from the bottom


def finish(path, dense_path):
    """Bleeds the leaves' colour out into the transparent texels, sets in the dense tile, and writes the atlas."""
    image, px = pixels(path, SIZE)
    rgb, a = px[:, :, :3].copy(), px[:, :, 3].copy()
    known = a > 0.5
    mean = rgb[known].mean(axis=0)
    filled = np.where(known[:, :, None], rgb, 0.0)
    weight = known.astype(np.float32)
    for _ in range(64):                                 # each pass: unknown texels take their known neighbours' mean
        acc, w = np.zeros_like(filled), np.zeros_like(weight)
        for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)):
            acc += np.roll(filled * weight[:, :, None], (dy, dx), axis=(0, 1))
            w += np.roll(weight, (dy, dx), axis=(0, 1))
        grow = (weight == 0) & (w > 0)
        filled[grow] = acc[grow] / w[grow][:, None]
        weight[grow] = 1
    filled[weight == 0] = mean
    rgb = np.where(known[:, :, None], rgb, filled)
    # the dense tile (bottom right): opaque, its gaps the dark of a crown's inside
    h = SIZE // 2
    d = int(h * DENSE)
    o = (h - d) // 2
    _, dpx = pixels(dense_path, d)
    da = dpx[:, :, 3:4]
    wall = dpx[:, :, :3] * da + mean * 0.4 * (1 - da)
    rgb[:h, h:] = wall.reshape(-1, 3).mean(axis=0)     # (the gutter: the wall's colour, clear)
    a[:h, h:] = 0.0
    rgb[o:o + d, h + o:h + o + d] = wall
    a[o:o + d, h + o:h + o + d] = 1.0
    out = np.concatenate([rgb, a[:, :, None]], axis=2).astype(np.float32)
    image.pixels.foreach_set(out.ravel())
    image.filepath_raw = path
    image.file_format = 'PNG'
    image.save()
    os.remove(dense_path)
    cover = [float((a[y:y + h, x:x + h] > 0.5).mean()) for y, x in ((h, 0), (h, h), (0, 0))]
    print(f'[gamekit] {os.path.relpath(path, G.scene.ASSETS)}: {SIZE} px, clusters cover '
          f'{", ".join(f"{c:.0%}" for c in cover)} of their tiles; mean leaf colour {[round(float(c), 3) for c in mean]}')


def main():
    G.reset()
    rng = random.Random(7)
    # near white: the part's colour tints it, as it does the tiling foliage albedo. Three depths, the top the lightest.
    tones = [(0.8, 0.0), (0.7, 0.35), (0.74, -0.3), (0.86, 0.5), (0.66, -0.6)]
    mats = [[_material(f'leaf{d}_{i}', v * k, w) for i, (v, w) in enumerate(tones)] for d, k in enumerate((1.0, 0.76, 0.56))]
    wood = _material('wood', 0.4, 1.2)
    h = TILE / 2
    cluster('a', rng, (-h, h), mats, wood, twigs=6, per=6, fill=10)
    cluster('b', rng, (h, h), mats, wood, twigs=5, per=7, fill=8, turn=0.6)
    cluster('c', rng, (-h, -h), mats, wood, twigs=7, per=5, fill=12, turn=2.2)
    dense('d', rng, (0, -TILE), (TILE, 0), mats)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    cam = setup()
    render(cam, OUT, (0, 0), 2 * TILE, SIZE, ('a_', 'b_', 'c_'))
    part = OUT[:-4] + '.dense.png'
    render(cam, part, (h, -h), TILE * DENSE, int(SIZE // 2 * DENSE), ('d',))
    finish(OUT, part)


main()
