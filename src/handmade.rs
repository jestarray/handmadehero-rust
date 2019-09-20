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
type memory_index = usize;
type bool32 = i32;
mod handmade_intrinsics;
use core::mem::size_of;
use handmade_intrinsics::*;
mod handmade_tile;
use handmade_tile::*;
use std::{convert::TryInto, ffi::c_void, ptr::null_mut};
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
    pub dtForFrame: f32,
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
struct world<'a> {
    TileMap: Option<&'a mut tile_map<'a>>,
}

struct memory_arena {
    Size: memory_index,
    Base: *mut u8,
    Used: memory_index,
}
impl Default for memory_arena {
    fn default() -> Self {
        memory_arena {
            Size: 0,
            Base: 0 as *mut u8,
            Used: 0,
        }
    }
}
#[derive(Default)]
pub struct GameState<'a> {
    WorldArena: memory_arena,
    world: Option<&'a mut world<'a>>,
    PlayerP: tile_map_position,
}

fn InitializeArena(Arena: &mut memory_arena, Size: memory_index, Base: *mut u8) {
    Arena.Size = Size;
    Arena.Base = Base;
    Arena.Used = 0;
}

/* #define PushStruct(Arena, type) (type *)PushSize_(Arena, sizeof(type))
#define PushArray(Arena, Count, type) (type *)PushSize_(Arena, (Count)*sizeof(type)) */

