//! The tools on a station while a resident cooks (the chopping board, the pan, food on the grill, the mixing bowl),
//! the bites eaten by hand and the cook's tool in hand.

use super::M;
use super::common::{F, aim, done, t3};
use super::plates::{_cookie, _halves, _pizza_slice, _sandwich_half};
use super::quick::{_burger, _toast_half};
use super::town::_hot_dog;
use artkit::forms::{by, rounded_rect};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, V3, cos, pow, radians, sin};
use artkit::parts::new_parts;
use artkit::place::Facing;
use artkit::py;
use artkit::scene::Obj;
use artkit::{cylinder, fextrude, fsweep, jitter, join, sphere};

/// The chopping board: rounded corners, a juice groove round its face and a slot to hang it by; a carrot half
/// chopped into coins and chopped parsley. (The knife is in the cook's hand: `knife`, actions.ts `PROPS.knife`.)
pub fn board() -> Vec<Obj> {
    let hole: Vec<(f64, f64)> = rounded_rect(0.016, 0.07, 0.008, 2)
        .into_iter()
        .map(|(u, v)| (0.152 + u, v))
        .collect();
    fextrude!(
        F,
        "board",
        &rounded_rect(0.34, 0.24, 0.02, 2),
        0.025,
        (0, 0.0125, 0),
        M.board,
        plane = Facing::Top,
        holes = vec![hole],
        bevel = 0.004,
        bevel_segments = 1
    );
    let groove: Vec<(f64, f64, f64)> = rounded_rect(0.27, 0.205, 0.018, 2)
        .into_iter()
        .map(|(x, z)| (x - 0.01, 0.0251, z))
        .collect();
    fsweep!(
        F,
        "groove",
        &groove,
        0.0,
        M.board_dark,
        profile = [
            (-0.0004, -0.0025),
            (0.0004, -0.0025),
            (0.0004, 0.0025),
            (-0.0004, 0.0025)
        ],
        closed = true
    );
    // the carrot: tapering to its root, cut flat where the coins came off
    let path = [
        (-0.145, 0.04, 0.035),
        (-0.13, 0.04, 0.034),
        (-0.1, 0.04, 0.032),
        (-0.06, 0.04, 0.03),
    ];
    let car = fsweep!(
        F,
        "carrot",
        &path,
        1.0,
        M.carrot,
        verts = 8,
        scale = [0.003, 0.008, 0.013, 0.015]
    );
    jitter!(car, 0.0006, size = 0.008, seed = 1.0);
    for (i, (x, z, t)) in [
        (-0.04, 0.04, 0.0),
        (-0.025, 0.062, 8.0),
        (-0.035, 0.015, -6.0),
        (-0.012, 0.036, 4.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("coin{i}"),
            0.028,
            0.005,
            (x, 0.025, z),
            (M.carrot, M.carrot_core),
            tilt = (t, 0.0)
        );
    }
    for (i, (x, z, yaw)) in [
        (0.02, 0.07, 10.0),
        (0.04, 0.05, 40.0),
        (0.03, 0.09, 70.0),
        (0.055, 0.075, 20.0),
        (0.065, 0.055, 120.0),
    ]
    .into_iter()
    .enumerate()
    {
        leaf!(
            format!("parsley{i}"),
            (x, 0.0252, z),
            0.014,
            0.009,
            yaw,
            M.pea,
            tilt = (10 * (i % 2)) as f64,
            n = 3
        );
    }
    done("board")
}

