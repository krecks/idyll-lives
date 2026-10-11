"""
Catmull-Clark subdivision of a closed polygon cage, as sparse linear operators (positions, skin
weights and any other per-vertex attribute go through the same matrices):

    S = subdivision(faces, n)       # S.M: (n1, n) cage -> level 1; S.quads: level-1 faces
    limit_matrix(S.quads, n1)       # level 1 -> its limit surface (every face a quad)
    fit(targets, faces)             # a cage whose limit surface passes through `targets`

Level-1 vertices are ordered: the cage's vertex points, then edge points, then face points, so the
first `n` of them are the cage's own vertices. Faces keep the cage's winding.
"""

import numpy as np
from scipy import sparse
from scipy.sparse.linalg import spsolve


class Subdivision:
    def __init__(self, M, quads, edges, n):
        self.M = M            # (n1, n) sparse
        self.quads = quads    # (q, 4) level-1 faces
        self.edges = edges    # (e, 2) cage edges (edge point k is level-1 vertex n + k)
        self.n = n


def _edges(faces):
    """Undirected cage edges, and per face its edge ids in order (edge i joins corner i and i + 1)."""
    key = {}
    edges = []
    face_edges = []
    for f in faces:
        ids = []
        for i in range(len(f)):
            a, b = f[i], f[(i + 1) % len(f)]
            k = (a, b) if a < b else (b, a)
            e = key.get(k)
            if e is None:
                e = key[k] = len(edges)
                edges.append(k)
            ids.append(e)
        face_edges.append(ids)
    return np.array(edges, np.int64), face_edges


def check_closed(faces, n):
    """Every edge is shared by exactly two faces, in opposite directions (a closed, oriented 2-manifold)."""
    seen = {}
    for fi, f in enumerate(faces):
        for i in range(len(f)):
            a, b = f[i], f[(i + 1) % len(f)]
            if (a, b) in seen:
                raise ValueError(f'edge {a}-{b} is used twice in the same direction (faces {seen[(a, b)]}, {fi})')
            seen[(a, b)] = fi
    for (a, b), fi in seen.items():
        if (b, a) not in seen:
            raise ValueError(f'edge {a}-{b} of face {fi} is open')
    used = np.zeros(n, bool)
    for f in faces:
        used[list(f)] = True
    if not used.all():
        raise ValueError(f'unused cage vertices: {np.where(~used)[0][:10]}')


def orient(faces, pos):
    """Faces wound consistently (flood fill over shared edges), counter-clockwise from outside."""
    faces = [list(f) for f in faces]
    owner = {}
    for fi, f in enumerate(faces):
        for i in range(len(f)):
            a, b = f[i], f[(i + 1) % len(f)]
            owner.setdefault((min(a, b), max(a, b)), []).append(fi)
    done = np.zeros(len(faces), bool)
    for seed in range(len(faces)):
        if done[seed]:
            continue
        done[seed] = True
        stack = [seed]
        comp = [seed]
        while stack:
            fi = stack.pop()
            f = faces[fi]
            for i in range(len(f)):
                a, b = f[i], f[(i + 1) % len(f)]
                for g in owner[(min(a, b), max(a, b))]:
                    if done[g]:
                        continue
                    h = faces[g]
                    # A neighbour must use the shared edge the other way round.
                    same = any(h[j] == a and h[(j + 1) % len(h)] == b for j in range(len(h)))
                    if same:
                        faces[g] = h[::-1]
                    done[g] = True
                    stack.append(g)
                    comp.append(g)
        # Outward: positive signed volume.
        vol = 0.0
        for fi in comp:
            f = faces[fi]
            p0 = pos[f[0]]
            for j in range(1, len(f) - 1):
                vol += np.dot(p0, np.cross(pos[f[j]], pos[f[j + 1]]))
        if vol < 0:
            for fi in comp:
                faces[fi] = faces[fi][::-1]
    return faces


