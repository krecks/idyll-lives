//! The transforms a family script sets by hand on what a builder returned (an Euler turn in any order, a
//! quaternion, a whole matrix, an origin moved on a turned part, a part put inside another, a mode changed), against
//! what Blender 5.2 makes of the same lines: tests/golden/api_transform.txt holds, for each case, the part's place,
//! turn, scale and world matrix after each step, its mesh once joined, its clips' keys and what the exporter wrote.

mod common;

use artkit::forms::rounded_rect;
use artkit::glb;
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::{Axis, M3, M4, PI, Quat, radians, v3};
use artkit::motion::{self, Path};
use artkit::parts::{hinged, join, set_origin};
use artkit::scene::{self, Obj, RotationMode};
use artkit::{cube, cylinder, grow, jitter, revolve, rod, sphere, spin, sweep, swing, torus};
use common::{f32s, golden, ulps};
use std::collections::BTreeMap;

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");
const WOOD: Mat = Mat::new("wood", "#8E7A6A", "wood");

/// One case's lines: (kind, label, the rest).
type Lines = Vec<(String, String, Vec<String>)>;

fn cases() -> BTreeMap<String, Lines> {
    let mut out: BTreeMap<String, Lines> = BTreeMap::new();
    let mut case = String::new();
    for line in golden("api_transform.txt").lines() {
        let w: Vec<&str> = line.split(' ').collect();
        if w[0] == "case" {
            case = w[1].to_string();
            continue;
        }
        let rest = w[2..].iter().map(|s| s.to_string()).collect();
        out.entry(case.clone())
            .or_default()
            .push((w[0].to_string(), w[1].to_string(), rest));
    }
    out
}

/// How far from Blender's a number may be for a part put inside a parent that is turned and scaled unevenly: its
/// own matrix in the parent's frame is then sheared, and one of the Euler angles read back from it comes out a unit
/// in the last place from Blender's (its world matrix with it). Everything else is Blender's to the bit.
const SHEARED: f32 = 1e-7;

/// What a case's checks found: how many numbers were compared, and those not Blender's (to the bit, or within
/// `reach` where it is set).
#[derive(Default)]
struct Found {
    compared: usize,
    reach: f32,
    off: Vec<String>,
}

impl Found {
    /// Compares numbers with a golden's; a zero's sign is not a difference.
    fn same(&mut self, what: &str, got: &[f32], want: &[f32]) {
        assert_eq!(got.len(), want.len(), "{what}: how many");
        for (i, (g, w)) in got.iter().zip(want).enumerate() {
            self.compared += 1;
            if g != w && (g - w).abs() > self.reach {
                self.off.push(format!(
                    "{what}[{i}]: {g:e}, Blender's {w:e} ({} in the last place)",
                    ulps(*g, *w)
                ));
            }
        }
    }
}

struct Case<'a> {
    name: &'a str,
    lines: &'a Lines,
    found: Found,
}

