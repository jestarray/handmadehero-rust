//comment out for println! to work
//#![windows_subsystem = "windows"]

mod win32_handmade;

use crate::win32_handmade::*;
use winapi::ctypes::c_void;
pub struct GameOffScreenBuffer {
    memory: *mut c_void,
    width: i32,
    height: i32,
    pitch: i32,
}
fn main() {
    create_window();
}

pub fn game_update_and_render(mut buffer: &mut GameOffScreenBuffer, offset_x: i32, offset_y: i32) {
    unsafe { render_weird_gradient(&mut buffer, 0, 0) }
}

unsafe fn render_weird_gradient(
    buffer: &mut GameOffScreenBuffer,
    blue_offset: i32,
    green_offset: i32,
) {
    let mut row = buffer.memory as *mut u8;
    for y in 0..buffer.height {
        let mut pixel = row as *mut [u8; 4]; //array of 4, u8s
        for x in 0..buffer.width {
            *pixel = [(x + blue_offset) as u8, (y + green_offset) as u8, 0, 0];
            pixel = pixel.offset(1); // adds sizeof(pixel), 4
        }
        row = row.offset(buffer.pitch as isize);
    }
}
