use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Neg;
use core::ops::Sub;

#[derive(Copy, Clone, Debug, Default)]
pub struct v2 {
    pub X: f32,
    pub Y: f32,
}

impl Add for v2 {
    type Output = Self;
    fn add(self, B: Self) -> Self {
        let result = Self {
            X: self.X + B.X,
            Y: self.Y + B.Y,
        };
        result
    }
}

impl AddAssign for v2 {
    fn add_assign(&mut self, B: Self) {
        *self = *self + B;
    }
}

impl Sub for v2 {
    type Output = Self;
    fn sub(self, B: Self) -> Self {
        let result = Self {
            X: self.X - B.X,
            Y: self.Y - B.Y,
        };
        result
    }
}

impl Neg for v2 {
    type Output = Self;
    fn neg(self) -> Self {
        let result = Self {
            X: -self.X,
            Y: -self.Y,
        };
        result
    }
}

impl MulAssign<f32> for v2 {
    fn mul_assign(&mut self, B: f32) {
        *self = *self * B;
    }
}

impl Mul<f32> for v2 {
    type Output = v2;
    fn mul(self, B: f32) -> v2 {
        let result = v2 {
            X: self.X * B,
            Y: self.Y * B,
        };
        result
    }
}

impl Mul<v2> for f32 {
    type Output = v2;
    fn mul(self: f32, B: v2) -> v2 {
        let result = v2 {
            X: self * B.X,
            Y: self * B.Y,
        };
        result
    }
}

pub fn Square(x: f32) -> f32 {
    x * x
}

pub fn Inner(a: v2, b: v2) -> f32 {
    let result = a.X * b.X + a.Y * b.Y;
    return result;
}
