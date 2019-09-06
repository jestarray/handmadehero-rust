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
use std::ptr::null_mut;
pub struct GameOffScreenBuffer {
    memory: *mut c_void,
    width: i32,
    height: i32,
    pitch: i32,
}

#[derive(Default)]
pub struct GameInput {
    //TODO(jest): insert clock values
    controllers: [GameControllerInput; 5],
}
#[derive(Default)]
struct GameControllerInput {
    is_connected: i32,
    is_analog: i32,

    stick_average_x: f32,
    stick_average_y: f32,

    buttons: [GameButtonState; 12],
}

impl GameControllerInput {
    fn move_up(&mut self) -> &mut GameButtonState {
        &mut self.buttons[0]
    }
    fn move_down(&mut self) -> &mut GameButtonState {
        &mut self.buttons[1]
    }
    fn move_left(&mut self) -> &mut GameButtonState {
        &mut self.buttons[2]
    }
    fn move_right(&mut self) -> &mut GameButtonState {
        &mut self.buttons[3]
    }

    fn action_up(&mut self) -> &mut GameButtonState {
        &mut self.buttons[4]
    }
    fn action_down(&mut self) -> &mut GameButtonState {
        &mut self.buttons[5]
    }
    fn action_left(&mut self) -> &mut GameButtonState {
        &mut self.buttons[6]
    }
    fn action_right(&mut self) -> &mut GameButtonState {
        &mut self.buttons[7]
    }

    fn left_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[8]
    }
    fn right_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[9]
    }
    fn back(&mut self) -> &mut GameButtonState {
        &mut self.buttons[10]
    }
    fn start(&mut self) -> &mut GameButtonState {
        &mut self.buttons[11]
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

pub struct DebugReadFile {
    content_size: u32,
    contents: *mut c_void,
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
        let mut game_state = memory.permanent_storage as *mut GameState;
        /*        if memory.is_initalized == 0 {
            (*game_state).tonehz = 256;
            (*game_state).green_offset = 0;
            (*game_state).blue_offset = 0;
            memory.is_initalized = 1;
            let file = debug_platform_read_entire_file("D:\\handmadehero-rust\\src\\main.rs");

            if file.contents != null_mut() {
                debug_platform_write_entire_file("HH_TEST.out", file.content_size, file.contents);
                debug_platform_free_file_memory(file.contents);
            }
        } */
        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
                (*game_state).blue_offset += (4.0 * controller.stick_average_x) as i32;
            } else {
                if controller.move_left().ended_down != 0 {
                    (*game_state).blue_offset -= 1;
                }
                if controller.move_right().ended_down != 0 {
                    (*game_state).blue_offset += 1;
                }
            }

            if controller.action_down().ended_down != 0 {
                (*game_state).green_offset += 1;
            }
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
