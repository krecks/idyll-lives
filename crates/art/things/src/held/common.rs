//! What the held things are built from: the palette, the frame (`F`: the game's axes) and the helpers, a lathe with a
//! material a band, prisms, boxes, discs, lumps of food, sausages, slices, leaves, smears, crusts and piped icing,
//! and the plate everything is served on.

use artkit::forms::{self, rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::{P3, PI, TAU, V3, asin, atan2, cos, degrees, pow, radians, sin};
use artkit::mesh::{Mesh, edge};
use artkit::palette;
use artkit::parts::{Settle, join_all, measure, settle_with};
use artkit::place::{Coord, Facing, GameFrame, Spot};
use artkit::scene::{self, Obj};
use artkit::{boulder, cube, cylinder, fextrude, fsweep, jitter, sphere};
use std::collections::BTreeMap;
use std::rc::Rc;

palette! { M:
    // dishes and pans
    china = ("#F1EEE6", gloss),
    china_well = ("#E4DFD4", gloss),
    bowl = ("#E9E4D8", gloss),
    dish = ("#E6E0D2", gloss),
    tray = ("#A9AEB4", metal),
    iron = ("#3A3D42", metal),
    iron_inside = ("#7C8188", metal),
    steel = ("#C9CED4", metal),
    grip = ("#2B2D31", satin),
    board = ("#B88A57", wood),
    board_dark = ("#9A6A3E", wood),
    peel = ("#A8784A", wood),
    kraft = ("#C49A6C", matte),
    kraft_fold = ("#A9804F", matte),
    kraft_inside = ("#6E5236", matte),
    carton = ("#B98B5A", matte),     // the parcel's corrugated card
    carton_seam = ("#8E6840", matte),
    tape = ("#DDBA85", satin),       // brown packing tape, a little shiny
    // food
    lettuce = ("#5FA33A", satin),
    leaf = ("#3F7F2A", satin),
    carrot = ("#E8772E", satin),
    tomato = ("#D93A2B", gloss, roughness(0.3)),
    potato = ("#E8D9A0", matte),
    heap = ("#C9B06E", matte),       // the prep bowl's heap under the diced vegetables
    pea = ("#6BA33A", satin),
    stew = ("#B0562C", satin),
    bread = ("#E9CD98", matte),
    crust = ("#B9874E", matte),
    cheese = ("#E9C46A", satin),
    cheese_cut = ("#F2D88E", satin),
    salami = ("#B83A2C", satin),
    grape = ("#6E9A3E", gloss),
    cracker = ("#D9B26A", matte),
    sausage = ("#8A4A2A", satin),
    sausage_b = ("#93512E", satin),
    char = ("#3E2414", matte),
    patty = ("#5E3A24", satin),
    corn = ("#F2C230", satin),
    cob = ("#E8DDB0", matte),
    dough = ("#E2B064", matte),
    dough_baked = ("#C98E4A", matte),
    sauce = ("#C8452E", satin),
    mozzarella = ("#F3E3B0", satin),
    pepperoni = ("#9C3024", satin),
    bun = ("#D69A55", satin),
    bun_dark = ("#B5743A", satin),
    bun_base = ("#E8C48A", matte),
    pastry = ("#D8A257", matte),
    pastry_baked = ("#C08040", matte),
    filling = ("#8E2433", gloss),
    cream = ("#FBF6EA", satin),
    cookie = ("#C98A45", matte),
    chocolate = ("#4A2C1C", satin),
    cupcake_case = ("#E58FA8", satin),
    icing = ("#F8E4EA", satin),
    mixing = ("#D9DCD6", gloss),     // the mixing bowl's glaze
    batter = ("#EBCB8C", satin),
    spoon = ("#C49A62", satin),      // a wooden spoon (plain, so the bowl stays one draw)
    // leftovers
    smear = ("#64452B", matte),
    smear_sauce = ("#94401F", matte),
    grease = ("#6E5338", matte),
    burnt = ("#3F3022", matte),
    residue = ("#6B4A30", matte),
    crumb = ("#D9B98A", matte),
    melt = ("#D2A48E", satin),       // melted ice cream in the bowl, the three flavours run together
    smear_filling = ("#6E2A2E", matte),
    // the quick foods (review M-held-01): ice cream, toast, a smoothie, noodles, takeout
    vanilla = ("#F6EBC8", satin),
    strawberry = ("#F0A3B2", satin),
    choc_ice = ("#7A4E34", satin),
    toast = ("#D39A55", matte),
    toast_crust = ("#94602F", matte),
    butter = ("#F6E08E", satin),
    glass = ("#DDE8EA", gloss),      // a tumbler's glass, opaque (plain: one draw)
    smoothie = ("#C85A86", satin),   // berry, seen through the glass
    smoothie_top = ("#D97AA0", satin),
    straw = ("#F2F0EA", satin),
    paper = ("#F4F2EC", matte),      // the noodle pail's white card
    paper_red = ("#C8402E", matte),
    noodle = ("#E7C97E", satin),
    chopstick = ("#D8B887", satin),
    fries = ("#E9BF55", satin),
    // the detail pass (2026-10-22)
    carrot_core = ("#F2A05A", satin),  // a carrot coin's paler middle
    beef = ("#6B3A22", satin),
    broccoli = ("#3E7A2C", satin),
    pepper_red = ("#C93226", gloss),
    pepper_yellow = ("#E9B92E", gloss),
    sesame = ("#F1E2B8", satin),
    jam = ("#7A2A3A", gloss),          // the tasting board's ramekin of fig jam
    berry = ("#8E2446", satin),        // a raspberry on the smoothie
    wafer = ("#E2B872", matte),
    napkin = ("#F2EFE6", matte),
    cheese_brown = ("#C98A3E", satin),  // browned spots on a baked pizza
    straw_stripe = ("#D9484A", satin),
    wire = ("#9EA4AA", metal),         // the noodle pail's wire handle
    // props in hand (G-70): what residents hold while they use a thing
    mug = ("#E8E2D6", gloss),
    mug_band = ("#4F7A9A", gloss),
    coffee = ("#4A2E1E", gloss),
    crema = ("#B88A5A", satin),
    juice = ("#F0A23A", satin),        // an orange drink, seen through the glass
    ice = ("#E9F1F2", gloss),
    wine = ("#6E1A2A", gloss),
    can = ("#C8302A", gloss),
    can_band = ("#F2F0EA", gloss),
    can_top = ("#C9CED4", metal),
    phone = ("#2A2C30", satin),
    screen = ("#141A22", gloss),
    screen_lit = ("#3C5A7A", gloss),
    cover = ("#7A2E2A", satin),        // a cloth-bound book's boards
    page = ("#F2EEE2", matte),
    page_edge = ("#E2DCCB", matte),
    print = ("#9A968C", matte),        // the lines of type, a grey band each
    brass = ("#C9A54A", metal),
    seed = ("#B8964E", matte),
    dart_flight = ("#2E6FB8", satin),
    dart_shaft = ("#1E1E1E", satin),
    cue = ("#D9B98A", satin),
    cue_butt = ("#3B2A20", satin),
    cue_wrap = ("#141414", satin),
    pearl = ("#EDE6D6", gloss),
    cue_tip = ("#3E8EDE", matte),      // chalked
    rubber = ("#C8322A", satin),       // a ping-pong bat's faces, red over black
    rubber_black = ("#1E1E1E", satin),
    blade = ("#C9A06A", satin),
    chrome = ("#D0D3D6", chrome),
    // the food from town (review M-held-03)
    cup_paper = ("#F4F1EA", matte),
    cup_stripe = ("#E58FA8", matte),
    paddle = ("#D8BE8E", matte),       // a wooden ice cream spoon
    mustard = ("#E2B02A", satin),
    ketchup = ("#B8281E", satin),
    foil = ("#C9CED4", metal),         // a crisp packet's silvered inside
    packet = ("#2F5FA8", gloss),
    packet_band = ("#F2C230", gloss),
    crisp = ("#E8C46A", matte),
    apple = ("#B8282A", gloss),
    apple_blush = ("#D8A23A", gloss),
    stalk = ("#5A3A22", matte),
    bag_white = ("#F2EEE4", matte),
    // props in hand, the rest of G-70
    can_green = ("#3F8A4A", satin),    // a plastic watering can
    rose = ("#2E6A38", satin),
    pruner = ("#C8322A", satin),       // secateurs' red grips
    bristle = ("#2A2420", matte),
    paint_blue = ("#3A6EB8", satin),
    handle_wood = ("#B8874E", satin),
    chalk = ("#F4F2EA", matte),
    bottle = ("#CFE3EC", gloss),       // a baby's bottle, the milk seen through it
    milk = ("#F7F4EC", satin),
    bottle_ring = ("#7FB2D8", satin),
    teat = ("#E2B88A", satin),
    card = ("#F6F3EA", matte),
    card_sky = ("#6EA7D8", matte),
    card_sea = ("#2F6E9E", matte),
    card_sand = ("#E2C68A", matte),
    ball = ("#D2652A", matte),         // a basketball's pebbled leather
    ball_seam = ("#2A1E18", matte),
    sponge = ("#E8C88A", matte),       // a slice of layer cake
    sponge_crumb = ("#D9B272", matte),
    cake_icing = ("#FBF4EA", satin),
    cake_jam = ("#B8283A", satin),
    cake_berry = ("#C42A30", gloss),
    fork = ("#E4E8EC", chrome),        // a polished fork, bright to be seen in a hand
    pen = ("#1F3F7A", gloss),          // a ballpoint's barrel
    pencil = ("#E8B830", satin),       // a pencil's yellow paint
    pencil_wood = ("#D9B488", matte),
    graphite = ("#3A3A3A", satin),
    marker = ("#F2F0EA", satin),       // a whiteboard marker's body
    marker_cap = ("#2A5FB8", satin),
    handset = ("#1E2120", satin),      // the phone box's receiver (street_furniture's `phoneBooth` `dark`)
    // the cut flowers a vase takes (Q-18): the bunch's heads, eyes and stems
    petal_white = ("#F2F0EA", satin),
    petal_blush = ("#E4B8AE", satin),
    petal_rose = ("#E07B7B", satin),
    petal_yellow = ("#F2C14E", satin),
    flower_eye = ("#E8A62A", matte),
    stem_green = ("#4E8A3A", satin),   // a cut stem, a shade lighter than `leaf`
    // something fresh from the garden: a salad
    lettuce_pale = ("#A6CC5E", satin),
    tomato_flesh = ("#E8664E", satin),
    cucumber = ("#3E6B2C", satin),
    cucumber_flesh = ("#D4E6A4", satin),
    radish = ("#C8324A", gloss),
    radish_white = ("#F4EEF0", satin),
    berry_seed = ("#E8D07A", satin),   // a strawberry's seeds
}

/// Built in the game's axes (x, height, depth; front +depth), as the old parts are.
pub const F: GameFrame = GameFrame::new(0.0);

/// A game point as Blender's, through `F`, given where a helper takes one by name.
#[derive(Clone, Copy, Debug)]
pub struct G3(pub P3);

impl Default for G3 {
    fn default() -> G3 {
        G3(F.at(0, 0, 0))
    }
}
impl<A: Coord, B: Coord, C: Coord> artkit::Arg<G3> for (A, B, C) {
    fn arg(self) -> G3 {
        G3(F.at(self.0, self.1, self.2))
    }
}

/// `tuple(vector)`: a Blender vector's parts as numbers.
pub fn t3(v: V3) -> (f64, f64, f64) {
    (v.x as f64, v.y as f64, v.z as f64)
}

/// One material for every band, or one a band.
pub trait Bands {
    fn bands(self, n: usize) -> Vec<Mat>;
}
impl Bands for Mat {
    fn bands(self, n: usize) -> Vec<Mat> {
        vec![self; n]
    }
}
impl Bands for Vec<Mat> {
    fn bands(self, _: usize) -> Vec<Mat> {
        self
    }
}
impl<const N: usize> Bands for [Mat; N] {
    fn bands(self, _: usize) -> Vec<Mat> {
        self.to_vec()
    }
}

/// One material, or (edge, face).
pub trait Pair {
    fn pair(self) -> (Mat, Mat);
}
impl Pair for Mat {
    fn pair(self) -> (Mat, Mat) {
        (self, self)
    }
}
impl Pair for (Mat, Mat) {
    fn pair(self) -> (Mat, Mat) {
        self
    }
}

/// The materials in the order they first come.
fn ordered(mats: &[Mat]) -> Vec<Mat> {
    let mut order: Vec<Mat> = Vec::new();
    for &m in mats {
        if !order.contains(&m) {
            order.push(m);
        }
    }
    order
}

fn index_of(order: &[Mat], m: Mat) -> u16 {
    order
        .iter()
        .position(|&o| o == m)
        .expect("a material of the part") as u16
}

/// A moved point (`lathe`'s `deform`): Blender (x, y, z) to Blender (x, y, z).
pub type Deform = Rc<dyn Fn(V3) -> (f64, f64, f64)>;

/// `lathe`'s options.
#[derive(Clone)]
pub struct Lathe {
    pub at: G3,
    pub segments: usize,
    pub scale: (f64, f64),
    pub rot: P3,
    pub sharp: f64,
    pub wave: Option<(i32, Vec<(usize, f64)>)>,
    pub deform: Option<Deform>,
}

impl Default for Lathe {
    fn default() -> Lathe {
        Lathe {
            at: G3::default(),
            segments: 16,
            scale: (1.0, 1.0),
            rot: P3::ZERO,
            sharp: 40.0,
            wave: None,
            deform: None,
        }
    }
}

/// A closed surface of revolution about the game's up axis: `profile` [(radius, height)] from the
/// bottom's middle (radius 0) round to the top's (radius 0), each end a pole (no doubled points at the
/// axis). `mats`: one material, or one per band between profile points. `scale`: (game x, depth) to make
/// it oval. Edges sharper than `sharp` degrees are kept sharp (a rim, a lip); the rest is smooth.
/// `wave` (count, {profile index: amount}) flutes or lobes those rings, their radius times
/// 1 + amount * cos(count * angle): a fluted dish, a pleated cake case, a tomato's lobes, an ice cream scoop's
/// ragged lip. `deform` moves each point (local x, y, z; z along the axis) after: a sausage bent.
pub fn lathe(name: impl AsRef<str>, profile: &[(f64, f64)], mats: impl Bands, o: Lathe) -> Obj {
    let name = name.as_ref();
    let mats = mats.bands(profile.len() - 1);
    assert!(
        mats.len() == profile.len() - 1,
        "{name}: {} materials for {} bands",
        mats.len(),
        profile.len() - 1
    );
    let (count, amounts) = o.wave.clone().unwrap_or((0, Vec::new()));
    let mut verts: Vec<V3> = Vec::new();
    let mut rings: Vec<Vec<u32>> = Vec::new();
    for (k, &(r, h)) in profile.iter().enumerate() {
        if r < 1e-6 {
            rings.push(vec![verts.len() as u32]);
            verts.push(V3::from((0.0, 0.0, h)));
        } else {
            let amp = amounts
                .iter()
                .find(|&&(i, _)| i == k)
                .map_or(0.0, |&(_, a)| a);
            let mut ring = Vec::new();
            for i in 0..o.segments {
                let a = TAU * i as f64 / o.segments as f64;
                let rr = r * (1.0 + amp * cos(count as f64 * a));
                ring.push(verts.len() as u32);
                verts.push(V3::from((
                    rr * cos(a) * o.scale.0,
                    rr * sin(a) * o.scale.1,
                    h,
                )));
            }
            rings.push(ring);
        }
    }
    if let Some(deform) = &o.deform {
        for v in &mut verts {
            *v = V3::from(deform(*v));
        }
    }
    let order = ordered(&mats);
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let mut slots: Vec<u16> = Vec::new();
    for (band, w) in rings.windows(2).enumerate() {
        let (a, b) = (&w[0], &w[1]);
        for i in 0..o.segments {
            let j = (i + 1) % o.segments;
            faces.push(if a.len() == 1 {
                vec![a[0], b[j], b[i]]
            } else if b.len() == 1 {
                vec![a[i], a[j], b[0]]
            } else {
                vec![a[i], a[j], b[j], b[i]]
            });
            slots.push(index_of(&order, mats[band]));
        }
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    for &m in &order {
        mesh.slot(m);
    }
    for (f, &s) in mesh.faces.iter_mut().zip(&slots) {
        (f.mat, f.smooth) = (s, true);
    }
    // the edges between two faces turning more than `sharp` kept sharp
    let normals = mesh.face_normals();
    let mut by_edge: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (f, c) in faces.iter().enumerate() {
        for (i, &v) in c.iter().enumerate() {
            by_edge
                .entry(edge(v, c[(i + 1) % c.len()]))
                .or_default()
                .push(f);
        }
    }
    let sharp = radians(o.sharp);
    for (e, fs) in by_edge {
        if fs.len() == 2 && normals[fs[0]].angle(normals[fs[1]]) > sharp {
            mesh.sharp.insert(e);
        }
    }
    _object(name, mesh, o.at.0, o.rot)
}

/// `lathe` with its options by name: `lathe!(name, profile, mats, key = value, ...)`.
macro_rules! lathe {
    ($name:expr, $profile:expr, $mats:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $profile, $mats);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Lathe::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::lathe(args.0, args.1, args.2, o)
    }};
}

