//! The hard edges themselves.

use super::*;

/// A mesh edge that is hard (the module's rule, but for how much of it shows).
struct Hard {
    ends: [u32; 2],
    faces: [usize; 2],
    length: f64,
    shown: f64,
}

/// `each(what it keeps from one to the next, i)` for every i under `n`, in order, shared out over the machine's
/// threads when there is enough of it (each thread keeping its own).
fn spread<T: Send, K>(
    n: usize,
    keep: impl Fn() -> K + Sync,
    each: impl Fn(&mut K, usize) -> T + Sync,
) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(1, |t| t.get().min(8));
    if threads < 2 || n < 64 {
        let mut kept = keep();
        return (0..n).map(|i| each(&mut kept, i)).collect();
    }
    let share = n.div_ceil(threads);
    std::thread::scope(|scope| {
        let (keep, each) = (&keep, &each);
        let shares: Vec<_> = (0..n)
            .step_by(share)
            .map(|from| {
                scope.spawn(move || {
                    let mut kept = keep();
                    (from..(from + share).min(n))
                        .map(|i| each(&mut kept, i))
                        .collect::<Vec<T>>()
                })
            })
            .collect();
        shares
            .into_iter()
            .flat_map(|s| {
                s.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

/// The hard edges of a shape that show unbevelled (the module's rule), the longest first.
pub fn hard_edges(meshes: &[Mesh]) -> Vec<Edge> {
    hard_edges_from(meshes, &|_| true)
}

/// `hard_edges` where only the play camera's views `from` takes (each by the way to it) see the shape: a house
/// with the walls on one side cut away is what the cameras on that side see, and no other.
pub fn hard_edges_from(meshes: &[Mesh], from: &(dyn Fn(V3) -> bool + Sync)) -> Vec<Edge> {
    let shape = Shape::new(meshes);
    let mut pieces: Option<Vec<f64>> = None;
    let mut hard: Vec<Hard> = Vec::new();
    for key in &shape.order {
        let Some(pair) = shape.pair(&shape.edges[key]) else {
            continue;
        };
        let [(a, ka), (b, kb)] = pair;
        let (fa, fb) = (&shape.faces[a], &shape.faces[b]);
        if !meshes[fa.mesh as usize].edged || !meshes[fb.mesh as usize].edged {
            continue;
        }
        if dot(fa.n, fb.n) > TURN + SLACK {
            continue;
        }
        let ends = [fa.v[ka], fa.v[(ka + 1) % 3]];
        // Convex: the far corner of one face lies behind the other.
        let far = shape.place[fb.v[(kb + 2) % 3] as usize];
        if dot(fa.n, sub(far, shape.place[ends[0] as usize])) > -1e-6 || !shape.split(pair) {
            continue;
        }
        let pieces = pieces.get_or_insert_with(|| shape.pieces());
        if pieces[ends[0] as usize] < PIECE - SLACK {
            continue;
        }
        if !shape.wide(a, ka, WIDTH) || !shape.wide(b, kb, WIDTH) {
            continue;
        }
        let [p0, p1] = ends.map(|e| shape.place[e as usize]);
        hard.push(Hard {
            ends,
            faces: [a, b],
            length: length(sub(p1, p0)),
            shown: 0.0,
        });
    }
    // Runs: at each place, two hard edges go on as one where each is the other's straightest way on.
    let mut at_place: Map<u32, Vec<u32>> = Map::default();
    for (i, h) in hard.iter().enumerate() {
        for end in h.ends {
            at_place.entry(end).or_default().push(i as u32);
        }
    }
    let way_from = |i: u32, place: u32| {
        let h = &hard[i as usize];
        let other = h.ends[usize::from(h.ends[0] == place)];
        unit(sub(
            shape.place[other as usize],
            shape.place[place as usize],
        ))
    };
    let mut run: Vec<u32> = (0..hard.len() as u32).collect();
    fn root(run: &mut [u32], mut i: u32) -> u32 {
        while run[i as usize] != i {
            run[i as usize] = run[run[i as usize] as usize];
            i = run[i as usize];
        }
        i
    }
    for (&place, edges) in at_place.iter().filter(|(_, e)| e.len() > 1) {
        // Each edge's straightest way on: the edge leaving most nearly opposite to it.
        let on = |i: u32| {
            let way = way_from(i, place);
            edges
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| (j, -dot(way, way_from(j, place))))
                .filter(|(_, straight)| *straight >= RUN - SLACK)
                .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
                .map(|(j, _)| j)
        };
        for &i in edges {
            if let Some(j) = on(i).filter(|&j| j > i && on(j) == Some(i)) {
                let (a, b) = (root(&mut run, i), root(&mut run, j));
                run[b.max(a) as usize] = a.min(b);
            }
        }
    }
    let run: Vec<u32> = (0..hard.len() as u32).map(|i| root(&mut run, i)).collect();
    let mut whole = vec![0.0; hard.len()];
    for (i, h) in hard.iter().enumerate() {
        whole[run[i] as usize] += h.length;
    }
    // How much of each shows: none of an underside or of one on the floor (nor is one looked at whose run is too
    // short whatever shows of it); else looked at every 10 cm or so.
    let out_of = |h: &Hard| unit(add(shape.faces[h.faces[0]].n, shape.faces[h.faces[1]].n));
    let open: Vec<usize> = (0..hard.len())
        .filter(|&i| {
            let [p0, p1] = hard[i].ends.map(|e| shape.place[e as usize]);
            whole[run[i] as usize] >= LENGTH - SLACK
                && out_of(&hard[i])[1] >= DOWN
                && p0[1].max(p1[1]) > FLOOR
        })
        .collect();
    if !open.is_empty() {
        let solid = Solid::new(meshes);
        let shown = spread(
            open.len(),
            || [None; 48],
            |last: &mut Hid, i| {
                let h = &hard[open[i]];
                let [p0, p1] = h.ends.map(|e| shape.place[e as usize]);
                let along = sub(p1, p0);
                let [na, nb] = h.faces.map(|f| shape.faces[f].n);
                let faces = [(na, unit(cross(na, along))), (nb, unit(cross(along, nb)))];
                let n = ((h.length / (2.0 * LENGTH)).round() as usize).clamp(1, 6);
                let seen = (0..n)
                    .filter(|&k| {
                        let p = add(p0, mul(along, (k as f64 + 0.5) / n as f64));
                        solid.shows(p, out_of(h), faces, last, from)
                    })
                    .count();
                h.length * seen as f64 / n as f64
            },
        );
        for (i, shown) in open.into_iter().zip(shown) {
            hard[i].shown = shown;
        }
    }
    // Each run: how much of it shows, and its longest stretch that does.
    let mut runs: BTreeMap<u32, (f64, usize)> = BTreeMap::new();
    for i in 0..hard.len() {
        let (shown, longest) = runs.entry(run[i]).or_insert((0.0, i));
        *shown += hard[i].shown;
        if hard[i].shown > hard[*longest].shown {
            *longest = i;
        }
    }
    let mut out: Vec<Edge> = runs
        .values()
        .filter(|(shown, _)| *shown >= LENGTH - SLACK)
        .map(|&(shown, longest)| {
            let h = &hard[longest];
            let mesh = &meshes[shape.faces[h.faces[0]].mesh as usize];
            let [a, b] = h.ends.map(|e| shape.place[e as usize]);
            Edge {
                part: mesh.part.clone(),
                material: mesh.material.clone(),
                length: shown,
                at: mul(add(a, b), 0.5),
                along: mul(sub(b, a), 1.0 / h.length.max(1e-12)),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.length
            .total_cmp(&a.length)
            .then_with(|| a.at.map(f64::to_bits).cmp(&b.at.map(f64::to_bits)))
    });
    out
}
