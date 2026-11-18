//! What the playground is built from: its palette, its frame, and the few shapes it makes itself.

use artkit::forms;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::palette;
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::shapes::CLOSED;

palette! { M:
    body = ("#2E6FA8", metal),         // painted steel frames: the slide's tower, the seesaw's stand
    chute = ("#F2C14E", gloss),        // the slide's chute
    roof = ("#C8402F", gloss),         // the slide's roof
    tread = ("#C9A06A", wood),         // the slide's deck and ladder rungs
    rail = ("#C3C7CB", metal),         // galvanised rails, handles and poles
    steel = ("#5C6168", metal),        // axles, brackets, hubs, base plates
    rubber = ("#2A2C2E", matte),       // bumpers, grips, the ball's seams
    plank = ("#F2C14E", gloss),        // the seesaw's plank
    seat = ("#D9483B", gloss),         // the seesaw's seats
    horse = ("#D9483B", gloss),        // the spring rider's horse
    saddle = ("#2E6FA8", gloss),       // its saddle
    mane = ("#F2F0EA", matte),         // its mane and tail
    eye = ("#1A1A1A", gloss),
    grip = ("#F2C14E", gloss),         // the spring rider's handles and foot pegs
    spring = ("#3C6B3A", metal),       // the spring rider's coil
    platform = ("#C8402F", gloss),     // the merry-go-round's deck
    deck = ("#F2C14E", gloss),         // its stripes
    timber = ("#C49A62", wood),        // the climbing frame
    rope = ("#D8C9A6", fabric),        // its net
    shingle = ("#B5463A", satin),      // its roof
    flag = ("#F2C14E", satin),
    post = ("#3E4347", metal),         // the hoop's post, arm and ballast
    board = ("#F4F4F2", gloss),        // the backboard
    board_mark = ("#C8402F", gloss),   // its border and target square
    rim = ("#E0602A", gloss),          // the hoop
    net = ("#F2F2F0", fabric),         // the net (its own part: one draw)
    ball = ("#D86A2A", matte),
    table_top = ("#2F6B4F", satin),    // the ping-pong table's top
    lines = ("#F2F2F0", satin),        // its white lines
    concrete = ("#A9ADB0", matte),     // its legs
    ping = ("#F7F5EE", gloss),         // the ping-pong ball
    stone = ("#C9C2B4", matte),        // the chess table's top
    stool = ("#8E8C88", matte),        // its pedestal and stools
    square_light = ("#E8DFC8", satin),
    square_dark = ("#3A3330", satin),
    piece_white = ("#F2EDE2", gloss),
    piece_black = ("#2A2624", gloss),
}

pub const F: GameFrame = GameFrame::new(0.0);
pub const X: V3 = V3::new(1.0, 0.0, 0.0);

/// A mesh object from points and faces (Blender world), its normals pointing out.
pub fn mesh(name: &str, verts: Vec<V3>, faces: &[Vec<u32>], material: Mat, smooth: bool) -> Obj {
    forms::object(name, verts, faces, Some(material), smooth, CLOSED, false)
}
