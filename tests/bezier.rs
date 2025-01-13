use linewise::{
    polymer::{OwnedPolymer, PolymerStorage},
    Vector,
};
use svg::{
    node::element::{Circle, Group, Line},
    Document, Node,
};

#[test]
fn visual_test() {
    let stroke_width = 15.0;
    let steps = 10;
    let owned_polymer = OwnedPolymer::new(
        vec![Vector::new(300.0, 300.0), Vector::new(600.0, 300.0)],
        vec![Vector::new(60.0, 300.0), Vector::new(60.0, -300.0)],
    );
    let mut storage = PolymerStorage::new();
    let polymer = storage.add_polymer(owned_polymer);
    let mut point_group = Group::new();
    let mut vec_group = Group::new();
    for seg in storage.get_borrowed_segments(&polymer) {
        for pos in seg.get_coord_matrix_0().column_iter() {
            point_group.append(
                Circle::new()
                    .set("cx", pos.x)
                    .set("cy", pos.y)
                    .set("r", stroke_width)
                    .set("fill", "yellow"),
            );
        }
        for (pos, der, _der2) in seg.all_iters(steps) {
            point_group.append(
                Circle::new()
                    .set("cx", pos.x)
                    .set("cy", pos.y)
                    .set("r", stroke_width)
                    .set("fill", "red"),
            );
            let end = pos + 1.0 / steps as f32 * der;
            vec_group.append(
                Line::new()
                    .set("x1", pos.x)
                    .set("y1", pos.y)
                    .set("x2", end.x)
                    .set("y2", end.y)
                    .set("stroke-width", stroke_width / 8.0)
                    .set("stroke", "#000000"),
            );
        }
    }
    let mut doc = Document::new().set("viewBox", (900.0, 900.0));
    doc.append(storage.as_path(&polymer, stroke_width));
    doc.append(point_group);
    doc.append(vec_group);
    svg::save("test_out/test.svg", &doc).unwrap()
}