/// `prism`'s options.
#[derive(Clone, Debug, Default)]
pub struct Prism {
    pub bottom: Option<Mat>,
    pub sides: Option<Vec<Mat>>,
    pub at: (f64, f64),
}

/// A flat-shaded upright prism over a polygon of game (x, depth) points, from height `y0` to `y1`:
/// `top` on its top face, `side` (or `sides`, one per edge from point i to i + 1) round it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn prism(
    name: impl AsRef<str>,
    points: &[(f64, f64)],
    y0: f64,
    y1: f64,
    top: Mat,
    side: Mat,
    o: Prism,
) -> Obj {
    let at = o.at;
    let mut verts: Vec<V3> = points
        .iter()
        .map(|&(x, z)| V3::from(F.at(x + at.0, y0, z + at.1)))
        .collect();
    verts.extend(
        points
            .iter()
            .map(|&(x, z)| V3::from(F.at(x + at.0, y1, z + at.1))),
    );
    let bottom = o.bottom.unwrap_or(side);
    let mut mats = vec![top, bottom];
    mats.extend(o.sides.clone().unwrap_or_else(|| vec![side; points.len()]));
    let order = ordered(&mats);
    let n = points.len() as u32;
    let mut faces: Vec<Vec<u32>> = vec![(n..2 * n).collect(), (0..n).rev().collect()];
    let mut slots = vec![index_of(&order, top), index_of(&order, bottom)];
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push(vec![i, j, n + j, n + i]);
        slots.push(index_of(&order, mats[2 + i as usize]));
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    for &m in &order {
        mesh.slot(m);
    }
    for (f, &s) in mesh.faces.iter_mut().zip(&slots) {
        (f.mat, f.smooth) = (s, false);
    }
    mesh.recalc_normals();
    _object(name.as_ref(), mesh, P3::ZERO, P3::ZERO)
}

