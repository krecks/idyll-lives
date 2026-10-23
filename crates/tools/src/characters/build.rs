//! Builds the residents' character set from code (rig.json version 4 and its binaries; see
//! docs/characters-format.md): `cargo characters build [out dir]` (default
//! web/public/assets/characters).
//!
//! Per family and life stage it asks the head interface (head.rs) where the face is, then builds
//! the body (body.rs), the face (face.rs: eyes, lids, lashes, brows, mouth and the morph channels),
//! the hair (hair.rs) and the clothes (clothes.rs), merges the face's skin pieces into the body,
//! bakes the base stage's ambient occlusion into tone.w (ao.rs; the stages share it), and writes
//! the base stage (young adult) whole and the other stages as deltas. Then the checks
//! run. Everything is deterministic (seeded).

use std::io;
use std::path::Path;
use std::sync::Arc;

use serde::Serialize;

use super::ao;
use super::body::{Meta, body_fields_and_tone, build_body};
use super::clothes::{Cut, Cuts, build_clothes, garment_cuts};
use super::contract::channels;
use super::face::{Deltas, FaceOut, build_face, follow_mouth};
use super::hair::build_hair;
use super::head::{HeadFrame, head_frame};
use super::json::{OMap, to_string_py};
use super::kdtree::KdTree;
use super::meshops::OnMesh;
use super::np::py_round;
use super::rig::{BASE_STAGE, BONES, CLIP_BONES, FACE_BONES, FAMILIES, NB, STAGES, head, leg_lengths, parents, skeleton, source_pelvis};
use super::write::{FIELD_SCALE, MORPH_SCALE, NORMAL_SCALE, PartEntry, STAGE_SCALE, StagePart, TONE_SCALE, pack_morph, r4, write_parts, write_stage};
use super::{Mesh, V3, merge};

/// Every part of a family at a stage (no fields / tone / morphs yet), with the body's metadata.
struct Assembled {
    parts: Vec<(String, Mesh)>,
    meta: Meta,
    p: [V3; NB],
    head: Arc<HeadFrame>,
    face: FaceOut,
    n_skin: usize,
    n_lod: usize,
    cuts: Cuts,
    /// The body's skin positions without its eye sockets (the occlusion bake), and its skin triangles on
    /// them (`lid_shading`, the lids' tone).
    unsocketed: Vec<V3>,
    skin: OnMesh,
}

fn part<'a>(parts: &'a mut [(String, Mesh)], name: &str) -> &'a mut Mesh {
    &mut parts.iter_mut().find(|e| e.0 == name).unwrap_or_else(|| panic!("no part {name}")).1
}

fn get<'a>(parts: &'a [(String, Mesh)], name: &str) -> &'a Mesh {
    &parts.iter().find(|e| e.0 == name).unwrap_or_else(|| panic!("no part {name}")).1
}

