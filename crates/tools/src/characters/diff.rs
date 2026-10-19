//! `cargo characters diff <a> <b>`: decodes two character sets (docs/characters-format.md) and
//! reports the largest difference per part and array. It fails when the topology, an index array
//! or rig.json's structure, keys or bones differ, when f32 positions differ by more than 1e-4 m,
//! when a quantised i16 / i8 / u8 value differs by more than one unit, or when a rig.json float
//! differs by more than 1e-5 relative. Weights are compared as joint -> weight maps (a different
//! joint at weight 0 is no difference).

use std::collections::BTreeMap;
use std::path::Path;

use super::json::JValue;

const POS_TOL: f64 = 1e-4;
const REL_TOL: f64 = 1e-5;

/// The largest difference seen per array kind, and the failures.
#[derive(Default)]
pub struct Report {
    /// kind -> (largest difference, where).
    pub max: BTreeMap<String, (f64, String)>,
    pub failures: Vec<String>,
    /// Per part and array, the largest difference (for the listing).
    pub rows: Vec<(String, String, f64)>,
}

impl Report {
    fn note(&mut self, kind: &str, at: &str, part: &str, d: f64) {
        self.rows.push((part.to_string(), kind.to_string(), d));
        let e = self.max.entry(kind.to_string()).or_insert((0.0, String::new()));
        if d > e.0 || e.1.is_empty() {
            *e = (d.max(e.0), at.to_string());
        }
    }

    fn fail(&mut self, m: String) {
        self.failures.push(m);
    }
}

fn le<const N: usize>(buf: &[u8], at: usize, n: usize, what: &str) -> Option<Vec<[u8; N]>> {
    let end = at.checked_add(n * N)?;
    if end > buf.len() {
        eprintln!("{what}: out of the file");
        return None;
    }
    Some(buf[at..end].chunks(N).map(|c| c.try_into().unwrap()).collect())
}

/// rig.json, recursively: same keys, lengths, strings and ints; floats within REL_TOL.
fn cmp_json(a: &JValue, b: &JValue, path: &str, r: &mut Report, worst: &mut (f64, String)) {
    match (a, b) {
        (JValue::Obj(x), JValue::Obj(y)) => {
            let kx: Vec<&str> = x.iter().map(|e| e.0.as_str()).collect();
            let ky: Vec<&str> = y.iter().map(|e| e.0.as_str()).collect();
            if kx != ky {
                r.fail(format!("rig.json {path}: keys differ ({kx:?} vs {ky:?})"));
            }
            for (k, v) in x {
                if let Some((_, w)) = y.iter().find(|e| &e.0 == k) {
                    cmp_json(v, w, &format!("{path}.{k}"), r, worst);
                }
            }
        }
        (JValue::Arr(x), JValue::Arr(y)) => {
            if x.len() != y.len() {
                r.fail(format!("rig.json {path}: {} vs {} items", x.len(), y.len()));
                return;
            }
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                cmp_json(v, w, &format!("{path}[{i}]"), r, worst);
            }
        }
        (JValue::Int(x), JValue::Int(y)) => {
            if x != y {
                r.fail(format!("rig.json {path}: {x} vs {y}"));
            }
        }
        (JValue::Float(x), JValue::Float(y)) => {
            let rel = if x == y { 0.0 } else { (x - y).abs() / x.abs().max(y.abs()) };
            if rel > worst.0 {
                *worst = (rel, path.to_string());
            }
            if rel > REL_TOL {
                r.fail(format!("rig.json {path}: {x} vs {y} ({rel:.2e} relative)"));
            }
        }
        _ => {
            if a != b {
                r.fail(format!("rig.json {path}: {a:?} vs {b:?}"));
            }
        }
    }
}

