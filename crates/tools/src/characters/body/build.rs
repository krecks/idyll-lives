//! Building the body: its parts, meta and joints.

use super::*;

/// The seat drop, eye height and grip of a body.
#[derive(Clone, Copy)]
pub struct Meta {
    pub seat_drop: f64,
    pub eye_y: f64,
    pub grip: [f64; 3],
}

pub struct Body {
    pub body: Mesh,
    pub lod: Mesh,
    pub meta: Meta,
    pub joints: [V3; NB],
    /// `body`'s positions without the eye sockets (`Surface::unsocketed`; its normals are so already).
    pub unsocketed: Vec<V3>,
    /// The body at level 2 (the close-up body's skin, `hd.body`) and its positions without the eye sockets.
    pub hd: Mesh,
    pub hd_unsocketed: Vec<V3>,
}

/// The body of `family` at `stage` (and its low-detail copy); see the module docs.
pub fn build_body(family: &str, stage: &str) -> Body {
    let s = surface(family, stage);
    let topo = topology();
    let body = Mesh {
        pos: s.pos.clone(),
        idx: tris(family).to_vec(),
        dense: s.dense.clone(),
        arm: s.tags.arm.clone(),
        nor: s.nor.clone(),
        ..Mesh::default()
    };
    let lod = Mesh {
        pos: s.lod_pos.clone(),
        idx: topo.lod_tris.concat(),
        dense: s.lod_dense.clone(),
        arm: s.lod_tags.arm.clone(),
        nor: s.lod_nor.clone(),
        ..Mesh::default()
    };
    let meta = Meta {
        seat_drop: seat_drop(&body, &s.p),
        eye_y: s.p[b("eye_l")].y,
        grip: grip(family, stage),
    };
    let hd = Mesh {
        pos: s.fine_pos.clone(),
        idx: tris_hd(family).to_vec(),
        dense: s.fine_dense.clone(),
        arm: s.fine_tags.arm.clone(),
        nor: s.fine_nor.clone(),
        ..Mesh::default()
    };
    Body {
        body,
        lod,
        meta,
        joints: s.p,
        unsocketed: s.unsocketed.clone(),
        hd,
        hd_unsocketed: s.fine_bare.clone(),
    }
}
