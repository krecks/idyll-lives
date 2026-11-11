//! Builds the residents' character set from code (rig.json version 4 and its binaries; see
//! docs/characters-format.md): `cargo characters build [out dir]` (default
//! web/public/assets/characters).
//!
//! Per family and life stage it asks the head interface (head.rs) where the face is, then builds
//! the body (body.rs), the face (face.rs: eyes, lids, lashes, brows, mouth and the morph channels),
//! the hair (hair.rs) and the clothes (clothes.rs), merges the face's skin pieces and the ears (ear.rs)
//! into the body,
//! bakes the base stage's ambient occlusion into tone.w (ao.rs; the stages share it), adds the
//! identity channels (identity.rs: a face per resident) to the base stage's morphs, and writes
//! the base stage (young adult) whole and the other stages as deltas. Then the checks
//! run. Everything is deterministic (seeded).

use std::io;
use std::path::Path;
use std::sync::Arc;

use serde::Serialize;

use super::ao;
use super::body::{Meta, body_fields_and_tone, build_body};
use super::clothes::{Cut, Cuts, build_clothes, garment_cuts};
use super::contract::{IDENTITY, channels, is_wisps, wisps_part};
use super::ear::{Ears, ears};
use super::face::{Deltas, FaceOut, build_face, follow_mouth};
use super::hair::build_hair;
use super::head::{HeadFrame, head_frame};
use super::identity;
use super::json::{OMap, to_string_py};
use super::kdtree::KdTree;
use super::meshops::OnMesh;
use super::np::{py_round, smoothstep};
use super::rig::{
    BASE_STAGE, BONES, CLIP_BONES, FACE_BONES, FAMILIES, NB, STAGES, head, leg_lengths, parents,
    skeleton, source_pelvis,
};
use super::write::{
    FIELD_SCALE, MORPH_SCALE, NORMAL_SCALE, PartEntry, STAGE_SCALE, StagePart, TONE_SCALE,
    pack_morph, r4, write_parts, write_stage,
};
use super::{Mesh, V3, merge};

/// Every part of a family at a stage (no fields / tone / morphs yet), with the body's metadata.
struct Assembled {
    parts: Vec<(String, Mesh)>,
    meta: Meta,
    p: [V3; NB],
    head: Arc<HeadFrame>,
    face: FaceOut,
    /// The ears: the last pieces of `body` and `hd.body` (whole) and of `lod.body` (plain).
    ears: Arc<Ears>,
    n_skin: usize,
    n_lod: usize,
    cuts: Cuts,
    /// The body's skin positions without its eye sockets (the occlusion bake), and its skin triangles on
    /// them (`lid_shading`, the lids' tone).
    unsocketed: Vec<V3>,
    skin: OnMesh,
    /// SPIKE (`hd.body`, the body at level 2 with the lids and lips after it; the last part): how many of
    /// its vertices are the skin's, the skin without its eye sockets and its triangles on that.
    n_hd: usize,
    hd_unsocketed: Vec<V3>,
    hd_skin: OnMesh,
}

/// SPIKE: the close-up detail level's part (the whole body at level 2).
const HD: &str = "hd.body";

fn part<'a>(parts: &'a mut [(String, Mesh)], name: &str) -> &'a mut Mesh {
    &mut parts
        .iter_mut()
        .find(|e| e.0 == name)
        .unwrap_or_else(|| panic!("no part {name}"))
        .1
}

fn get<'a>(parts: &'a [(String, Mesh)], name: &str) -> &'a Mesh {
    &parts
        .iter()
        .find(|e| e.0 == name)
        .unwrap_or_else(|| panic!("no part {name}"))
        .1
}

