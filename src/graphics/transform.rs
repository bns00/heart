#[derive(Clone, Copy)]
struct Row {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Transform {
    x: Row,
    y: Row,
    z: Row,
}

impl Transform {
    pub(crate) fn identity() -> Self {
        Self {
            x: Row {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y: Row {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z: Row {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    pub(crate) fn translation(x: f32, y: f32) -> Self {
        Self {
            x: Row {
                x: 1.0,
                y: 0.0,
                z: x,
            },
            y: Row {
                x: 0.0,
                y: 1.0,
                z: y,
            },
            z: Row {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    pub(crate) fn scaling(x: f32, y: f32) -> Self {
        Self {
            x: Row { x, y: 0.0, z: 0.0 },
            y: Row { x: 0.0, y, z: 0.0 },
            z: Row {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    pub(crate) fn rotation(angle: f32) -> Self {
        Self {
            x: Row {
                x: angle.cos(),
                y: -angle.sin(),
                z: 0.0,
            },
            y: Row {
                x: angle.sin(),
                y: angle.cos(),
                z: 0.0,
            },
            z: Row {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    pub(crate) fn shearing(x: f32, y: f32) -> Self {
        Self {
            x: Row {
                x: 1.0,
                y: x,
                z: 0.0,
            },
            y: Row {
                x: y,
                y: 1.0,
                z: 0.0,
            },
            z: Row {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    pub(crate) fn combine(&self, other: &Self) -> Self {
        Self {
            x: Row {
                x: self.x.x * other.x.x + self.x.y * other.y.x + self.x.z * other.z.x,
                y: self.x.x * other.x.y + self.x.y * other.y.y + self.x.z * other.z.y,
                z: self.x.x * other.x.z + self.x.y * other.y.z + self.x.z * other.z.z,
            },
            y: Row {
                x: self.y.x * other.x.x + self.y.y * other.y.x + self.y.z * other.z.x,
                y: self.y.x * other.x.y + self.y.y * other.y.y + self.y.z * other.z.y,
                z: self.y.x * other.x.z + self.y.y * other.y.z + self.y.z * other.z.z,
            },
            z: Row {
                x: self.z.x * other.x.x + self.z.y * other.y.x + self.z.z * other.z.x,
                y: self.z.x * other.x.y + self.z.y * other.y.y + self.z.z * other.z.y,
                z: self.z.x * other.x.z + self.z.y * other.y.z + self.z.z * other.z.z,
            },
        }
    }

    pub(crate) fn translate(&self, x: f32, y: f32) -> Self {
        Self::translation(x, y).combine(self)
    }

    pub(crate) fn scale(&self, x: f32, y: f32) -> Self {
        Self::scaling(x, y).combine(self)
    }

    pub(crate) fn rotate(&self, angle: f32) -> Self {
        Self::rotation(angle).combine(self)
    }

    pub(crate) fn shear(&self, x: f32, y: f32) -> Self {
        Self::shearing(x, y).combine(self)
    }

    pub(crate) fn apply(&self, x: f32, y: f32) -> [f32; 2] {
        let vec = Row {
            x: self.x.x * x + self.x.y * y + self.x.z,
            y: self.y.x * x + self.y.y * y + self.y.z,
            z: self.z.x * x + self.z.y * y + self.z.z,
        };
        [vec.x / vec.z, vec.y / vec.z]
    }
}
