pub struct Camera {
    pub res: [f32; 2],
}

impl Camera {
    pub fn new() -> Self {
        Self {
            res: [1.0, 1.0],
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.res = [
            width.max(1.0),
            height.max(1.0),
        ];
    }
}