fn assemble(family: &str, stage: &str, cuts: Option<&Cuts>) -> Assembled {
    let head = head_frame(family, stage);
    let bd = build_body(family, stage);
    let p = bd.joints;
    let f = build_face(family, stage, head.clone());
    let n_skin = bd.body.pos.len();
    let n_lod = bd.lod.pos.len();
    let extras: Vec<Mesh> = f
        .body
        .iter()
        .map(|m| Mesh {
            arm: vec![0.0; m.pos.len()],
            ..m.clone()
        })
        .collect();
    let strip = |m: &Mesh| Mesh {
        pos: m.pos.clone(),
        nor: m.nor.clone(),
        dense: m.dense.clone(),
        arm: m.arm.clone(),
        idx: m.idx.clone(),
        ..Mesh::default()
    };
    // The ears come last, the left then the right (ear.rs): the face's channels pass them by.
    let ears = ears(family, stage);
    let mut body_list: Vec<Mesh> = vec![strip(&bd.body)];
    body_list.extend(extras.iter().map(strip));
    body_list.extend(ears.full.iter().map(|e| strip(&e.mesh)));
    // The low-detail body has the eyelids too (they come first among the face's skin pieces), and
    // plain ears.
    let mut lod_list: Vec<Mesh> = vec![strip(&bd.lod)];
    lod_list.extend(extras.iter().filter(|m| m.skin_tone).map(strip));
    lod_list.extend(ears.lod.iter().map(|e| strip(&e.mesh)));
    let mut parts = vec![
        (
            "body".to_string(),
            merge(&body_list.iter().collect::<Vec<_>>()),
        ),
        (
            "lod.body".to_string(),
            merge(&lod_list.iter().collect::<Vec<_>>()),
        ),
    ];
    let n_lids: usize = extras.iter().filter(|m| m.skin_tone).map(Mesh::len).sum();
    let skin = OnMesh::new(&bd.unsocketed, &parts[0].1.idx, n_skin);
    let lids: Vec<V3> = {
        let shade = lid_shading(&skin, &parts[0].1.nor);
        parts[0].1.pos[n_skin..n_skin + n_lids]
            .iter()
            .map(|&q| shade(q))
            .collect()
    };
    parts[0].1.nor[n_skin..n_skin + n_lids].copy_from_slice(&lids);
    parts.extend(f.parts.iter().cloned());
    parts.extend(build_hair(family, stage, &head, &p));
    let cuts = match cuts {
        Some(c) => c.clone(),
        None => garment_cuts(family, &p),
    };
    parts.extend(build_clothes(family, stage, &cuts));
    // SPIKE (`hd.body`): the level-2 skin with the same lids, lips and ears after it, the lids shaded by
    // the level-2 skin they lie on. Last, so every other part stays where it was in the binaries.
    let n_hd = bd.hd.pos.len();
    let mut hd_list: Vec<Mesh> = vec![strip(&bd.hd)];
    hd_list.extend(extras.iter().map(strip));
    hd_list.extend(ears.full.iter().map(|e| strip(&e.mesh)));
    let mut hd = merge(&hd_list.iter().collect::<Vec<_>>());
    let hd_skin = OnMesh::new(&bd.hd_unsocketed, &hd.idx, n_hd);
    let hd_lids: Vec<V3> = {
        let shade = lid_shading(&hd_skin, &hd.nor);
        hd.pos[n_hd..n_hd + n_lids]
            .iter()
            .map(|&q| shade(q))
            .collect()
    };
    hd.nor[n_hd..n_hd + n_lids].copy_from_slice(&hd_lids);
    parts.push((HD.to_string(), hd));
    Assembled {
        parts,
        meta: bd.meta,
        p,
        head,
        face: f,
        ears,
        n_skin,
        n_lod,
        cuts,
        unsocketed: bd.unsocketed,
        skin,
        n_hd,
        hd_unsocketed: bd.hd_unsocketed,
        hd_skin,
    }
}

/// Hair this near the skin (m) takes all of an identity channel's shape there, and hair further off than
/// the second none of it (`build_family`; the beard all of it).
const HAIR_FOLLOWS: (f64, f64) = (0.006, 0.03);
/// Where a hair vertex lies on the skin (its nearest triangle's corners and their weights) and how much
/// of the skin's shape it takes.
type OnSkin = ([usize; 3], [f64; 3], f64);

/// A lid's normal at a point: the face's it lies on, the skin as it would be without the eye sockets
/// (`skin`; `nor`: the body's normals, which are the skin's so), at its nearest point, not
/// the eyeball's the lid is made round. The lids reach over the sockets out to the face (face.rs), where
/// that is the skin's own, so a lid, open or closed, carries on the face round it rather than showing as a
/// patch. (Not the low-detail body's: its coarse socket would turn them inside out.)
fn lid_shading<'a>(skin: &'a OnMesh, nor: &'a [V3]) -> impl Fn(V3) -> V3 + 'a {
    move |q: V3| {
        let (t, w) = skin.at(q);
        (nor[t[0]] * w[0] + nor[t[1]] * w[1] + nor[t[2]] * w[2]).normalize()
    }
}

