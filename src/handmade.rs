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
#![allow(bad_style)]
type memory_index = usize;
type bool32 = i32;
mod handmade_intrinsics;
use core::mem::size_of;
use handmade_intrinsics::*;
use std::convert::TryFrom;
mod handmade_tile;
use handmade_tile::*;
use std::{convert::TryInto, ffi::c_void, ptr::null_mut};
mod handmade_random;
use crate::handmade_random::RandomNumberTable;
mod handmade_math;
use crate::handmade_math::*;
struct loaded_bitmap {
    Width: i32,
    Height: i32,
    Pixels: *mut u32,
}

impl Default for loaded_bitmap {
    fn default() -> Self {
        loaded_bitmap {
            Width: 0,
            Height: 0,
            Pixels: 0 as *mut u32,
        }
    }
}
#[derive(Default)]
pub struct hero_bitmaps {
    AlignX: i32,
    AlignY: i32,
    Head: loaded_bitmap,
    Cape: loaded_bitmap,
    Torso: loaded_bitmap,
}
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
    TileMap: Option<&'a mut tile_map>,
}

#[derive(Debug)]
pub struct memory_arena {
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
#[derive(Default, Copy, Clone)]
struct entity {
    Exists: bool,
    P: tile_map_position,
    dP: v2,
    FacingDirection: u32,
    Width: f32,
    Height: f32,
}
pub struct GameState<'a> {
    WorldArena: memory_arena,
    world: Option<&'a mut world<'a>>,

    CameraFollowingEntityIndex: u32,
    CameraP: tile_map_position,

    PlayerIndexForController: [u32; 5], //GameInput::default().controllers.len() IS THE LENGTH, DOUBLE CHECK TO MATCH
    EntityCount: u32,
    Entities: [entity; 256],

    Backdrop: loaded_bitmap,
    HeroBitmaps: [hero_bitmaps; 4],

    //REMOVE
    HeroFacingDirection: u32,
    dPlayerP: v2,
    PlayerP: tile_map_position,
}

/* impl Default for GameState<'_, 'b> {
    fn default() -> Self {
        GameState {
            WorldArena: memory_arena::default(),
            CameraFollowingEntityIndex: 0,
            world: 0 as *mut world,
            CameraP: tile_map_position::default(),
            PlayerIndexForController: [0, 0, 0, 0, 0],
            EntityCount: 0,
            Entities: [entity::default(); 256],
            Backdrop: loaded_bitmap::default(),
            HeroBitmaps: [
                hero_bitmaps::default(),
                hero_bitmaps::default(),
                hero_bitmaps::default(),
                hero_bitmaps::default(),
            ],
        }
    }
} */

fn InitializeArena(Arena: &mut memory_arena, Size: memory_index, Base: *mut u8) {
    Arena.Size = Size;
    Arena.Base = Base;
    Arena.Used = 0;
}

/* #define PushStruct(Arena, type) (type *)PushSize_(Arena, sizeof(type))
#define PushArray(Arena, Count, type) (type *)PushSize_(Arena, (Count)*sizeof(type)) */

