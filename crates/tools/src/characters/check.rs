//! Checks a built character set against the contract (docs/characters-format.md) and the
//! runtime's names; fails on any violation: `cargo characters check [set dir]`. Runs after every
//! build.

use std::collections::BTreeSet;
use std::path::Path;

use regex::Regex;

use super::body;
use super::contract::{
    HAIR_STYLES, HD, IDENTITY, VISEMES, channels, garments, hair_parts, is_wisps, required_parts,
};
use super::hair::{EAR_MIN, Ears};
use super::head::head_frame;
use super::identity;
use super::json::JValue;
use super::np::{degrees, linspace_open, radians};
use super::rig::{
    BASE_STAGE, BONES, CLIP_BONES, RUNTIME_BONES, STAGES, proportions, target_height,
};
use super::{V3, repo_root, v3};

/// body: the skin (~4.8k), the lids and lips (~0.5k) and the ears (324 each, ear.rs).
/// lod.body: the cage at its limit (~0.9k) plus the eyelids (~0.2k) and plain ears (24 each).
///
/// hair: the styles are built at `hair::DETAIL` (1.5) times their layout's rows, each lock on 6
/// columns rather than 4 (`hair::LOCK`), so the
/// densest (`hair.layered`) comes to 7.3k and the rest to 2.1k-5.6k. It was 3k, which left the cap
/// and the curtains' locks at 40 columns and 8-16 rows: plates, with a polyline for a hem (see
/// `hair.rs`). At 5.2k the curtains still read as ribbons of six or nine, the width of a thumb at
/// arm's length; ten locks over four rows is the first count whose silhouette is a curve rather
/// than a few straight edges. `layered` is the tallest of them, three flared tiers under a cap, so
/// it sets the ceiling.
fn budget(name: &str) -> Option<usize> {
    match name {
        "body" => Some(6000),
        "lod.body" => Some(1400),
        // The close-up body: `body`'s skin split in four, its lids, lips and ears as they are (20,200).
        HD => Some(20_400),
        _ if is_wisps(name) => Some(WISPS_BUDGET),
        _ if name.starts_with("hair.") => Some(7400),
        _ if shell_re(true).is_match(name) => Some(1500),
        _ => None,
    }
}

/// A style's wisps: thin quads past its edges, drawn only close up (the afro's fins come nearest,
/// about 400).
const WISPS_BUDGET: usize = 600;

fn shell_re(lod: bool) -> Regex {
    Regex::new(if lod {
        r"^(lod\.)?(top|bottom|shoes)\."
    } else {
        r"^(top|bottom|shoes)\."
    })
    .unwrap()
}

fn cut_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "top" => &["hem", "sleeve"],
        "bottom" => &["leg", "waist"],
        _ => &["colour", "shoe"],
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The string items of `const NAME ... = [ ... ]` in TypeScript source.
fn ts_list(src: &str, name: &str) -> Vec<String> {
    let re = Regex::new(&format!(r"(?s){}[^=]*=\s*\[(.*?)\]", regex::escape(name))).unwrap();
    let item = Regex::new(r"'([^']+)'").unwrap();
    match re.captures(src) {
        Some(m) => item
            .captures_iter(&m[1])
            .map(|c| c[1].to_string())
            .collect(),
        None => Vec::new(),
    }
}

/// No identity channel moves anything further than this at 1 (m): a face of its own, not another head.
const IDENTITY_REACH: f64 = 0.006;

struct Runtime {
    faces: Vec<String>,
    units: Vec<String>,
    moods: BTreeSet<String>,
    visemes: Vec<String>,
    garments: [(String, BTreeSet<String>); 2],
    hair: Vec<String>,
}