/// `prism` with its options by name: `prism!(name, points, y0, y1, top, side, key = value, ...)`.
macro_rules! prism {
    ($name:expr, $points:expr, $y0:expr, $y1:expr, $top:expr, $side:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $points, $y0, $y1, $top, $side);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Prism::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::prism(args.0, args.1, args.2, args.3, args.4, args.5, o)
    }};
}

/// A part of its own mesh (its materials in it), at `location` (Blender), turned `rot` degrees.
pub fn _object(name: &str, mesh: Mesh, location: P3, rot: P3) -> Obj {
    scene::add(name, mesh)
        .set_location(location)
        .set_rotation(rot)
}

/// `box_`'s options.
#[derive(Clone, Copy, Debug, Default)]
pub struct Box_ {
    pub yaw: f64,
    pub tilt: (f64, f64),
    pub bevel: f64,
}

/// A box of game width, height and depth centred at a game point, turned `yaw` degrees about the up
/// axis (and tilted (about game x, about game depth)).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn box_(
    name: impl AsRef<str>,
    w: f64,
    h: f64,
    d: f64,
    at: impl Spot,
    material: Mat,
    o: Box_,
) -> Obj {
    cube!(
        name,
        (w, d, h),
        at = at.through(&F),
        material = material,
        rot = (o.tilt.0, o.tilt.1, o.yaw),
        bevel = o.bevel,
        bevel_segments = 1
    )
}