/// Ambient occlusion into every toned part's tone.w (ao.rs), at rest: the bodies by themselves (the
/// lids, the lips and the ears are in them) and the mouth (not the eyes: the lower lids would take a dark crescent
/// from the eyeball); the shells by the body (the low-detail ones by the low-detail body) and
/// themselves; the hair and beard by the body, the eyes and themselves. On the face (the skin, the first
/// `spread.0` of the body, where `spread.1` says) each vertex's shade is spread over its triangles
/// (`ao::occlusion_spread`). The lids then take the shade of the skin they lie on (`build_family`).
/// The hair's wisps take the shade of the hair they grow from (a wisp's vertices come in twos, its
/// root, where a vertex of the hair is, then its end: hair.rs `wisps`), so an edge and its wisps are
/// one colour.
fn bake_ao(parts: &mut [(String, Mesh)], spread: (usize, &dyn Fn(V3) -> bool), n_hd: usize) {
    let (body, lod) = (get(parts, "body").clone(), get(parts, "lod.body").clone());
    let face = ["eyes", "teeth", "tongue"].map(|n| get(parts, n).clone());
    let [eyes, teeth, tongue] = &face;
    for (name, m) in parts.iter_mut() {
        if is_wisps(name) {
            continue;
        }
        // (SPIKE: `hd.body` is baked as `body` is, over its own skin.)
        let full = name == "body" || name == HD;
        let n_spread = if name == HD { n_hd } else { spread.0 };
        let around: Vec<&Mesh> = if full || name == "lod.body" {
            vec![teeth, tongue]
        } else if name.starts_with("lod.") {
            vec![&lod]
        } else if ["top.", "bottom.", "shoes."]
            .iter()
            .any(|p| name.starts_with(p))
        {
            vec![&body]
        } else if name.starts_with("hair.") {
            vec![&body, eyes]
        } else {
            continue;
        };
        let (reach, passes, samples) = if full || name == "lod.body" {
            (ao::REACH_BODY, 2, ao::SAMPLES)
        } else if name.starts_with("hair.") {
            (ao::REACH, 0, ao::SAMPLES_HAIR)
        } else {
            (ao::REACH_SHELL, 2, ao::SAMPLES)
        };
        let occ = if full {
            ao::occlusion_spread(m, &around, reach, samples, n_spread, spread.1)
        } else {
            ao::occlusion(m, &around, reach, samples)
        };
        // The face's creases are narrower than the triangles, and `occlusion_spread` has them; the rest
        // of the skin and the shells are relaxed over the mesh instead, so a ring's two or three dark
        // vertices read as a crease rather than as a blotch (`ao::smooth`).
        let face = |i: usize| full && i < n_spread && (spread.1)(m.pos[i]);
        let occ = ao::smooth(m, &occ, passes, &face);
        ao::bake(m, occ);
    }
    let hair: Vec<String> = parts.iter().map(|e| e.0.clone()).collect();
    for name in hair.iter().filter(|n| is_wisps(n)) {
        let solid = get(parts, name.strip_suffix(".wisps").expect("a wisps part"));
        let near = KdTree::new(&solid.pos);
        let shade: Vec<f64> = get(parts, name)
            .pos
            .chunks(2)
            .map(|q| solid.tone[near.query(&q[0], 1)[0].1][3])
            .collect();
        for (t, w) in part(parts, name).tone.chunks_mut(2).zip(shade) {
            t[0][3] = w;
            t[1][3] = w;
        }
    }
}

/// A body's (or a stage's) skeleton and metadata in rig.json.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkeletonJson {
    height: f64,
    rest_t: Vec<f64>,
    inverse_bind: Vec<f64>,
    pelvis_scale: f64,
    seat_drop: f64,
    eye_y: f64,
    grip: Vec<f64>,
    /// How far this stage's face takes a resident's identity (identity.rs `face_scale`).
    face_scale: f64,
}

fn skeleton_json(family: &str, stage: &str, meta: &Meta) -> SkeletonJson {
    let sk = skeleton(family, stage);
    SkeletonJson {
        height: py_round(sk.height, 4),
        rest_t: r4(&sk.rest_t),
        inverse_bind: r4(&sk.inverse_bind),
        pelvis_scale: py_round(sk.pelvis_scale, 5),
        seat_drop: py_round(meta.seat_drop, 4),
        eye_y: py_round(meta.eye_y, 4),
        grip: meta.grip.iter().map(|&v| py_round(v, 4)).collect(),
        face_scale: py_round(identity::face_scale(family, stage), 4),
    }
}