impl Case<'_> {
    fn line(&self, kind: &str, label: &str) -> Vec<String> {
        let l = self
            .lines
            .iter()
            .find(|(k, l, _)| k == kind && l == label)
            .unwrap_or_else(|| panic!("{}: no {kind} {label}", self.name));
        l.2.clone()
    }

    /// The part as it stands against a golden \`state\` line: its mode, place, turn, scale and world matrix
    /// (\`matrix\` false: Blender's matrix on that line is one the kernel does not keep).
    fn state_with(&mut self, o: Obj, label: &str, matrix: bool) {
        let w = self.line("state", label);
        let w: Vec<&str> = w.iter().map(String::as_str).collect();
        let what = format!("{} {label}", self.name);
        let mode = match o.rotation_mode() {
            RotationMode::Quaternion => "QUATERNION".to_string(),
            m => format!("{m:?}"),
        };
        assert_eq!(mode, w[0], "{what}: mode");
        let field = |key: &str, n: usize| -> Vec<f32> {
            let at = w.iter().position(|x| *x == key).unwrap();
            f32s(&w[at + 1..at + 1 + n])
        };
        self.found.same(
            &format!("{what} loc"),
            &o.location().array(),
            &field("loc", 3),
        );
        self.found.same(
            &format!("{what} eul"),
            &o.rotation_euler().array(),
            &field("eul", 3),
        );
        if o.rotation_mode() == RotationMode::Quaternion {
            self.found.same(
                &format!("{what} quat"),
                &o.rotation_quaternion().array(),
                &field("quat", 4),
            );
        }
        self.found.same(
            &format!("{what} scale"),
            &o.scale().array(),
            &field("scale", 3),
        );
        if matrix {
            self.found.same(
                &format!("{what} matrix"),
                &o.matrix_world().rows(),
                &field("mw", 16),
            );
        }
    }

    fn state(&mut self, o: Obj, label: &str) {
        self.state_with(o, label, true);
    }

    /// The part's points against a golden \`verts\` line, in order.
    fn verts(&mut self, o: Obj, label: &str) {
        let w = self.line("verts", label);
        let want = f32s(&w.iter().map(String::as_str).collect::<Vec<_>>());
        let got: Vec<f32> = o.read(|p| p.mesh.verts.iter().flat_map(|v| v.array()).collect());
        self.found
            .same(&format!("{} {label} points", self.name), &got, &want);
    }

    /// The part's clips' keys against the golden \`keys\` lines.
    fn keys(&mut self, o: Obj, label: &str) {
        let lines: Vec<&(String, String, Vec<String>)> = self
            .lines
            .iter()
            .filter(|(k, l, _)| k == "keys" && l == label)
            .collect();
        let tracks = o.read(|p| p.tracks.clone());
        let channels: usize = tracks.iter().map(|t| t.channels.len()).sum();
        assert_eq!(channels, lines.len(), "{} {label}: channels", self.name);
        for (_, _, w) in lines {
            let path = match w[1].as_str() {
                "location" => Path::Location,
                "rotation_euler" => Path::Rotation,
                _ => Path::Scale,
            };
            let index: usize = w[2].parse().unwrap();
            let track = tracks.iter().find(|t| t.name == w[0]).unwrap();
            let ch = track
                .channels
                .iter()
                .find(|c| c.path == path && c.index == index)
                .unwrap();
            let want: Vec<(i32, f32)> = w[3..]
                .iter()
                .map(|k| {
                    let (f, v) = k.split_once(':').unwrap();
                    (f.parse().unwrap(), f32s(&[v])[0])
                })
                .collect();
            let frames: Vec<i32> = ch.keys.iter().map(|k| k.frame).collect();
            assert_eq!(
                frames,
                want.iter().map(|k| k.0).collect::<Vec<_>>(),
                "{} {label}: frames",
                self.name
            );
            let values: Vec<f32> = ch.keys.iter().map(|k| k.value).collect();
            let wanted: Vec<f32> = want.iter().map(|k| k.1).collect();
            self.found
                .same(&format!("{} {label} keys", self.name), &values, &wanted);
        }
    }
}

fn finishes() -> Finishes {
    let mut f = Finishes::new();
    for (name, textured) in [("gloss", false), ("wood", true)] {
        f.insert(
            name.to_string(),
            Finish {
                roughness: 0.5,
                metallic: 0.0,
                textured,
            },
        );
    }
    f
}

fn a_rod(name: &str) -> Obj {
    rod!(
        name,
        (0.1, 0.2, 0.3),
        (0.4, -0.1, 0.9),
        0.02,
        material = BODY,
        verts = 8
    )
}

