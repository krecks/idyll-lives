//! What the stalls and the machines are built from: the palette, and the wheels, striped canopies and valances
//! they share.

use artkit::forms::{self, dedupe};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, V3, atan2, cos, degrees, sin};
use artkit::mesh::Mesh;
use artkit::parts::{Settle, settle_with};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, revolve};

/// The family's materials, by the names the pack gives them (a palette written out: one is named `crate`).
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub cart: Mat,
    pub stripe: Mat,
    pub parasol: Mat,
    pub parasol_white: Mat,
    pub cream_pink: Mat,
    pub cream_vanilla: Mat,
    pub cream_choc: Mat,
    pub cream_mint: Mat,
    pub wafer: Mat,
    pub stand: Mat,
    pub awning: Mat,
    pub awning_white: Mat,
    pub sign: Mat,
    pub bun: Mat,
    pub sausage: Mat,
    pub mustard: Mat,
    pub ketchup: Mat,
    pub grill: Mat,
    pub snack_red: Mat,
    pub drink_blue: Mat,
    pub lit_sign: Mat,
    pub lit_panel: Mat,
    pub band: Mat,
    pub glass: Mat,
    pub inside: Mat,
    pub tray: Mat,
    pub button: Mat,
    pub display: Mat,
    pub label: Mat,
    pub snack_a: Mat,
    pub snack_b: Mat,
    pub snack_c: Mat,
    pub snack_d: Mat,
    pub snack_e: Mat,
    pub snack_f: Mat,
    pub snack_g: Mat,
    pub can_green: Mat,
    pub timber: Mat,
    pub board: Mat,
    pub crate_: Mat,
    pub awning_green: Mat,
    pub tomato: Mat,
    pub orange: Mat,
    pub apple: Mat,
    pub lemon: Mat,
    pub slate: Mat,
    pub chalk: Mat,
    pub steel: Mat,
    pub tyre: Mat,
}

#[allow(non_upper_case_globals)]
pub const M: Palette = Palette {
    // the ice cream cart
    cart: Mat::new("cart", "#F4EFE6", "gloss"), // its cream body; the main part the player colours
    stripe: Mat::new("stripe", "#E9A6B4", "gloss"), // the pink bands round it
    parasol: Mat::new("parasol", "#E9A6B4", "fabric"), // the parasol's coloured gores and valance
    parasol_white: Mat::new("parasol_white", "#F6F3EE", "fabric"), // its white gores
    cream_pink: Mat::new("cream_pink", "#F3B6C4", "satin"), // the ice creams in their tubs
    cream_vanilla: Mat::new("cream_vanilla", "#F3E6C4", "satin"),
    cream_choc: Mat::new("cream_choc", "#7A4E33", "satin"),
    cream_mint: Mat::new("cream_mint", "#BFE3C8", "satin"),
    wafer: Mat::new("wafer", "#D9A35F", "matte"), // the cone on the sign
    // the hot dog stand
    stand: Mat::new("stand", "#B8322B", "gloss"), // its red body
    awning: Mat::new("awning", "#B8322B", "fabric"), // the awning's red stripes
    awning_white: Mat::new("awning_white", "#F2F0EA", "fabric"), // and its white ones
    sign: Mat::new("sign", "#F2F0EA", "gloss"),   // the sign on its front
    bun: Mat::new("bun", "#D9A35F", "satin"),
    sausage: Mat::new("sausage", "#9E3B2A", "gloss"),
    mustard: Mat::new("mustard", "#D6A64A", "gloss"),
    ketchup: Mat::new("ketchup", "#C0392B", "gloss"),
    grill: Mat::new("grill", "#2A2F36", "satin"), // the hot plate
    // the machines
    snack_red: Mat::new("snack_red", "#B8322B", "gloss"), // the snack machine's cabinet
    drink_blue: Mat::new("drink_blue", "#2E6FA8", "gloss"), // the drinks machine's cabinet
    lit_sign: Mat::new("lit_sign", "#F2E6C2", "gloss").glow("#FFE2A6"), // the header sign over the window, lit at night
    lit_panel: Mat::new("lit_panel", "#EAF4F8", "gloss").glow("#CFE8FF"), // the drinks machine's lit front
    band: Mat::new("band", "#2E5C8A", "gloss"), // the bands on the lit front
    glass: Mat::new("glass", "#CFE3EA", "glass"), // the snack machine's window (see-through: its own draw call)
    inside: Mat::new("inside", "#2A2F36", "satin"), // the dark inside, the keypad panel, the tray
    tray: Mat::new("tray", "#1E2226", "satin"),   // the pick-up tray's dark well
    button: Mat::new("button", "#C3C7CB", "satin"), // keypad buttons, the card reader
    display: Mat::new("display", "#7FD1A8", "gloss"), // the price display
    label: Mat::new("label", "#F2F0EA", "satin"), // the shelf labels
    snack_a: Mat::new("snack_a", "#F2C14E", "gloss"), // the snacks' wrappers and the cans
    snack_b: Mat::new("snack_b", "#4E7A7F", "gloss"),
    snack_c: Mat::new("snack_c", "#D9483B", "gloss"),
    snack_d: Mat::new("snack_d", "#8C5A3C", "gloss"),
    snack_e: Mat::new("snack_e", "#9DAF8C", "gloss"),
    snack_f: Mat::new("snack_f", "#E9A6B4", "gloss"),
    snack_g: Mat::new("snack_g", "#3E5470", "gloss"),
    can_green: Mat::new("can_green", "#5EC28B", "gloss"),
    // the market stall
    timber: Mat::new("timber", "#7A5234", "wood"), // its frame: posts, beams, apron; the main part
    board: Mat::new("board", "#A8754A", "wood"),   // the table top
    crate_: Mat::new("crate", "#CDB08A", "wood"),  // the fruit crates
    awning_green: Mat::new("awning_green", "#4E8A5A", "fabric"), // the awning's green stripes
    tomato: Mat::new("tomato", "#D9483B", "gloss").roughness(0.3),
    orange: Mat::new("orange", "#E8902E", "gloss").roughness(0.45),
    apple: Mat::new("apple", "#8DB84A", "gloss"),
    lemon: Mat::new("lemon", "#F2C14E", "gloss").roughness(0.4),
    slate: Mat::new("slate", "#2A2C30", "matte"), // the chalk board
    chalk: Mat::new("chalk", "#E8E6E0", "matte"),
    // shared
    steel: Mat::new("steel", "#C3C7CB", "metal"), // tops, frames, push bars, poles
    tyre: Mat::new("tyre", "#26282A", "matte"),
};