fn assemble(family: &str, stage: &str, cuts: Option<&Cuts>) -> Assembled {
    let head = head_frame(family, stage);
    let bd = build_body(family, stage);
    let p = bd.joints;
    let f = build_face(family, stage, head.clone());
    let n_skin = bd.body.pos.len();
    let n_lod = bd.lod.pos.len();
    let extras: Vec<Mesh> = f.body.iter().map(|m| Mesh { arm: vec![0.0; m.pos.len()], ..m.clone() }).collect();
    let strip = |m: &Mesh| Mesh { pos: m.pos.clone(), nor: m.nor.clone(), dense: m.dense.clone(), arm: m.arm.clone(), idx: m.idx.clone(), ..Mesh::default() };
    let mut body_list: Vec<Mesh> = vec![strip(&bd.body)];
    body_list.extend(extras.iter().map(strip));
    // The low-detail body has the eyelids too (they come first among the face's skin pieces).
    let mut lod_list: Vec<Mesh> = vec![strip(&bd.lod)];
    lod_list.extend(extras.iter().filter(|m| m.skin_tone).map(strip));
    let mut parts = vec![("body".to_string(), merge(&body_list.iter().collect::<Vec<_>>())), ("lod.body".to_string(), merge(&lod_list.iter().collect::<Vec<_>>()))];
    let n_lids: usize = extras.iter().filter(|m| m.skin_tone).map(Mesh::len).sum();
    let skin = OnMesh::new(&bd.unsocketed, &parts[0].1.idx, n_skin);
    let lids: Vec<V3> = {
        let shade = lid_shading(&skin, &parts[0].1.nor);
        parts[0].1.pos[n_skin..n_skin + n_lids].iter().map(|&q| shade(q)).collect()
    };
    parts[0].1.nor[n_skin..n_skin + n_lids].copy_from_slice(&lids);
    parts.extend(f.parts.iter().cloned());
    parts.extend(build_hair(family, stage, &head, &p));
    let cuts = match cuts {
        Some(c) => c.clone(),
        None => garment_cuts(family, &p),
    };
    parts.extend(build_clothes(family, stage, &cuts));
    Assembled { parts, meta: bd.meta, p, head, face: f, n_skin, n_lod, cuts, unsocketed: bd.unsocketed, skin }
}

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
/// lids and lips are in them) and the mouth (not the eyes: the lower lids would take a dark crescent
/// from the eyeball); the shells by the body (the low-detail ones by the low-detail body) and
/// themselves; the hair and beard by the body, the eyes and themselves. On the face (the skin, the first
/// `spread.0` of the body, where `spread.1` says) each vertex's shade is spread over its triangles
/// (`ao::occlusion_spread`). The lids then take the shade of the skin they lie on (`build_family`).
fn bake_ao(parts: &mut [(String, Mesh)], spread: (usize, &dyn Fn(V3) -> bool)) {
    let (body, lod) = (get(parts, "body").clone(), get(parts, "lod.body").clone());
    let face = ["eyes", "teeth", "tongue"].map(|n| get(parts, n).clone());
    let [eyes, teeth, tongue] = &face;
    for (name, m) in parts.iter_mut() {
        let around: Vec<&Mesh> = if name == "body" || name == "lod.body" {
            vec![teeth, tongue]
        } else if name.starts_with("lod.") {
            vec![&lod]
        } else if ["top.", "bottom.", "shoes."].iter().any(|p| name.starts_with(p)) {
            vec![&body]
        } else if name.starts_with("hair.") {
            vec![&body, eyes]
        } else {
            continue;
        };
        let occ = if name == "body" { ao::occlusion_spread(m, &around, ao::REACH, spread.0, spread.1) } else { ao::occlusion(m, &around, ao::REACH) };
        ao::bake(m, occ);
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

fn build_family(family: &str, out: &Path) -> io::Result<BodyEntry> {
    let mut base = assemble(family, BASE_STAGE, None);
    let head = base.head.clone();
    // Fields and tone (base stage only; the stages share them).
    for name in ["body", "lod.body"] {
        let (f, t) = body_fields_and_tone(family, part(&mut base.parts, name), &base.p, &head);
        let m = part(&mut base.parts, name);
        m.fields = f;
        m.tone = t;
    }
    // The skin is baked as it is without the eye sockets: they lie under the lids, which carry the face on
    // over them (face.rs), so the skin round a lid and the lid take the shade the face has there.
    let socketed = part(&mut base.parts, "body").pos[..base.n_skin].to_vec();
    part(&mut base.parts, "body").pos[..base.n_skin].copy_from_slice(&base.unsocketed);
    bake_ao(&mut base.parts, (base.n_skin, &|p: V3| p.y > head.chin_y - 0.1 * head.hh));
    part(&mut base.parts, "body").pos[..base.n_skin].copy_from_slice(&socketed);
    // The face's skin pieces: the lids take the tone of the face they lie on (its shade too; the low-detail
    // body's the mean of the skin round them), the upper lid's crease its own; the lips keep theirs.
    for (name, k0) in [("body", base.n_skin), ("lod.body", base.n_lod)] {
        let m = part(&mut base.parts, name);
        let near = KdTree::new(&m.pos[..k0]);
        let mut k = k0;
        for piece in &base.face.body {
            if name == "lod.body" && !piece.skin_tone {
                continue;
            }
            let n = piece.pos.len();
            for (i, q) in piece.pos.iter().enumerate() {
                m.tone[k + i] = if piece.skin_tone && name == "body" {
                    // (The face's tone at the lid's nearest point, as the face is without the socket; the
                    // upper lid's crease its own.)
                    let (t, w) = base.skin.at(*q);
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
    }
    // Face morphs: the face's own parts, the body's lids (after its skin), the beard.
    let mut morphs: Vec<(String, Deltas)> = base.face.morphs.clone();
    if let Some(face_d) = base.face.morphs.iter().find(|e| e.0 == "body").map(|e| e.1.clone()) {
        let n_lids: usize = base.face.body.iter().filter(|m| m.skin_tone).map(|m| m.pos.len()).sum();
        let body = get(&base.parts, "body");
        let shade = lid_shading(&base.skin, &body.nor);
        for (name, k, count) in [("body", base.n_skin, None), ("lod.body", base.n_lod, Some(n_lids))] {
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
    if base.parts.iter().any(|e| e.0 == "hair.beard") {
        let follow = base.face.follow.as_ref().expect("lip lines");
        let d = follow_mouth(part(&mut base.parts, "hair.beard"), follow, &head);
        morphs.push(("hair.beard".into(), d));
    }
    let ch = channels();
    for (name, d) in &morphs {
        let m = part(&mut base.parts, name);
        if let Some(pm) = pack_morph(m.pos.len(), d, &ch) {
            m.morph = Some(pm);
        }
    }
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
        let t = write_stage(&out.join(&mesh), &base.parts, &st.parts)?;
        stages.insert(stage, StageEntry { mesh, sk: skeleton_json(family, stage, &st.meta), parts: t });
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
        reference: Reference { body: "male", thigh: py_round(thigh, 5), shin: py_round(shin, 5) },
        field_scale: FIELD_SCALE,
        tone_scale: TONE_SCALE,
        stage_scale: STAGE_SCALE,
        morphs: Morphs { channels: channels(), scale: MORPH_SCALE, normal_scale: NORMAL_SCALE },
        bodies,
    };
    std::fs::write(out.join("rig.json"), to_string_py(&rig))?;
    println!("wrote {}", out.display());
    Ok(())
}
