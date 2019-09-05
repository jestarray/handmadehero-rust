//comment out for println! to work
//#![windows_subsystem = "windows"]
/*

type alies
typedef int8_t int8;
typedef int16_t int16;
typedef int32_t int32;
typedef int64_t int64;
typedef int32 bool32;

typedef uint8_t uint8;
typedef uint16_t uint16;
typedef uint32_t uint32;
typedef uint64_t uint64;

typedef float real32;
typedef double real64;

*/
mod win32_handmade;

use crate::win32_handmade::*;
use std::ffi::c_void;
pub struct GameOffScreenBuffer {
    memory: *mut c_void,
    width: i32,
    height: i32,
    pitch: i32,
}

#[derive(Default)]
pub struct GameInput {
    controllers: [GameControllerInput; 4],
}
#[derive(Default)]
struct GameControllerInput {
    is_analog: i32,

    is_down: i32,

    start_x: f32,
    start_y: f32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    end_x: f32,
    end_y: f32,

    buttons: [GameButtonState; 6],
}

impl GameControllerInput {
    fn up(&mut self) -> &mut GameButtonState {
        &mut self.buttons[0]
    }
    fn down(&mut self) -> &mut GameButtonState {
        &mut self.buttons[1]
    }
    fn left(&mut self) -> &mut GameButtonState {
        &mut self.buttons[2]
    }
    fn right(&mut self) -> &mut GameButtonState {
        &mut self.buttons[3]
    }
    fn left_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[4]
    }
    fn right_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[5]
    }
}

#[derive(Default)]
struct GameButtonState {
    half_transition_count: i32,
    ended_down: i32,
}
#[derive(Default)]
pub struct GameState {
    green_offset: i32,
    blue_offset: i32,
    tonehz: i32,
}
pub struct GameMemory {
    is_initalized: i32,
    permanent_storage_size: u64,
    transient_storage_size: u64,
    transient_storage: *mut c_void,
    permanent_storage: *mut c_void,
}
fn main() {
    create_window();
}

pub fn game_update_and_render(
    memory: &mut GameMemory,
    input: &mut GameInput,
    mut buffer: &mut GameOffScreenBuffer,
) {
     unsafe {
        //let game_state = memory.permanent_storage;
        let mut game_state = memory.permanent_storage as *mut GameState;
        if memory.is_initalized == 0 {
            (*game_state).tonehz = 256;
            (*game_state).green_offset = 0;
            (*game_state).blue_offset = 0;
            memory.is_initalized = 1;
        }
        let input_0 = &mut input.controllers[0];
        if input_0.is_analog != 0 {
            (*game_state).blue_offset += (4.0 * input_0.end_y) as i32;
        }

        if input_0.down().ended_down != 0 {
            (*game_state).green_offset += 1;
        }
        render_weird_gradient(
            &mut buffer,
            (*game_state).blue_offset,
            (*game_state).green_offset,
        )
    } 
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