/// `box_` with its options by name: `box_!(name, w, h, d, at, material, key = value, ...)`.
macro_rules! box_ {
    ($name:expr, $w:expr, $h:expr, $d:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $w, $h, $d, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Box_::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::box_(args.0, args.1, args.2, args.3, args.4, args.5, o)
    }};
}

/// `disc`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Disc {
    pub verts: usize,
    pub depth: Option<f64>,
    pub yaw: f64,
    pub tilt: (f64, f64),
}

impl Default for Disc {
    fn default() -> Disc {
        Disc {
            verts: 10,
            depth: None,
            yaw: 0.0,
            tilt: (0.0, 0.0),
        }
    }
}

/// An upright disc `d` across (`depth` deep, if oval) and `h` tall, its base at the game point `at`.
pub fn disc(
    name: impl AsRef<str>,
    d: f64,
    h: f64,
    at: (f64, f64, f64),
    material: Mat,
    o: Disc,
) -> Obj {
    let depth = o.depth.filter(|&v| v != 0.0).unwrap_or(d);
    cylinder!(
        name,
        d / 2.0,
        h,
        at = F.at(at.0, at.1 + h / 2.0, at.2),
        material = material,
        verts = o.verts,
        scale = (1.0, depth / d, 1.0),
        rot = (o.tilt.0, o.tilt.1, o.yaw),
        smooth = false
    )
}

