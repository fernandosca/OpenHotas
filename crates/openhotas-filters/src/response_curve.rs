/// Piecewise linear response curve with 5 control points.
///
/// P0=(-1,-1), P2=(0,0), P4=(1,1) are fixed endpoints + center.
/// P1 (point_left) and P3 (point_right) are variable control points.
#[derive(Debug, Clone, Copy)]
pub struct ResponseCurve {
    points: [(f32, f32); 5],
}

impl ResponseCurve {
    pub fn new(p1: (f32, f32), p3: (f32, f32)) -> Self {
        Self {
            points: [
                (-1.0, -1.0),
                (p1.0.clamp(-1.0, 0.0), p1.1.clamp(-1.0, 1.0)),
                (0.0, 0.0),
                (p3.0.clamp(0.0, 1.0), p3.1.clamp(-1.0, 1.0)),
                (1.0, 1.0),
            ],
        }
    }

    pub fn apply(&self, input: f32) -> f32 {
        let input = input.clamp(-1.0, 1.0);

        for i in 0..4 {
            let (x0, y0) = self.points[i];
            let (x1, y1) = self.points[i + 1];
            if input <= x1 {
                let dx = x1 - x0;
                if dx.abs() < f32::EPSILON {
                    return y0.clamp(-1.0, 1.0);
                }
                let t = (input - x0) / dx;
                return (y0 + t * (y1 - y0)).clamp(-1.0, 1.0);
            }
        }

        1.0
    }

    pub fn set_points(&mut self, p1: (f32, f32), p3: (f32, f32)) {
        *self = Self::new(p1, p3);
    }
}

impl Default for ResponseCurve {
    fn default() -> Self {
        Self::new((-0.5, -0.5), (0.5, 0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_default() {
        let curve = ResponseCurve::default();
        assert!((curve.apply(-1.0) - (-1.0)).abs() < f32::EPSILON);
        assert!((curve.apply(0.0)).abs() < f32::EPSILON);
        assert!((curve.apply(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_output() {
        let curve = ResponseCurve::default();
        assert!(curve.apply(-1.5) >= -1.0);
        assert!(curve.apply(1.5) <= 1.0);
    }

    #[test]
    fn segment_interpolation() {
        let curve = ResponseCurve::new((-0.5, -0.8), (0.5, 0.8));
        let mid_left = curve.apply(-0.25);
        let mid_right = curve.apply(0.25);
        assert!(mid_left < -0.25);
        assert!(mid_right > 0.25);
    }

    #[test]
    fn set_points_roundtrip() {
        let mut rc = ResponseCurve::default();
        rc.set_points((-0.5, -1.0), (0.5, 1.0));
        assert!((rc.apply(0.5) - 1.0).abs() < f32::EPSILON);
    }
}
