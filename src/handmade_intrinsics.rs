#![allow(non_snake_case)]
pub fn RoundReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32.round() as i32;
    return (result);
}

pub fn RoundReal32ToUInt32(Real32: f32) -> u32 {
    let result = (Real32.round() as u32);
    return (result);
}

pub fn FloorReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32.floor() as i32;
    return result;
}

pub fn TruncateReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32 as i32;
    return (result);
}

pub fn Sin(Angle: f32) -> f32 {
    let result = (Angle.sin());
    return (result);
}

pub fn Cos(Angle: f32) -> f32 {
    let result = (Angle.cos());
    return (result);
}

pub fn ATan2(Y: f32, X: f32) -> f32 {
    //is this equivilant to atan2(y, x)?
    let result = Y.atan2(X);
    return (result);
}