/// `disc` with its options by name: `disc!(name, d, h, at, material, key = value, ...)`.
macro_rules! disc {
    ($name:expr, $d:expr, $h:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $d, $h, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Disc::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::disc(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `lump`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Lump {
    pub scale: (f64, f64, f64),
    pub seed: f64,
    pub subdivisions: usize,
    pub noise: f64,
    pub rough: f64,
}

impl Default for Lump {
    fn default() -> Lump {
        Lump {
            scale: (1.0, 1.0, 1.0),
            seed: 0.0,
            subdivisions: 1,
            noise: 0.25,
            rough: 0.0,
        }
    }
}

/// A lumpy bit of food (an icosphere shaken by noise), its base at the game point `at`; `scale` in
/// the game's (x, height, depth). `rough` shakes it again in features a third of it across (the boulder's own
/// noise is too broad to show on a thing a few centimetres across).
pub fn lump(name: impl AsRef<str>, r: f64, at: impl Spot, material: Mat, o: Lump) -> Obj {
    let obj = boulder!(
        name,
        r,
        at = at.through(&F),
        material = material,
        scale = (o.scale.0, o.scale.2, o.scale.1),
        seed = o.seed,
        subdivisions = o.subdivisions,
        noise = o.noise
    );
    if o.rough != 0.0 {
        jitter!(obj, o.rough * r, size = r * 0.7, seed = o.seed);
    }
    obj
}

/// `lump` with its options by name: `lump!(name, r, at, material, key = value, ...)`.
macro_rules! lump {
    ($name:expr, $r:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $r, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Lump::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::lump(args.0, args.1, args.2, args.3, o)
    }};
}

/// `ball`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Ball {
    pub scale: (f64, f64, f64),
    pub seg: usize,
    pub rings: usize,
}

impl Default for Ball {
    fn default() -> Ball {
        Ball {
            scale: (1.0, 1.0, 1.0),
            seg: 8,
            rings: 5,
        }
    }
}

/// A low sphere centred at the game point `at`; `scale` in the game's (x, height, depth).
pub fn ball(name: impl AsRef<str>, r: f64, at: impl Spot, material: Mat, o: Ball) -> Obj {
    sphere!(
        name,
        r,
        at = at.through(&F),
        material = material,
        scale = (o.scale.0, o.scale.2, o.scale.1),
        segments = o.seg,
        rings = o.rings
    )
}

/// `ball` with its options by name: `ball!(name, r, at, material, key = value, ...)`.
macro_rules! ball {
    ($name:expr, $r:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $r, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Ball::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::ball(args.0, args.1, args.2, args.3, o)
    }};
}

/// `chunk`'s and `dice`'s options.
#[derive(Clone, Copy, Debug, Default)]
pub struct Turn {
    pub yaw: f64,
    pub tilt: (f64, f64),
}

/// A chopped cube of food `s` across, centred at the game point `at`.
pub fn chunk(name: impl AsRef<str>, s: f64, at: impl Spot, material: Mat, o: Turn) -> Obj {
    box_(
        name,
        s,
        s * 0.8,
        s,
        at,
        material,
        Box_ {
            yaw: o.yaw,
            tilt: o.tilt,
            bevel: 0.0,
        },
    )
}

/// A diced piece of vegetable `s` across, its edges softened, centred at the game point `at`.
pub fn dice(name: impl AsRef<str>, s: f64, at: impl Spot, material: Mat, o: Turn) -> Obj {
    box_(
        name,
        s,
        s * 0.85,
        s,
        at,
        material,
        Box_ {
            yaw: o.yaw,
            tilt: o.tilt,
            bevel: s * 0.18,
        },
    )
}

/// `chunk` with its options by name: `chunk!(name, s, at, material, key = value, ...)`.
macro_rules! chunk {
    ($name:expr, $s:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $s, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Turn::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::chunk(args.0, args.1, args.2, args.3, o)
    }};
}

/// `dice` with its options by name: `dice!(name, s, at, material, key = value, ...)`.
macro_rules! dice {
    ($name:expr, $s:expr, $at:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $s, $at, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Turn::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::dice(args.0, args.1, args.2, args.3, o)
    }};
}

/// `roll`'s options.
#[derive(Clone, Debug)]
pub struct Roll {
    pub yaw: f64,
    pub segments: usize,
    pub marks: Vec<(f64, f64)>,
    pub bend: f64,
}

impl Default for Roll {
    fn default() -> Roll {
        Roll {
            yaw: 0.0,
            segments: 8,
            marks: Vec::new(),
            bend: 0.0,
        }
    }
}

