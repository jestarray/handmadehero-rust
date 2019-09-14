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
use std::convert::TryInto;
use std::ffi::c_void;
use std::ptr::null_mut;
pub struct GameOffScreenBuffer {
    pub memory: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub pitch: i32,
    pub bytes_per_pixel: i32,
}
pub struct game_sound_output_buffer {
    pub SamplesPerSecond: u32,
    pub SampleCount: u32,

    pub samples: *mut i16,
}

#[derive(Default)]
pub struct GameInput {
    pub MouseButtons: [GameButtonState; 5],
    pub MouseX: i32,
    pub MouseY: i32,
    pub MouseZ: i32,
    pub SecondsToAdvanceOverUpdate: f32,
    pub controllers: [GameControllerInput; 5],
}
#[derive(Default)]
pub struct thread_context {
    pub place_hodler: i32,
}
#[derive(Default)]
pub struct GameControllerInput {
    pub is_connected: i32,
    pub is_analog: i32,

    pub stick_average_x: f32,
    pub stick_average_y: f32,

    pub buttons: [GameButtonState; 12],
}

impl GameControllerInput {
    pub fn move_up(&mut self) -> &mut GameButtonState {
        &mut self.buttons[0]
    }
    pub fn move_down(&mut self) -> &mut GameButtonState {
        &mut self.buttons[1]
    }
    pub fn move_left(&mut self) -> &mut GameButtonState {
        &mut self.buttons[2]
    }
    pub fn move_right(&mut self) -> &mut GameButtonState {
        &mut self.buttons[3]
    }

    pub fn action_up(&mut self) -> &mut GameButtonState {
        &mut self.buttons[4]
    }
    pub fn action_down(&mut self) -> &mut GameButtonState {
        &mut self.buttons[5]
    }
    pub fn action_left(&mut self) -> &mut GameButtonState {
        &mut self.buttons[6]
    }
    pub fn action_right(&mut self) -> &mut GameButtonState {
        &mut self.buttons[7]
    }

    pub fn left_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[8]
    }
    pub fn right_shoulder(&mut self) -> &mut GameButtonState {
        &mut self.buttons[9]
    }
    pub fn back(&mut self) -> &mut GameButtonState {
        &mut self.buttons[10]
    }
    pub fn start(&mut self) -> &mut GameButtonState {
        &mut self.buttons[11]
    }
}

#[derive(Default)]
pub struct GameButtonState {
    pub half_transition_count: i32,
    pub ended_down: i32,
}
#[derive(Default)]
pub struct GameState {}
pub struct GameMemory {
    pub is_initalized: i32,
    pub permanent_storage_size: u64,
    pub transient_storage_size: u64,
    pub transient_storage: *mut c_void,
    pub permanent_storage: *mut c_void,
    pub debug_platform_read_entire_file:
        unsafe fn(thread: &thread_context, file_name: &str) -> DebugReadFile,
    pub debug_platform_free_file_memory:
        unsafe fn(thread: &thread_context, memory: *mut std::ffi::c_void),
    pub debug_platform_write_entire_file: unsafe fn(
        thread: &thread_context,
        file_name: &str,
        memory_size: u32,
        memory: *mut std::ffi::c_void,
    ) -> bool,
}

pub struct DebugReadFile {
    pub content_size: u32,
    pub contents: *mut c_void,
}

#[no_mangle]
pub extern "C" fn game_update_and_render(
    thread: &thread_context,
    memory: &mut GameMemory,
    input: &mut GameInput,
    mut buffer: &mut GameOffScreenBuffer,
) {
    unsafe {
        let mut game_state = memory.permanent_storage as *mut GameState;
        if memory.is_initalized == 0 {
            memory.is_initalized = 1;
        }
        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
            } else {
            };
        }

        let width = buffer.width;
        let height = buffer.height;

        DrawRectangle(
            &mut buffer,
            0.0,
            0.0,
            width as f32,
            height as f32,
            0x00FF00FF,
        );
        DrawRectangle(&mut buffer, 10.0, 10.0, 40.0, 40.0, 0x0000FFFF);
    }
}

unsafe fn DrawRectangle(
    Buffer: &mut GameOffScreenBuffer,
    RealMinX: f32,
    RealMinY: f32,
    RealMaxX: f32,
    RealMaxY: f32,
    Color: u32,
) {
    // TODO(casey): Floating point color tomorrow!!!!!!

    let mut MinX = (RealMinX).round() as i32;
    let mut MinY = (RealMinY).round() as i32;
    let mut MaxX = (RealMaxX).round() as i32;
    let mut MaxY = (RealMaxY).round() as i32;

    if (MinX < 0) {
        MinX = 0;
    }

    if (MinY < 0) {
        MinY = 0;
    }

    if (MaxX > Buffer.width) {
        MaxX = Buffer.width;
    }

    if (MaxY > Buffer.height) {
        MaxY = Buffer.height;
    }

    let mut Row = Buffer
        .memory
        .offset((MinX * Buffer.bytes_per_pixel + MinY * Buffer.pitch) as isize)
        as *mut u8;
    let mut y = MinY;
    while y < MaxY {
        y += 1;
        let mut pixel = Row as *mut u32;

        let mut x = MinX;
        while x < MaxX {
            x += 1;

            *pixel = Color;
            pixel = pixel.offset(1);
        }
        Row = Row.offset(Buffer.pitch.try_into().unwrap());
    }
    /*   for(int Y = MinY;
        Y < MaxY;
        ++Y)
    {
        uint32 *Pixel = (uint32 *)Row;
        for(int X = MinX;
            X < MaxX;
            ++X)
        {
            *Pixel++ = Color;
        }

        Row += Buffer.Pitch;
    } */
}

unsafe fn GameOutputSound(
    game_state: *mut GameState,
    buffer: &mut game_sound_output_buffer,
    tone_hz: u32,
) {
    /*     let tone_volume = 3000;
    let wave_period = buffer.SamplesPerSecond / tone_hz;

    let mut sample_out = buffer.samples;

    for _ in 0..buffer.SampleCount {
        unsafe {
            let sine_value = (*game_state).t_sine.sin();
            let sample_value = (sine_value * tone_volume as f32) as i16;
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);
            (*sample_out) = sample_value;
            sample_out = sample_out.add(1);

            (*game_state).t_sine += (1.0 / wave_period as f32) * 2.0 * std::f32::consts::PI;
        }
    } */
}

/* unsafe fn render_weird_gradient(
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
 */

#[no_mangle]
pub unsafe extern "C" fn GameGetSoundSamples(
    thread: &thread_context,
    Memory: &mut GameMemory,
    SoundBuffer: &mut game_sound_output_buffer,
) {
    let GameState = Memory.permanent_storage as *mut GameState;
    GameOutputSound(GameState, SoundBuffer, 400);
}
