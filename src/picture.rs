use std::{f32, usize};

use convolve2d::{convolve2d, kernel, Matrix};

use crate::{quad_tree::Rect, sampler::Samples2d, Vector};

pub fn get_image() -> (Samples2d<Vector>, Samples2d<f32>) {
    let berset = image::open("./berset dunkel.jpg").unwrap().to_luma32f();
    let x = convolve2d(&berset, &kernel::sobel::x::<f32>());
    let y = convolve2d(&berset, &kernel::sobel::y::<f32>());
    let field: Vec<_> = x
        .get_data()
        .into_iter()
        .zip(y.get_data().into_iter())
        .map(|(x, y)| Vector::new(*x, *y))
        .collect();
    (
        Samples2d::new(
            field,
            berset.width() as usize,
            berset.height() as usize,
            Rect::new(0.0, 1.0, 0.0, 1.0),
        ),
        Samples2d::new(
            berset.to_vec(),
            berset.width() as usize,
            berset.height() as usize,
            Rect::new(0.0, 1.0, 0.0, 1.0),
        ),
    )
}