impl Palette {
    pub fn palette(&self) -> artkit::materials::Palette {
        artkit::materials::Palette::new(&[
            self.cart,
            self.stripe,
            self.parasol,
            self.parasol_white,
            self.cream_pink,
            self.cream_vanilla,
            self.cream_choc,
            self.cream_mint,
            self.wafer,
            self.stand,
            self.awning,
            self.awning_white,
            self.sign,
            self.bun,
            self.sausage,
            self.mustard,
            self.ketchup,
            self.grill,
            self.snack_red,
            self.drink_blue,
            self.lit_sign,
            self.lit_panel,
            self.band,
            self.glass,
            self.inside,
            self.tray,
            self.button,
            self.display,
            self.label,
            self.snack_a,
            self.snack_b,
            self.snack_c,
            self.snack_d,
            self.snack_e,
            self.snack_f,
            self.snack_g,
            self.can_green,
            self.timber,
            self.board,
            self.crate_,
            self.awning_green,
            self.tomato,
            self.orange,
            self.apple,
            self.lemon,
            self.slate,
            self.chalk,
            self.steel,
            self.tyre,
        ])
    }
}

/// A float in a part's name, as Python writes it.
pub fn fr(x: f64) -> String {
    float_repr(x)
}

/// Stands the model on the floor where it was built, not centred: its `centre` says where it stands.
pub fn settle_in_place() {
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
}