// can remove Size and be called with PushStruct::<TileMap>(&memory_arena)
unsafe fn PushStruct<T>(arena: &mut memory_arena) -> *mut T {
    PushSize_(arena, size_of::<T>()) as *mut T
}
unsafe fn PushArray<T>(arena: &mut memory_arena, count: u32) -> *mut T {
    PushSize_(arena, size_of::<T>()) as *mut T
}
unsafe fn PushSize_(Arena: &mut memory_arena, Size: memory_index) -> *mut u8 {
    //Assert((Arena->Used + Size) <= Arena->Size);
    let result = Arena.Base.offset(Arena.Used.try_into().unwrap());
    Arena.Used += Size;

    return result;
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
        let PlayerHeight: f32 = 1.4;
        let PlayerWidth: f32 = 0.75 * PlayerHeight;

        let mut game_state = &mut *(memory.permanent_storage as *mut GameState);
        if !(memory.is_initalized != 0) {
            game_state.PlayerP.AbsTileX = 1;
            game_state.PlayerP.AbsTileY = 3;
            game_state.PlayerP.TileRelX = 5.0;
            game_state.PlayerP.TileRelY = 5.0;

            InitializeArena(
                &mut game_state.WorldArena,
                (memory.permanent_storage_size - size_of::<GameState>() as u64)
                    .try_into()
                    .unwrap(),
                (memory.permanent_storage as *mut u8)
                    .offset(size_of::<GameState>().try_into().unwrap()),
            );

            game_state.world = Some(&mut *PushStruct::<world>(&mut game_state.WorldArena));
            let World = game_state.world.as_ref().unwrap();
            World.TileMap = Some(&mut *PushStruct::<tile_map>(&mut game_state.WorldArena));

            let mut TileMap = World.TileMap.unwrap();

            TileMap.ChunkShift = 4;
            TileMap.ChunkMask = (1 << TileMap.ChunkShift) - 1;
            TileMap.ChunkDim = (1 << TileMap.ChunkShift);

            TileMap.TileChunkCountX = 128;
            TileMap.TileChunkCountY = 128;
            TileMap.TileChunkCountZ = 2;
            TileMap.TileChunks = Some(&mut *PushArray::<tile_chunk>(
                &mut game_state.WorldArena,
                TileMap.TileChunkCountX * TileMap.TileChunkCountY * TileMap.TileChunkCountZ,
            ));

            TileMap.TileSideInMeters = 1.4;

            let mut RandomNumberIndex = 0;
            let mut TilesPerWidth = 17;
            let mut TilesPerHeight = 9;
            let mut ScreenX = 0;
            let mut ScreenY = 0;
            let mut AbsTileZ = 0;

            // TODO(casey): Replace all this with real world generation!
            let mut DoorLeft = false;
            let mut DoorRight = false;
            let mut DoorTop = false;
            let mut DoorBottom = false;
            let mut DoorUp = false;
            let mut DoorDown = false;
            for ScreenIndex in 0..100
            /*    (uint32 ScreenIndex = 0;
            ScreenIndex < 100;
            ++ScreenIndex) */
            {
                // TODO(casey): Random number generator!
                // Assert(RandomNumberIndex < ArrayCount(RandomNumberTable));

                let RandomChoice = 0;
                if (DoorUp || DoorDown) {
                    //RandomChoice = RandomNumberTable[RandomNumberIndex += 1] % 2;
                } else {
                    //RandomChoice = RandomNumberTable[RandomNumberIndex += 1] % 3;
                }

                if (RandomChoice == 2) {
                    if (AbsTileZ == 0) {
                        DoorUp = true;
                    } else {
                        DoorDown = true;
                    }
                } else if (RandomChoice == 1) {
                    DoorRight = true;
                } else {
                    DoorTop = true;
                }

                for TileY in 0..TilesPerHeight
                /*   (uint32 TileY = 0;
                TileY < TilesPerHeight;
                ++TileY) */
                {
                    for TileX in 0..TilesPerWidth
                    /* (uint32 TileX = 0;
                    TileX < TilesPerWidth;
                    ++TileX) */
                    {
                        let AbsTileX = ScreenX * TilesPerWidth + TileX;
                        let AbsTileY = ScreenY * TilesPerHeight + TileY;

                        let mut TileValue = 1;
                        if ((TileX == 0) && (!DoorLeft || (TileY != (TilesPerHeight / 2)))) {
                            TileValue = 2;
                        }

                        if ((TileX == (TilesPerWidth - 1))
                            && (!DoorRight || (TileY != (TilesPerHeight / 2))))
                        {
                            TileValue = 2;
                        }

                        if ((TileY == 0) && (!DoorBottom || (TileX != (TilesPerWidth / 2)))) {
                            TileValue = 2;
                        }

                        if ((TileY == (TilesPerHeight - 1))
                            && (!DoorTop || (TileX != (TilesPerWidth / 2))))
                        {
                            TileValue = 2;
                        }

                        if ((TileX == 10) && (TileY == 6)) {
                            if (DoorUp) {
                                TileValue = 3;
                            }

                            if (DoorDown) {
                                TileValue = 4;
                            }
                        }

                        SetTileValue(
                            &mut game_state.WorldArena,
                            World.TileMap.unwrap(),
                            AbsTileX,
                            AbsTileY,
                            AbsTileZ,
                            TileValue,
                        );
                    }
                }

                DoorLeft = DoorRight;
                DoorBottom = DoorTop;

                if (DoorUp) {
                    DoorDown = true;
                    DoorUp = false;
                } else if (DoorDown) {
                    DoorUp = true;
                    DoorDown = false;
                } else {
                    DoorUp = false;
                    DoorDown = false;
                }

                DoorRight = false;
                DoorTop = false;

                if (RandomChoice == 2) {
                    if (AbsTileZ == 0) {
                        AbsTileZ = 1;
                    } else {
                        AbsTileZ = 0;
                    }
                } else if (RandomChoice == 1) {
                    ScreenX += 1;
                } else {
                    ScreenY += 1;
                }
            }

            memory.is_initalized = true as bool32;
        }

        let mut World = game_state.world.unwrap();
        let mut TileMap = World.TileMap.unwrap();

        let TileSideInPixels = 60;
        let MetersToPixels = TileSideInPixels as f32 / TileMap.TileSideInMeters as f32;

        let LowerLeftX = (-TileSideInPixels / 2) as f32;
        let LowerLeftY = buffer.height as f32;

        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
            } else {
                // NOTE(casey): Use digital movement tuning
                let mut dPlayerX: f32 = 0.0; // pixels/second
                let mut dPlayerY: f32 = 0.0; // pixels/second

                if controller.move_up().ended_down != 0 {
                    dPlayerY = 1.0;
                }
                if controller.move_down().ended_down != 0 {
                    dPlayerY = -1.0;
                }
                if controller.move_left().ended_down != 0 {
                    dPlayerX = -1.0;
                }
                if controller.move_right().ended_down != 0 {
                    dPlayerX = 1.0;
                }
                dPlayerX *= 64.0;
                dPlayerY *= 64.0;

                // TODO(casey): Diagonal will be faster!  Fix once we have vectors :)
                let mut NewPlayerP = game_state.PlayerP;
                NewPlayerP.TileRelX += input.dtForFrame * dPlayerX;
                NewPlayerP.TileRelY += input.dtForFrame * dPlayerY;
                NewPlayerP = RecanonicalizePosition(TileMap, NewPlayerP);
                // TODO(casey): Delta function that auto-recanonicalizes

                let mut PlayerLeft = NewPlayerP;
                PlayerLeft.TileRelX -= 0.5 * PlayerWidth;
                PlayerLeft = RecanonicalizePosition(TileMap, PlayerLeft);

                let mut PlayerRight = NewPlayerP;
                PlayerRight.TileRelX += 0.5 * PlayerWidth;
                PlayerRight = RecanonicalizePosition(TileMap, PlayerRight);

                if IsTileMapPointEmpty(TileMap, NewPlayerP)
                    && IsTileMapPointEmpty(TileMap, PlayerLeft)
                    && IsTileMapPointEmpty(TileMap, PlayerRight)
                {
                    game_state.PlayerP = NewPlayerP;
                }
            }
        }

        DrawRectangle(
            &mut buffer,
            0.0,
            0.0,
            buffer.width as f32,
            buffer.height as f32,
            1.0,
            0.0,
            0.1,
        );

        let ScreenCenterX = 0.5 * buffer.width as f32;
        let ScreenCenterY = 0.5 * buffer.height as f32;

        for RelRow in -10..10
        /*     (int32 RelRow = -10;
        RelRow < 10;
        ++RelRow) */
        {
            for RelColumn in -20..20
            /*       (int32 RelColumn = -20;
            RelColumn < 20;
            ++RelColumn) */
            {
                let Column = (game_state.PlayerP.AbsTileX as i32 + RelColumn as i32) as u32;
                let Row = (game_state.PlayerP.AbsTileY as i32 + RelRow as i32) as u32;
                let TileID = GetTileValue(TileMap, Column, Row, game_state.PlayerP.AbsTileZ);

                if (TileID > 0) {
                    let mut Gray = 0.5;
                    if (TileID == 2) {
                        Gray = 1.0;
                    }

                    if (TileID > 2) {
                        Gray = 0.25;
                    }

                    if ((Column == game_state.PlayerP.AbsTileX)
                        && (Row == game_state.PlayerP.AbsTileY))
                    {
                        Gray = 0.0;
                    }

                    let CenX = ScreenCenterX - MetersToPixels * game_state.PlayerP.TileRelX
                        + (RelColumn as f32) * TileSideInPixels as f32;
                    let CenY = ScreenCenterY + MetersToPixels * game_state.PlayerP.TileRelY
                        - (RelRow as f32) * TileSideInPixels as f32;
                    let MinX = CenX - 0.5 * TileSideInPixels as f32;
                    let MinY = CenY - 0.5 * TileSideInPixels as f32;
                    let MaxX = CenX + 0.5 * TileSideInPixels as f32;
                    let MaxY = CenY + 0.5 * TileSideInPixels as f32;
                    DrawRectangle(buffer, MinX, MinY, MaxX, MaxY, Gray, Gray, Gray);
                }
            }
        }

        let PlayerR = 1.0;
        let PlayerG = 1.0;
        let PlayerB = 0.0;
        let PlayerLeft = ScreenCenterX - 0.5 * MetersToPixels * PlayerWidth;
        let PlayerTop = ScreenCenterY - MetersToPixels * PlayerHeight;
        DrawRectangle(
            buffer,
            PlayerLeft,
            PlayerTop,
            PlayerLeft + MetersToPixels * PlayerWidth,
            PlayerTop + MetersToPixels * PlayerHeight,
            PlayerR,
            PlayerG,
            PlayerB,
        );
    }
}

unsafe fn DrawRectangle(
    Buffer: &mut GameOffScreenBuffer,
    RealMinX: f32,
    RealMinY: f32,
    RealMaxX: f32,
    RealMaxY: f32,
    R: f32,
    G: f32,
    B: f32,
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

    let Color = ((R * 255.0).round() as u32) << 16
        | ((G * 255.0).round() as u32) << 8
        | ((B * 255.0).round() as u32) << 0;
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
