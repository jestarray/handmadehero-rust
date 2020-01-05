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

pub fn LengthSq(a: v2) -> f32 {
    let result = Inner(a, a);
    return result;
}

pub fn GetMinCorner(Rect: rectangle2) -> v2 {
    let result = Rect.Min;
    return (result);
}

pub fn GetMaxCorner(Rect: rectangle2) -> v2 {
    let result = Rect.Max;
    return (result);
}

pub fn GetCenter(Rect: rectangle2) -> v2 {
    let result = 0.5 * (Rect.Min + Rect.Max);
    return (result);
}

#[derive(Default, Copy, Clone)]
pub struct rectangle2 {
    Min: v2,
    Max: v2,
}

pub fn RectMinMax(Min: v2, Max: v2) -> rectangle2 {
    let mut result = rectangle2::default();

    result.Min = Min;
    result.Max = Max;

    return result;
}

fn RectMinDim(Min: v2, Dim: v2) -> rectangle2 {
    let mut result = rectangle2::default();

    result.Min = Min;
    result.Max = Min + Dim;

    return result;
}

pub fn RectCenterHalfDim(Center: v2, HalfDim: v2) -> rectangle2 {
    let mut result = rectangle2::default();

    result.Min = Center - HalfDim;
    result.Max = Center + HalfDim;

    return result;
}

pub fn RectCenterDim(Center: v2, Dim: v2) -> rectangle2 {
    let result = RectCenterHalfDim(Center, 0.5 * Dim);

    return result;
}

pub fn IsInRectangle(Rectangle: rectangle2, Test: v2) -> bool {
    let result = (Test.X >= Rectangle.Min.X)
        && (Test.Y >= Rectangle.Min.Y)
        && (Test.X < Rectangle.Max.X)
        && (Test.Y < Rectangle.Max.Y);

    return result;
}
