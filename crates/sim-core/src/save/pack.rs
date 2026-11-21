//! The compact encodings a save uses (run lengths, base64, edges, looks, floors, diagonals).

use super::*;

pub(super) const RUN: usize = 1;
/// Strings of edges or diagonals this long or shorter stay as they are (`run_length`).
pub(super) const DENSE: usize = 4096;

/// A string of edge or diagonal characters, run-length coded (v35) when it is long and that makes
/// it shorter: `~` and then `<char><count>` runs split by commas (`~0120,1,04` is 120 `0`s, a `1`
/// and four `0`s). Short ones (a single lot's) stay dense, as before.
pub(super) fn run_length(dense: &str) -> String {
    if dense.len() <= DENSE {
        return dense.to_string();
    }
    let mut out = String::from("~");
    let bytes = dense.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] == c {
            j += 1;
        }
        if i > 0 {
            out.push(',');
        }
        out.push(char::from(c));
        if j - i > RUN {
            out.push_str(&(j - i).to_string());
        }
        i = j;
    }
    if out.len() < dense.len() {
        out
    } else {
        dense.to_string()
    }
}

/// The dense string a saved one stands for: run-length coded (`~`, v35) or as it is (older).
pub(super) fn expand(s: &str) -> Result<std::borrow::Cow<'_, str>, Error> {
    let Some(runs) = s.strip_prefix('~') else {
        return Ok(std::borrow::Cow::Borrowed(s));
    };
    let corrupt = || Error::new("corrupt lot edges in save");
    let mut out = String::new();
    for run in runs.split(',').filter(|r| !r.is_empty()) {
        let mut chars = run.chars();
        let c = chars.next().ok_or_else(corrupt)?;
        let count: usize = match chars.as_str() {
            "" => 1,
            n => n.parse().map_err(|_| corrupt())?,
        };
        // (No lot is bigger than this: a corrupt count can't ask for more.)
        if out.len().saturating_add(count) > (crate::lot::MAX_SIDE + 1) * (crate::lot::MAX_ROWS + 1)
        {
            return Err(corrupt());
        }
        out.extend(std::iter::repeat_n(c, count));
    }
    Ok(std::borrow::Cow::Owned(out))
}