/// A wheel at game (x, z), its axle along the depth: a rounded rubber tyre, a steel rim, spokes and a domed hub.
pub fn wheel(f: &GameFrame, name: &str, x: f64, z: f64, r: f64, spokes: usize) {
    let w = 0.06;
    let tyre = [
        (r * 0.72, -w / 2.0),
        (r * 0.93, -w / 2.0),
        (r, -w * 0.22),
        (r, w * 0.22),
        (r * 0.93, w / 2.0),
        (r * 0.72, w / 2.0),
    ];
    let closed: Vec<(f64, f64)> = tyre.iter().copied().chain([tyre[0]]).collect();
    revolve!(
        format!("{name}_tyre"),
        &closed,
        at = f.at(x, r, z),
        material = M.tyre,
        segments = 14,
        rot = (90, 0, 0)
    );
    let rim = [
        (r * 0.5, -w * 0.3),
        (r * 0.74, -w * 0.38),
        (r * 0.74, w * 0.38),
        (r * 0.5, w * 0.3),
    ];
    let closed: Vec<(f64, f64)> = rim.iter().copied().chain([rim[0]]).collect();
    revolve!(
        format!("{name}_rim"),
        &closed,
        at = f.at(x, r, z),
        material = M.steel,
        segments = 14,
        rot = (90, 0, 0)
    );
    revolve!(
        format!("{name}_hub"),
        &[
            (0.0, -w * 0.62),
            (r * 0.24, -w * 0.62),
            (r * 0.24, w * 0.5),
            (r * 0.12, w * 0.7),
            (0.0, w * 0.7)
        ],
        at = f.at(x, r, z),
        material = M.steel,
        segments = 8,
        rot = (90, 0, 0)
    );
    for k in 0..spokes {
        let a = TAU * k as f64 / spokes as f64;
        fbox!(
            f,
            format!("{name}_spoke{k}"),
            0.012,
            r * 0.52,
            0.01,
            (x + r * 0.46 * sin(a), r + r * 0.46 * cos(a), z),
            M.steel,
            rot = (0.0, 0.0, degrees(a))
        );
    }
}

/// A polygon's middle as Blender's `polygon.center` gives it: the mean of its corners, each weighted by one
/// over their count, summed in single precision.
fn centre(mesh: &Mesh, f: usize) -> V3 {
    let corners = mesh.corners(f);
    let w = 1.0f32 / corners.len() as f32;
    let mut c = [0.0f32; 3];
    for &v in corners {
        let p = mesh.verts[v as usize];
        c[0] += p.x * w;
        c[1] += p.y * w;
        c[2] += p.z * w;
    }
    V3::new(c[0], c[1], c[2])
}

/// Gives a canopy's faces alternate materials round it by the angle of their middles (a parasol's gores); its
/// underside the second.
pub fn striped(obj: Obj, gores: usize, _first: Mat, second: Mat) -> Obj {
    obj.edit(|o| {
        let mesh = &mut o.mesh;
        mesh.mats.push(second);
        for f in 0..mesh.faces.len() {
            let n = mesh.face_normal(f);
            if (n.z as f64).abs() > 0.9 && n.z < 0.0 {
                mesh.faces[f].mat = 1;
                continue;
            }
            let c = centre(mesh, f);
            let a = artkit::py::fmod(atan2(c.y as f64, c.x as f64), TAU);
            mesh.faces[f].mat = (artkit::py::int(a / (TAU / gores as f64)) % 2) as u16;
        }
    });
    obj
}

/// A scalloped valance along the front of an awning, from game x0 to x1 hanging from height y at depth z: one
/// flat strip per stripe, each `scallops` round tongues across, in the stripes' materials in turn.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn valance(
    f: &GameFrame,
    name: &str,
    x0: f64,
    x1: f64,
    y: f64,
    z: f64,
    drop: f64,
    materials: &[Mat],
    scallops: usize,
) {
    let n = materials.len();
    let w = (x1 - x0) / n as f64;
    for (i, &material) in materials.iter().enumerate() {
        let cx = x0 + w * (i as f64 + 0.5);
        let mut pts = vec![(w / 2.0, 0.0), (-w / 2.0, 0.0)];
        for k in 0..scallops {
            let u0 = -w / 2.0 + w * k as f64 / scallops as f64;
            for j in 0..5 {
                let fj = j as f64 / 4.0;
                pts.push((
                    u0 + w / scallops as f64 * fj,
                    -drop + 0.022 * (1.0 - sin(PI * fj)),
                ));
            }
        }
        let pts = dedupe(&pts, 1e-7);
        fextrude!(f, format!("{name}{i}"), &pts, 0.008, (cx, y, z), material);
    }
}

/// A closed thin band through four loops of equal count (outer top, outer bottom, inner bottom, inner top), each
/// joined to the next and the last back to the first: a valance.
pub fn forms_band(name: &str, a: &[V3], b: &[V3], c: &[V3], d: &[V3], material: Mat) -> Obj {
    let m = a.len();
    let vs: Vec<V3> = [a, b, c, d].concat();
    let mut faces = Vec::new();
    for r0 in 0..4 {
        let r1 = (r0 + 1) % 4;
        for k in 0..m {
            let k2 = (k + 1) % m;
            faces.push(vec![
                (r0 * m + k) as u32,
                (r0 * m + k2) as u32,
                (r1 * m + k2) as u32,
                (r1 * m + k) as u32,
            ]);
        }
    }
    forms::object(name, vs, &faces, Some(material), false, 40.0, false)
}