/// A sausage (a capsule `r` thick, `length` long) lying along the game's x axis turned `yaw` degrees,
/// centred at the game point `at`. `marks`: the (from, to) stretches along it, from -1 to 1, that take
/// the second material (grill marks). `bend`: how far its middle bows sideways (towards -depth at yaw 0),
/// its ends staying where they were.
pub fn roll(name: impl AsRef<str>, r: f64, length: f64, mats: impl Pair, at: G3, o: Roll) -> Obj {
    let (body, mark) = mats.pair();
    let half = length / 2.0;
    let cap = r * 0.75;
    let mut stops: Vec<f64> = vec![-1.0, 1.0];
    stops.extend(o.marks.iter().flat_map(|m| [m.0, m.1]));
    stops.sort_by(f64::total_cmp);
    stops.dedup();
    let mut profile = vec![(0.0, -half), (r * 0.7, -half + r * 0.3)];
    profile.extend(stops.iter().map(|&t| (r, t * (half - cap))));
    profile.extend([(r * 0.7, half - r * 0.3), (0.0, half)]);
    let mut bands = vec![body, body];
    bands.extend(stops.windows(2).map(|w| {
        let (a, b) = (w[0], w[1]);
        if o.marks.iter().any(|m| m.0 <= a && b <= m.1) {
            mark
        } else {
            body
        }
    }));
    bands.extend([body, body]);
    let bend = o.bend;
    let bow: Deform = Rc::new(move |p: V3| {
        (
            p.x as f64,
            p.y as f64 + bend * (1.0 - pow(p.z as f64 / half, 2.0)),
            p.z as f64,
        )
    });
    lathe(
        name,
        &profile,
        bands,
        Lathe {
            at,
            segments: o.segments,
            rot: P3::from((0.0, 90.0, -o.yaw)),
            deform: if bend != 0.0 { Some(bow) } else { None },
            ..Lathe::default()
        },
    )
}

/// `roll` with its options by name: `roll!(name, r, length, mats, at, key = value, ...)`.
macro_rules! roll {
    ($name:expr, $r:expr, $length:expr, $mats:expr, $at:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $r, $length, $mats, artkit::Arg::<$crate::held::common::G3>::arg($at));
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Roll::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::roll(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `coin`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Coin {
    pub segments: usize,
    pub cup: f64,
    pub tilt: (f64, f64),
}

impl Default for Coin {
    fn default() -> Coin {
        Coin {
            segments: 8,
            cup: 0.35,
            tilt: (0.0, 0.0),
        }
    }
}

/// A thin round slice `d` across and `h` thick at its edge, the edge curled up a little (`cup`): pepperoni,
/// salami, a carrot coin; its base at the game point `at`. `mats`: one material or (edge, face).
pub fn coin(name: impl AsRef<str>, d: f64, h: f64, at: G3, mats: impl Pair, o: Coin) -> Obj {
    let (edge, face) = mats.pair();
    let r = d / 2.0;
    lathe(
        name,
        &[
            (0.0, 0.0),
            (r, 0.0),
            (r * 0.96, h * (1.0 + o.cup)),
            (0.0, h),
        ],
        vec![edge, edge, face],
        Lathe {
            at,
            segments: o.segments,
            rot: P3::from((o.tilt.0, o.tilt.1, 0.0)),
            ..Lathe::default()
        },
    )
}

/// `coin` with its options by name: `coin!(name, d, h, at, mats, key = value, ...)`.
macro_rules! coin {
    ($name:expr, $d:expr, $h:expr, $at:expr, $mats:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $d, $h, artkit::Arg::<$crate::held::common::G3>::arg($at), $mats);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Coin::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::coin(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `leaf`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Leaf {
    pub tilt: f64,
    pub t: f64,
    pub n: usize,
}

impl Default for Leaf {
    fn default() -> Leaf {
        Leaf {
            tilt: 0.0,
            t: 0.0012,
            n: 4,
        }
    }
}

/// A flat leaf (basil, parsley, mint, a scrap of greens) `length` long and `width` wide, its middle at the
/// game point `at` and its underside there, along game x turned `yaw` degrees, tipped `tilt` about its length.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn leaf(
    name: impl AsRef<str>,
    at: (f64, f64, f64),
    length: f64,
    width: f64,
    yaw: f64,
    material: Mat,
    o: Leaf,
) -> Obj {
    let n = o.n as f64;
    let side: Vec<(f64, f64)> = (0..=o.n)
        .map(|i| {
            let i = i as f64;
            (
                length * (i / n - 0.5),
                width / 2.0 * pow(sin(PI * i / n), 0.8),
            )
        })
        .collect();
    let mut outline = side.clone();
    outline.extend(side[1..side.len() - 1].iter().rev().map(|&(u, v)| (u, -v)));
    fextrude!(
        F,
        name,
        &outline,
        o.t,
        (at.0, at.1 + o.t / 2.0, at.2),
        material,
        plane = Facing::Top,
        rot = (o.tilt, 0.0, yaw)
    )
}

/// `leaf` with its options by name: `leaf!(name, at, length, width, yaw, material, key = value, ...)`.
macro_rules! leaf {
    ($name:expr, $at:expr, $length:expr, $width:expr, $yaw:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $at, $length, $width, $yaw, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Leaf::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::leaf(args.0, args.1, args.2, args.3, artkit::forms::num(args.4), args.5, o)
    }};
}

/// `blob`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Blob {
    pub seed: f64,
    pub yaw: f64,
    pub n: usize,
}

impl Default for Blob {
    fn default() -> Blob {
        Blob {
            seed: 0.0,
            yaw: 0.0,
            n: 8,
        }
    }
}

/// A thin, irregular smear or puddle `h` thick and about `rx` by `rz`, lying on the game point `at`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn blob(
    name: impl AsRef<str>,
    at: (f64, f64, f64),
    rx: f64,
    rz: f64,
    h: f64,
    material: Mat,
    o: Blob,
) -> Obj {
    let seed = o.seed;
    let pts: Vec<(f64, f64)> = (0..o.n)
        .map(|i| {
            let a = TAU * i as f64 / o.n as f64;
            let k = 1.0 + 0.16 * sin(3.0 * a + seed) + 0.09 * sin(5.0 * a + 2.3 * seed);
            (rx * k * cos(a), rz * k * sin(a))
        })
        .collect();
    fextrude!(
        F,
        name,
        &smooth_outline(&pts, 1, true),
        h,
        (at.0, at.1 + h / 2.0, at.2),
        material,
        plane = Facing::Top,
        rot = (0.0, 0.0, o.yaw)
    )
}

