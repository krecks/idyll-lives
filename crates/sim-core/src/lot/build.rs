//! Making a lot: empty, from a file, from edges, and growing it deeper.

use super::*;

impl Lot {
    /// An open lot whose every row is ground (all its chunks allocated).
    pub fn new(width: usize, depth: usize) -> Self {
        Self::with_ground(width, depth, depth)
    }

    /// An open lot whose first `ground` rows are always there; the rows past them are
    /// allocated chunk by chunk as they're built on.
    pub fn with_ground(width: usize, depth: usize, ground: usize) -> Self {
        let (cols, rows) = (width / CHUNK + 1, depth / CHUNK + 1);
        let mut lot = Self {
            width,
            depth,
            cols,
            rows,
            base: vec![0; cols * rows],
            order: Vec::new(),
            built: vec![0; cols * rows],
            indoor: vec![0; cols * rows],
            h_edges: vec![Edge::Open],
            v_edges: vec![Edge::Open],
            diags: vec![None],
            rooms: vec![OUTDOORS],
            halves: vec![[OUTDOORS; 2]],
            looks: BTreeMap::new(),
            floors: BTreeMap::new(),
            room_meta: Vec::new(),
            pending: None,
            changes: RoomChanges {
                full: true,
                ..Default::default()
            },
            log: None,
        };
        lot.ensure(0, 0, width, ground.min(depth));
        lot
    }

    pub fn from_file(file: &LotFile) -> Result<Self, Error> {
        Self::from_file_styled(file, &[], &[])
    }

    /// `from_file`, its fences' styles found among `fence_styles` (the content's fence style ids)
    /// and its floors' coverings among `floor_ids` (the content's floor covering ids).
    pub fn from_file_styled(
        file: &LotFile,
        fence_styles: &[&str],
        floor_ids: &[&str],
    ) -> Result<Self, Error> {
        let (w, d) = (file.width as usize, file.depth as usize);
        if w == 0 || d == 0 || w > MAX_SIDE || d > MAX_SIDE {
            return Err(Error::new(format!(
                "lot size must be within 1..={MAX_SIDE}"
            )));
        }
        // Every storey's rows, and the basement's after them (see `storeys.rs`).
        let ground = d;
        let d = d * (file.storeys.unwrap_or(1).clamp(1, 4) as usize + usize::from(file.basement));
        let mut lot = Self::with_ground(w, d, ground);
        for &[x0, z0, x1, z1] in &file.walls {
            let (x0, z0, x1, z1) = (x0 as usize, z0 as usize, x1 as usize, z1 as usize);
            if z0 == z1 && z0 <= d && x0.max(x1) <= w {
                for x in x0.min(x1)..x0.max(x1) {
                    lot.set_h(x, z0, Edge::Wall);
                }
            } else if x0 == x1 && x0 <= w && z0.max(z1) <= d {
                for z in z0.min(z1)..z0.max(z1) {
                    lot.set_v(x0, z, Edge::Wall);
                }
            } else {
                return Err(Error::new(format!(
                    "wall {x0},{z0}-{x1},{z1} is diagonal or out of bounds"
                )));
            }
        }
        let openings = file.doors.iter().map(|d| (d, Edge::Door, "door"));
        let openings = openings.chain(file.windows.iter().map(|w| (w, Edge::Window, "window")));
        for (door, kind, name) in openings {
            let (x, z) = (door.x as usize, door.z as usize);
            let current = match door.axis {
                Axis::X if x < w && z <= d => lot.h_edge(x, z),
                Axis::Z if x <= w && z < d => lot.v_edge(x, z),
                _ => return Err(Error::new(format!("{name} at {x},{z} is out of bounds"))),
            };
            if current != Edge::Wall {
                return Err(Error::new(format!("{name} at {x},{z} is not on a wall")));
            }
            match door.axis {
                Axis::X => lot.set_h(x, z, kind),
                Axis::Z => lot.set_v(x, z, kind),
            }
        }
        for d in &file.diagonals {
            let (x, z) = (d.x as usize, d.z as usize);
            if x >= w || z >= lot.depth {
                return Err(Error::new(format!("diagonal at {x},{z} is out of bounds")));
            }
            let edge = match d.kind.as_deref().unwrap_or("wall") {
                "wall" => Edge::Wall,
                "door" => Edge::Door,
                "window" => Edge::Window,
                other => return Err(Error::new(format!("unknown diagonal kind '{other}'"))),
            };
            lot.set_diag(x, z, Some(Diagonal { dir: d.dir, edge }));
        }
        // Fences on the ground storey, never on a wall (they make no rooms).
        for f in &file.fences {
            let [x0, z0, x1, z1] = f.line.map(usize::from);
            let edge = if f.gate { Edge::Gate } else { Edge::Fence };
            let style = f
                .style
                .as_deref()
                .and_then(|id| fence_styles.iter().position(|s| *s == id))
                .unwrap_or(0) as u8;
            let edges: Vec<EdgeRef> = if z0 == z1 && z0 <= ground && x0.max(x1) <= w {
                (x0.min(x1)..x0.max(x1))
                    .map(|x| EdgeRef::H(x as u16, z0 as u16))
                    .collect()
            } else if x0 == x1 && x0 <= w && z0.max(z1) <= ground {
                (z0.min(z1)..z0.max(z1))
                    .map(|z| EdgeRef::V(x0 as u16, z as u16))
                    .collect()
            } else {
                return Err(Error::new(format!(
                    "fence {x0},{z0}-{x1},{z1} is diagonal or off the ground"
                )));
            };
            for e in edges {
                let (current, x, z) = match e {
                    EdgeRef::H(x, z) => (lot.h_edge(x as usize, z as usize), x, z),
                    EdgeRef::V(x, z) => (lot.v_edge(x as usize, z as usize), x, z),
                    EdgeRef::Diag(..) => unreachable!(),
                };
                if current.is_wall() {
                    return Err(Error::new(format!("fence at {x},{z} is on a wall")));
                }
                match e {
                    EdgeRef::H(..) => lot.set_h(x as usize, z as usize, edge),
                    _ => lot.set_v(x as usize, z as usize, edge),
                }
                lot.set_look(
                    e,
                    EdgeLook {
                        style,
                        ..EdgeLook::default()
                    },
                );
            }
        }
        lot.lay_floors(&file.floors, floor_ids)?;
        lot.compute_rooms();
        Ok(lot)
    }