fn build(c: &mut Case) {
    scene::reset();
    scene::set_finishes(finishes());
    let order_of = |name: &str| match name {
        "XZY" => RotationMode::XZY,
        "YXZ" => RotationMode::YXZ,
        "YZX" => RotationMode::YZX,
        "ZXY" => RotationMode::ZXY,
        _ => RotationMode::ZYX,
    };
    match c.name {
        "euler_xyz" => {
            let o = cube!(
                "c",
                (0.2, 0.3, 0.4),
                at = (0.5, -0.25, 0.75),
                material = BODY,
                bevel = 0
            );
            o.set_rotation_euler((0.3, -0.7, 1.9));
            o.set_scale((1.5, 0.5, 2.0));
            c.state(o, "c");
            o.set_rotation_euler_axis(1, radians(37.0));
            o.set_scale_axis(2, 0.001);
            o.set_location_axis(1, o.location().y as f64 - 0.07);
            o.set_location_axis(2, o.location().z as f64 + 0.0123);
            o.set_location_axis(0, 0.4 - o.location().x as f64 / 3.0);
            c.state(o, "c2");
            c.verts(join(&[o], "j", None), "j");
        }
        name if name.starts_with("euler_") => {
            let o = cube!(
                "c",
                (0.2, 0.3, 0.4),
                at = (0.5, -0.25, 0.75),
                material = BODY,
                bevel = 0
            );
            o.set_rotation_mode(order_of(&name[6..]));
            o.set_rotation_euler((0.3, -0.7, 1.9));
            o.set_scale_axis(1, 0.25);
            c.state(o, "c");
            let j = join(&[o], "j", None);
            c.state(j, "j");
            c.verts(j, "j");
        }
        "quat_rain" => {
            let parts: Vec<Obj> = (0..3)
                .map(|i| {
                    cylinder!(
                        format!("rain{i}"),
                        0.005,
                        0.6,
                        at = (0.02 * i as f64, 0.01 * i as f64, 0.9),
                        material = BODY,
                        verts = 6,
                        smooth = false
                    )
                })
                .collect();
            let rain = join(&parts, "water", Some((0, 0, 1.2).into()));
            c.state(rain, "joined");
            let aim = v3(0.13, -0.21, -0.8);
            rain.set_rotation_mode(RotationMode::Quaternion);
            rain.set_rotation_quaternion(v3(0, 0, -1).rotation_difference(aim));
            rain.set_scale_axis(2, 0.001);
            c.state(rain, "rain");
        }
        "quat_chain" => {
            let (a, b) = (v3(0.1, -0.3, 1.4), v3(0.35, 0.2, 0.2));
            let d = b - a;
            let q0 = d.to_track_quat(Axis::X, Axis::Z);
            let mut made = Vec::new();
            for k in 0..2 {
                let at = a + d * ((k as f64 + 0.5) / 2.0);
                let link = torus!(
                    format!("link{k}"),
                    0.011,
                    0.0028,
                    at = at,
                    material = BODY,
                    major_segments = 8,
                    minor_segments = 3
                );
                link.set_rotation_mode(RotationMode::Quaternion);
                link.set_rotation_quaternion(
                    q0 * Quat::from_axis_angle((1, 0, 0), PI / 2.0 * (k % 2) as f64),
                );
                link.set_scale((d.length() / 2.0 / (2.0 * 0.011) * 1.25, 0.75, 1.0));
                c.state(link, &format!("link{k}"));
                made.push(link);
            }
            let j = join(&made, "chain", None);
            c.state(j, "chain");
            c.verts(j, "chain");
        }
        "quat_plume" => {
            let (base, along) = (v3(0.05, -0.02, 0.6), v3(0.12, 0.3, 0.45));
            let plume = sphere!(
                "plume",
                0.045,
                at = base + along.normalized() * 0.045 * 3.2 * 0.25,
                material = BODY,
                scale = (1, 1, 3.2),
                segments = 12,
                rings = 10
            );
            plume.set_rotation_mode(RotationMode::Quaternion);
            plume.set_rotation_quaternion(along.to_track_quat_zy());
            jitter!(plume, 0.004, size = 0.02, seed = 2.3 + 0.045);
            c.state(plume, "plume");
            c.verts(plume, "plume");
            c.verts(join(&[plume], "j", None), "j");
        }
        "mw_board" => {
            let (a, b, face) = (v3(0.2, -0.1, 0.05), v3(0.5, 0.25, 0.85), v3(0.3, -1, 0.1));
            let z = (b - a).normalized();
            let x = face.cross(z).normalized();
            let y = z.cross(x);
            let o = cube!(
                "board",
                (0.08, 0.02, (b - a).length()),
                material = WOOD,
                bevel = 0.0
            );
            let rot = M3([x.array(), y.array(), z.array()]);
            o.set_matrix_world(M4::translation((a + b) / 2) * rot.to_4x4());
            // (read back before an update, Blender's matrix is the one given, not the part's own put together)
            c.state_with(o, "unread", false);
            c.state(o, "board");
            c.verts(join(&[o], "j", None), "j");
        }
        "mw_turn" => {
            let parts = [
                cube!(
                    "p0",
                    (0.1, 0.2, 0.3),
                    at = (0.3, 0.1, 0.4),
                    material = BODY,
                    rot = (10, 20, 30),
                    bevel = 0
                ),
                cylinder!(
                    "p1",
                    0.05,
                    0.4,
                    at = (-0.2, 0.3, 0.2),
                    material = BODY,
                    verts = 8,
                    rot = (0, 90, 0),
                    scale = (1, 0.5, 1)
                ),
                revolve!(
                    "p2",
                    &[(0.0, 0.0), (0.1, 0.0), (0.06, 0.2), (0.0, 0.2)],
                    at = (0.1, -0.4, 0.0),
                    material = BODY,
                    segments = 8,
                    rot = (0, 0, 45),
                    scale = (1, 2, 1)
                ),
            ];
            let turn = M4::translation(v3(0.7, 0, -0.2)) * M4::rotation(radians(-90.0), Axis::Z);
            for &o in &parts {
                o.set_matrix_world(turn * o.matrix_world());
            }
            for &o in &parts {
                c.state(o, &o.name());
            }
            c.verts(join(&parts, "j", None), "j");
        }
        "mw_modes" => {
            let r = a_rod("rod");
            c.state(r, "rod");
            let m = M4::translation(v3(0.3, 0.2, 0.1))
                * M4::rotation(radians(50.0), Axis::X)
                * M4::rotation(radians(-20.0), Axis::Y)
                * M4::diagonal(v3(1.5, 0.5, 2.0));
            r.set_matrix_world(m);
            c.state(r, "rod_set");
            let o = cube!("c", (0.2, 0.3, 0.4), material = BODY, bevel = 0);
            o.set_rotation_mode(RotationMode::YXZ);
            o.set_matrix_world(m);
            c.state(o, "yxz_set");
            let e = cube!("e", (0.2, 0.3, 0.4), material = BODY, bevel = 0);
            e.set_matrix_world(M4::diagonal(v3(-1, 1, 1)) * m);
            c.state(e, "mirrored_set");
            let e2 = cube!("e2", (0.2, 0.3, 0.4), material = BODY, bevel = 0);
            e2.set_matrix_world(M4::IDENTITY);
            c.state(e2, "identity_set");
        }
        "rod_join" => {
            let r = a_rod("rod");
            let b = cube!(
                "box",
                (0.1, 0.1, 0.1),
                at = (0.2, 0.2, 0.2),
                material = BODY,
                bevel = 0
            );
            let j = join(&[r, b], "j", None);
            c.state(j, "joined");
            c.verts(j, "joined");
            set_origin(j, (0.1, 0.2, 0.3));
            j.set_rotation_euler((0.2, 0.0, 0.4));
            c.state(j, "ignored");
            j.set_rotation_mode(RotationMode::XYZ);
            c.state(j, "xyz");
            j.set_rotation_euler((radians(-12.0), 0, radians(30.0)));
            c.state(j, "turned");
            c.verts(join(&[j], "k", None), "k");
        }
        "origin" => {
            let o = cube!(
                "c",
                (0.2, 0.3, 0.4),
                at = (0.5, -0.25, 0.75),
                material = BODY,
                rot = (10, 20, 30),
                bevel = 0
            );
            o.set_scale((1.2, 0.8, 1.1));
            set_origin(o, (0.1, 0.2, 0.3));
            c.state(o, "c");
            c.verts(o, "c");
            let r = a_rod("rod");
            hinged(r, (0.1, 0.2, 0.3));
            c.state(r, "rod");
            c.verts(r, "rod");
        }
        "mode_change" => {
            let r = a_rod("rod");
            r.set_rotation_mode(RotationMode::XYZ);
            c.state(r, "q_xyz");
            let r2 = rod!(
                "rod2",
                (0.1, 0.2, 0.3),
                (-0.4, 0.5, 0.1),
                0.02,
                material = BODY,
                verts = 8
            );
            r2.set_rotation_mode(RotationMode::ZXY);
            c.state(r2, "q_zxy");
            let o = cube!(
                "c",
                (0.2, 0.3, 0.4),
                material = BODY,
                rot = (17, -40, 109),
                bevel = 0
            );
            o.set_rotation_mode(RotationMode::Quaternion);
            c.state(o, "xyz_q");
            let o2 = cube!("c2", (0.2, 0.3, 0.4), material = BODY, bevel = 0);
            o2.set_rotation_mode(RotationMode::YXZ);
            o2.set_rotation_euler((0.3, -0.7, 1.9));
            o2.set_rotation_mode(RotationMode::Quaternion);
            c.state(o2, "yxz_q");
            let o3 = cube!(
                "c3",
                (0.2, 0.3, 0.4),
                material = BODY,
                rot = (17, -40, 109),
                bevel = 0
            );
            o3.set_rotation_mode(RotationMode::ZYX);
            c.state(o3, "xyz_zyx");
        }
        "attach" => {
            let door = cube!(
                "door",
                (0.6, 0.04, 1.0),
                at = (0.1, -0.3, 0.6),
                material = BODY,
                rot = (0, 0, 25),
                bevel = 0
            );
            door.set_scale((1.0, 1.5, 1.0));
            let lever = cube!(
                "lever",
                (0.1, 0.02, 0.03),
                at = (0.3, -0.36, 0.7),
                material = BODY,
                rot = (15, 0, 40),
                bevel = 0
            );
            motion::attach(lever, door);
            c.state(door, "door");
            c.found.reach = SHEARED;
            c.state(lever, "lever");
            c.found.reach = 0.0;
            let r = a_rod("rod");
            motion::attach(r, door);
            c.state(r, "rod");
        }
        "swing_rod" => {
            let r = a_rod("rod");
            swing!(r, "open", 30, seconds = 0.5, axis = Axis::X);
            c.state(r, "rod");
            c.keys(r, "rod");
            let j = join(
                &[
                    a_rod("a"),
                    cube!("b", (0.1, 0.1, 0.1), material = BODY, bevel = 0),
                ],
                "j",
                None,
            );
            spin!(j, "round", turns = 1, seconds = 0.5, axis = Axis::Y);
            c.state(j, "j");
            c.keys(j, "j");
        }
        "page" => {
            let (hx, hy, hz) = (0.2, -0.1, 1.05);
            let curl: Vec<(f64, f64, f64)> = (0..7)
                .map(|k| {
                    let k = k as f64;
                    (
                        hx - 0.215 * k / 6.0,
                        hy,
                        hz + 0.012 * artkit::math::sin(PI * k / 6.0) * (1.0 - k / 12.0),
                    )
                })
                .collect();
            let leaf = sweep!(
                "page_leaf",
                &curl,
                0.02,
                material = BODY,
                profile = rounded_rect(0.003, 0.3, 0.001, 1),
                up = (0, 1, 0)
            );
            let page = join(&[leaf], "page", Some((hx, hy, hz).into()));
            page.set_rotation_mode(RotationMode::YXZ);
            page.set_rotation_euler((radians(25.0), 0, 0));
            motion::keys(
                page,
                "page",
                Path::Rotation,
                1,
                &[(0.0, 0.0), (0.9, radians(-172.0))],
            );
            c.state(page, "page");
            c.keys(page, "page");
            file(c, &[page]);
        }
        "rain_file" => {
            let drop = cylinder!(
                "rain0",
                0.005,
                0.6,
                at = (0.0, 0.0, 0.9),
                material = BODY,
                verts = 6,
                smooth = false
            );
            let rain = join(&[drop], "water", Some((0, 0, 1.2).into()));
            rain.set_rotation_mode(RotationMode::Quaternion);
            rain.set_rotation_quaternion(v3(0, 0, -1).rotation_difference(v3(0.13, -0.21, -0.8)));
            rain.set_scale_axis(2, 0.001);
            grow!(rain, "water_on", seconds = 0.4, axis = Axis::Z);
            c.state(rain, "rain");
            file(c, &[rain]);
        }
        other => panic!("no case {other}"),
    }
}

