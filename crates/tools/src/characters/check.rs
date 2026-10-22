//! Checks a built character set against the contract (docs/characters-format.md) and the
//! runtime's names; fails on any violation: `cargo characters check [set dir]`. Runs after every
//! build.

use std::collections::BTreeSet;
use std::path::Path;

use regex::Regex;

use super::contract::{HAIR_STYLES, VISEMES, channels, garments, hair_parts, required_parts};
use super::head::head_frame;
use super::json::JValue;
use super::np::{degrees, linspace_open, radians};
use super::rig::{BASE_STAGE, BONES, CLIP_BONES, RUNTIME_BONES, STAGES, target_height};
use super::{V3, repo_root, v3};

/// lod.body: the cage at its limit (~0.9k) plus the eyelids (~0.2k).
fn budget(name: &str) -> Option<usize> {
    match name {
        "body" => Some(4500),
        "lod.body" => Some(1200),
        _ if name.starts_with("hair.") => Some(3000),
        _ if shell_re(true).is_match(name) => Some(1500),
        _ => None,
    }
}

fn shell_re(lod: bool) -> Regex {
    Regex::new(if lod { r"^(lod\.)?(top|bottom|shoes)\." } else { r"^(top|bottom|shoes)\." }).unwrap()
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
        Some(m) => item.captures_iter(&m[1]).map(|c| c[1].to_string()).collect(),
        None => Vec::new(),
    }
}

struct Runtime {
    units: Vec<String>,
    moods: BTreeSet<String>,
    visemes: Vec<String>,
    garments: [(String, BTreeSet<String>); 2],
    hair: Vec<String>,
}

fn runtime_names() -> Runtime {
    let web = repo_root().join("web").join("src");
    let face = read(&web.join("render/babylon/characters/face.ts"));
    let units = ts_list(&face, "const UNITS");
    let moods_src = &face[face.find("export const MOODS").expect("MOODS")..face.find("/** Idle micro-movements").expect("micro-movements")];
    let mood_re = Regex::new(r"\b([a-z]+[A-Z][A-Za-z]+)\s*:").unwrap();
    let moods = mood_re.captures_iter(moods_src).map(|c| c[1].to_string()).collect();
    let visemes = ts_list(&read(&web.join("voice/visemes.ts")), "export const VISEMES");
    let outfit = read(&web.join("render/babylon/characters/outfit.ts"));
    let id_re = Regex::new(r"'([a-z]+\.[A-Za-z]+)'").unwrap();
    let fam_ids = |fam: &str| -> BTreeSet<String> {
        let mut ids = Vec::new();
        for table in ["TOPS", "BOTTOMS"] {
            let block_re = Regex::new(&format!(r"(?s)const {table}[^=]*=\s*\{{(.*?)\n\}};")).unwrap();
            let block = block_re.captures(&outfit).unwrap_or_else(|| panic!("outfit.ts: no {table}"))[1].to_string();
            // (Python: `(?<![a-z])fam:`; the regex crate has no look-behind.)
            let fam_re = Regex::new(&format!(r"(?s)(?:^|[^a-z]){fam}:\s*\[(.*?)\]")).unwrap();
            let list = fam_re.captures(&block).unwrap_or_else(|| panic!("outfit.ts: no {fam} in {table}"))[1].to_string();
            ids.extend(id_re.captures_iter(&list).map(|c| c[1].to_string()));
        }
        ids.extend(ts_list(&outfit, "const SHOE_PARTS"));
        ids.into_iter().collect()
    };
    let garments = [("male".to_string(), fam_ids("male")), ("female".to_string(), fam_ids("female"))];
    let hair = ts_list(&read(&web.join("game/household.ts")), "export const HAIR_STYLES");
    Runtime { units, moods, visemes, garments, hair }
}

/// A MOODS name is covered by the channel itself or by its Left / Right pair.
fn resolve(name: &str, ch: &[String]) -> bool {
    ch.iter().any(|c| c == name) || (ch.contains(&format!("{name}Left")) && ch.contains(&format!("{name}Right")))
}

