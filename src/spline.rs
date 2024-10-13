use crate::Vector;
pub struct BSpline {
    points: Vec<Vector>,
    vectors: Vec<Vector>,
}

impl BSpline {
    pub fn new(points: Vec<Vector>, vectors: Vec<Vector>) -> Self {
        assert_eq!(
            points.len(),
            vectors.len(),
            "points and vectors must be of same lenght"
        );
        assert!(
            points.len() >= 2,
            "must contain at least two points and vectors"
        );
        Self { points, vectors }
    }

    pub fn push(&mut self, point: Vector, vector: Vector) {
        self.points.push(point);
        self.vectors.push(vector);
    }

    pub fn count(&self) -> usize {
        self.points.len() - 1
    }
}

impl BSpline {
    pub fn path(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t < self.count() as f32);
        let floor = t.floor() as usize;
        let fract = t.fract();
        (1.0 - fract).powi(3) * self.points[floor]
            + (1.0 - fract).powi(2) * fract * (self.points[floor] + self.vectors[floor])
            + (1.0 - fract) * fract.powi(2) * (self.points[floor + 1] - self.vectors[floor + 1])
            + fract.powi(3) * self.points[floor + 1]
    }

    pub fn derivative(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t < self.count() as f32);
        let floor = t.floor() as usize;
        let fract = t.fract();
        3.0 * (1.0 - fract).powi(2) * self.vectors[floor]
            + 6.0
                * (1.0 - fract)
                * fract
                * (self.points[floor + 1]
                    - self.points[floor]
                    - self.vectors[floor + 1]
                    - self.vectors[floor])
            + 3.0 * fract.powi(2) * self.points[floor + 1]
    }

    pub fn derivative2(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t < self.count() as f32);
        let floor = t.floor() as usize;
        let fract = t.fract();
        6.0 * (1.0 - fract)
            * (self.points[floor + 1]
                - self.points[floor]
                - self.vectors[floor + 1]
                - 2.0 * self.vectors[floor])
            - 6.0
                * fract
                * (self.points[floor] - self.points[floor + 1]
                    + self.vectors[floor]
                    + 2.0 * self.vectors[floor + 1])
    }

    pub fn length(&self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        let mut last_point = *self.points.first().unwrap();
        for i in 0..(self.count()) {
            for j in 0..steps_per_segment {
                let t = i as f32 + (j + 1) as f32 / steps_per_segment as f32;
                let new_point = self.path(t);
                sum += (new_point - last_point).norm();
                last_point = new_point;
            }
        }
        sum
    }

    pub fn curvature(&self, t: f32) -> f32 {
        let der = self.derivative(t);
        let der2 = self.derivative2(t);
        (der[0] * der2[1] - der2[0] * der[1]) / der.norm().powi(3)
    }

    pub fn curvature_integral(&self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in
            (0..self.count() * steps_per_segment).map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let der = self.derivative(t);
            let der2 = self.derivative2(t);
            sum += (der[0] * der2[1] - der2[0] * der[1]) / der.norm_squared();
        }

        sum
    }
}