def subdivision(faces, n):
    """One Catmull-Clark step of a closed cage of `n` vertices (faces: lists of vertex ids)."""
    edges, face_edges = _edges(faces)
    ne, nf = len(edges), len(faces)
    rows, cols, vals = [], [], []

    def put(r, c, v):
        rows.append(r)
        cols.append(c)
        vals.append(v)

    # Face points: centroids.
    fp0 = n + ne
    for fi, f in enumerate(faces):
        for v in f:
            put(fp0 + fi, v, 1.0 / len(f))
    # Edge points: (a + b + the two face points) / 4, written in cage terms.
    edge_faces = [[] for _ in range(ne)]
    for fi, ids in enumerate(face_edges):
        for e in ids:
            edge_faces[e].append(fi)
    for e, (a, b) in enumerate(edges):
        fs = edge_faces[e]
        assert len(fs) == 2, f'open or non-manifold edge {a}-{b}'
        put(n + e, a, 0.25)
        put(n + e, b, 0.25)
        for fi in fs:
            f = faces[fi]
            for v in f:
                put(n + e, v, 0.25 / len(f))
    # Vertex points: (F + 2R + (k - 3) P) / k.
    vert_faces = [[] for _ in range(n)]
    for fi, f in enumerate(faces):
        for v in f:
            vert_faces[v].append(fi)
    vert_edges = [[] for _ in range(n)]
    for e, (a, b) in enumerate(edges):
        vert_edges[a].append(e)
        vert_edges[b].append(e)
    for v in range(n):
        k = len(vert_edges[v])
        assert k >= 3 and len(vert_faces[v]) == k, f'vertex {v}: valence {k}, {len(vert_faces[v])} faces'
        put(v, v, (k - 3) / k)
        for e in vert_edges[v]:
            a, b = edges[e]
            put(v, a, 2 / k * 0.5 / k)
            put(v, b, 2 / k * 0.5 / k)
        for fi in vert_faces[v]:
            f = faces[fi]
            for u in f:
                put(v, u, 1 / k / k / len(f))
    M = sparse.coo_matrix((vals, (rows, cols)), shape=(n + ne + nf, n)).tocsr()
    # Level-1 quads: corner, next edge point, face point, previous edge point.
    quads = []
    for fi, f in enumerate(faces):
        ids = face_edges[fi]
        m = len(f)
        for i in range(m):
            quads.append([f[i], n + ids[i], fp0 + fi, n + ids[(i - 1) % m]])
    return Subdivision(M, np.array(quads, np.int64), edges, n)


def limit_matrix(quads, n):
    """Limit positions of an all-quad mesh: (k^2 v + 4 sum(edge neighbours) + sum(diagonals)) / (k (k + 5))."""
    nbr = [set() for _ in range(n)]
    diag = [[] for _ in range(n)]
    for q in quads:
        for i in range(4):
            v = q[i]
            nbr[v].add(q[(i + 1) % 4])
            nbr[v].add(q[(i - 1) % 4])
            diag[v].append(q[(i + 2) % 4])
    rows, cols, vals = [], [], []
    for v in range(n):
        k = len(nbr[v])
        d = k * (k + 5)
        rows.append(v)
        cols.append(v)
        vals.append(k * k / d)
        for u in sorted(nbr[v]):
            rows.append(v)
            cols.append(u)
            vals.append(4 / d)
        for u in diag[v]:
            rows.append(v)
            cols.append(u)
            vals.append(1 / d)
    return sparse.coo_matrix((vals, (rows, cols)), shape=(n, n)).tocsr()


def fit(targets, S, keep=None, stiff=0.0):
    """
    Cage positions whose limit surface interpolates `targets` (n, 3) at the cage vertices.
    `stiff` (0..1) blends towards plain targets (less overshoot on sharp features); `keep` (bool
    mask) are vertices pinned to their targets in the cage itself.
    """
    n = S.n
    L = limit_matrix(S.quads, S.M.shape[0])
    A = (L @ S.M)[:n].tocsr()
    if keep is not None and keep.any():
        I = sparse.identity(n, format='csr')
        D = sparse.diags(keep.astype(float))
        A = (D @ I + (I - D) @ A).tocsr()
    c = np.column_stack([spsolve(A.tocsc(), targets[:, k]) for k in range(targets.shape[1])])
    return c + stiff * (targets - c)


def triangles(quads, pos):
    """Quads split along their shorter diagonal (counter-clockwise kept)."""
    q = np.asarray(quads)
    d02 = np.linalg.norm(pos[q[:, 0]] - pos[q[:, 2]], axis=1)
    d13 = np.linalg.norm(pos[q[:, 1]] - pos[q[:, 3]], axis=1)
    a = np.stack([q[:, 0], q[:, 1], q[:, 2], q[:, 0], q[:, 2], q[:, 3]], 1)
    b = np.stack([q[:, 1], q[:, 2], q[:, 3], q[:, 1], q[:, 3], q[:, 0]], 1)
    return np.where((d02 <= d13)[:, None], a, b).reshape(-1, 3)


def polygon_triangles(faces):
    """Fan triangles of polygons (for the cage-level mesh)."""
    out = []
    for f in faces:
        for j in range(1, len(f) - 1):
            out.append([f[0], f[j], f[j + 1]])
    return np.array(out, np.int64)