pub(super) const BASE64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]);
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(char::from(BASE64[(n >> (18 - 6 * k) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

pub(super) fn unbase64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let digit = |c: u8| BASE64.iter().position(|&d| d == c).map(|v| v as u32);
    for chunk in s.as_bytes().chunks(4) {
        if chunk.len() != 4 {
            return None;
        }
        let pad = chunk.iter().rev().take_while(|&&c| c == b'=').count();
        let mut n = 0u32;
        for &c in &chunk[..4 - pad] {
            n = n << 6 | digit(c)?;
        }
        n <<= 6 * pad as u32;
        let bytes = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

/// Chemistry between every pair `a < b` of `n` residents, row by row, as base64 of little-endian
/// f32s (empty for fewer than two).
pub(super) fn pack_chemistry(n: usize, chemistry: impl Fn(usize, usize) -> f32) -> String {
    let mut bytes = Vec::with_capacity(n * n.saturating_sub(1) * 2);
    for a in 0..n {
        for b in a + 1..n {
            bytes.extend_from_slice(&chemistry(a, b).to_le_bytes());
        }
    }
    base64(&bytes)
}

pub(super) fn unpack_chemistry(s: &str, n: usize) -> Result<Vec<f32>, Error> {
    let corrupt = || Error::new("corrupt chemistry in save");
    let bytes = unbase64(s).ok_or_else(corrupt)?;
    if bytes.len() != n * n.saturating_sub(1) / 2 * 4 {
        return Err(corrupt());
    }
    Ok(bytes
        .chunks(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

pub(super) fn encode(edges: &[Edge]) -> String {
    edges
        .iter()
        .map(|e| match e {
            Edge::Open => '0',
            Edge::Wall => '1',
            Edge::Door => '2',
            Edge::Window => '3',
            Edge::Fence => '4',
            Edge::Gate => '5',
        })
        .collect()
}

/// The kind of wall standing at `at` (open where there's none).
pub(super) fn edge_at(lot: &Lot, at: EdgeRef) -> Edge {
    match at {
        EdgeRef::H(x, z) => lot.h_edge(x as usize, z as usize),
        EdgeRef::V(x, z) => lot.v_edge(x as usize, z as usize),
        EdgeRef::Diag(x, z) => lot.diag(x as i32, z as i32).map_or(Edge::Open, |d| d.edge),
    }
}

pub(super) fn encode_looks(lot: &Lot, content: &Content) -> Vec<LookSave> {
    let b = &content.build;
    let covering = |c: u8| {
        c.checked_sub(1)
            .and_then(|i| b.coverings.get(i as usize))
            .map_or(String::new(), |s| s.id.clone())
    };
    lot.looks()
        .filter_map(|(at, look)| {
            let edge = edge_at(lot, at);
            if edge == Edge::Open {
                return None;
            }
            let styles = match edge {
                Edge::Door => &b.doors,
                Edge::Window => &b.windows,
                Edge::Fence | Edge::Gate => &b.fences,
                _ => &Vec::new(),
            };
            let (tag, x, z) = match at {
                EdgeRef::H(x, z) => ("h", x, z),
                EdgeRef::V(x, z) => ("v", x, z),
                EdgeRef::Diag(x, z) => ("d", x, z),
            };
            Some(LookSave {
                at: tag.into(),
                x,
                z,
                faces: look.sides.map(covering),
                form: look.form,
                style: styles
                    .get(look.style as usize)
                    .map_or(String::new(), |s| s.id.clone()),
                sill: look.sill,
                head: look.head,
            })
        })
        .collect()
}

/// Puts saved looks back on the walls they belong to; unknown ids, and looks of walls that
/// aren't there, are dropped.
pub(super) fn decode_looks(lot: &mut Lot, content: &Content, looks: &[LookSave]) {
    let b = &content.build;
    let index =
        |list: &[crate::content::BuildStyle], id: &str| list.iter().position(|s| s.id == id);
    for l in looks {
        let at = match l.at.as_str() {
            "h" if (l.x as usize) < lot.width && (l.z as usize) <= lot.depth => {
                EdgeRef::H(l.x, l.z)
            }
            "v" if (l.x as usize) <= lot.width && (l.z as usize) < lot.depth => {
                EdgeRef::V(l.x, l.z)
            }
            "d" if lot.in_bounds(l.x as i32, l.z as i32) => EdgeRef::Diag(l.x, l.z),
            _ => continue,
        };
        let edge = edge_at(lot, at);
        if edge == Edge::Open {
            continue;
        }
        let styles: &[crate::content::BuildStyle] = match edge {
            Edge::Door => &b.doors,
            Edge::Window => &b.windows,
            Edge::Fence | Edge::Gate => &b.fences,
            _ => &[],
        };
        let side = |id: &str| index(&b.coverings, id).map_or(0, |i| i as u8 + 1);
        lot.set_look(
            at,
            EdgeLook {
                sides: [side(&l.faces[0]), side(&l.faces[1])],
                form: if edge == Edge::Wall {
                    l.form.min(crate::lot::FORM_HALF)
                } else {
                    0
                },
                style: index(styles, &l.style).map_or(0, |i| i as u8),
                // A size no door or window may have is dropped.
                sill: l.sill.filter(|&s| {
                    edge == Edge::Window
                        && s + crate::command::MIN_WINDOW <= crate::command::MAX_HEAD
                }),
                head: l.head.filter(|&h| {
                    matches!(edge, Edge::Door | Edge::Window) && h <= crate::command::MAX_HEAD
                }),
            },
        );
    }
}

/// The lot's floor coverings as rectangles of one covering each: runs along each row, a run
/// joined to the one just above it where that is as long and starts where it does (a room's
/// floor is one rectangle, a town's thousands of tiles a few entries a room).
pub(super) fn encode_floors(lot: &Lot, content: &Content) -> Vec<FloorSave> {
    let floors = &content.build.floors;
    let mut tiles: Vec<(u16, u16, u8)> = lot.floors().collect();
    tiles.sort_unstable_by_key(|&(x, z, _)| (z, x));
    // Row runs, in row order: (x, z, w, covering).
    let mut runs: Vec<(u16, u16, u16, u8)> = Vec::new();
    for (x, z, c) in tiles {
        match runs.last_mut() {
            Some(r) if r.1 == z && r.0 + r.2 == x && r.3 == c => r.2 += 1,
            _ => runs.push((x, z, 1, c)),
        }
    }
    // Rectangles: each run grows the open one ending on the row above with its start, width and covering.
    let mut rects: Vec<(u16, u16, u16, u16, u8)> = Vec::new();
    let mut open: HashMap<(u16, u16, u8), usize> = HashMap::new();
    for (x, z, w, c) in runs {
        let key = (x, w, c);
        match open.get(&key).copied() {
            Some(i) if rects[i].1 + rects[i].3 == z => rects[i].3 += 1,
            _ => {
                open.insert(key, rects.len());
                rects.push((x, z, w, 1, c));
            }
        }
    }
    rects
        .into_iter()
        .filter_map(|(x, z, w, d, c)| {
            let id = &floors.get(c.checked_sub(1)? as usize)?.id;
            Some(FloorSave {
                x,
                z,
                w,
                d,
                covering: id.clone(),
            })
        })
        .collect()
}

/// Puts saved floor coverings back; unknown ids, and tiles off the lot, are dropped.
pub(super) fn decode_floors(lot: &mut Lot, content: &Content, floors: &[FloorSave]) {
    let list = &content.build.floors;
    for f in floors {
        let Some(i) = list.iter().position(|s| s.id == f.covering) else {
            continue;
        };
        let Ok(covering) = u8::try_from(i + 1) else {
            continue;
        };
        // (Rows and tiles in order: the first off the lot ends them, so a corrupt size costs nothing.)
        for z in f.z..f.z.saturating_add(f.d) {
            if !lot.in_bounds(i32::from(f.x), i32::from(z)) {
                break;
            }
            for x in f.x..f.x.saturating_add(f.w) {
                if !lot.in_bounds(i32::from(x), i32::from(z)) {
                    break;
                }
                lot.set_floor(x, z, covering);
            }
        }
    }
}

pub(super) fn encode_diagonals(diags: &[Option<Diagonal>]) -> String {
    if diags.iter().all(Option::is_none) {
        return String::new();
    }
    diags
        .iter()
        .map(|d| {
            let Some(d) = d else { return '0' };
            let base = match d.dir {
                DiagDir::Dp => 0,
                DiagDir::Dn => 3,
            };
            let kind = match d.edge {
                Edge::Door => 2,
                Edge::Window => 3,
                _ => 1,
            };
            char::from(b'0' + base + kind)
        })
        .collect()
}

pub(super) fn decode_diagonals(s: &str) -> Result<Vec<Option<Diagonal>>, Error> {
    s.chars()
        .map(|c| {
            let n = c
                .to_digit(10)
                .filter(|&n| n <= 6)
                .ok_or_else(|| Error::new("corrupt diagonal walls in save"))?;
            if n == 0 {
                return Ok(None);
            }
            let dir = if n <= 3 { DiagDir::Dp } else { DiagDir::Dn };
            let edge = match (n - 1) % 3 {
                0 => Edge::Wall,
                1 => Edge::Door,
                _ => Edge::Window,
            };
            Ok(Some(Diagonal { dir, edge }))
        })
        .collect()
}

pub(super) fn decode(s: &str) -> Result<Vec<Edge>, Error> {
    s.chars()
        .map(|c| match c {
            '0' => Ok(Edge::Open),
            '1' => Ok(Edge::Wall),
            '2' => Ok(Edge::Door),
            '3' => Ok(Edge::Window),
            '4' => Ok(Edge::Fence),
            '5' => Ok(Edge::Gate),
            _ => Err(Error::new("corrupt lot edges in save")),
        })
        .collect()
}

// ---- Planner ------------------------------------------------------------------------------
