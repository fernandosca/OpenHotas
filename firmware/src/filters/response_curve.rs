#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub struct ResponseCurve {
    enabled: bool,
}

impl ResponseCurve {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn apply(&self, input: f32) -> f32 {
        input.clamp(-1.0, 1.0)
    }
}