/// Little-endian arrays of a binary.
fn f32s(buf: &[u8], at: usize, n: usize) -> Vec<f32> {
    buf[at..at + 4 * n].chunks(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()
}

fn i16s(buf: &[u8], at: usize, n: usize) -> Vec<i16> {
    buf[at..at + 2 * n].chunks(2).map(|c| i16::from_le_bytes(c.try_into().unwrap())).collect()
}

fn u16s(buf: &[u8], at: usize, n: usize) -> Vec<u16> {
    buf[at..at + 2 * n].chunks(2).map(|c| u16::from_le_bytes(c.try_into().unwrap())).collect()
}

/// Python's `repr` of a list of strings.
fn py_list(v: &[&str]) -> String {
    format!("[{}]", v.iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(", "))
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
    if rig.get("version").map(JValue::f64) != Some(4.0) {
        fail("rig.json is not version 4".into());
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
    if rig.at("parents").arr().iter().enumerate().any(|(i, p)| p.f64() >= i as f64) {
        fail("a parent comes after its child".into());
    }
    if rig.at("clipBones").usize() != CLIP_BONES || bones.iter().skip(CLIP_BONES).any(|b| !b.starts_with("eye_")) {
        fail("clip bones: the eyes must come last".into());
    }
    let q = rig.at("sourceRestQ").f64s();
    let ident = [0.0, 0.0, 0.0, 1.0];
    if !q.len().is_multiple_of(4) || q.iter().enumerate().any(|(i, &v)| (v - ident[i % 4]).abs() > 1e-8 + 1e-5 * ident[i % 4]) {
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

    let mut moved: BTreeSet<String> = BTreeSet::new();
    let shell = shell_re(false);
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
        let runtime_g = &rt.garments.iter().find(|g| g.0 == fam).map(|g| g.1.clone()).unwrap_or_default();
        if gkeys.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>() != *runtime_g || gkeys != garments(fam) {
            fail(format!("{fam}: garment ids differ from outfit.ts / contract.py"));
        }
        for (gid, cut) in body.at("garments").obj() {
            let have = cut.keys();
            let miss: Vec<&str> = cut_keys(gid.split('.').next().unwrap()).iter().copied().filter(|k| !have.contains(k)).collect();
            if !miss.is_empty() {
                fail(format!("{fam}: {gid} lacks {}", py_list(&miss)));
            }
        }
        for p in &pkeys {
            if shell.is_match(p) {
                if !gkeys.contains(p) {
                    fail(format!("{fam}: shell {p} has no garment entry"));
                }
                if !pkeys.contains(&format!("lod.{p}").as_str()) {
                    fail(format!("{fam}: shell {p} has no lod copy"));
                }
            }
        }
        let chunks_ok = matches!(body.get("bodyChunks"), Some(JValue::Obj(m)) if m.len() == 1 && m[0].0 == "body" && m[0].1 == JValue::Arr(Vec::new()));
        if !chunks_ok {
            fail(format!("{fam}: bodyChunks must be {{body: []}}"));
        }
        for (name, e) in parts.obj() {
            let n = e.at("vertices").usize();
            let tris = e.at("indices").usize() / 3;
            if n >= 65536 {
                fail(format!("{fam}:{name} has {n} vertices"));
            }
            let pos = f32s(&buf, e.at("positions").usize(), n * 3);
            let w = &buf[e.at("weights").usize()..e.at("weights").usize() + n * 4];
            let idx = u16s(&buf, e.at("index").usize(), tris * 3);
            if pos.iter().any(|v| !v.is_finite()) {
                fail(format!("{fam}:{name} has NaN positions"));
            }
            if w.chunks(4).any(|c| c.iter().map(|&v| v as u32).sum::<u32>() != 255) {
                fail(format!("{fam}:{name} weights do not sum to 255"));
            }
            if idx.iter().any(|&i| i as usize >= n) {
                fail(format!("{fam}:{name} index out of range"));
            }
            // The baked occlusion (tone.w) is a share of the sky: 0..1 on every part.
            let tone = i16s(&buf, e.at("tone").usize(), n * 4);
            if tone.chunks(4).any(|t| !(0.0..=1.0).contains(&(t[3] as f64 / tone_scale))) {
                fail(format!("{fam}:{name} has tone.w outside 0..1"));
            }
            if let Some(bg) = budget(name)
                && tris > bg
            {
                fail(format!("{fam}:{name} has {tris} triangles (budget {bg})"));
            }
            // Babylon front faces are clockwise: cross(b - a, c - a) points against the normals.
            let nrm = i16s(&buf, e.at("normals").usize(), n * 4);
            let pt = |i: u16| -> [f32; 3] { [pos[3 * i as usize], pos[3 * i as usize + 1], pos[3 * i as usize + 2]] };
            let nv = |i: u16| -> [f64; 3] { [nrm[4 * i as usize] as f64, nrm[4 * i as usize + 1] as f64, nrm[4 * i as usize + 2] as f64] };
            let (mut big, mut wrong) = (0usize, 0usize);
            for t in idx.chunks(3) {
                let (a, b, c) = (pt(t[0]), pt(t[1]), pt(t[2]));
                let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
                let cr = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                if (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt() > 1e-10 {
                    big += 1;
                    let (n0, n1, n2) = (nv(t[0]), nv(t[1]), nv(t[2]));
                    let s: f64 = (0..3).map(|k| cr[k] as f64 * (n0[k] + n1[k] + n2[k])).sum();
                    if s > 0.0 {
                        wrong += 1;
                    }
                }
            }
            let frac = if big > 0 { wrong as f64 / big as f64 } else { 0.0 };
            if frac > 0.05 {
                fail(format!("{fam}:{name}: {} of the faces wind the wrong way for Babylon", pct(frac)));
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
            let info = if stage == BASE_STAGE { Some(body) } else { body.at("stages").get(stage) };
            let Some(info) = info else {
                fail(format!("{fam}: missing stage {stage}"));
                continue;
            };
            let target = target_height(fam, stage);
            let height = info.at("height").f64();
            if (height / target - 1.0).abs() > 0.03 {
                fail(format!("{fam}.{stage}: height {} is more than 3% off {}", super::np::py_repr(height), super::np::py_repr(target)));
            }
            let drop = info.at("seatDrop").f64() / height;
            if !(0.04 < drop && drop < 0.12) {
                fail(format!("{fam}.{stage}: seatDrop {} is implausible", super::np::py_repr(info.at("seatDrop").f64())));
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
                let Some(base) = parts.get(name) else { continue };
                let end = e.at("positions").usize() + base.at("vertices").usize() * 8;
                if end > sbuf.len() {
                    fail(format!("{fam}.{stage}:{name} topology differs from the base"));
                }
            }
        }
        // Eyes in their sockets (base stage).
        let h = head_frame(fam, BASE_STAGE);
        if h.eyes.iter().any(|&e| h.sdf(e) >= 0.0) {
            fail(format!("{fam}: an eye centre is outside the skull"));
        }
        if h.eyes.iter().any(|&e| h.sdf(e + v3(0.0, 0.0, h.eye_r)) <= 0.0) {
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
                            let p = e + v3(phi.sin() * a.cos(), phi.sin() * a.sin(), phi.cos()) * hs.eye_r;
                            hs.surface_z(p.x, p.y) - p.z
                        })
                        .fold(f64::NEG_INFINITY, f64::max);
                    if over > -0.0005 {
                        fail(format!("{fam}.{stage}: skin covers an eye ({:.1} mm at {:.0} degrees)", over * 1000.0, degrees(phi)));
                        break;
                    }
                }
            }
        }
        // Hair more than 2 mm off the head at every life stage (stage positions: base + deltas).
        for &stage in &STAGES {
            let hs = if stage == BASE_STAGE { h.clone() } else { head_frame(fam, stage) };
            let info = if stage == BASE_STAGE { None } else { body.at("stages").get(stage) };
            let sbuf = info.map(|i| std::fs::read(out.join(i.at("mesh").str())).expect("stage mesh"));
            for (name, e) in parts.obj() {
                if !name.starts_with("hair.") {
                    continue;
                }
                let n = e.at("vertices").usize();
                let pos = f32s(&buf, e.at("positions").usize(), n * 3);
                let d = match (info, &sbuf) {
                    (Some(i), Some(sb)) => Some(i16s(sb, i.at("parts").at(name).at("positions").usize(), n * 4)),
                    _ => None,
                };
                let gap = (0..n)
                    .map(|v| {
                        let mut p: V3 = v3(pos[3 * v] as f64, pos[3 * v + 1] as f64, pos[3 * v + 2] as f64);
                        if let Some(d) = &d {
                            p += v3(d[4 * v] as f64, d[4 * v + 1] as f64, d[4 * v + 2] as f64) / stage_scale;
                        }
                        hs.sdf(p)
                    })
                    .fold(f64::INFINITY, f64::min);
                if gap < 0.002 {
                    fail(format!("{fam}.{stage}:{name} comes within 2 mm of the head ({:.1} mm)", gap * 1000.0));
                }
            }
        }
        // Face morphs: small, finite shapes (no channel moves anything further than 4 cm).
        let mscale = rig.at("morphs").at("scale").f64();
        for (name, e) in parts.obj() {
            if let Some(m) = e.get("morph") {
                let total = m.at("ranges").arr().last().unwrap();
                let dp = i16s(&buf, m.at("morphPositions").usize(), total.usize() * 4);
                let mx = dp.chunks(4).flat_map(|c| c[..3].iter().map(|&v| (v as f64 / mscale).abs())).fold(0.0, f64::max);
                if mx > 0.04 {
                    fail(format!("{fam}:{name} has a morph moving {:.1} cm", mx * 100.0));
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
    println!("check: {} ok ({} bodies, {} channels)", out.display(), rig.at("bodies").obj().len(), ch.len());
    true
}