/// The frying pan on the stove, its handle to the back (+depth: away from the cook, as the old part's,
/// out of the way of the hands): a heavy rolled rim, the handle riveted on through a bracket and ending in a
/// rounded grip; a heap of stir-fry, strips of beef, broccoli, peppers and carrot.
pub fn pan() -> Vec<Obj> {
    lathe!(
        "pan",
        &[
            (0.0, 0.0),
            (0.112, 0.0),
            (0.118, 0.003),
            (0.1385, 0.044),
            (0.14, 0.0482),
            (0.1372, 0.0502),
            (0.1342, 0.0472),
            (0.113, 0.0115),
            (0.105, 0.009),
            (0.0, 0.009)
        ],
        M.iron,
        segments = 24
    );
    kit::tube!(
        F,
        "bracket",
        &[(0, 0.03, 0.128), (0, 0.04, 0.152), (0, 0.044, 0.172)],
        0.0075,
        M.iron,
        verts = 6,
        round = 1
    );
    for t in [-1, 1] {
        cylinder!(
            format!("rivet{t}"),
            0.004,
            0.004,
            at = F.at(t as f64 * 0.008, 0.03, 0.129),
            material = M.iron,
            rot = (90, 0, 0),
            verts = 6
        );
    }
    fsweep!(
        F,
        "handle",
        &[
            (0, 0.043, 0.166),
            (0, 0.045, 0.2),
            (0, 0.047, 0.235),
            (0, 0.048, 0.25),
            (0, 0.048, 0.2545)
        ],
        0.0,
        M.grip,
        profile = rounded_rect(0.018, 0.026, 0.008, 2),
        up = (0, 0, 1),
        scale = [0.85, 0.92, 1.0, 0.88, 0.55]
    );
    let heap = lump!(
        "heap",
        0.075,
        (0.0, 0.006, 0.0),
        M.stew,
        scale = (1.0, 0.32, 1.0),
        seed = 8.5,
        noise = 0.15,
        subdivisions = 2
    );
    jitter!(heap, 0.003, size = 0.025, seed = 8.5);
    let top = |x: f64, z: f64| {
        let k = 1.0 - pow(py::hypot(x, z) / 0.08, 2.0);
        0.012 + 0.04 * (if k > 0.0 { k } else { 0.0 }).sqrt()
    };
    for (i, (x, z, yaw)) in [
        (0.02, -0.01, 10.0),
        (-0.04, 0.02, 40.0),
        (0.05, 0.04, 70.0),
        (-0.02, -0.05, 25.0),
        (0.0, 0.03, 120.0),
    ]
    .into_iter()
    .enumerate()
    {
        box_!(
            format!("beef{i}"),
            0.034,
            0.008,
            0.012,
            (x, top(x, z) + 0.002, z),
            M.beef,
            yaw = yaw,
            tilt = ((10 * (i as i64 % 3 - 1)) as f64, 6.0),
            bevel = 0.0025
        );
    }
    for (i, (x, z)) in [
        (0.05, -0.035),
        (-0.055, -0.02),
        (-0.03, 0.055),
        (0.07, 0.01),
    ]
    .into_iter()
    .enumerate()
    {
        let y = top(x, z) - 0.006;
        cylinder!(
            format!("stem{i}"),
            0.004,
            0.012,
            at = F.at(x, y + 0.006, z),
            material = M.pea,
            verts = 5
        );
        lump!(
            format!("floret{i}"),
            0.0115,
            (x, y + 0.009, z),
            M.broccoli,
            scale = (1.0, 0.8, 1.0),
            seed = i as f64 * 2.3,
            rough = 0.25
        );
    }
    for (i, (x, z, yaw, m)) in [
        (-0.01, -0.025, 30.0, M.pepper_red),
        (0.035, 0.015, 100.0, M.pepper_yellow),
        (-0.05, 0.035, 150.0, M.pepper_red),
        (0.02, 0.06, 60.0, M.pepper_yellow),
    ]
    .into_iter()
    .enumerate()
    {
        let a = radians(yaw);
        let (ca, sa) = (cos(a), sin(a));
        let y = top(x, z) + 0.002;
        fsweep!(
            F,
            format!("pepper{i}"),
            &[
                (x - 0.017 * ca, y - 0.002, z + 0.017 * sa),
                (x + 0.004 * sa, y + 0.001, z + 0.004 * ca),
                (x + 0.017 * ca, y - 0.002, z - 0.017 * sa)
            ],
            0.0,
            m,
            profile = [
                (-0.002, -0.0045),
                (0.002, -0.0045),
                (0.002, 0.0045),
                (-0.002, 0.0045)
            ],
            resolution = 2
        );
    }
    for (i, (x, z, t)) in [
        (0.0, -0.06, 14.0),
        (-0.065, 0.0, -10.0),
        (0.04, -0.06, 8.0),
        (0.06, 0.06, -12.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("carrot{i}"),
            0.02,
            0.004,
            (x, top(x, z) - 0.004, z),
            (M.carrot, M.carrot_core),
            tilt = (t, t * 0.5)
        );
    }
    done("pan")
}

/// On the grill: two sausages, bent and marked by the bars, and a burger with its rounded, rough edge and
/// the bars' marks across it.
pub fn grill_food() -> Vec<Obj> {
    for (i, (z, m, b)) in [(-0.045, M.sausage, -0.007), (0.027, M.sausage_b, -0.007)]
        .into_iter()
        .enumerate()
    {
        roll!(
            format!("sausage{i}"),
            0.014,
            0.15,
            (m, M.char),
            (-0.08, 0.014, z),
            segments = 12,
            bend = b,
            marks = vec![(-0.55, -0.4), (-0.05, 0.1), (0.45, 0.6)]
        );
    }
    super::dishes::_patty("patty", (0.08, 0.0, 0.0), 0.1, 0.03, true);
    done("grillFood")
}