#[derive(Serialize)]
struct StageEntry {
    mesh: String,
    #[serde(flatten)]
    sk: SkeletonJson,
    parts: OMap<StagePart>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BodyEntry {
    mesh: String,
    stage: &'static str,
    #[serde(flatten)]
    sk: SkeletonJson,
    skin_ref: [i64; 3],
    garments: OMap<Cut>,
    body_chunks: OMap<[i64; 0]>,
    parts: OMap<PartEntry>,
    stages: OMap<StageEntry>,
}

/// SPIKE: level-2 values guessed from level 1's by the plainest stencils (a level-1 vertex keeps its own,
/// an edge's new vertex takes the mean of its two ends, a face's the mean of its four corners).
fn spike_stencil(v1: &[V3]) -> Vec<V3> {
    let topo = super::body::topology();
    let mut out = v1.to_vec();
    out.extend(topo.s2.edges.iter().map(|&(a, b)| (v1[a] + v1[b]) / 2.0));
    out.extend(
        topo.s
            .quads
            .iter()
            .map(|q| (v1[q[0]] + v1[q[1]] + v1[q[2]] + v1[q[3]]) / 4.0),
    );
    out
}

/// SPIKE: the largest and the root-mean-square difference of two lists, over the vertices `keep` picks.
fn spike_err(a: &[V3], b: &[V3], keep: &dyn Fn(usize) -> bool) -> (f64, f64) {
    let (mut mx, mut sq, mut n) = (0.0f64, 0.0, 0usize);
    for i in (0..a.len()).filter(|&i| keep(i)) {
        let d = (a[i] - b[i]).norm();
        mx = mx.max(d);
        sq += d * d;
        n += 1;
    }
    (mx, (sq / n.max(1) as f64).sqrt())
}

/// SPIKE: where the two lists differ most: the vertex, what it is (a level-1 vertex, an edge's or a face's
/// new one), its head / ear / neck shares, where it is and how deep its eye socket is carved there (mm).
fn spike_worst(family: &str, a: &[V3], b: &[V3]) -> String {
    let s = super::body::surface(family, BASE_STAGE);
    let topo = super::body::topology();
    let i = (0..a.len())
        .max_by(|&i, &j| (a[i] - b[i]).norm().total_cmp(&(a[j] - b[j]).norm()))
        .unwrap_or(0);
    let n1 = s.pos.len();
    let kind = if i < n1 {
        "level-1 vertex"
    } else if i < n1 + topo.s2.edges.len() {
        "edge vertex"
    } else {
        "face vertex"
    };
    let r = &s.fine_region[i];
    let p = s.fine_pos[i];
    format!(
        "worst at {kind} {i}: head {:.2} ear {:.2} neck {:.2}, at ({:.3}, {:.3}, {:.3}), socket {:.1} mm deep",
        r[super::body::region("head")],
        r[super::body::region("ear")],
        r[super::body::region("neck")],
        p.x,
        p.y,
        p.z,
        (s.fine_bare[i].z - p.z) * 1e3
    )
}

/// SPIKE: with `HD_SPIKE_DUMP=<dir>`, what a download would hold if level 2 were made at load time from
/// level 1 by the plain stencils and only the rest shipped: per new vertex (an edge's or a face's) how far
/// the real one is from the stencils' (int16 x 3 at `STAGE_SCALE`), for gzip to size.
fn spike_dump(name: &str, derived: &[V3], real: &[V3], n1: usize) {
    let Ok(dir) = std::env::var("HD_SPIKE_DUMP") else {
        return;
    };
    let bytes: Vec<u8> = (n1..real.len())
        .flat_map(|i| {
            let d = real[i] - derived[i];
            [d.x, d.y, d.z].map(|v| (v * STAGE_SCALE).round().clamp(-32767.0, 32767.0) as i16)
        })
        .flat_map(i16::to_le_bytes)
        .collect();
    std::fs::write(Path::new(&dir).join(format!("{name}.resid.bin")), bytes).expect("dump");
}

/// SPIKE: what the close-up level is made of and how well its morphs and stages could be derived at
/// load time from level 1's (printed by the build; nothing reads it).
fn spike_report(family: &str, base: &Assembled) {
    let s = super::body::surface(family, BASE_STAGE);
    let topo = super::body::topology();
    let (rh, re, rn) = (
        super::body::region("head"),
        super::body::region("ear"),
        super::body::region("neck"),
    );
    let share =
        |r: &[f64; super::body::NR], neck: bool| r[rh] + r[re] + if neck { r[rn] } else { 0.0 };
    let count = |reg: &[[f64; super::body::NR]], idx: &[usize], n: usize, neck: bool| {
        let v = reg.iter().filter(|r| share(r, neck) > 0.5).count();
        let t = idx
            .chunks(3)
            .filter(|t| t.iter().all(|&i| i < n && share(&reg[i], neck) > 0.5))
            .count();
        (v, t)
    };
    let (b1, b2) = (get(&base.parts, "body"), get(&base.parts, HD));
    let (n1, n2) = (base.n_skin, base.n_hd);
    println!(
        "SPIKE {family}: cage {} vertices; level 1 skin {n1} v, {} t; level 2 skin {n2} v, {} t; lids and lips {} v, {} t",
        topo.n,
        b1.idx
            .chunks(3)
            .filter(|t| t.iter().all(|&i| i < n1))
            .count(),
        b2.idx
            .chunks(3)
            .filter(|t| t.iter().all(|&i| i < n2))
            .count(),
        b1.pos.len() - n1,
        b1.idx
            .chunks(3)
            .filter(|t| t.iter().all(|&i| i >= n1))
            .count(),
    );
    for neck in [true, false] {
        let (v1, t1) = count(&s.region, &b1.idx, n1, neck);
        let (v2, t2) = count(&s.fine_region, &b2.idx, n2, neck);
        println!(
            "SPIKE {family}: head + ears{}: level 1 {v1} v, {t1} t; level 2 {v2} v, {t2} t",
            if neck { " + neck" } else { "" }
        );
    }
    // Level 2's first vertices against level 1's (they should be the same points).
    let same = spike_err(&s.pos, &s.fine_pos[..n1], &|_| true);
    let same_off_ears = spike_err(&s.pos, &s.fine_pos[..n1], &|i| s.fine_region[i][re] < 0.01);
    println!(
        "SPIKE {family}: level 2's first vertices are off level 1's by at most {:.4} mm ({:.4} mm away from the ears); {}",
        same.0 * 1e3,
        same_off_ears.0 * 1e3,
        spike_worst(family, &s.pos, &s.fine_pos[..n1])
    );
    spike_dump(
        &format!("{family}.base"),
        &spike_stencil(&s.pos),
        &s.fine_pos,
        n1,
    );
    // Positions made at load time instead: one Catmull-Clark step of level 1's points (they are limit
    // points already, so the step rounds the surface off a little), against the real level 2.
    let cc = topo.s2.m.apply(&s.pos);
    let headish = |i: usize| share(&s.fine_region[i], true) > 0.5;
    let ear = |i: usize| s.fine_region[i][re] > 0.5;
    for (what, keep) in [
        ("whole body", &(|_| true) as &dyn Fn(usize) -> bool),
        ("head + ears + neck", &headish),
        ("ears", &ear),
    ] {
        let e = spike_err(&cc, &s.fine_pos, keep);
        let l = spike_err(&spike_stencil(&s.pos), &s.fine_pos, keep);
        println!(
            "SPIKE {family}: level 2 from level 1 at load, {what}: Catmull-Clark step max {:.3} mm rms {:.3} mm; plain means max {:.3} mm rms {:.3} mm",
            e.0 * 1e3,
            e.1 * 1e3,
            l.0 * 1e3,
            l.1 * 1e3
        );
    }
    // Identity channels on level 2 derived from level 1's deltas by the plain stencils, against the real ones.
    let (mut worst_p, mut worst_n, mut worst_cc) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64), 0.0f64);
    for ch in identity::channels(family).iter() {
        let reach = ch.skin_hd.0.iter().map(|d| d.norm()).fold(0.0, f64::max);
        let p = spike_err(&spike_stencil(&ch.skin.0), &ch.skin_hd.0, &|_| true);
        let n = spike_err(&spike_stencil(&ch.skin.1), &ch.skin_hd.1, &|_| true);
        let c = spike_err(&topo.s2.m.apply(&ch.skin.0), &ch.skin_hd.0, &|_| true);
        let moved = |d: &[V3]| d.iter().filter(|v| v.abs().max() > 1e-5).count();
        println!(
            "SPIKE {family} {}: reach {:.2} mm, moves {} of level 1 and {} of level 2; derived positions off by max {:.3} mm rms {:.4} mm (Catmull-Clark step: max {:.3} mm); derived normals off by max {:.3} rms {:.4} (of 1)",
            ch.name,
            reach * 1e3,
            moved(&ch.skin.0),
            moved(&ch.skin_hd.0),
            p.0 * 1e3,
            p.1 * 1e3,
            c.0 * 1e3,
            n.0,
            n.1
        );
        if p.0 > 2e-4 {
            println!(
                "SPIKE {family} {}: {}",
                ch.name,
                spike_worst(family, &spike_stencil(&ch.skin.0), &ch.skin_hd.0)
            );
        }
        worst_p = (worst_p.0.max(p.0), worst_p.1.max(p.1));
        worst_n = (worst_n.0.max(n.0), worst_n.1.max(n.1));
        worst_cc = worst_cc.max(c.0);
    }
    println!(
        "SPIKE {family}: identity derived at load, worst channel: positions max {:.3} mm rms {:.4} mm (Catmull-Clark step max {:.3} mm); normals max {:.3} rms {:.4}",
        worst_p.0 * 1e3,
        worst_p.1 * 1e3,
        worst_cc * 1e3,
        worst_n.0,
        worst_n.1
    );
}