fn runtime_names() -> Runtime {
    let web = repo_root().join("web").join("src");
    let face = read(&web.join("render/shared/characters/face.ts"));
    let units = ts_list(&face, "const UNITS");
    let moods_src = &face[face.find("export const MOODS").expect("MOODS")
        ..face
            .find("/** Idle micro-movements")
            .expect("micro-movements")];
    let mood_re = Regex::new(r"\b([a-z]+[A-Z][A-Za-z]+)\s*:").unwrap();
    let moods = mood_re
        .captures_iter(moods_src)
        .map(|c| c[1].to_string())
        .collect();
    let visemes = ts_list(&read(&web.join("voice/visemes.ts")), "export const VISEMES");
    let outfit = read(&web.join("render/shared/characters/outfit.ts"));
    let id_re = Regex::new(r"'([a-z]+\.[A-Za-z]+)'").unwrap();
    let fam_ids = |fam: &str| -> BTreeSet<String> {
        let mut ids = Vec::new();
        for table in ["TOPS", "BOTTOMS"] {
            let block_re =
                Regex::new(&format!(r"(?s)const {table}[^=]*=\s*\{{(.*?)\n\}};")).unwrap();
            let block = block_re
                .captures(&outfit)
                .unwrap_or_else(|| panic!("outfit.ts: no {table}"))[1]
                .to_string();
            // (Python: `(?<![a-z])fam:`; the regex crate has no look-behind.)
            let fam_re = Regex::new(&format!(r"(?s)(?:^|[^a-z]){fam}:\s*\[(.*?)\]")).unwrap();
            let list = fam_re
                .captures(&block)
                .unwrap_or_else(|| panic!("outfit.ts: no {fam} in {table}"))[1]
                .to_string();
            ids.extend(id_re.captures_iter(&list).map(|c| c[1].to_string()));
        }
        ids.extend(ts_list(&outfit, "const SHOE_PARTS"));
        // Workwear (the staff's apron) is a garment of every body but is not offered in the creator.
        ids.extend(ts_list(&outfit, "const WORKWEAR"));
        ids.into_iter().collect()
    };
    let garments = [
        ("male".to_string(), fam_ids("male")),
        ("female".to_string(), fam_ids("female")),
    ];
    let hair = ts_list(
        &read(&web.join("game/household.ts")),
        "export const HAIR_STYLES",
    );
    let faces = ts_list(
        &read(&web.join("game/household.ts")),
        "export const FACE_CHANNELS",
    );
    Runtime {
        faces,
        units,
        moods,
        visemes,
        garments,
        hair,
    }
}

/// A MOODS name is covered by the channel itself or by its Left / Right pair.
fn resolve(name: &str, ch: &[String]) -> bool {
    ch.iter().any(|c| c == name)
        || (ch.contains(&format!("{name}Left")) && ch.contains(&format!("{name}Right")))
}