/// On the counter while batter is mixed or pastry made (`mix`): a deep glazed bowl, its rim rolled, of batter
/// stirred into a swirl, a wooden spoon left in it, its bowl in the batter and its handle over the far rim (+depth:
/// away from the cook, who whisks: actions.ts `PROPS.whisk`).
pub fn mixing_bowl() -> Vec<Obj> {
    let mut mats = vec![M.mixing; 8];
    mats.extend([M.batter, M.batter]);
    lathe!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.07, 0.0),
            (0.085, 0.01),
            (0.128, 0.104),
            (0.13, 0.109),
            (0.1265, 0.113),
            (0.1215, 0.1115),
            (0.117, 0.1),
            (0.111, 0.086),
            (0.06, 0.0848),
            (0.0, 0.085)
        ],
        mats,
        segments = 20
    );
    // The swirl the whisk leaves, coiling in on the batter (its top 0.85 cm up), riding on it.
    let pts: Vec<(f64, f64, f64)> = (0..17)
        .map(|i| {
            let f = i as f64 / 16.0;
            let a = TAU * 1.6 * f;
            let rr = 0.09 * (1.0 - f) + 0.01;
            (
                0.008 + rr * cos(a),
                0.0862 + 0.0015 * f,
                -0.006 + rr * sin(a),
            )
        })
        .collect();
    fsweep!(
        F,
        "ridge",
        &pts,
        0.0,
        M.batter,
        profile = [
            (-0.002, -0.004),
            (0.0008, -0.0025),
            (0.0008, 0.0025),
            (-0.002, 0.004)
        ],
        scale = by(|f| 1.0 - 0.5 * f)
    );
    // The spoon: its bowl half sunk in the batter at the back, the handle resting on the rim (its top 11.3 cm up)
    // and reaching 10 cm past it.
    let (s0, rim) = (
        V3::from((-0.01, 0.087, 0.03)),
        V3::from((0.018, 0.12, 0.126)),
    );
    let d = (rim - s0).normalized();
    let s1 = rim + d * 0.1;
    fsweep!(
        F,
        "spoon_handle",
        &[
            t3(s0 + d * 0.012),
            t3(s0 + (s1 - s0) * 0.5),
            t3(s1 - d * 0.004),
            t3(s1)
        ],
        0.007,
        M.spoon,
        verts = 8,
        scale = [0.6, 0.85, 1.0, 0.75]
    );
    let b = s0 - d * 0.012;
    sphere!(
        "spoon_bowl",
        1.0,
        at = F.at(b.x as f64, b.y as f64, b.z as f64),
        material = M.spoon,
        scale = (0.028, 0.02, 0.007),
        rot = aim((d.x as f64, d.y as f64, d.z as f64)),
        segments = 8,
        rings = 5
    );
    done("mixingBowl")
}

// --- bites (2026-11-03): what is eaten by hand at the table, in the right hand on its way to the mouth ----------------
// Drawn by the bite's grip (actions.ts `BITES`, `grip: 'bite'`), as the fork is: lying face up, its front (+depth), the
// end bitten, to the mouth, held at the origin by its back end (the crust, the bun's end), the way it is held.
/// `parts` (made where their item has them) joined, turned `yaw` degrees about the up axis and moved (`dx`, `dz`).
fn _moved(parts: &[Obj], name: &str, dx: f64, dz: f64, yaw: f64) -> Obj {
    let obj = join!(parts, name, origin = (0, 0, 0));
    obj.set_rotation_euler((0.0, 0.0, radians(yaw)));
    obj.set_location(F.at(dx, 0.0, dz))
}

/// Half the sandwich (`sandwich`), its point to the front: crusts along its back and left, the cut along its
/// front right.
pub fn bite_sandwich() -> Vec<Obj> {
    let (hw, hd) = (0.058, 0.052);
    let [(half, pts, cut, _), _] = _halves((hw, hd), 0.012);
    let p = new_parts(|| _sandwich_half(half, &pts, cut, (0.0, 0.0), 0.0, 3.0));
    _moved(&p, "half", hw / 3.0, hd / 3.0 + 0.03, 0.0);
    done("biteSandwich")
}