/// The parts written and read back against what Blender's exporter wrote for them (\`node\` and \`anim\` lines): each
/// node's turn, scale and place, each clip's samples. The exporter takes a part's matrix apart, the kernel writes
/// the turn it was given: they agree to within \`FILE\`.
const FILE: f32 = 2e-6;

fn file(c: &mut Case, objs: &[Obj]) {
    let model = glb::read(&glb::write(objs, false, &finishes())).unwrap();
    let mut worst = 0.0_f32;
    let mut near = |what: String, got: &[f32], want: &[f32]| {
        assert_eq!(got.len(), want.len(), "{what}: how many");
        for (g, w) in got.iter().zip(want) {
            worst = worst.max((g - w).abs());
            assert!((g - w).abs() <= FILE, "{what}: {g}, Blender's {w}");
        }
    };
    for (kind, label, w) in c.lines {
        let w: Vec<&str> = w.iter().map(String::as_str).collect();
        match kind.as_str() {
            "node" => {
                let node = model.nodes.iter().find(|n| n.name == *label).unwrap();
                near(format!("{label} rotation"), &node.rotation, &f32s(&w[1..5]));
                near(format!("{label} scale"), &node.scale, &f32s(&w[6..9]));
                near(
                    format!("{label} translation"),
                    &node.translation,
                    &f32s(&w[10..13]),
                );
            }
            "anim" => {
                let anim = model.animations.iter().find(|a| a.name == *label).unwrap();
                let want_target = match w[1] {
                    "rotation" => glb::Target::Rotation,
                    "scale" => glb::Target::Scale,
                    _ => glb::Target::Translation,
                };
                let ch = anim
                    .channels
                    .iter()
                    .find(|ch| ch.target == want_target && model.nodes[ch.node].name == w[0])
                    .unwrap();
                near(format!("{label} {}", w[1]), &ch.values, &f32s(&w[2..]));
            }
            _ => {}
        }
    }
    c.found.compared += 1;
    println!("{}: the file within {worst:e} of Blender's", c.name);
}

#[test]
fn hand_set_transforms_are_blenders() {
    let all = cases();
    let mut off = Vec::new();
    let mut compared = 0;
    for (name, lines) in &all {
        let mut case = Case {
            name,
            lines,
            found: Found::default(),
        };
        build(&mut case);
        println!(
            "{name}: {} numbers, {} not Blender's to the bit",
            case.found.compared,
            case.found.off.len()
        );
        compared += case.found.compared;
        off.extend(case.found.off);
    }
    assert!(compared > 2500, "{compared} numbers compared");
    assert!(off.is_empty(), "{} off:\n{}", off.len(), off.join("\n"));
}