/// SPIKE: a life stage's level 2 derived at load time (the base's level 2 plus the stage's level-1
/// deltas spread by the plain stencils), against the real one.
fn spike_stage(family: &str, stage: &str, base: &Assembled, st: &Assembled) {
    let s = super::body::surface(family, BASE_STAGE);
    let (rh, re, rn) = (
        super::body::region("head"),
        super::body::region("ear"),
        super::body::region("neck"),
    );
    let headish =
        |i: usize| s.fine_region[i][rh] + s.fine_region[i][re] + s.fine_region[i][rn] > 0.5;
    let (n1, n2) = (base.n_skin, base.n_hd);
    let (b1, b2) = (get(&base.parts, "body"), get(&base.parts, HD));
    let (s1, s2) = (get(&st.parts, "body"), get(&st.parts, HD));
    let d1: Vec<V3> = (0..n1).map(|i| s1.pos[i] - b1.pos[i]).collect();
    let derived: Vec<V3> = spike_stencil(&d1)
        .iter()
        .zip(&b2.pos[..n2])
        .map(|(d, p)| p + d)
        .collect();
    let all = spike_err(&derived, &s2.pos[..n2], &|_| true);
    let hd = spike_err(&derived, &s2.pos[..n2], &headish);
    println!(
        "SPIKE {family} {stage}: {}",
        spike_worst(family, &derived, &s2.pos[..n2])
    );
    // (The other way to make a stage at load: the stage's own level 1 through the stencils, the rest shipped.)
    spike_dump(
        &format!("{family}.{stage}"),
        &spike_stencil(&s1.pos[..n1]),
        &s2.pos[..n2],
        n1,
    );
    spike_dump(
        &format!("{family}.{stage}.fromdelta"),
        &derived,
        &s2.pos[..n2],
        n1,
    );
    println!(
        "SPIKE {family} {stage}: level 2 derived from level 1's stage deltas: whole body max {:.3} mm rms {:.3} mm; head + ears + neck max {:.3} mm rms {:.3} mm",
        all.0 * 1e3,
        all.1 * 1e3,
        hd.0 * 1e3,
        hd.1 * 1e3
    );
}

