#![allow(non_snake_case)]
#![allow(dead_code)]
use crate::bool32;
use std::convert::TryInto;
pub fn RoundReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32.round() as i32;
    return result;
}

pub fn RoundReal32ToUInt32(Real32: f32) -> u32 {
    let result = Real32.round() as u32;
    return result;
}

pub fn FloorReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32.floor() as i32;
    return result;
}

pub fn TruncateReal32ToInt32(Real32: f32) -> i32 {
    let result = Real32 as i32;
    return result;
}

pub fn Sin(Angle: f32) -> f32 {
    let result = Angle.sin();
    return result;
}

pub fn Cos(Angle: f32) -> f32 {
    let result = Angle.cos();
    return result;
}

pub fn ATan2(Y: f32, X: f32) -> f32 {
    //is this equivilant to atan2(y, x)?
    let result = Y.atan2(X);
    return result;
}
#[derive(Default)]
pub struct bit_scan_result {
    pub Found: bool32,
    pub Index: u32,
}

pub fn FindLeastSignificantSetBit(Value: u32) -> bit_scan_result {
    let mut result = bit_scan_result::default();

    /* #if COMPILER_MSVC
        Result.Found = _BitScanForward((unsigned long *)&Result.Index, Value);
    #else */
    for Test in 0..32 {
        if (Value & (1 << Test)) != 0 {
            result.Index = Test;
            result.Found = (true as u32).try_into().unwrap();
            break;
        }
    }
    return result;
}
