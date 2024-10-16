use linewise::{
    polymer::Polymer,
    quad_tree::{BoundingBox, QuadTree},
    MyRng, Vector,
};
use rand::prelude::*;
use svg::{node::element::Group, Document, Node};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const ROOT_POLYMER_COUNT: usize = 25;

const MAX_SEGMENTS: usize = 4;
const OBJECTS_ON_LEAFS: usize = 5;

fn main() {
    let mut rng = MyRng::from_rng(thread_rng()).unwrap();
    let mut field = Vec::new();
    let mut potential = Vec::new();
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            let vec = Vector::new(i as f32, j as f32)
                - Vector::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
            field.push(nalgebra::Rotation2::new(std::f32::consts::FRAC_PI_2) * vec);
            potential.push((vec.norm() / 10.0).sin())
        }
    }
    let mut polymers = Vec::new();
    for i in 0..ROOT_POLYMER_COUNT {
        for j in 0..ROOT_POLYMER_COUNT {
            let x = (i as f32 + 0.5) * WIDTH as f32 / ROOT_POLYMER_COUNT as f32;
            let y = (j as f32 + 0.5) * WIDTH as f32 / ROOT_POLYMER_COUNT as f32;
            let position = Vector::new(x, y);
            polymers.push(Polymer::new_random(
                position,
                3.0,
                rng.gen_range(1..=MAX_SEGMENTS),
                &mut rng,
            ));
        }
    }

    let quad_tree = QuadTree::new(polymers, OBJECTS_ON_LEAFS);

    let mut selected = Group::new();
    let bounds = BoundingBox::new(30.0, 80.0, 100.0, 180.0);
    for polymer in quad_tree.query(bounds) {
        selected.append(polymer.as_path(0.8))
    }

    let as_vec: Vec<Polymer> = quad_tree.into();
    let mut group = Group::new();
    for polymer in as_vec {
        group.append(polymer.as_path(0.5))
    }

    let mut document = Document::new().set("viewBox", (0, 0, WIDTH, HEIGHT));
    document.append(group);
    document.append(selected);
    document.append(bounds.as_rect(0.4));
    svg::save("out/test.svg", &document).unwrap();
}
