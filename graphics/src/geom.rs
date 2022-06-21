use std::ops;

/// A 2-dimensional vector.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Constructs a 2-dimensional vector.
    #[inline]
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x: x, y: y }
    }

    /// Computes the dot product between two vectors.
    #[inline]
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Considering the two given vectors as 3-dimensional vectors lying in the
    /// XY-plane, finds the z-coordinate of their cross product.
    #[inline]
    pub fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Computes the distance between two points.
    #[inline]
    pub fn distance(self, other: Vec2) -> f32 {
        (other - self).length()
    }

    /// Computes the length of a vector.
    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Finds the vector with the same direction and a length of 1.
    #[inline]
    pub fn normalized(self) -> Vec2 {
        (1.0 / self.length()) * self
    }

    /// Linearly interpolates between two vectors by the parameter `t`.
    #[inline]
    pub fn lerp(t: f32, a: Vec2, b: Vec2) -> Vec2 {
        (1.0 - t) * a + t * b
    }

    /// Finds the componentwise minimum of two vectors.
    #[inline]
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    /// Finds the componentwise maximum of two vectors.
    #[inline]
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other;
    }
}

impl ops::Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, other: Vec2) {
        *self = *self - other;
    }
}

impl ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

/// A 2×2 matrix, in row-major order.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat2x2(pub [f32; 4]);

impl Mat2x2 {
    /// Constructs a 2×2 matrix. Arguments are given in row-major order.
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Mat2x2 {
        Mat2x2([a, b, c, d])
    }

    /// Constructs an identity matrix.
    pub fn id() -> Mat2x2 {
        Mat2x2([1.0, 0.0, 0.0, 1.0])
    }

    /// Constructs a uniform scaling matrix.
    pub fn scale(scale: f32) -> Mat2x2 {
        Mat2x2([scale, 0.0, 0.0, scale])
    }

    /// Constructs a rotation matrix.
    pub fn rotate(angle: f32) -> Mat2x2 {
        Mat2x2([angle.cos(), angle.sin(), -angle.sin(), angle.cos()])
    }
}

impl ops::Mul<Mat2x2> for Mat2x2 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Mat2x2 {
        Mat2x2([
            self.0[0] * rhs.0[0] + self.0[1] * rhs.0[2],
            self.0[0] * rhs.0[1] + self.0[1] * rhs.0[3],
            self.0[2] * rhs.0[0] + self.0[3] * rhs.0[2],
            self.0[2] * rhs.0[1] + self.0[3] * rhs.0[3],
        ])
    }
}

impl ops::Mul<Vec2> for Mat2x2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.0[0] * rhs.x + self.0[1] * rhs.y,
            y: self.0[2] * rhs.x + self.0[3] * rhs.y,
        }
    }
}

impl ops::Mul<Mat2x2> for f32 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: Mat2x2) -> Mat2x2 {
        Mat2x2([
            self * rhs.0[0],
            self * rhs.0[1],
            self * rhs.0[2],
            self * rhs.0[3],
        ])
    }
}

impl ops::Mul<f32> for Mat2x2 {
    type Output = Mat2x2;
    #[inline]
    fn mul(self, rhs: f32) -> Mat2x2 {
        rhs * self
    }
}

/// A 2-dimensional affine transformation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub matrix: Mat2x2,
    pub offset: Vec2,
}

impl Transform {
    /// Constructs an affine transformation from the given transformation
    /// matrix and translation vector.
    pub fn new(matrix: Mat2x2, offset: Vec2) -> Transform {
        Transform { matrix, offset }
    }

    /// Constructs an identity transformation.
    pub fn id() -> Transform {
        Transform {
            matrix: Mat2x2::id(),
            offset: Vec2::new(0.0, 0.0),
        }
    }

    /// Constructs a translation.
    pub fn translate(x: f32, y: f32) -> Transform {
        Transform {
            matrix: Mat2x2::id(),
            offset: Vec2::new(x, y),
        }
    }

    /// Constructs a uniform scaling.
    pub fn scale(scale: f32) -> Transform {
        Transform {
            matrix: Mat2x2::scale(scale),
            offset: Vec2::new(0.0, 0.0),
        }
    }

    /// Constructs a rotation.
    pub fn rotate(angle: f32) -> Transform {
        Transform {
            matrix: Mat2x2::rotate(angle),
            offset: Vec2::new(0.0, 0.0),
        }
    }

    /// Sequentially composes two affine transformations.
    pub fn then(self, transform: Transform) -> Transform {
        Transform {
            matrix: transform.matrix * self.matrix,
            offset: transform.matrix * self.offset + transform.offset,
        }
    }

    /// Applies the affine transformation to the given vector.
    pub fn apply(self, vec: Vec2) -> Vec2 {
        self.matrix * vec + self.offset
    }
}
