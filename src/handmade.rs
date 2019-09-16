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
type bool32 = i32;
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
    pub dtForFrame: f32,
    pub controllers: [GameControllerInput; 5],
}
#[derive(Default)]
pub struct thread_context {
    pub place_hodler: i32,
}

struct tile_map {
    pub CountX: i32,
    pub CountY: i32,

    pub UpperLeftX: f32,
    pub UpperLeftY: f32,
    pub TileWidth: f32,
    pub TileHeight: f32,

    pub Tiles: *const u32, //look into making it so type implements trait Sized?
}

struct world {
    // TODO(casey): Beginner's sparseness
    pub TileMapCountX: i32,
    pub TileMapCountY: i32,

    pub TileMaps: *const tile_map,
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
    pub PlayerX: f32,
    pub PlayerY: f32,
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

unsafe fn GetTileMap(World: &world, TileMapX: i32, TileMapY: i32) -> *mut tile_map {
    let mut TileMap = 0 as *mut tile_map;

    if (TileMapX >= 0)
        && (TileMapX < World.TileMapCountX)
        && (TileMapY >= 0)
        && (TileMapY < World.TileMapCountY.try_into().unwrap())
    {
        TileMap = World.TileMaps.offset(
            (TileMapY * World.TileMapCountX + TileMapX)
                .try_into()
                .unwrap(),
        ) as *mut tile_map;
    }

    return TileMap;
}

unsafe fn GetTileValueUnchecked(TileMap: &tile_map, TileX: i32, TileY: i32) -> u32 {
    let TileMapValue = TileMap
        .Tiles
        .offset((TileY * TileMap.CountX + TileX).try_into().unwrap());
    return *TileMapValue;
}

unsafe fn IsTileMapPointEmpty(TileMap: &tile_map, TestX: f32, TestY: f32) -> bool32 {
    let mut Empty: bool32 = false as i32;

    let PlayerTileX = ((TestX - TileMap.UpperLeftX) / TileMap.TileWidth) as i32;
    let PlayerTileY = ((TestY - TileMap.UpperLeftY) / TileMap.TileHeight) as i32;

    if (PlayerTileX >= 0)
        && (PlayerTileX < TileMap.CountX)
        && (PlayerTileY >= 0)
        && (PlayerTileY < TileMap.CountY)
    {
        let TileMapValue = GetTileValueUnchecked(&TileMap, PlayerTileX, PlayerTileY);
        Empty = (TileMapValue == 0) as i32;
    }

    return Empty;
}

unsafe fn IsWorldPointEmpty(
    World: &world,
    TileMapX: i32,
    TileMapY: i32,
    TestX: f32,
    TestY: f32,
) -> bool32 {
    let mut Empty: bool32 = false as bool32;

    let TileMap = GetTileMap(World, TileMapX, TileMapY);
    if TileMap != null_mut() {
        let PlayerTileX = ((TestX - (*TileMap).UpperLeftX) / (*TileMap).TileWidth) as i32;
        let PlayerTileY = ((TestY - (*TileMap).UpperLeftY) / (*TileMap).TileHeight) as i32;

        if (PlayerTileX >= 0)
            && (PlayerTileX < (*TileMap).CountX)
            && (PlayerTileY >= 0)
            && (PlayerTileY < (*TileMap).CountY)
        {
            let TileMapValue = GetTileValueUnchecked(&(*TileMap), PlayerTileX, PlayerTileY);
            Empty = (TileMapValue == 0) as bool32;
        }
    }

    return Empty;
}

#[no_mangle]
pub extern "C" fn game_update_and_render(
    thread: &thread_context,
    memory: &mut GameMemory,
    input: &mut GameInput,
    mut buffer: &mut GameOffScreenBuffer,
) {
    unsafe {
        let TILE_MAP_COUNT_X = 17;
        let TILE_MAP_COUNT_Y = 9;
        //[9][17]
        let Tiles00 = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
            [1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let Tiles01 = [
            [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let Tiles10 = [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let Tiles11 = [
            [1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        //  let mut TileMaps[2][2];
        let tilemap00 = tile_map {
            Tiles: Tiles00.as_ptr() as *mut u32,
            CountX: TILE_MAP_COUNT_X,
            CountY: TILE_MAP_COUNT_Y,
            UpperLeftX: -30.0,
            UpperLeftY: 0.0,
            TileWidth: 60.0,
            TileHeight: 60.0,
        };
        let mut TileMaps = [[tilemap00]];
   

        /*   TileMaps[0][0].Tiles = (uint32 *)Tiles00;

           TileMaps[0][1] = TileMaps[0][0];
           TileMaps[0][1].Tiles = (uint32 *)Tiles01;

           TileMaps[1][0] = TileMaps[0][0];
           TileMaps[1][0].Tiles = (uint32 *)Tiles10;

           TileMaps[1][1] = TileMaps[0][0];
           TileMaps[1][1].Tiles = (uint32 *)Tiles11;

           tile_map *TileMap = &TileMaps[0][0];
        */
        let TileMap = &TileMaps[0][0];
        let World = world {
            TileMapCountX: 2,
            TileMapCountY: 2,

            TileMaps: TileMaps.as_ptr() as *mut tile_map,
        };

        let mut PlayerWidth = 0.75 * TileMap.TileWidth as f32;
        let mut PlayerHeight = TileMap.TileHeight;
        let mut game_state = memory.permanent_storage as *mut GameState;

        if memory.is_initalized == 0 {
            (*game_state).PlayerX = 150.0;
            (*game_state).PlayerY = 150.0;
            memory.is_initalized = 1;
        }

        let PlayerWidth = 0.75 * TileMap.TileWidth;
        let PlayerHeight = TileMap.TileHeight;

        let UpperLeftX: f32 = -30.0;
        let UpperLeftY: f32 = 0.0;
        let TileWidth: f32 = 60.0;
        let TileHeight: f32 = 60.0;

        let width = buffer.width;
        let height = buffer.height;

        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
            } else {
                // NOTE(casey): Use digital movement tuning
                let mut dPlayerX: f32 = 0.0; // pixels/second
                let mut dPlayerY: f32 = 0.0; // pixels/second

                if controller.move_up().ended_down != 0 {
                    dPlayerY = -1.0;
                }
                if controller.move_down().ended_down != 0 {
                    dPlayerY = 1.0;
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
                let NewPlayerX = (*game_state).PlayerX + input.dtForFrame * dPlayerX as f32;
                let NewPlayerY = (*game_state).PlayerY + input.dtForFrame * dPlayerY as f32;

                if IsTileMapPointEmpty(&TileMap, NewPlayerX - 0.5 * PlayerWidth, NewPlayerY) != 0
                    && IsTileMapPointEmpty(&TileMap, NewPlayerX + 0.5 * PlayerWidth, NewPlayerY)
                        != 0
                    && IsTileMapPointEmpty(&TileMap, NewPlayerX, NewPlayerY) != 0
                {
                    (*game_state).PlayerX = NewPlayerX;
                    (*game_state).PlayerY = NewPlayerY;
                }
            }
        }

        DrawRectangle(
            &mut buffer,
            0.0,
            0.0,
            width as f32,
            height as f32,
            1.0,
            0.0,
            0.1,
        );

        for (row_index, row) in Tiles00.iter().enumerate() {
            for (column_index, column) in row.iter().enumerate() {
                let tileID = GetTileValueUnchecked(&TileMap, column_index.try_into().unwrap(), row_index.try_into().unwrap());
                let mut Gray = 0.5;
                match tileID {
                    1 => Gray = 1.0,
                    _ => {}
                }
                let mut MinX = TileMap.UpperLeftX + (column_index as f32) * TileWidth;
                let mut MinY = TileMap.UpperLeftY + (row_index as f32) * TileHeight;
                let mut MaxX = MinX + TileMap.TileWidth;
                let mut MaxY = MinY + TileMap.TileHeight;
                DrawRectangle(buffer, MinX, MinY, MaxX, MaxY, Gray, Gray, Gray);
            }
        }
        let PlayerR = 1.0;
        let PlayerG = 1.0;
        let mut PlayerB = 0.0;

        let mut PlayerLeft = (*game_state).PlayerX - 0.5 * PlayerWidth;
        let mut PlayerTop = (*game_state).PlayerY - PlayerHeight as f32;

        DrawRectangle(
            buffer,
            PlayerLeft,
            PlayerTop,
            PlayerLeft + PlayerWidth,
            PlayerTop + PlayerHeight as f32,
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