/// Half a slice of the toast (`toast`), buttered, its point to the front.
pub fn bite_toast() -> Vec<Obj> {
    let (hw, hd) = (0.055, 0.05);
    let [(half, pts, cut, _), _] = _halves((hw, hd), 0.012);
    let p = new_parts(|| _toast_half(half, &pts, cut, (0.0, 0.0), 0.0, -1));
    _moved(&p, "half", hw / 3.0, hd / 3.0 + 0.03, 0.0);
    done("biteToast")
}

/// The slice of pizza (`pizzaSlice`), held by its crust, its tip to the front.
pub fn bite_pizza() -> Vec<Obj> {
    _pizza_slice(0.0, 0.045);
    done("bitePizza")
}

/// A chocolate-chip cookie (`treatPlate`), held at its back edge.
pub fn bite_cookie() -> Vec<Obj> {
    _cookie("cookie", (0.0, 0.0, 0.03), (0.0, 0.0), 0.5, 2, 0.0);
    done("biteCookie")
}

/// The takeaway's burger (`takeout`), held at its back.
pub fn bite_burger() -> Vec<Obj> {
    _burger(0.0, 0.0, 0.035);
    done("biteBurger")
}

/// The hot dog (`hotDog`) out of its boat: the bun's two halves, the sausage longer than it, mustard and ketchup;
/// along the depth, held at its back end.
pub fn bite_hot_dog() -> Vec<Obj> {
    let p = new_parts(|| _hot_dog(0.0));
    _moved(&p, "hot_dog", 0.0, 0.065, 90.0);
    done("biteHotDog")
}

/// A cracker from the cheese board (`tastingBoard`) with a slice of cheese on it.
pub fn bite_cracker() -> Vec<Obj> {
    box_!(
        "cracker",
        0.042,
        0.005,
        0.042,
        (0.0, 0.0025, 0.02),
        M.cracker,
        yaw = 8.0,
        bevel = 0.0012
    );
    prism!(
        "cheese",
        &[(-0.014, 0.004), (0.016, 0.01), (-0.01, 0.036)],
        0.005,
        0.012,
        M.cheese_cut,
        M.cheese_cut,
        sides = vec![M.cheese_cut, M.cheese, M.cheese_cut]
    );
    done("biteCracker")
}

/// The bakery's croissant (`pastryBag`), lying, its horns curling back to the hand.
pub fn bite_croissant() -> Vec<Obj> {
    for (i, t) in [-1.0, -0.5, 0.0, 0.5, 1.0].into_iter().enumerate() {
        let r = 0.02 * (1.0 - 0.35 * f64::abs(t));
        lump!(
            format!("lobe{i}"),
            r,
            (0.04 * t, 0.0, 0.05 - 0.028 * t * t),
            M.pastry_baked,
            scale = (0.85, 0.75, 1.15),
            seed = 4.0 + i as f64,
            noise = 0.12
        );
    }
    done("biteCroissant")
}

/// A few crisps from the packet (`crisps`), one on another.
pub fn bite_crisps() -> Vec<Obj> {
    for (i, (x, y, z, a)) in [
        (0.0, 0.0, 0.02, 10.0),
        (0.008, 0.004, 0.03, 70.0),
        (-0.006, 0.008, 0.026, 140.0),
    ]
    .into_iter()
    .enumerate()
    {
        coin!(
            format!("crisp{i}"),
            0.034,
            0.0015,
            (x, y, z),
            M.crisp,
            segments = 8,
            cup = 1.6,
            tilt = (a / 8.0, a / 10.0)
        );
    }
    done("biteCrisps")
}

// --- cooking in hand (2026-11-03): the cook's tool while a step is done at a station (actions.ts `ACTIONS[tag].prop`) --
/// A chef's knife, 27 cm, lying flat, the blade to the front (+depth) and its edge to the left (-x), held at the
/// middle of its handle: a black riveted handle, a steel bolster, a blade curving up to its point.
pub fn knife() -> Vec<Obj> {
    box_!(
        "handle",
        0.022,
        0.017,
        0.1,
        (0.0, 0.0085, 0.0),
        M.grip,
        bevel = 0.005
    );
    for (k, z) in [-0.025, 0.02].into_iter().enumerate() {
        disc!(
            format!("rivet{k}"),
            0.005,
            0.0004,
            (0.0, 0.017, z),
            M.steel,
            verts = 6
        );
    }
    box_!(
        "bolster",
        0.024,
        0.016,
        0.012,
        (0.0, 0.0085, 0.056),
        M.steel,
        bevel = 0.002
    );
    let blade = [
        (0.0, 0.0105),
        (0.12, 0.0075),
        (0.165, 0.004),
        (0.17, 0.0025),
        (0.15, -0.012),
        (0.1, -0.026),
        (0.035, -0.032),
        (0.0, -0.032),
    ];
    _along("blade", &blade, 0.002, 0.0075, 0.062, M.steel);
    done("knife")
}