/// Little-endian arrays of a binary.
fn f32s(buf: &[u8], at: usize, n: usize) -> Vec<f32> {
    buf[at..at + 4 * n]
        .chunks(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn i16s(buf: &[u8], at: usize, n: usize) -> Vec<i16> {
    buf[at..at + 2 * n]
        .chunks(2)
        .map(|c| i16::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn u16s(buf: &[u8], at: usize, n: usize) -> Vec<u16> {
    buf[at..at + 2 * n]
        .chunks(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

/// A part's morph of channel `c` (m per vertex), if it has one.
fn morph_of(buf: &[u8], part: &JValue, c: usize, scale: f64) -> Option<Vec<V3>> {
    let m = part.get("morph")?;
    let r: Vec<usize> = m.at("ranges").arr().iter().map(JValue::usize).collect();
    if r[c + 1] == r[c] {
        return None;
    }
    let verts = u16s(buf, m.at("morphVerts").usize(), m.at("count").usize());
    let slots = u16s(buf, m.at("morphSlots").usize(), r[c + 1]);
    let dp = i16s(buf, m.at("morphPositions").usize(), r[c + 1] * 4);
    let mut out = vec![V3::zeros(); part.at("vertices").usize()];
    for e in r[c]..r[c + 1] {
        out[verts[slots[e] as usize] as usize] = v3(
            dp[4 * e] as f64 / scale,
            dp[4 * e + 1] as f64 / scale,
            dp[4 * e + 2] as f64 / scale,
        );
    }
    Some(out)
}

/// Python's `repr` of a list of strings.
fn py_list(v: &[&str]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|s| format!("'{s}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Python's `{:.0%}`.
fn pct(v: f64) -> String {
    format!("{:.0}%", v * 100.0)
}

/// Runs every check on the set in `out`; prints `FAIL ...` lines (and returns false) or the summary.
pub fn check(out: &Path) -> bool {
    let rig = JValue::parse(&read(&out.join("rig.json"))).expect("rig.json");
    let rt = runtime_names();
    let mut errors: Vec<String> = Vec::new();
    let mut fail = |m: String| errors.push(m);

    // Skeleton.
    if rig.get("version").map(JValue::f64) != Some(5.0) {
        fail("rig.json is not version 5".into());
    }
    let bones = rig.at("bones").strs();
    if bones != BONES {
        fail("bone list differs from rig.py".into());
    }
    for b in RUNTIME_BONES {
        if !bones.iter().any(|x| x == b) {
            fail(format!("missing runtime bone {b}"));
        }
    }
    if rig
        .at("parents")
        .arr()
        .iter()
        .enumerate()
        .any(|(i, p)| p.f64() >= i as f64)
    {
        fail("a parent comes after its child".into());
    }
    if rig.at("clipBones").usize() != CLIP_BONES
        || bones
            .iter()
            .skip(CLIP_BONES)
            .any(|b| !b.starts_with("eye_"))
    {
        fail("clip bones: the eyes must come last".into());
    }
    let q = rig.at("sourceRestQ").f64s();
    let ident = [0.0, 0.0, 0.0, 1.0];
    if !q.len().is_multiple_of(4)
        || q.iter()
            .enumerate()
            .any(|(i, &v)| (v - ident[i % 4]).abs() > 1e-8 + 1e-5 * ident[i % 4])
    {
        fail("sourceRestQ is not identity".into());
    }

    // Channels.
    let ch = rig.at("morphs").at("channels").strs();
    if ch.len() < 15 || ch[..15] != rt.visemes[..] || rt.visemes != VISEMES {
        fail("visemes are not first, in the voice engine order".into());
    }
    for u in &rt.units {
        if !ch.contains(u) {
            fail(format!("missing face unit {u}"));
        }
    }
    for m in &rt.moods {
        if !resolve(m, &ch) && m != "mouthPressLeft" {
            fail(format!("MOODS name {m} has no channel"));
        }
    }
    if ch != channels() {
        fail("channel list differs from contract.py".into());
    }
    if rt.hair != HAIR_STYLES {
        fail("hairstyles differ from household.ts".into());
    }
    // The identity channels: the last of the channels, the creator's (household.ts) in its order.
    let (first, count) = match rig.at("morphs").get("identity") {
        Some(i) => (i.at("first").usize(), i.at("count").usize()),
        None => (ch.len(), 0),
    };
    if first + count != ch.len()
        || ch[first.min(ch.len())..] != IDENTITY
        || ch[..first.min(ch.len())]
            .iter()
            .any(|c| c.starts_with("id."))
    {
        fail("the identity channels are not the last channels".into());
    }
    if rt
        .faces
        .iter()
        .map(|c| format!("id.{c}"))
        .collect::<Vec<_>>()
        != IDENTITY
    {
        fail("identity channels differ from household.ts FACE_CHANNELS".into());
    }

    let mut moved: BTreeSet<String> = BTreeSet::new();
    let shell = shell_re(false);
    let mscale = rig.at("morphs").at("scale").f64();
    let stage_scale = rig.at("stageScale").f64();
    let tone_scale = rig.at("toneScale").f64();
    for (fam, body) in rig.at("bodies").obj() {
        let fam = fam.as_str();
        let buf = std::fs::read(out.join(body.at("mesh").str())).expect("mesh");
        let parts = body.at("parts");
        let pkeys = parts.keys();
        let mut need = required_parts();
        need.extend(hair_parts(fam));
        for p in &need {
            if !pkeys.contains(&p.as_str()) {
                fail(format!("{fam}: missing part {p}"));
            }
        }
        // Garments: the runtime's ids, every cut, a lod copy of every shell.
        let gkeys = body.at("garments").keys();
        let runtime_g = &rt
            .garments
            .iter()
            .find(|g| g.0 == fam)
            .map(|g| g.1.clone())
            .unwrap_or_default();
        if gkeys.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>() != *runtime_g
            || gkeys != garments(fam)
        {
            fail(format!(
                "{fam}: garment ids differ from outfit.ts / contract.py"
            ));
        }
        for (gid, cut) in body.at("garments").obj() {
            let have = cut.keys();
            let miss: Vec<&str> = cut_keys(gid.split('.').next().unwrap())
                .iter()
                .copied()
                .filter(|k| !have.contains(k))
                .collect();
            if !miss.is_empty() {
                fail(format!("{fam}: {gid} lacks {}", py_list(&miss)));
            }
        }
        for p in &pkeys {
            if shell.is_match(p) {
                if !gkeys.contains(p) {
                    fail(format!("{fam}: shell {p} has no garment entry"));
                }
                // (But a shell the shader cuts, the trousers and the regular tops: far off the low-detail body is painted.)
                let painted_far = *p == "bottom.trousers" || *p == "top.long";
                if pkeys.contains(&format!("lod.{p}").as_str()) == painted_far {
                    fail(format!(
                        "{fam}: shell {p} has {} lod copy",
                        if painted_far { "a" } else { "no" }
                    ));
                }
            }
        }
        let chunks_ok = matches!(body.get("bodyChunks"), Some(JValue::Obj(m)) if m.len() == 1 && m[0].0 == "body" && m[0].1 == JValue::Arr(Vec::new()));
        if !chunks_ok {
            fail(format!("{fam}: bodyChunks must be {{body: []}}"));
        }
        // The close-up body: in files of its own (play never fetches it), that one part, laid out as the
        // body's parts are and held to the same checks; its lids, lips and ears are `body`'s, where they are.
        if pkeys.iter().any(|p| p.starts_with("hd.")) {
            fail(format!(
                "{fam}: the close-up body is among the parts play fetches"
            ));
        }
        let hd = body.get("hd");
        let hbuf = hd.map(|h| std::fs::read(out.join(h.at("mesh").str())).expect("close-up mesh"));
        if hd.map(|h| h.at("parts").keys()) != Some(vec![HD]) {
            fail(format!("{fam}: no close-up body (hd: {HD})"));
        }
        let all: Vec<(&String, &JValue, &[u8])> = parts
            .obj()
            .iter()
            .map(|(name, e)| (name, e, &buf[..]))
            .chain(hd.iter().flat_map(|h| {
                let hbuf = hbuf.as_deref().expect("close-up mesh");
                h.at("parts")
                    .obj()
                    .iter()
                    .map(move |(name, e)| (name, e, hbuf))
            }))
            .collect();
        if let (Some(h), Some(b)) = (hd.and_then(|h| h.at("parts").get(HD)), parts.get("body")) {
            let hbuf = hbuf.as_deref().expect("close-up mesh");
            let (nh, nb) = (h.at("vertices").usize(), b.at("vertices").usize());
            let tail = nb - body::surface(fam, BASE_STAGE).pos.len();
            let (ph, pb) = (
                f32s(hbuf, h.at("positions").usize(), nh * 3),
                f32s(&buf, b.at("positions").usize(), nb * 3),
            );
            if nh < tail || ph[(nh - tail) * 3..] != pb[(nb - tail) * 3..] {
                fail(format!(
                    "{fam}:{HD}: its lids, lips and ears are not `body`'s"
                ));
            }
        }
        for &(name, e, buf) in &all {
            let n = e.at("vertices").usize();
            let tris = e.at("indices").usize() / 3;
            if n >= 65536 {
                fail(format!("{fam}:{name} has {n} vertices"));
            }
            let pos = f32s(buf, e.at("positions").usize(), n * 3);
            let w = &buf[e.at("weights").usize()..e.at("weights").usize() + n * 4];
            let idx = u16s(buf, e.at("index").usize(), tris * 3);
            if pos.iter().any(|v| !v.is_finite()) {
                fail(format!("{fam}:{name} has NaN positions"));
            }
            if w.chunks(4)
                .any(|c| c.iter().map(|&v| v as u32).sum::<u32>() != 255)
            {
                fail(format!("{fam}:{name} weights do not sum to 255"));
            }
            if idx.iter().any(|&i| i as usize >= n) {
                fail(format!("{fam}:{name} index out of range"));
            }
            // The baked occlusion (tone.w) is a share of the sky: 0..1 on every part.
            let tone = i16s(buf, e.at("tone").usize(), n * 4);
            if tone
                .chunks(4)
                .any(|t| !(0.0..=1.0).contains(&(t[3] as f64 / tone_scale)))
            {
                fail(format!("{fam}:{name} has tone.w outside 0..1"));
            }
            if let Some(bg) = budget(name)
                && tris > bg
            {
                fail(format!("{fam}:{name} has {tris} triangles (budget {bg})"));
            }
            // The game's front faces are clockwise in its left-handed space (kept under PlayCanvas's
            // mirrored root): cross(b - a, c - a) points against the normals.
            let nrm = i16s(buf, e.at("normals").usize(), n * 4);
            let pt = |i: u16| -> [f32; 3] {
                [
                    pos[3 * i as usize],
                    pos[3 * i as usize + 1],
                    pos[3 * i as usize + 2],
                ]
            };
            let nv = |i: u16| -> [f64; 3] {
                [
                    nrm[4 * i as usize] as f64,
                    nrm[4 * i as usize + 1] as f64,
                    nrm[4 * i as usize + 2] as f64,
                ]
            };
            let (mut big, mut wrong) = (0usize, 0usize);
            for t in idx.chunks(3) {
                let (a, b, c) = (pt(t[0]), pt(t[1]), pt(t[2]));
                let (u, v) = (
                    [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                    [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
                );
                let cr = [
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                ];
                if (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt() > 1e-10 {
                    big += 1;
                    let (n0, n1, n2) = (nv(t[0]), nv(t[1]), nv(t[2]));
                    let s: f64 = (0..3).map(|k| cr[k] as f64 * (n0[k] + n1[k] + n2[k])).sum();
                    if s > 0.0 {
                        wrong += 1;
                    }
                }
            }
            let frac = if big > 0 {
                wrong as f64 / big as f64
            } else {
                0.0
            };
            // (Not the hair's wisps: drawn from both sides and shaded by the normals of the hair they grow
            // from, whichever way a quad of theirs faces; a fin stands square to them.)
            if frac > 0.05 && !is_wisps(name) {
                fail(format!(
                    "{fam}:{name}: {} of the faces wind the wrong way (front faces are clockwise)",
                    pct(frac)
                ));
            }
            if let Some(m) = e.get("morph") {
                let r: Vec<usize> = m.at("ranges").arr().iter().map(JValue::usize).collect();
                for (k, c) in ch.iter().enumerate() {
                    if r[k + 1] > r[k] {
                        moved.insert(c.clone());
                    }
                }
            }
        }
        // Stages: every part, same topology, heights.
        for &stage in &STAGES {
            let info = if stage == BASE_STAGE {
                Some(body)
            } else {
                body.at("stages").get(stage)
            };
            let Some(info) = info else {
                fail(format!("{fam}: missing stage {stage}"));
                continue;
            };
            let target = target_height(fam, stage);
            let height = info.at("height").f64();
            if (height / target - 1.0).abs() > 0.03 {
                fail(format!(
                    "{fam}.{stage}: height {} is more than 3% off {}",
                    super::np::py_repr(height),
                    super::np::py_repr(target)
                ));
            }
            // A real body's heads in its height (docs/review/reference.md: about 4 on a baby, 5-6 on a
            // child, 6.5-7 on a teen, 7-7.5 on the grown).
            let heads = 1.0 / proportions(fam, stage).hh;
            let (lo, hi) = match stage {
                "baby" => (3.8, 4.3),
                "child" => (5.0, 6.0),
                "teen" => (6.5, 7.0),
                _ => (7.0, 7.5),
            };
            if !(lo..=hi).contains(&heads) {
                fail(format!(
                    "{fam}.{stage}: {heads:.2} heads tall, a real one is {lo} to {hi}"
                ));
            }
            let drop = info.at("seatDrop").f64() / height;
            if !(0.04 < drop && drop < 0.12) {
                fail(format!(
                    "{fam}.{stage}: seatDrop {} is implausible",
                    super::np::py_repr(info.at("seatDrop").f64())
                ));
            }
            if stage == BASE_STAGE {
                continue;
            }
            let sbuf = std::fs::read(out.join(info.at("mesh").str())).expect("stage mesh");
            let skeys: BTreeSet<&str> = info.at("parts").keys().into_iter().collect();
            if skeys != pkeys.iter().copied().collect::<BTreeSet<_>>() {
                fail(format!("{fam}.{stage}: parts differ from the base"));
            }
            for (name, e) in info.at("parts").obj() {
                let Some(base) = parts.get(name) else {
                    continue;
                };
                let end = e.at("positions").usize() + base.at("vertices").usize() * 8;
                if end > sbuf.len() {
                    fail(format!(
                        "{fam}.{stage}:{name} topology differs from the base"
                    ));
                }
            }
            // The stage's close-up body: its deltas, in a file of their own.
            let ok = match (info.get("hd"), hd.and_then(|h| h.at("parts").get(HD))) {
                (Some(sh), Some(base)) if sh.at("parts").keys() == vec![HD] => {
                    let e = sh.at("parts").at(HD);
                    let end = e.at("positions").usize() + base.at("vertices").usize() * 8;
                    let len = std::fs::metadata(out.join(sh.at("mesh").str())).map(|m| m.len());
                    len.is_ok_and(|len| end as u64 <= len)
                }
                _ => false,
            };
            if !ok {
                fail(format!(
                    "{fam}.{stage}: no close-up body, or not the base's topology"
                ));
            }
        }
        // Eyes in their sockets (base stage).
        let h = head_frame(fam, BASE_STAGE);
        if h.eyes.iter().any(|&e| h.sdf(e) >= 0.0) {
            fail(format!("{fam}: an eye centre is outside the skull"));
        }
        if h.eyes
            .iter()
            .any(|&e| h.sdf(e + v3(0.0, 0.0, h.eye_r)) <= 0.0)
        {
            fail(format!("{fam}: an eye does not show in front of the skull"));
        }
        // The sockets: no skin over the eyes' fronts (within 40 degrees of looking ahead), every stage.
        for &stage in &STAGES {
            let hs = head_frame(fam, stage);
            let a = linspace_open(0.0, 2.0 * std::f64::consts::PI, 12);
            for e in hs.eyes {
                for phi in [0.0, 20.0, 40.0].map(radians) {
                    let over = a
                        .iter()
                        .map(|&a| {
                            let p = e + v3(phi.sin() * a.cos(), phi.sin() * a.sin(), phi.cos())
                                * hs.eye_r;
                            hs.surface_z(p.x, p.y) - p.z
                        })
                        .fold(f64::NEG_INFINITY, f64::max);
                    if over > -0.0005 {
                        fail(format!(
                            "{fam}.{stage}: skin covers an eye ({:.1} mm at {:.0} degrees)",
                            over * 1000.0,
                            degrees(phi)
                        ));
                        break;
                    }
                }
            }
        }
        // Hair more than 2 mm off the head, and off the ears (no vertex in or within EAR_MIN of one,
        // no edge through one), at every life stage (stage positions: base + deltas).
        for &stage in &STAGES {
            let hs = if stage == BASE_STAGE {
                h.clone()
            } else {
                head_frame(fam, stage)
            };
            let info = if stage == BASE_STAGE {
                None
            } else {
                body.at("stages").get(stage)
            };
            let sbuf =
                info.map(|i| std::fs::read(out.join(i.at("mesh").str())).expect("stage mesh"));
            let at = |name: &str| -> Vec<V3> {
                let e = parts.at(name);
                let n = e.at("vertices").usize();
                let pos = f32s(&buf, e.at("positions").usize(), n * 3);
                let d = match (info, &sbuf) {
                    (Some(i), Some(sb)) => Some(i16s(
                        sb,
                        i.at("parts").at(name).at("positions").usize(),
                        n * 4,
                    )),
                    _ => None,
                };
                (0..n)
                    .map(|v| {
                        let mut p: V3 = v3(
                            pos[3 * v] as f64,
                            pos[3 * v + 1] as f64,
                            pos[3 * v + 2] as f64,
                        );
                        if let Some(d) = &d {
                            p += v3(d[4 * v] as f64, d[4 * v + 1] as f64, d[4 * v + 2] as f64)
                                / stage_scale;
                        }
                        p
                    })
                    .collect()
            };
            // The ears: sound (no corner of a triangle that shows shaded as facing away from it).
            let turned = super::measure::ear(fam, stage)[6];
            if turned > 0.0 {
                fail(format!(
                    "{fam}.{stage}: an ear is shaded inside out ({turned:.0} corners shaded as facing away)"
                ));
            }
            // The ears as the set has them: the body's last vertices (ear.rs), the generator's faces.
            let skin = body::surface(fam, stage);
            let ears = Ears::new(
                &hs.ears,
                super::ear::ears(fam, stage).faces(Some(&at("body"))),
            );
            // The head as a face of its own deforms it (identity.rs): each identity channel at its two
            // ends, as far as this stage takes it (`faceScale`), and the box of what it moves.
            let Some(scale) = info.unwrap_or(body).get("faceScale").map(JValue::f64) else {
                fail(format!("{fam}.{stage}: no faceScale"));
                continue;
            };
            let own: Vec<_> = identity::channels(fam)
                .iter()
                .flat_map(|c| [-1.0, 1.0].map(|end| (c, end * scale)))
                .filter_map(|(c, k)| {
                    let moved = hs.moved(&skin.head_tri_ix, &c.samples.0, &c.samples.1, k);
                    let at = c.samples.0.iter().zip(&hs.samples);
                    let (lo, hi) = at.filter(|(d, _)| d.norm() > 0.0).fold(
                        (V3::repeat(f64::INFINITY), V3::repeat(f64::NEG_INFINITY)),
                        |(lo, hi), (_, p)| (lo.inf(p), hi.sup(p)),
                    );
                    let pad = V3::repeat(0.015);
                    (lo.x <= hi.x).then_some((c.name, k, moved, lo - pad, hi + pad))
                })
                .collect();
            for (name, e) in parts.obj() {
                if !name.starts_with("hair.") {
                    continue;
                }
                let pos = at(name);
                let gap = pos.iter().map(|&p| hs.sdf(p)).fold(f64::INFINITY, f64::min);
                if gap < 0.002 {
                    fail(format!(
                        "{fam}.{stage}:{name} comes within 2 mm of the head ({:.1} mm)",
                        gap * 1000.0
                    ));
                }
                // Hair as far off a face of its own (the beard moves with the skin: its own morphs).
                let mut worst = (0.002, "", 0.0);
                for (c, k, moved, lo, hi) in &own {
                    let d = morph_of(
                        &buf,
                        e,
                        ch.iter().position(|x| x == c).expect("channel"),
                        mscale,
                    );
                    let gap = (0..pos.len())
                        .map(|v| pos[v] + d.as_ref().map_or(V3::zeros(), |d| d[v] * *k))
                        .filter(|p| (0..3).all(|a| lo[a] <= p[a] && p[a] <= hi[a]))
                        .map(|p| moved.sdf(p))
                        .fold(f64::INFINITY, f64::min);
                    if gap < worst.0 {
                        worst = (gap, c, *k);
                    }
                }
                if !worst.1.is_empty() {
                    fail(format!(
                        "{fam}.{stage}:{name} comes within 2 mm of the head with {} at {:+.2} ({:.1} mm)",
                        worst.1,
                        worst.2,
                        worst.0 * 1000.0
                    ));
                }
                let into: Vec<f64> = pos
                    .iter()
                    .filter_map(|&p| ears.depth(p))
                    .map(|e| e.0)
                    .filter(|&d| d > -EAR_MIN)
                    .collect();
                let idx = u16s(&buf, e.at("index").usize(), e.at("indices").usize());
                let through = idx
                    .chunks(3)
                    .flat_map(|t| [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])])
                    .filter(|&(a, b)| a < b && ears.cut(pos[a as usize], pos[b as usize]))
                    .count();
                if !into.is_empty() || through > 0 {
                    let deep = into.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    fail(format!(
                        "{fam}.{stage}:{name} passes into an ear ({} vertices, {:.1} mm deep; {through} edges through)",
                        into.len(),
                        deep * 1000.0
                    ));
                }
            }
        }
        // Face morphs: small, finite shapes (no channel moves anything further than 4 cm, no identity
        // channel further than `IDENTITY_REACH`, and none of those the low-detail parts).
        for &(name, e, buf) in &all {
            if let Some(m) = e.get("morph") {
                let r: Vec<usize> = m.at("ranges").arr().iter().map(JValue::usize).collect();
                let dp = i16s(buf, m.at("morphPositions").usize(), r[ch.len()] * 4);
                // No channel moves an ear (the last vertices of the bodies that have the whole ones).
                let n = e.at("vertices").usize();
                let moved = u16s(buf, m.at("morphVerts").usize(), m.at("count").usize());
                if matches!(name.as_str(), "body" | HD)
                    && moved
                        .iter()
                        .any(|&v| v as usize >= n - 2 * super::ear::VERTS)
                {
                    fail(format!("{fam}:{name} has a channel that moves an ear"));
                }
                let reach = |from: usize, to: usize| {
                    dp[4 * r[from]..4 * r[to]]
                        .chunks(4)
                        .flat_map(|c| c[..3].iter().map(|&v| (v as f64 / mscale).abs()))
                        .fold(0.0, f64::max)
                };
                let mx = reach(0, ch.len());
                if mx > 0.04 {
                    fail(format!(
                        "{fam}:{name} has a morph moving {:.1} cm",
                        mx * 100.0
                    ));
                }
                for k in first.min(ch.len())..ch.len() {
                    if reach(k, k + 1) > IDENTITY_REACH {
                        fail(format!(
                            "{fam}:{name}: {} moves {:.1} mm (at most {:.0})",
                            ch[k],
                            reach(k, k + 1) * 1000.0,
                            IDENTITY_REACH * 1000.0
                        ));
                    }
                    if name.starts_with("lod.") && r[k + 1] > r[k] {
                        fail(format!("{fam}:{name} has the identity channel {}", ch[k]));
                    }
                }
            }
        }
    }
    for c in &ch {
        if c != "sil" && !moved.contains(c) {
            fail(format!("channel {c} moves nothing"));
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            println!("FAIL {e}");
        }
        return false;
    }
    println!(
        "check: {} ok ({} bodies, {} channels)",
        out.display(),
        rig.at("bodies").obj().len(),
        ch.len()
    );
    true
}