fn build_family(family: &str, out: &Path) -> io::Result<BodyEntry> {
    let mut base = assemble(family, BASE_STAGE, None);
    let head = base.head.clone();
    // Fields and tone (base stage only; the stages share them).
    for name in ["body", "lod.body", HD] {
        let (f, t) = body_fields_and_tone(family, part(&mut base.parts, name), &base.p, &head);
        let m = part(&mut base.parts, name);
        m.fields = f;
        m.tone = t;
    }
    // The skin is baked as it is without the eye sockets: they lie under the lids, which carry the face on
    // over them (face.rs), so the skin round a lid and the lid take the shade the face has there.
    let socketed = part(&mut base.parts, "body").pos[..base.n_skin].to_vec();
    part(&mut base.parts, "body").pos[..base.n_skin].copy_from_slice(&base.unsocketed);
    let hd_socketed = part(&mut base.parts, HD).pos[..base.n_hd].to_vec();
    part(&mut base.parts, HD).pos[..base.n_hd].copy_from_slice(&base.hd_unsocketed);
    bake_ao(
        &mut base.parts,
        (base.n_skin, &|p: V3| p.y > head.chin_y - 0.1 * head.hh),
        base.n_hd,
    );
    part(&mut base.parts, "body").pos[..base.n_skin].copy_from_slice(&socketed);
    part(&mut base.parts, HD).pos[..base.n_hd].copy_from_slice(&hd_socketed);
    // The face's skin pieces: the lids take the tone of the face they lie on (its shade too; the low-detail
    // body's the mean of the skin round them), the upper lid's crease its own; the lips keep theirs.
    for (name, k0) in [
        ("body", base.n_skin),
        ("lod.body", base.n_lod),
        (HD, base.n_hd),
    ] {
        let on_skin = if name == HD {
            &base.hd_skin
        } else {
            &base.skin
        };
        let m = part(&mut base.parts, name);
        let near = KdTree::new(&m.pos[..k0]);
        let mut k = k0;
        for piece in &base.face.body {
            if name == "lod.body" && !piece.skin_tone {
                continue;
            }
            let n = piece.pos.len();
            for (i, q) in piece.pos.iter().enumerate() {
                m.tone[k + i] = if piece.skin_tone && name != "lod.body" {
                    // (The face's tone at the lid's nearest point, as the face is without the socket; the
                    // upper lid's crease its own.)
                    let (t, w) = on_skin.at(*q);
                    let at = |c: usize| (0..3).map(|j| m.tone[t[j]][c] * w[j]).sum::<f64>();
                    [at(0), piece.tone[i][1], at(2), at(3)]
                } else if piece.skin_tone {
                    // (The mean of the skin round each lid vertex, so a lid is blushed and shaded as its
                    // socket is; the upper lid's crease its own.)
                    let nb = near.query(q, 16);
                    let mut t = [0.0; 4];
                    for &(_, j) in &nb {
                        for (tc, &sc) in t.iter_mut().zip(&m.tone[j]) {
                            *tc += sc / nb.len() as f64;
                        }
                    }
                    t[1] = t[1].max(piece.tone[i][1]);
                    t
                } else {
                    let [x, y, z, _] = piece.tone[i];
                    [x, y, z, m.tone[k + i][3]]
                };
            }
            k += n;
        }
        // The ears, after them: their own blush and the shade in their bowl (baked with the body; as much
        // of it as `Ear::shade` says), and where they run into the head (`Ear::seam`) the tone of the skin
        // there, so the join shows no line.
        let set = if name == "lod.body" {
            &base.ears.lod
        } else {
            &base.ears.full
        };
        for ear in set {
            for (i, q) in ear.mesh.pos.iter().enumerate() {
                let skin: [f64; 4] = if name == "lod.body" {
                    m.tone[near.nearest(q)]
                } else {
                    let (t, w) = on_skin.at(*q);
                    std::array::from_fn(|c| (0..3).map(|j| m.tone[t[j]][c] * w[j]).sum())
                };
                let (own, w) = (ear.mesh.tone[i], ear.seam[i]);
                let shade = m.tone[k + i][3] * ear.shade[i];
                m.tone[k + i] = [
                    own[0] + (skin[0] - own[0]) * w,
                    own[1],
                    0.0,
                    shade + (skin[3] - shade) * w,
                ];
            }
            k += ear.mesh.pos.len();
        }
        assert_eq!(k, m.pos.len(), "{name}: pieces");
    }
    // Face morphs: the face's own parts, the body's lids (after its skin), the beard.
    let mut morphs: Vec<(String, Deltas)> = base.face.morphs.clone();
    if let Some(face_d) = base
        .face
        .morphs
        .iter()
        .find(|e| e.0 == "body")
        .map(|e| e.1.clone())
    {
        let n_lids: usize = base
            .face
            .body
            .iter()
            .filter(|m| m.skin_tone)
            .map(|m| m.pos.len())
            .sum();
        for (name, k, count) in [
            ("body", base.n_skin, None),
            ("lod.body", base.n_lod, Some(n_lids)),
            (HD, base.n_hd, None),
        ] {
            // (SPIKE: `hd.body`'s lids and lips move as `body`'s do, shaded by its own skin.)
            let (body, on_skin) = if name == HD {
                (get(&base.parts, HD), &base.hd_skin)
            } else {
                (get(&base.parts, "body"), &base.skin)
            };
            let shade = lid_shading(on_skin, &body.nor);
            let n = get(&base.parts, name).pos.len();
            let mut d = Deltas::new();
            for (c, (dp, dn)) in &face_d {
                let cnt = count.unwrap_or(dp.len());
                let (dp, dn) = (&dp[..cnt], &dn[..cnt]);
                // (`not max > 0`: an all-zero or empty delta moves nothing.)
                if !dp.iter().any(|v| v.abs().max() > 0.0) {
                    continue;
                }
                let mut fp = vec![V3::zeros(); n];
                let mut fnn = vec![V3::zeros(); n];
                fp[k..k + dp.len()].copy_from_slice(dp);
                fnn[k..k + dn.len()].copy_from_slice(dn);
                if count.is_none() {
                    // The body's lids are shaded the same way in every shape.
                    for i in k..k + n_lids {
                        fnn[i] = shade(body.pos[i] + fp[i]) - body.nor[i];
                    }
                }
                d.insert(c, (fp, fnn));
            }
            match morphs.iter_mut().find(|e| e.0 == name) {
                Some(e) => e.1 = d,
                None => morphs.push((name.to_string(), d)),
            }
        }
    }
    // (The beard's wisps as the beard: a wisp's root is a vertex of the beard's, and moves as it does.)
    for name in ["hair.beard".to_string(), wisps_part("hair.beard")] {
        if base.parts.iter().any(|e| e.0 == name) {
            let follow = base.face.follow.as_ref().expect("lip lines");
            let d = follow_mouth(part(&mut base.parts, &name), follow, &head);
            morphs.push((name, d));
        }
    }
    // A face of its own (identity.rs): each channel's shape on the skin with the lids and lips after it (and
    // the ears, which it leaves alone),
    // on the face's parts, and on the hair that lies on the face (the beard, a lock on a cheek, and the
    // wisps of either, by the same rule: a wisp's root moves as the vertex it grows from), which
    // takes the skin's shape where it lies (`HAIR_FOLLOWS`). Not the low-detail body.
    let on_face: Vec<(&str, Vec<OnSkin>)> = base
        .parts
        .iter()
        .filter(|e| e.0.starts_with("hair."))
        .map(|(name, m)| {
            let at = m.pos.iter().map(|&q| {
                let (t, w) = base.skin.at(q);
                let near = (0..3).fold(V3::zeros(), |a, j| a + base.unsocketed[t[j]] * w[j]);
                let k = if name.starts_with("hair.beard") {
                    1.0
                } else {
                    1.0 - smoothstep(HAIR_FOLLOWS.0, HAIR_FOLLOWS.1, (near - q).norm())
                };
                (t, w, k)
            });
            (name.as_str(), at.collect())
        })
        .collect();
    for ch in identity::channels(family).iter() {
        let mut shapes: Vec<(&str, Vec<V3>, Vec<V3>)> = Vec::new();
        for (name, dp, dn) in &ch.parts {
            if *name == "body" {
                // (The ears, the body's last vertices, stay as they are.)
                let join = |a: &[V3], b: &[V3]| {
                    let mut v = [a, b].concat();
                    v.resize(v.len() + 2 * super::ear::VERTS, V3::zeros());
                    v
                };
                shapes.push((name, join(&ch.skin.0, dp), join(&ch.skin.1, dn)));
                // SPIKE: the channel on the level-2 skin, the same lids, lips and ears after it.
                shapes.push((HD, join(&ch.skin_hd.0, dp), join(&ch.skin_hd.1, dn)));
            } else {
                shapes.push((name, dp.clone(), dn.clone()));
            }
        }
        for (name, at) in &on_face {
            let d = &ch.skin.0;
            let dp: Vec<V3> = at
                .iter()
                .map(|(t, w, k)| (d[t[0]] * w[0] + d[t[1]] * w[1] + d[t[2]] * w[2]) * *k)
                .map(|v| {
                    if v.abs().max() < identity::TRIM.0 {
                        V3::zeros()
                    } else {
                        v
                    }
                })
                .collect();
            let none = vec![V3::zeros(); dp.len()];
            shapes.push((name, dp, none));
        }
        for (name, dp, dn) in shapes {
            assert_eq!(dp.len(), get(&base.parts, name).pos.len(), "{name}");
            if !dp.iter().any(|v| v.abs().max() > 0.0) {
                continue;
            }
            match morphs.iter_mut().find(|e| e.0 == name) {
                Some(e) => {
                    e.1.insert(ch.name, (dp, dn));
                }
                None => {
                    let mut d = Deltas::new();
                    d.insert(ch.name, (dp, dn));
                    morphs.push((name.to_string(), d));
                }
            }
        }
    }
    let ch = channels();
    for (name, d) in &morphs {
        let m = part(&mut base.parts, name);
        if let Some(pm) = pack_morph(m.pos.len(), d, &ch) {
            m.morph = Some(pm);
        }
    }
    spike_report(family, &base);
    let table = write_parts(&out.join(format!("{family}.bin")), &base.parts)?;
    let mut stages = OMap::new();
    for &stage in &STAGES {
        if stage == BASE_STAGE {
            continue;
        }
        let st = assemble(family, stage, Some(&base.cuts));
        let mut a: Vec<&str> = st.parts.iter().map(|e| e.0.as_str()).collect();
        let mut b: Vec<&str> = base.parts.iter().map(|e| e.0.as_str()).collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "{family} {stage}: parts differ");
        let mesh = format!("{family}.{stage}.bin");
        spike_stage(family, stage, &base, &st);
        let t = write_stage(&out.join(&mesh), &base.parts, &st.parts)?;
        stages.insert(
            stage,
            StageEntry {
                mesh,
                sk: skeleton_json(family, stage, &st.meta),
                parts: t,
            },
        );
    }
    let mut chunks = OMap::new();
    chunks.insert("body", []);
    Ok(BodyEntry {
        mesh: format!("{family}.bin"),
        stage: BASE_STAGE,
        sk: skeleton_json(family, BASE_STAGE, &base.meta),
        skin_ref: [1, 1, 1],
        garments: base.cuts.clone(),
        body_chunks: chunks,
        parts: table,
        stages,
    })
}