/// The largest |a - b| of two arrays of numbers.
fn max_abs(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

fn as_i16(v: &[[u8; 2]]) -> Vec<f64> {
    v.iter().map(|c| i16::from_le_bytes(*c) as f64).collect()
}

fn as_u16(v: &[[u8; 2]]) -> Vec<u16> {
    v.iter().map(|c| u16::from_le_bytes(*c)).collect()
}

fn as_i8(v: &[[u8; 1]]) -> Vec<f64> {
    v.iter().map(|c| c[0] as i8 as f64).collect()
}

/// Compares one quantised array (at most one unit apart).
fn quantised(r: &mut Report, kind: &str, part: &str, a: &[f64], b: &[f64]) {
    let d = max_abs(a, b);
    r.note(kind, part, part, d);
    if d > 1.0 {
        r.fail(format!("{part}: {kind} differ by {d} units"));
    }
}

/// Compares one index array (exactly).
fn exact(r: &mut Report, kind: &str, part: &str, a: &[u16], b: &[u16]) {
    let n = a.iter().zip(b).filter(|(x, y)| x != y).count();
    r.note(kind, part, part, n as f64);
    if n > 0 || a.len() != b.len() {
        r.fail(format!("{part}: {kind} differ at {n} places"));
    }
}

fn cmp_part(r: &mut Report, label: &str, ea: &JValue, eb: &JValue, ba: &[u8], bb: &[u8]) {
    let n = ea.at("vertices").usize();
    let ni = ea.at("indices").usize();
    if n != eb.at("vertices").usize() || ni != eb.at("indices").usize() {
        r.fail(format!("{label}: topology differs ({n} / {ni} vs {} / {})", eb.at("vertices").usize(), eb.at("indices").usize()));
        return;
    }
    let get = |e: &JValue, k: &str| e.at(k).usize();
    let (Some(pa), Some(pb)) = (le::<4>(ba, get(ea, "positions"), n * 3, label), le::<4>(bb, get(eb, "positions"), n * 3, label)) else {
        r.fail(format!("{label}: positions out of range"));
        return;
    };
    let f = |v: &[[u8; 4]]| -> Vec<f64> { v.iter().map(|c| f32::from_le_bytes(*c) as f64).collect() };
    let d = max_abs(&f(&pa), &f(&pb));
    r.note("positions (m)", label, label, d);
    if d > POS_TOL {
        r.fail(format!("{label}: positions differ by {d:.3e} m"));
    }
    for (k, kind) in [("normals", "normals i16"), ("fields", "fields i16"), ("tone", "tone i16")] {
        match (le::<2>(ba, get(ea, k), n * 4, label), le::<2>(bb, get(eb, k), n * 4, label)) {
            (Some(x), Some(y)) => quantised(r, kind, label, &as_i16(&x), &as_i16(&y)),
            _ => r.fail(format!("{label}: {k} out of range")),
        }
    }
    // Weights as joint -> weight maps.
    let (ja, wa, jb, wb) = (le::<4>(ba, get(ea, "joints"), n, label), le::<4>(ba, get(ea, "weights"), n, label), le::<4>(bb, get(eb, "joints"), n, label), le::<4>(bb, get(eb, "weights"), n, label));
    if let (Some(ja), Some(wa), Some(jb), Some(wb)) = (ja, wa, jb, wb) {
        let mut d = 0.0f64;
        for v in 0..n {
            let mut m: BTreeMap<u8, (f64, f64)> = BTreeMap::new();
            for s in 0..4 {
                m.entry(ja[v][s]).or_default().0 += wa[v][s] as f64;
                m.entry(jb[v][s]).or_default().1 += wb[v][s] as f64;
            }
            for (x, y) in m.values() {
                d = d.max((x - y).abs());
            }
        }
        r.note("weights u8", label, label, d);
        if d > 1.0 {
            r.fail(format!("{label}: weights differ by {d} units"));
        }
    } else {
        r.fail(format!("{label}: joints / weights out of range"));
    }
    match (le::<2>(ba, get(ea, "index"), ni, label), le::<2>(bb, get(eb, "index"), ni, label)) {
        (Some(x), Some(y)) => exact(r, "index", label, &as_u16(&x), &as_u16(&y)),
        _ => r.fail(format!("{label}: index out of range")),
    }
    match (ea.get("morph"), eb.get("morph")) {
        (None, None) => {}
        (Some(ma), Some(mb)) => {
            let (ca, cb) = (ma.at("count").usize(), mb.at("count").usize());
            let ra: Vec<usize> = ma.at("ranges").arr().iter().map(JValue::usize).collect();
            let rb: Vec<usize> = mb.at("ranges").arr().iter().map(JValue::usize).collect();
            if ca != cb || ra != rb {
                r.fail(format!("{label}: morph topology differs (count {ca} vs {cb}, {} vs {} entries)", ra.last().unwrap_or(&0), rb.last().unwrap_or(&0)));
                return;
            }
            let e = *ra.last().unwrap_or(&0);
            let g = |m: &JValue, k: &str| m.at(k).usize();
            match (le::<2>(ba, g(ma, "morphVerts"), ca, label), le::<2>(bb, g(mb, "morphVerts"), ca, label)) {
                (Some(x), Some(y)) => exact(r, "morphVerts", label, &as_u16(&x), &as_u16(&y)),
                _ => r.fail(format!("{label}: morphVerts out of range")),
            }
            match (le::<2>(ba, g(ma, "morphSlots"), e, label), le::<2>(bb, g(mb, "morphSlots"), e, label)) {
                (Some(x), Some(y)) => exact(r, "morphSlots", label, &as_u16(&x), &as_u16(&y)),
                _ => r.fail(format!("{label}: morphSlots out of range")),
            }
            match (le::<2>(ba, g(ma, "morphPositions"), e * 4, label), le::<2>(bb, g(mb, "morphPositions"), e * 4, label)) {
                (Some(x), Some(y)) => quantised(r, "morphPositions i16", label, &as_i16(&x), &as_i16(&y)),
                _ => r.fail(format!("{label}: morphPositions out of range")),
            }
            match (le::<1>(ba, g(ma, "morphNormals"), e * 4, label), le::<1>(bb, g(mb, "morphNormals"), e * 4, label)) {
                (Some(x), Some(y)) => quantised(r, "morphNormals i8", label, &as_i8(&x), &as_i8(&y)),
                _ => r.fail(format!("{label}: morphNormals out of range")),
            }
        }
        _ => r.fail(format!("{label}: morph in one set only")),
    }
}

fn cmp_stage_part(r: &mut Report, label: &str, n: usize, ea: &JValue, eb: &JValue, ba: &[u8], bb: &[u8]) {
    let get = |e: &JValue, k: &str| e.at(k).usize();
    match (le::<2>(ba, get(ea, "positions"), n * 4, label), le::<2>(bb, get(eb, "positions"), n * 4, label)) {
        (Some(x), Some(y)) => quantised(r, "stage positions i16", label, &as_i16(&x), &as_i16(&y)),
        _ => r.fail(format!("{label}: stage positions out of range")),
    }
    match (le::<1>(ba, get(ea, "normals"), n * 4, label), le::<1>(bb, get(eb, "normals"), n * 4, label)) {
        (Some(x), Some(y)) => quantised(r, "stage normals i8", label, &as_i8(&x), &as_i8(&y)),
        _ => r.fail(format!("{label}: stage normals out of range")),
    }
}

fn load(dir: &Path) -> Result<JValue, String> {
    let s = std::fs::read_to_string(dir.join("rig.json")).map_err(|e| format!("{}: {e}", dir.display()))?;
    JValue::parse(&s).map_err(|e| format!("{}: rig.json: {e}", dir.display()))
}

/// Compares the sets in `a` and `b`.
pub fn diff(a: &Path, b: &Path) -> Result<Report, String> {
    let (ra, rb) = (load(a)?, load(b)?);
    let mut r = Report::default();
    let mut worst = (0.0, String::new());
    cmp_json(&ra, &rb, "", &mut r, &mut worst);
    r.max.insert("rig.json floats (relative)".into(), worst);
    if ra.at("bones") != rb.at("bones") {
        r.fail("bones differ".into());
    }
    let read = |dir: &Path, f: &str| std::fs::read(dir.join(f)).map_err(|e| format!("{}: {e}", dir.join(f).display()));
    for (fam, body_a) in ra.at("bodies").obj() {
        let Some(body_b) = rb.at("bodies").get(fam) else { continue };
        let (ba, bb) = (read(a, body_a.at("mesh").str())?, read(b, body_b.at("mesh").str())?);
        for (name, ea) in body_a.at("parts").obj() {
            let Some(eb) = body_b.at("parts").get(name) else { continue };
            cmp_part(&mut r, &format!("{fam}:{name}"), ea, eb, &ba, &bb);
        }
        for (stage, sa) in body_a.at("stages").obj() {
            let Some(sb) = body_b.at("stages").get(stage) else { continue };
            let (ba, bb) = (read(a, sa.at("mesh").str())?, read(b, sb.at("mesh").str())?);
            for (name, ea) in sa.at("parts").obj() {
                let (Some(eb), Some(base)) = (sb.at("parts").get(name), body_a.at("parts").get(name)) else { continue };
                cmp_stage_part(&mut r, &format!("{fam}.{stage}:{name}"), base.at("vertices").usize(), ea, eb, &ba, &bb);
            }
        }
    }
    Ok(r)
}

/// Prints the report: per part and array the largest difference, then the largest per array kind.
pub fn print(r: &Report, verbose: bool) {
    if verbose {
        for (part, kind, d) in &r.rows {
            println!("{part:<40} {kind:<22} {d}");
        }
    }
    println!("largest difference per array:");
    for (kind, (d, at)) in &r.max {
        println!("  {kind:<28} {d:<12.4e} {at}");
    }
    for f in &r.failures {
        println!("FAIL {f}");
    }
    if r.failures.is_empty() {
        println!("diff: ok");
    }
}