/// A wooden spoon, 30 cm, lying on its back, its bowl to the front (+depth), held at the origin 10 cm from the
/// handle's end.
pub fn wooden_spoon() -> Vec<Obj> {
    fsweep!(
        F,
        "handle",
        &[(0.0, 0.006, -0.1), (0.0, 0.006, 0.05), (0.0, 0.0065, 0.165)],
        0.0065,
        M.spoon,
        verts = 8,
        scale = [1.0, 0.85, 0.6]
    );
    sphere!(
        "bowl",
        1.0,
        at = F.at(0.0, 0.007, 0.185),
        material = M.spoon,
        scale = (0.02, 0.028, 0.007),
        segments = 10,
        rings = 6
    );
    done("woodenSpoon")
}

/// A slotted turner for the grill, 31 cm: a black grip, a steel shank bent down to a blade 8 by 10 cm with three
/// slots, lying on its blade, the blade to the front (+depth), held at the middle of the grip.
pub fn spatula() -> Vec<Obj> {
    fsweep!(
        F,
        "grip",
        &[(0.0, 0.022, -0.07), (0.0, 0.022, 0.04)],
        0.0,
        M.grip,
        profile = rounded_rect(0.022, 0.014, 0.006, 1),
        up = (0, 0, 1),
        scale = [0.9, 1.0]
    );
    fsweep!(
        F,
        "shank",
        &[(0.0, 0.022, 0.04), (0.0, 0.02, 0.08), (0.0, 0.004, 0.11)],
        0.0,
        M.steel,
        profile = rounded_rect(0.014, 0.003, 0.001, 1),
        up = (0, 0, 1),
        resolution = 2
    );
    let slots: Vec<Vec<(f64, f64)>> = [-0.018, 0.0, 0.018]
        .into_iter()
        .map(|x| {
            rounded_rect(0.008, 0.05, 0.004, 2)
                .into_iter()
                .map(|(u, w)| (x + u, 0.15 + w))
                .collect()
        })
        .collect();
    let outline: Vec<(f64, f64)> = rounded_rect(0.08, 0.1, 0.01, 2)
        .into_iter()
        .map(|(x, z)| (x, z + 0.155))
        .collect();
    fextrude!(
        F,
        "blade",
        &outline,
        0.0018,
        (0.0, 0.0009, 0.0),
        M.steel,
        plane = Facing::Top,
        holes = slots
    );
    done("spatula")
}

/// A balloon whisk, 28 cm, lying, its wires to the front (+depth), held at the middle of its handle: a steel
/// handle and four loops of wire bowing out 6 cm across.
pub fn whisk() -> Vec<Obj> {
    lathe!(
        "handle",
        &[
            (0.0, -0.07),
            (0.009, -0.068),
            (0.011, -0.05),
            (0.011, 0.03),
            (0.008, 0.05),
            (0.0, 0.052)
        ],
        M.steel,
        at = (0, 0.03, 0),
        segments = 8,
        rot = (90, 0, 0)
    );
    for k in 0..4 {
        let a = PI * k as f64 / 4.0;
        let mut pts = vec![(0.0, 0.03, 0.05)];
        for i in 1..12 {
            let ph = 0.35 + (TAU - 0.7) * i as f64 / 12.0;
            let r = 0.03 * sin(ph);
            pts.push((r * cos(a), 0.03 + r * sin(a), 0.155 - 0.105 * cos(ph)));
        }
        pts.push((0.0, 0.03, 0.05));
        fsweep!(F, format!("wire{k}"), &pts, 0.0011, M.steel, verts = 4);
    }
    done("whisk")
}

/// A flat piece lying along the depth (u to the front, v across), `t` thick with its underside at `y`, moved
/// `shift` along the depth.
pub fn _along(
    name: &str,
    outline: &[(f64, f64)],
    t: f64,
    y: f64,
    shift: f64,
    material: Mat,
) -> Obj {
    let pts: Vec<(f64, f64)> = outline.iter().map(|&(u, v)| (v, u + shift)).collect();
    fextrude!(
        F,
        name,
        &pts,
        t,
        (0.0, y + t / 2.0, 0.0),
        material,
        plane = Facing::Top
    )
}