// can remove Size and be called with PushStruct::<TileMap>(&memory_arena)
unsafe fn PushStruct<T>(arena: &mut memory_arena) -> &mut T {
    &mut *(PushSize_(arena, size_of::<T>()) as *mut T)
}
unsafe fn PushArray<T>(arena: &mut memory_arena, count: u32) -> &mut T {
    &mut *(PushSize_(arena, count as usize * size_of::<T>()) as *mut T)
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
#[allow(unused)]
#[repr(packed)]
struct bitmap_header {
    FileType: u16,
    FileSize: u32,
    Reserved1: u16,
    Reserved2: u16,
    BitmapOffset: u32,
    Size: u32,
    Width: i32,
    Height: i32,
    Planes: u16,
    BitsPerPixel: u16,
    Compression: u32,
    SizeOfBitmap: u32,
    HorzResolution: i32,
    VertResolution: i32,
    ColorsUsed: u32,
    ColorsImportant: u32,

    RedMask: u32,
    GreenMask: u32,
    BlueMask: u32,
}

fn DrawBitmap(
    Buffer: &GameOffScreenBuffer,
    Bitmap: &loaded_bitmap,
    mut RealX: f32,
    mut RealY: f32,
    AlignX: i32,
    AlignY: i32,
) {
    RealX -= AlignX as f32;
    RealY -= AlignY as f32;
    let mut MinX = RoundReal32ToInt32(RealX);
    let mut MinY = RoundReal32ToInt32(RealY);
    let mut MaxX = RoundReal32ToInt32(RealX + Bitmap.Width as f32);
    let mut MaxY = RoundReal32ToInt32(RealY + Bitmap.Height as f32);

    let mut SourceOffsetX = 0;
    if MinX < 0 {
        SourceOffsetX = -MinX;
        MinX = 0;
    }

    let mut SourceOffsetY = 0;
    if MinY < 0 {
        SourceOffsetY = -MinY;
        MinY = 0;
    }

    if MaxX > Buffer.width {
        MaxX = Buffer.width;
    }

    if MaxY > Buffer.height {
        MaxY = Buffer.height;
    }

    // TODO(casey): SourceRow needs to be changed based on clipping.
    unsafe {
        let mut SourceRow: *mut u32 = Bitmap
            .Pixels
            .offset((Bitmap.Width * (Bitmap.Height - 1)).try_into().unwrap());
        SourceRow = SourceRow.offset(
            (-SourceOffsetY * Bitmap.Width + SourceOffsetX)
                .try_into()
                .unwrap(),
        );
        let mut DestRow: *mut u8 = (Buffer.memory as *mut u8).offset(
            (MinX * Buffer.bytes_per_pixel + MinY * Buffer.pitch)
                .try_into()
                .unwrap(),
        );
        for Y in MinY..MaxY
        /*    (int Y = MinY;
        Y < MaxY;
        ++Y) */
        {
            let mut Dest: *mut u32 = DestRow as *mut u32;
            let mut Source: *mut u32 = SourceRow;
            for X in MinX..MaxX
            /*    (int X = MinX;
            X < MaxX;
            ++X) */
            {
                let A = ((*Source >> 24) & 0xFF) as f32 / 255.0;
                let SR = ((*Source >> 16) & 0xFF) as f32;
                let SG = ((*Source >> 8) & 0xFF) as f32;
                let SB = ((*Source >> 0) & 0xFF) as f32;

                let DR = ((*Dest >> 16) & 0xFF) as f32;
                let DG = ((*Dest >> 8) & 0xFF) as f32;
                let DB = ((*Dest >> 0) & 0xFF) as f32;

                // TODO(casey): Someday, we need to talk about premultiplied alpha!
                // (this is not premultiplied alpha)
                let R = (1.0 - A) * DR + A * SR;
                let G = (1.0 - A) * DG + A * SG;
                let B = (1.0 - A) * DB + A * SB;

                *Dest = (((R + 0.5) as u32) << 16)
                    | (((G + 0.5) as u32) << 8)
                    | (((B + 0.5) as u32) << 0);

                Dest = Dest.offset(1);
                Source = Source.offset(1);
                /* ++Dest;
                ++Source; */
            }

            DestRow = DestRow.offset(Buffer.pitch.try_into().unwrap());
            SourceRow = SourceRow.offset((-Bitmap.Width).try_into().unwrap());
        }
    }
}

unsafe fn DEBUGLoadBMP(
    Thread: &thread_context,
    ReadEntireFile: unsafe fn(thread: &thread_context, file_name: &str) -> DebugReadFile,
    FileName: &str,
) -> loaded_bitmap {
    let mut result = loaded_bitmap::default();

    let ReadResult = ReadEntireFile(Thread, FileName);
    if ReadResult.content_size != 0 {
        let Header = &mut *((ReadResult.contents) as *mut bitmap_header);
        let Pixels: *mut u32 = ((ReadResult.contents as *mut u8)
            .offset(Header.BitmapOffset.try_into().unwrap()))
            as *mut u32;
        result.Pixels = Pixels;
        result.Width = Header.Width;
        result.Height = Header.Height;

        //Assert(Header.Compression == 3);

        // NOTE(casey): If you are using this generically for some reason,
        // please remember that BMP files CAN GO IN EITHER DIRECTION and
        // the height will be negative for top-down.
        // (Also, there can be compression, etc., etc... DON'T think this
        // is complete BMP loading code because it isn't!!)

        // NOTE(casey): Byte order in memory is determined by the Header itself,
        // so we have to read out the masks and convert the pixels ourselves.
        let RedMask = Header.RedMask;
        let GreenMask = Header.GreenMask;
        let BlueMask = Header.BlueMask;
        let AlphaMask = !(RedMask | GreenMask | BlueMask);
        let RedShift = FindLeastSignificantSetBit(RedMask);
        let GreenShift = FindLeastSignificantSetBit(GreenMask);
        let BlueShift = FindLeastSignificantSetBit(BlueMask);
        let AlphaShift = FindLeastSignificantSetBit(AlphaMask);

        //Assert(RedShift.Found);
        //Assert(GreenShift.Found);
        //Assert(BlueShift.Found);
        //Assert(AlphaShift.Found);
        let mut SourceDest: *mut u32 = Pixels;
        for Y in 0..Header.Height
        /*  (int32 Y = 0;
        Y < Header.Height;
        ++Y) */
        {
            for X in 0..Header.Width
            /*    (int32 X = 0;
            X < Header.Width;
            ++X) */
            {
                let C = *SourceDest;
                *SourceDest = (((C >> AlphaShift.Index) & 0xFF) << 24)
                    | (((C >> RedShift.Index) & 0xFF) << 16)
                    | (((C >> GreenShift.Index) & 0xFF) << 8)
                    | (((C >> BlueShift.Index) & 0xFF) << 0);
                SourceDest = SourceDest.offset(1);
            }
        }
    }

    return result;
}

/* fn GetEntity(GameState: &GameState, Index: u32) -> Option<&mut entity> {
    let mut Entity = None;

    if (Index > 0) && (Index < (GameState.Entities.len()).try_into().unwrap()) {
        Entity = Some(&mut GameState.Entities[usize::try_from(Index).unwrap()]);
    }

    return Entity;
}
fn InitializePlayer(GameState: &GameState, EntityIndex: u32) {
    let Entity = GetEntity(GameState, EntityIndex);

    match Entity {
        Some(Entity) => {
            Entity.Exists = true;
            Entity.P.AbsTileX = 1;
            Entity.P.AbsTileY = 3;
            Entity.P.Offset.X = 5.0;
            Entity.P.Offset.Y = 5.0;
            Entity.Height = 1.4;
            Entity.Width = 0.75 * Entity.Height;
        }
        None => {}
    }

    //if(!GetEntity(GameState, GameState->CameraFollowingEntityIndex)) c++ version
    if GetEntity(GameState, GameState.CameraFollowingEntityIndex).is_none() {
        GameState.CameraFollowingEntityIndex = EntityIndex;
    }
} */

fn AddEntity(GameState: &mut GameState) -> u32 {
    GameState.EntityCount += 1;
    let EntityIndex = GameState.EntityCount;

    let Entity = &mut GameState.Entities[usize::try_from(EntityIndex).unwrap()];
    *Entity = entity::default();

    return EntityIndex;
}

/*
//DO NOT FIX UNTIL TILEMAP IS MADE SAFE
//original takes gamestate opposed to tilemap but should probably pass in tilemap if it doesn't utilzie the gamestate struct fields
fn MovePlayer(GameState: &mut GameState, Entity: &mut entity, dt: f32, mut ddP: v2) {
    let TileMap = GameState.world.as_mut().unwrap().TileMap.as_mut().unwrap();

    if (ddP.X != 0.0) && (ddP.Y != 0.0) {
        ddP *= 0.707106781187;
    }

    let PlayerSpeed = 50.0; // m/s^2
    ddP *= PlayerSpeed;

    // TODO(casey): ODE here!
    ddP += -8.0 * Entity.dP;

    let OldPlayerP = Entity.P;
    let mut NewPlayerP = OldPlayerP;
    let PlayerDelta = 0.5 * ddP * Square(dt) + Entity.dP * dt;
    NewPlayerP.Offset += PlayerDelta;
    Entity.dP = ddP * dt + Entity.dP;
    NewPlayerP = RecanonicalizePosition(TileMap, NewPlayerP);
    // TODO(casey): Delta function that auto-recanonicalizes

    let mut PlayerLeft = NewPlayerP;
    PlayerLeft.Offset.X -= 0.5 * Entity.Width;
    PlayerLeft = RecanonicalizePosition(TileMap, PlayerLeft);
    let mut PlayerRight = NewPlayerP;
    PlayerRight.Offset.X += 0.5 * Entity.Width;
    PlayerRight = RecanonicalizePosition(TileMap, PlayerRight);

    let mut Collided = false;
    let mut ColP = tile_map_position::default();
    if (!IsTileMapPointEmpty(TileMap, NewPlayerP)) {
        ColP = NewPlayerP;
        Collided = true;
    }
    if (!IsTileMapPointEmpty(TileMap, PlayerLeft)) {
        ColP = PlayerLeft;
        Collided = true;
    }
    if (!IsTileMapPointEmpty(TileMap, PlayerRight)) {
        ColP = PlayerRight;
        Collided = true;
    }

    if (Collided) {
        let mut r = v2::default();
        if (ColP.AbsTileX < Entity.P.AbsTileX) {
            r = v2 { X: 1.0, Y: 0.0 };
        }
        if (ColP.AbsTileX > Entity.P.AbsTileX) {
            r = v2 { X: -1.0, Y: 0.0 };
        }
        if (ColP.AbsTileY < Entity.P.AbsTileY) {
            r = v2 { X: 0.0, Y: 1.0 };
        }
        if (ColP.AbsTileY > Entity.P.AbsTileY) {
            r = v2 { X: 0.0, Y: -1.0 };
        }

        Entity.dP = Entity.dP - 1.0 * Inner(Entity.dP, r) * r;
    } else {
        Entity.P = NewPlayerP;
    }
    /* #else
        uint32 MinTileX = 0;
        uint32 MinTileY = 0;
        uint32 OnePastMaxTileX = 0;
        uint32 OnePastMaxTileY = 0;
        uint32 AbsTileZ = Entity->P.AbsTileZ;
        tile_map_position BestPlayerP = Entity->P;
        real32 BestDistanceSq = LengthSq(PlayerDelta);
        for(uint32 AbsTileY = MinTileY;
            AbsTileY != OnePastMaxTileY;
            ++AbsTileY)
        {
            for(uint32 AbsTileX = MinTileX;
                AbsTileX != OnePastMaxTileX;
                ++AbsTileX)
            {
                tile_map_position TestTileP = CenteredTilePoint(AbsTileX, AbsTileY, AbsTileZ);
                uint32 TileValue = GetTileValue(TileMap, TestTileP);
                if(IsTileValueEmpty(TileValue))
                {
                    v2 MinCorner = -0.5f*v2{TileMap->TileSideInMeters, TileMap->TileSideInMeters};
                    v2 MaxCorner = 0.5f*v2{TileMap->TileSideInMeters, TileMap->TileSideInMeters};

                    tile_map_difference RelNewPlayerP = Subtract(TileMap, &TestTileP, &NewPlayerP);
                    v2 TestP = ClosestPointInRectangle(MinCorner, MaxCorner, RelNewPlayerP);
                    TestDistanceSq = ;
                    if(BestDistanceSq > TestDistanceSq)
                    {
                        BestPlayerP = ;
                        BestDistanceSq = ;
                    }
                }
            }
        }
    #endif */

    //
    // NOTE(casey): Update camera/player Z based on last movement.
    //
    if !AreOnSameTile(&OldPlayerP, &Entity.P) {
        //TODO: MAKE TILEMAP NOT TAKE IN RAW POINTER
        let NewTileValue = GetTileValue_P(TileMap, Entity.P);

        if NewTileValue == 3 {
            Entity.P.AbsTileZ += 1;
        } else if NewTileValue == 4 {
            Entity.P.AbsTileZ -= 1;
        }
    }

    if (Entity.dP.X == 0.0) && (Entity.dP.Y == 0.0) {
        // NOTE(casey): Leave FacingDirection whatever it was
    } else if Entity.dP.X.abs() > Entity.dP.Y.abs() {
        if Entity.dP.X > 0.0 {
            Entity.FacingDirection = 0;
        } else {
            Entity.FacingDirection = 2;
        }
    } else {
        if Entity.dP.Y > 0.0 {
            Entity.FacingDirection = 1;
        } else {
            Entity.FacingDirection = 3;
        }
    }
}
 */
