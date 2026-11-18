//! `Mesh::shade_smooth` (Blender's shade_smooth + set_sharp_from_angle) on edges that have more than two faces,
//! or two wound against each other, against Blender 5.2 (tests/golden/api_shade.txt): an edge is marked sharp by
//! the angle between the first two faces made on it, and a third face neither marks it nor takes its mark off.

mod common;

use artkit::math::V3;
use artkit::mesh::Mesh;
use common::golden;

#[test]
fn an_edge_of_three_faces_keeps_the_mark_its_first_two_gave_it() {
    let text = golden("api_shade.txt");
    let lines: Vec<&str> = text.lines().collect();
    let mut cases = 0;
    for block in lines.chunks(4) {
        let head: Vec<&str> = block[0].split(' ').collect();
        let (name, angle): (&str, f64) = (head[1], head[3].parse().unwrap());
        let coords: Vec<f32> = block[1]
            .split(' ')
            .skip(1)
            .map(|w| w.parse().unwrap())
            .collect();
        let verts: Vec<V3> = coords
            .chunks(3)
            .map(|c| V3::new(c[0], c[1], c[2]))
            .collect();
        let faces: Vec<Vec<u32>> = block[2]
            .split(' ')
            .skip(1)
            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
            .collect();
        let mut mesh = Mesh::from_pydata(verts, &faces);
        mesh.shade_smooth(angle);
        let got: Vec<String> = mesh.sharp.iter().map(|(a, b)| format!("{a}-{b}")).collect();
        let want: Vec<&str> = block[3]
            .split(' ')
            .skip(1)
            .filter(|w| !w.is_empty())
            .collect();
        assert_eq!(got, want, "{name}");
        cases += 1;
    }
    assert_eq!(cases, 9);
}