#[derive(Serialize)]
struct Reference {
    body: &'static str,
    thigh: f64,
    shin: f64,
}

#[derive(Serialize)]
struct Morphs {
    channels: Vec<&'static str>,
    scale: f64,
    #[serde(rename = "normalScale")]
    normal_scale: f64,
    identity: IdentityJson,
}

/// The identity channels (identity.rs): the first of them among the channels, how many, and per body
/// the eyeballs a channel moves (at 1: the right eye's centre x, y, z in metres, the left's x mirrored,
/// and how much larger they are, of 1), which the runtime moves by their bones.
#[derive(Serialize)]
struct IdentityJson {
    first: usize,
    count: usize,
    eyes: OMap<OMap<Vec<f64>>>,
}

#[derive(Serialize)]
struct FaceBones {
    bones: [&'static str; 2],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Rig {
    version: i64,
    bones: Vec<&'static str>,
    parents: Vec<i64>,
    clip_bones: usize,
    face: FaceBones,
    head: usize,
    fps: i64,
    source_pelvis: Vec<f64>,
    source_rest_q: Vec<i64>,
    reference: Reference,
    field_scale: f64,
    tone_scale: f64,
    stage_scale: f64,
    morphs: Morphs,
    bodies: OMap<BodyEntry>,
}

/// Writes the set into `out` (rig.json and the binaries).
pub fn build(out: &Path) -> io::Result<()> {
    std::fs::create_dir_all(out)?;
    let mut bodies = OMap::new();
    for &family in &FAMILIES {
        bodies.insert(family, build_family(family, out)?);
    }
    let mut eyes = OMap::new();
    for &family in &FAMILIES {
        let mut moved = OMap::new();
        for ch in identity::channels(family).iter() {
            if ch.eye.iter().any(|v| v.abs() > 1e-9) {
                moved.insert(ch.name, r4(&ch.eye));
            }
        }
        eyes.insert(family, moved);
    }
    let (thigh, shin) = leg_lengths();
    let sp = source_pelvis();
    let rig = Rig {
        version: 4,
        bones: BONES.to_vec(),
        parents: parents(),
        clip_bones: CLIP_BONES,
        face: FaceBones { bones: FACE_BONES },
        head: head(),
        fps: 30,
        source_pelvis: r4(&[sp.x, sp.y, sp.z]),
        source_rest_q: (0..NB).flat_map(|_| [0, 0, 0, 1]).collect(),
        reference: Reference {
            body: "male",
            thigh: py_round(thigh, 5),
            shin: py_round(shin, 5),
        },
        field_scale: FIELD_SCALE,
        tone_scale: TONE_SCALE,
        stage_scale: STAGE_SCALE,
        morphs: Morphs {
            channels: channels(),
            scale: MORPH_SCALE,
            normal_scale: NORMAL_SCALE,
            identity: IdentityJson {
                first: channels().len() - IDENTITY.len(),
                count: IDENTITY.len(),
                eyes,
            },
        },
        bodies,
    };
    std::fs::write(out.join("rig.json"), to_string_py(&rig))?;
    println!("wrote {}", out.display());
    Ok(())
}
