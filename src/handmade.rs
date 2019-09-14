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
pub struct GameState {
    pub green_offset: i32,
    pub blue_offset: i32,
    pub tonehz: u32,
    pub t_sine: f32,

    pub player_x: i32,
    pub player_y: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub t_jump: f32,
}
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
            (*game_state).tonehz = 256;
            (*game_state).green_offset = 0;
            (*game_state).blue_offset = 0;
            (*game_state).t_sine = 0.0;
            memory.is_initalized = 1;
            (*game_state).player_x = 100;
            (*game_state).player_y = 100;
            /*       let file =
                (memory.debug_platform_read_entire_file)("D:\\handmadehero-rust\\src\\handmade.rs");

            if file.contents != null_mut() {
                (memory.debug_platform_write_entire_file)(
                    "HH_TEST.out",
                    file.content_size,
                    file.contents,
                );
                (memory.debug_platform_free_file_memory)(file.contents);
            } */
        }
        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
                (*game_state).blue_offset += (4.0 * controller.stick_average_x) as i32;
                (*game_state).tonehz = 256u32
                    .overflowing_add((128.0 * controller.stick_average_y) as u32)
                    .0;
            } else {
                if controller.move_left().ended_down != 0 {
                    (*game_state).blue_offset -= 1;
                }
                if controller.move_right().ended_down != 0 {
                    (*game_state).blue_offset += 1;
                }
            }

            (*game_state).player_x += (4.0 as f32 * controller.stick_average_x) as i32;
            (*game_state).player_y -= (5.0 as f32 * controller.stick_average_y) as i32;
            if (*game_state).t_jump > 0.0 {
                (*game_state).player_x -= (5.0 as f32
                    * (0.5 as f32 * std::f32::consts::PI * (*game_state).t_jump).sin())
                    as i32;
            }
            if controller.action_down().ended_down != 0 {
                (*game_state).t_jump = 4.0;
            }
            (*game_state).t_jump -= 0.033 as f32;
        }

        render_weird_gradient(
            &mut buffer,
            (*game_state).blue_offset,
            (*game_state).green_offset,
        );
        RenderPlayer(&mut buffer, (*game_state).player_x, (*game_state).player_y);

        RenderPlayer(&mut buffer, (*game_state).mouse_x, (*game_state).mouse_y);

        for ButtonIndex in 0..input.MouseButtons.len()
        /*
        (int ButtonIndex = 0;
            ButtonIndex < ArrayCount(Input->MouseButtons);
            ++ButtonIndex) */
        {
            if input.MouseButtons[ButtonIndex].ended_down != 0 {
                RenderPlayer(&mut buffer, 10 + 20 * ButtonIndex as i32, 10);
            }
        }
    }
}

unsafe fn RenderPlayer(Buffer: &mut GameOffScreenBuffer, PlayerX: i32, PlayerY: i32) {
    //uint8 *EndOfBuffer = (uint8 *)Buffer->Memory + Buffer->Pitch*Buffer->Height;
    let EndOfBuffer: *mut u8 = Buffer
        .memory
        .offset((Buffer.pitch * Buffer.height) as isize) as *mut u8;

    let Color: u32 = 0xFFFFFFFF;
    let Top = PlayerY;
    let Bottom = PlayerY + 10;
    let mut x = PlayerX;
    while x < PlayerX + 10 {
        x += 1;

        let mut Pixel: *mut u8 = Buffer
            .memory
            .offset((x * Buffer.bytes_per_pixel + Top * Buffer.pitch) as isize)
            as *mut u8;

        let mut y = Top;
        while y < Bottom {
            y += 1;
            if Pixel >= Buffer.memory as *mut u8 && (Pixel.offset(4)) <= EndOfBuffer {
                //let p = Pixel as *mut u32;
                *(Pixel as *mut u32) = Color;
            }
            Pixel = Pixel.offset(Buffer.pitch as isize);
        }
        /*     for(int Y = Top;
            Y < Bottom;
            ++Y)
        {
            if((Pixel >= Buffer->Memory) &&
               ((Pixel + 4) <= EndOfBuffer))
            {
                *(uint32 *)Pixel = Color;
            }

            Pixel += Buffer->Pitch;
        } */
    }
}

unsafe fn GameOutputSound(
    game_state: *mut GameState,
    buffer: &mut game_sound_output_buffer,
    tone_hz: u32,
) {
    let tone_volume = 3000;
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

#[no_mangle]
pub unsafe extern "C" fn GameGetSoundSamples(
    thread: &thread_context,
    Memory: &mut GameMemory,
    SoundBuffer: &mut game_sound_output_buffer,
) {
    let GameState = Memory.permanent_storage as *mut GameState;
    GameOutputSound(GameState, SoundBuffer, (*GameState).tonehz);
}