/// `blob` with its options by name: `blob!(name, at, rx, rz, h, material, key = value, ...)`.
macro_rules! blob {
    ($name:expr, $at:expr, $rx:expr, $rz:expr, $h:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $at, $rx, $rz, $h, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Blob::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::blob(args.0, args.1, args.2, args.3, args.4, args.5, o)
    }};
}

/// The (x, depth) points of a wedge's round edge (a slice of something round, its tip at `apex`, pointing
/// to the front): `steps` + 1 points from one side to the other.
pub fn arc(r: f64, a: f64, apex: (f64, f64), steps: usize) -> Vec<(f64, f64)> {
    (0..=steps)
        .map(|k| {
            let t = radians(-a + 2.0 * a * k as f64 / steps as f64);
            (apex.0 + r * sin(t), apex.1 - r * cos(t))
        })
        .collect()
}

/// `crust`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Crust {
    pub flat: f64,
    pub crimp: i32,
    pub bulge: f64,
    pub taper: f64,
    pub verts: usize,
    pub closed: bool,
}

impl Default for Crust {
    fn default() -> Crust {
        Crust {
            flat: 0.8,
            crimp: 0,
            bulge: 0.16,
            taper: 0.65,
            verts: 6,
            closed: false,
        }
    }
}

/// A crust or a pastry edge `r` round (squashed to `flat` of that in height) swept along game (x, depth) `pts`
/// at height `y` (its middle): its ends tapering unless `closed`, crimped `crimp` times along (a wave in its
/// thickness).
pub fn crust(
    name: impl AsRef<str>,
    pts: &[(f64, f64)],
    y: f64,
    r: f64,
    material: Mat,
    o: Crust,
) -> Obj {
    let prof: Vec<(f64, f64)> = (0..o.verts)
        .map(|i| {
            let a = TAU * i as f64 / o.verts as f64;
            (r * o.flat * cos(a), r * sin(a))
        })
        .collect();
    let (crimp, bulge, closed, taper) = (o.crimp, o.bulge, o.closed, o.taper);
    let scale = forms::by(move |f| {
        let mut s = if crimp != 0 {
            1.0 + bulge * cos(TAU * crimp as f64 * f)
        } else {
            1.0
        };
        if !closed {
            let m = if f < 1.0 - f { f } else { 1.0 - f } * 6.0;
            s *= taper + (1.0 - taper) * pow(if m < 1.0 { m } else { 1.0 }, 0.5);
        }
        s
    });
    let points: Vec<(f64, f64, f64)> = pts.iter().map(|&(x, z)| (x, y, z)).collect();
    fsweep!(
        F,
        name,
        &points,
        0.0,
        material,
        profile = prof,
        scale = scale,
        closed = o.closed,
        angle = 75
    )
}