    /// Lays floor coverings (`FloorRaw` rectangles) by their ids among `floor_ids` (index + 1 is
    /// the covering); an unknown id leaves its tiles automatic, a rectangle off the lot is an error.
    pub fn lay_floors(&mut self, floors: &[FloorRaw], floor_ids: &[&str]) -> Result<(), Error> {
        for f in floors {
            let (x1, z1) = (
                usize::from(f.x) + usize::from(f.w),
                usize::from(f.z) + usize::from(f.d),
            );
            if f.w == 0 || f.d == 0 || x1 > self.width || z1 > self.depth {
                return Err(Error::new(format!(
                    "floor {},{} {}x{} is out of bounds",
                    f.x, f.z, f.w, f.d
                )));
            }
            let Some(i) = floor_ids.iter().position(|id| *id == f.covering) else {
                continue;
            };
            let Ok(covering) = u8::try_from(i + 1) else {
                continue;
            };
            for z in f.z..f.z + f.d {
                for x in f.x..f.x + f.w {
                    self.set_floor(x, z, covering);
                }
            }
        }
        Ok(())
    }

    /// Rebuilds a lot from saved edge arrays (row-major, as `h_edge` / `v_edge` number them:
    /// `width × (depth + 1)` and `(width + 1) × depth`) and diagonals (row-major per tile; empty
    /// for saves made before diagonal walls existed). The first `ground` rows are always there.
    pub fn from_edges(
        width: usize,
        depth: usize,
        ground: usize,
        h: Vec<Edge>,
        v: Vec<Edge>,
        diags: Vec<Option<Diagonal>>,
    ) -> Result<Self, Error> {
        // Deeper than a town file may be: the storeys above the ground are rows too.
        if width == 0 || depth == 0 || width > MAX_SIDE || depth > MAX_ROWS {
            return Err(Error::new("saved lot size is out of range"));
        }
        if h.len() != width * (depth + 1) || v.len() != (width + 1) * depth {
            return Err(Error::new("saved lot edges have the wrong size"));
        }
        if !diags.is_empty() && diags.len() != width * depth {
            return Err(Error::new("saved diagonal walls have the wrong size"));
        }
        if diags.iter().flatten().any(|d| d.edge == Edge::Open) {
            return Err(Error::new("corrupt diagonal walls in save"));
        }
        let mut lot = Self::with_ground(width, depth, ground.clamp(1, depth));
        for (i, &e) in h.iter().enumerate().filter(|(_, e)| **e != Edge::Open) {
            lot.set_h(i % width, i / width, e);
        }
        for (i, &e) in v.iter().enumerate().filter(|(_, e)| **e != Edge::Open) {
            lot.set_v(i % (width + 1), i / (width + 1), e);
        }
        for (i, &d) in diags.iter().enumerate().filter(|(_, d)| d.is_some()) {
            lot.set_diag(i % width, i / width, d);
        }
        lot.compute_rooms();
        Ok(lot)
    }

    /// Adds rows at the far end, all open (room for the storeys above the ground; see
    /// `World::storeys`). Edges and tiles already there keep their places, and nothing is
    /// allocated for the new rows until something is built there.
    pub fn grow_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.rows = depth / CHUNK + 1;
        self.base.resize(self.cols * self.rows, 0);
        self.built.resize(self.cols * self.rows, 0);
        self.indoor.resize(self.cols * self.rows, 0);
        self.depth = depth;
        self.pending = None;
        self.compute_rooms();
    }
}