#[no_mangle]
pub extern "C" fn game_update_and_render(
    thread: &thread_context,
    memory: &mut GameMemory,
    input: &mut GameInput,
    buffer: &mut GameOffScreenBuffer,
) {
    unsafe {
        let PlayerHeight: f32 = 1.4;
        let PlayerWidth: f32 = 0.75 * PlayerHeight;

        if !(memory.is_initalized != 0) {
            let mut game_state = &mut *(memory.permanent_storage as *mut GameState);
            println!("{:#?}", game_state.CameraFollowingEntityIndex);
            game_state.Backdrop = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_background.bmp",
            );
            let mut Bitmap = &mut game_state.HeroBitmaps[0];
            Bitmap.Head = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_right_head.bmp",
            );
            Bitmap.Cape = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_right_cape.bmp",
            );
            Bitmap.Torso = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_right_torso.bmp",
            );
            Bitmap.AlignX = 72;
            Bitmap.AlignY = 182;
            Bitmap = &mut game_state.HeroBitmaps[1];

            Bitmap.Head = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_back_head.bmp",
            );
            Bitmap.Cape = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_back_cape.bmp",
            );
            Bitmap.Torso = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_back_torso.bmp",
            );
            Bitmap.AlignX = 72;
            Bitmap.AlignY = 182;
            Bitmap = &mut game_state.HeroBitmaps[2];

            Bitmap.Head = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_left_head.bmp",
            );
            Bitmap.Cape = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_left_cape.bmp",
            );
            Bitmap.Torso = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_left_torso.bmp",
            );
            Bitmap.AlignX = 72;
            Bitmap.AlignY = 182;
            Bitmap = &mut game_state.HeroBitmaps[3];

            Bitmap.Head = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_front_head.bmp",
            );
            Bitmap.Cape = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_front_cape.bmp",
            );
            Bitmap.Torso = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_front_torso.bmp",
            );
            Bitmap.AlignX = 72;
            Bitmap.AlignY = 182;

            game_state.CameraP.AbsTileX = 17 / 2;
            game_state.CameraP.AbsTileY = 9 / 2;

            game_state.PlayerP.AbsTileX = 1;
            game_state.PlayerP.AbsTileY = 3;
            game_state.PlayerP.Offset.X = 5.0;
            game_state.PlayerP.Offset.Y = 5.0;
            {
                let mut game_state = &mut *(memory.permanent_storage as *mut GameState);
                let world_arena = &mut game_state.WorldArena;
                InitializeArena(
                    world_arena,
                    (memory.permanent_storage_size - size_of::<GameState>() as u64)
                        .try_into()
                        .unwrap(),
                    (memory.permanent_storage as *mut u8)
                        .offset(size_of::<GameState>().try_into().unwrap()),
                );
                game_state.world = Some(PushStruct::<world>(world_arena));
            }
            {
                let World = game_state.world.as_mut().unwrap();
                let world_arena = &mut game_state.WorldArena;
                World.TileMap = Some(PushStruct::<tile_map>(world_arena)); //REFERENCES CAN BE __MOVED__, THEY ARE NOT COPIED, SO WORLD.TILEMAP IS NOW THE NEW SCOPE OF THE ENTIRE REFERENCE LIFE TIME.
                let mut TileMap = World.TileMap.as_mut().unwrap();
                TileMap.ChunkShift = 4;
                TileMap.ChunkMask = (1 << TileMap.ChunkShift) - 1;
                TileMap.ChunkDim = 1 << TileMap.ChunkShift;

                TileMap.TileChunkCountX = 128;
                TileMap.TileChunkCountY = 128;
                TileMap.TileChunkCountZ = 2;
                TileMap.TileSideInMeters = 1.4;

                {
                    let mut game_state = &mut *(memory.permanent_storage as *mut GameState);
                    let World = game_state.world.as_mut().unwrap();
                    let world_arena = &mut game_state.WorldArena;
                    TileMap.TileChunks = &mut *PushArray::<tile_chunk>(
                        world_arena,
                        TileMap.TileChunkCountX * TileMap.TileChunkCountY * TileMap.TileChunkCountZ,
                    );
                }
            }

            let mut RandomNumberIndex = 0;
            let TilesPerWidth = 17;
            let TilesPerHeight = 9;
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

                let mut RandomChoice = 0;
                if DoorUp || DoorDown {
                    RandomNumberIndex += 1;
                    RandomChoice = RandomNumberTable[RandomNumberIndex] % 2;
                } else {
                    RandomNumberIndex += 1;
                    RandomChoice = RandomNumberTable[RandomNumberIndex] % 3;
                }

                let mut createdZDoor = false;
                if RandomChoice == 2 {
                    createdZDoor = true;
                    if AbsTileZ == 0 {
                        DoorUp = true;
                    } else {
                        DoorDown = true;
                    }
                } else if RandomChoice == 1 {
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
                        if (TileX == 0) && (!DoorLeft || (TileY != (TilesPerHeight / 2))) {
                            TileValue = 2;
                        }

                        if (TileX == (TilesPerWidth - 1))
                            && (!DoorRight || (TileY != (TilesPerHeight / 2)))
                        {
                            TileValue = 2;
                        }

                        if (TileY == 0) && (!DoorBottom || (TileX != (TilesPerWidth / 2))) {
                            TileValue = 2;
                        }

                        if (TileY == (TilesPerHeight - 1))
                            && (!DoorTop || (TileX != (TilesPerWidth / 2)))
                        {
                            TileValue = 2;
                        }

                        if (TileX == 10) && (TileY == 6) {
                            if DoorUp {
                                TileValue = 3;
                            }

                            if DoorDown {
                                TileValue = 4;
                            }
                        }

                        //2 MUTABLE BORROW IS ALLOWED IF THE LAST INSTANCE OF THE MUTABLE BORROW ISNT REUSED
                        let game_state = &mut *(memory.permanent_storage as *mut GameState);
                        let World = game_state.world.as_mut().unwrap();
                        SetTileValue(
                            &mut game_state.WorldArena,
                            &mut World.TileMap.as_mut().unwrap(),
                            AbsTileX,
                            AbsTileY,
                            AbsTileZ,
                            TileValue,
                        );
                    }
                }

                DoorLeft = DoorRight;
                DoorBottom = DoorTop;

                if createdZDoor {
                    DoorDown = !DoorDown;
                    DoorUp = !DoorUp;
                } else {
                    DoorUp = false;
                    DoorDown = false;
                }

                DoorRight = false;
                DoorTop = false;

                if RandomChoice == 2 {
                    if AbsTileZ == 0 {
                        AbsTileZ = 1;
                    } else {
                        AbsTileZ = 0;
                    }
                } else if RandomChoice == 1 {
                    ScreenX += 1;
                } else {
                    ScreenY += 1;
                }
            }
            memory.is_initalized = true as bool32;
        }

        let mut game_state = &mut *(memory.permanent_storage as *mut GameState);
        let World = game_state.world.as_ref().unwrap();
        let TileMap = World.TileMap.as_ref().unwrap();

        let TileSideInPixels = 60;
        let MetersToPixels = TileSideInPixels as f32 / TileMap.TileSideInMeters as f32;

        let LowerLeftX = -(TileSideInPixels / 2) as f32;
        let LowerLeftY = buffer.height as f32;

        for controller_index in 0..input.controllers.len() {
            let controller = &mut input.controllers[controller_index];
            if controller.is_analog != 0 {
            } else {
                let mut ddPlayer = v2::default();

                if controller.move_up().ended_down != 0 {
                    game_state.HeroFacingDirection = 1;
                    ddPlayer.Y = 1.0;
                }
                if controller.move_down().ended_down != 0 {
                    game_state.HeroFacingDirection = 3;
                    ddPlayer.Y = -1.0;
                }
                if controller.move_left().ended_down != 0 {
                    game_state.HeroFacingDirection = 2;
                    ddPlayer.X = -1.0;
                }
                if controller.move_right().ended_down != 0 {
                    game_state.HeroFacingDirection = 0;
                    ddPlayer.X = 1.0;
                }
                if (ddPlayer.X != 0.0) && (ddPlayer.Y != 0.0) {
                    ddPlayer *= 0.707106781187;
                }
                let mut PlayerSpeed = 10.0;
                if controller.action_up().ended_down != 0 {
                    PlayerSpeed = 50.0;
                }
                ddPlayer = ddPlayer * PlayerSpeed;

                ddPlayer += -1.5 * game_state.dPlayerP;

                let mut NewPlayerP = game_state.PlayerP;
                NewPlayerP.Offset = 0.5 * ddPlayer * Square(input.dtForFrame)
                    + game_state.dPlayerP * input.dtForFrame
                    + NewPlayerP.Offset;
                game_state.dPlayerP = ddPlayer * input.dtForFrame + game_state.dPlayerP;

                NewPlayerP = RecanonicalizePosition(TileMap, NewPlayerP);
                // TODO(casey): Delta function that auto-recanonicalizes

                let mut PlayerLeft = NewPlayerP;
                PlayerLeft.Offset.X -= 0.5 * PlayerWidth;
                PlayerLeft = RecanonicalizePosition(TileMap, PlayerLeft);

                let mut PlayerRight = NewPlayerP;
                PlayerRight.Offset.X += 0.5 * PlayerWidth;
                PlayerRight = RecanonicalizePosition(TileMap, PlayerRight);

                let mut Collided: bool = false;
                let mut ColP = tile_map_position::default();

                if !IsTileMapPointEmpty(TileMap, NewPlayerP) {
                    ColP = NewPlayerP;
                    Collided = true;
                }
                if !IsTileMapPointEmpty(TileMap, PlayerLeft) {
                    ColP = PlayerLeft;
                    Collided = true;
                }
                if !IsTileMapPointEmpty(TileMap, PlayerRight) {
                    ColP = PlayerRight;
                    Collided = true;
                }
                if Collided {
                    let mut r = v2 { X: 0.0, Y: 0.0 };

                    if ColP.AbsTileX < game_state.PlayerP.AbsTileX {
                        r = v2 { X: 1.0, Y: 0.0 }
                    }
                    if ColP.AbsTileX > game_state.PlayerP.AbsTileX {
                        r = v2 { X: -1.0, Y: 0.0 }
                    }
                    if ColP.AbsTileY < game_state.PlayerP.AbsTileY {
                        r = v2 { X: 0.0, Y: -1.0 }
                    }
                    if ColP.AbsTileY > game_state.PlayerP.AbsTileY {
                        r = v2 { X: 0.0, Y: -1.0 }
                    }

                    game_state.dPlayerP =
                        game_state.dPlayerP - 1.0 * Inner(game_state.dPlayerP, r) * r;
                } else {
                    if !AreOnSameTile(&game_state.PlayerP, &NewPlayerP) {
                        let NewTileValue = GetTileValue_P(TileMap, NewPlayerP);

                        if NewTileValue == 3 {
                            NewPlayerP.AbsTileZ += 1;
                        } else if NewTileValue == 4 {
                            NewPlayerP.AbsTileZ -= 1;
                        }
                    }
                    game_state.PlayerP = NewPlayerP;
                }

                game_state.CameraP.AbsTileZ = game_state.PlayerP.AbsTileZ;

                let Diff = Subtract(TileMap, &game_state.PlayerP, &game_state.CameraP);
                if Diff.dXY.X > (9.0 * TileMap.TileSideInMeters) {
                    game_state.CameraP.AbsTileX += 17;
                }
                if Diff.dXY.X < -(9.0 * TileMap.TileSideInMeters) {
                    game_state.CameraP.AbsTileX -= 17;
                }
                if Diff.dXY.Y > (5.0 * TileMap.TileSideInMeters) {
                    game_state.CameraP.AbsTileY += 9;
                }
                if Diff.dXY.Y < -(5.0 * TileMap.TileSideInMeters) {
                    game_state.CameraP.AbsTileY -= 9;
                }
            }
        }
        let width = buffer.width;
        let height = buffer.height;
        DrawBitmap(buffer, &game_state.Backdrop, 0.0, 0.0, 0, 0);
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
                let Column = game_state.CameraP.AbsTileX as u32 + RelColumn as u32;
                let Row = game_state.CameraP.AbsTileY as u32 + RelRow as u32;
                let TileID = GetTileValue(TileMap, Column, Row, game_state.CameraP.AbsTileZ);

                if TileID > 1 {
                    let mut Gray = 0.5;
                    if TileID == 2 {
                        Gray = 1.0;
                    }

                    if TileID > 2 {
                        Gray = 0.25;
                    }

                    if (Column == game_state.CameraP.AbsTileX)
                        && (Row == game_state.CameraP.AbsTileY)
                    {
                        Gray = 0.0;
                    }

                    let TileSide = v2 {
                        X: 0.5 * TileSideInPixels as f32,
                        Y: 0.5 * TileSideInPixels as f32,
                    };
                    let Cen = v2 {
                        X: ScreenCenterX - MetersToPixels * game_state.CameraP.Offset.X
                            + (RelColumn as f32) * TileSideInPixels as f32,
                        Y: ScreenCenterY + MetersToPixels * game_state.CameraP.Offset.Y
                            - (RelRow as f32) * TileSideInPixels as f32,
                    };
                    let Min = Cen - TileSide;
                    let Max = Cen + TileSide;
                    /*  v2 Min = Cen - TileSide;
                    v2 Max = Cen + TileSide; */
                    DrawRectangle(buffer, Min, Max, Gray, Gray, Gray);
                }
            }
        }

        let Diff = Subtract(TileMap, &game_state.PlayerP, &game_state.CameraP);

        let PlayerR = 1.0;
        let PlayerG = 1.0;
        let PlayerB = 0.0;
        let PlayerGroundPointX = ScreenCenterX + MetersToPixels * Diff.dXY.X;
        let PlayerGroundPointY = ScreenCenterY - MetersToPixels * Diff.dXY.Y;
        let PlayerLeftTop = v2 {
            X: PlayerGroundPointX - 0.5 * MetersToPixels * PlayerWidth,
            Y: PlayerGroundPointY - MetersToPixels * PlayerHeight,
        };
        let PlayerWidthHeight = v2 {
            X: PlayerWidth,
            Y: PlayerHeight,
        };
        DrawRectangle(
            buffer,
            PlayerLeftTop,
            PlayerLeftTop + MetersToPixels * PlayerWidthHeight,
            PlayerR,
            PlayerG,
            PlayerB,
        );

        let HeroBitmaps = &game_state.HeroBitmaps[game_state.HeroFacingDirection as usize];
        DrawBitmap(
            buffer,
            &HeroBitmaps.Torso,
            PlayerGroundPointX,
            PlayerGroundPointY,
            HeroBitmaps.AlignX,
            HeroBitmaps.AlignY,
        );
        DrawBitmap(
            buffer,
            &HeroBitmaps.Cape,
            PlayerGroundPointX,
            PlayerGroundPointY,
            HeroBitmaps.AlignX,
            HeroBitmaps.AlignY,
        );
        DrawBitmap(
            buffer,
            &HeroBitmaps.Head,
            PlayerGroundPointX,
            PlayerGroundPointY,
            HeroBitmaps.AlignX,
            HeroBitmaps.AlignY,
        );
    }
}

unsafe fn DrawRectangle(
    Buffer: &mut GameOffScreenBuffer,
    vMin: v2,
    vMax: v2,
    R: f32,
    G: f32,
    B: f32,
) {
    // TODO(casey): Floating point color tomorrow!!!!!!

    let mut MinX = (vMin.X).round() as i32;
    let mut MinY = (vMin.Y).round() as i32;
    let mut MaxX = (vMax.X).round() as i32;
    let mut MaxY = (vMax.Y).round() as i32;

    if MinX < 0 {
        MinX = 0;
    }

    if MinY < 0 {
        MinY = 0;
    }

    if MaxX > Buffer.width {
        MaxX = Buffer.width;
    }

    if MaxY > Buffer.height {
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