/// `crust` with its options by name: `crust!(name, pts, y, r, material, key = value, ...)`.
macro_rules! crust {
    ($name:expr, $pts:expr, $y:expr, $r:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $pts, $y, $r, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Crust::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::crust(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `swirl`'s options.
#[derive(Clone, Copy, Debug)]
pub struct Swirl {
    pub tube: f64,
    pub turns: f64,
    pub steps: usize,
    pub star: usize,
}

impl Default for Swirl {
    fn default() -> Swirl {
        Swirl {
            tube: 0.008,
            turns: 2.2,
            steps: 14,
            star: 8,
        }
    }
}

/// Piped icing or cream: a tube `tube` round with `star` ridges (0: smooth, 6 sides) coiling up from radius
/// `r0` round the game point `at` (its base) to a peak `h` above it, thinning as it goes.
pub fn swirl(
    name: impl AsRef<str>,
    at: (f64, f64, f64),
    r0: f64,
    h: f64,
    material: Mat,
    o: Swirl,
) -> Obj {
    let tube = o.tube;
    let pts: Vec<(f64, f64, f64)> = (0..=o.steps)
        .map(|i| {
            let f = i as f64 / o.steps as f64;
            let a = TAU * o.turns * f;
            let rr = r0 * pow(1.0 - f, 0.85);
            (
                at.0 + rr * cos(a),
                at.1 + tube * 0.8 + (h - tube * 0.8) * pow(f, 0.9),
                at.2 + rr * sin(a),
            )
        })
        .collect();
    let prof: Vec<(f64, f64)> = if o.star != 0 {
        (0..o.star)
            .map(|i| {
                let a = TAU * i as f64 / o.star as f64;
                let k = if i % 2 == 0 { 1.0 } else { 0.78 };
                (cos(a) * k, sin(a) * k)
            })
            .collect()
    } else {
        forms::circle(1.0, 6)
    };
    fsweep!(
        F,
        name,
        &pts,
        0.0,
        material,
        profile = prof,
        scale = forms::by(move |f| tube * (1.0 - 0.8 * pow(f, 1.6))),
        angle = 80
    )
}

/// `swirl` with its options by name: `swirl!(name, at, r0, h, material, key = value, ...)`.
macro_rules! swirl {
    ($name:expr, $at:expr, $r0:expr, $h:expr, $m:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($name, $at, $r0, $h, $m);
        #[allow(unused_mut)]
        let mut o = $crate::held::common::Swirl::default();
        $(o.$k = artkit::Arg::arg($v);)*
        $crate::held::common::swirl(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// The Blender rotation (degrees) that turns Blender +x along the game direction `d`, rolling it no more than
/// that takes: the long axis of a spoon's bowl laid along its handle.
pub fn aim(d: (f64, f64, f64)) -> (f64, f64, f64) {
    let (bx, by, bz) = (-d.0, -d.2, d.1);
    let n = (bx * bx + by * by + bz * bz).sqrt();
    (0.0, -degrees(asin(bz / n)), degrees(atan2(by, bx)))
}

/// A rounded rectangle's (x, depth) points, `hw` by `hd` from its middle (`forms::ring`'s superellipse).
pub fn rrect(hw: f64, hd: f64, n: usize, square: f64) -> Vec<(f64, f64)> {
    let e = 2.0 / square;
    (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            let (c, s) = (cos(a), sin(a));
            (
                hw * pow(c.abs(), e).copysign(c),
                hd * pow(s.abs(), e).copysign(s),
            )
        })
        .collect()
}

/// A body lofted through `sections`, each [(x, height, depth)] in game points with the same count, bottom
/// first (`forms::loft`): a paper bag, a tray turned up at its edges, a box's lid.
pub fn loft(
    name: impl AsRef<str>,
    sections: &[Vec<(f64, f64, f64)>],
    material: Mat,
    angle: f64,
) -> Obj {
    let secs: Vec<Vec<P3>> = sections
        .iter()
        .map(|s| s.iter().map(|&p| p.through(&F)).collect())
        .collect();
    artkit::loft!(name, &secs, material = material, angle = angle)
}

/// A still model: everything joined into one node and stood on the floor, where it was built.
pub fn done(name: &str) -> Vec<Obj> {
    let obj = join_all(name);
    if std::env::var_os("HELD_MEASURE").is_some() {
        let (lo, hi) = measure(&[obj]);
        println!(
            "[art] {name} built centred on game ({:.4}, {:.4}), size {:.3} x {:.3} x {:.3}, bottom {:.4}",
            -(lo[0] + hi[0]) / 2.0,
            -(lo[1] + hi[1]) / 2.0,
            hi[0] - lo[0],
            hi[2] - lo[2],
            hi[1] - lo[1],
            lo[2]
        );
        return settle_with(&[obj], Settle::default());
    }
    settle_with(
        &[obj],
        Settle {
            centre: false,
            floor: true,
        },
    )
}

// --- the plate everything is served on ----------------------------------------------------------------------
/// The height of the plate's well, where the food sits.
pub const WELL: f64 = 0.009;

/// A dinner plate 29 cm across: a flat foot, a rim rising to 1.7 cm with a rolled edge, a well at 0.9 cm in a
/// paler glaze; its foot at the game point `at`. `covered`: under another plate (the stack), its well flat at the
/// rim's height, out of sight.
pub fn plate(name: &str, at: G3, covered: bool) {
    let mut profile = vec![
        (0.0, 0.0),
        (0.094, 0.0),
        (0.1438, 0.0156),
        (0.1408, 0.0172),
        (0.124, 0.0162),
    ];
    let mut mats = vec![M.china; 5];
    if covered {
        profile.push((0.0, 0.0162));
    } else {
        profile.extend([(0.088, WELL), (0.0, WELL)]);
        mats.push(M.china_well);
    }
    lathe(
        name,
        &profile,
        mats,
        Lathe {
            at,
            segments: if covered { 20 } else { 22 },
            ..Lathe::default()
        },
    );
}

/// The plate as it nearly always is: `plate`, at the origin.
pub fn plate0() {
    plate("plate", G3::default(), false);
}

/// A steel dessert spoon, its bowl at the game point `at`, the handle out along `yaw` (degrees from +x)
/// rising at `tilt` degrees: a flat handle widening to its end, an oval bowl along it.
pub fn _spoon(at: (f64, f64, f64), yaw: f64, tilt: f64, name: &str, reach: f64) {
    let (x, y, z) = at;
    let (a, t) = (radians(yaw), radians(tilt));
    let d = (cos(a) * cos(t), sin(t), -sin(a) * cos(t));
    let along = |k: f64| (x + d.0 * k, y + d.1 * k, z + d.2 * k);
    fsweep!(
        F,
        format!("{name}_handle"),
        &[along(0.012), along(0.05), along(reach)],
        0.0,
        M.steel,
        profile = rounded_rect(0.0024, 0.0075, 0.001, 1),
        scale = [0.55, 0.8, 1.25]
    );
    sphere!(
        format!("{name}_bowl"),
        1.0,
        at = F.at(x, y, z),
        material = M.steel,
        scale = (0.017, 0.012, 0.0042),
        rot = aim(d),
        segments = 8,
        rings = 5
    );
}
