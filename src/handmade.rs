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
#[derive(Default, Debug)]
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
struct high_entity {
    P: v2, // NOTE(casey): Relative to the camera!
    dP: v2,
    AbsTileZ: u32,
    FacingDirection: u32,

    Z: f32,
    dZ: f32,

    LowEntityIndex: u32,
}

#[derive(Copy, Clone)]
enum entity_type {
    EntityType_Null,
    EntityType_Hero,
    EntityType_Wall,
}
#[derive(Copy, Clone)]
struct low_entity {
    Type: entity_type,

    P: tile_map_position,
    Width: f32,
    Height: f32,
    // NOTE(casey): This is for "stairs"
    Collides: bool,
    dAbsTileZ: i32,

    HighEntityIndex: u32,
}

#[derive(Default)]
struct entity<'a> {
    LowIndex: u32,
    Low: Option<&'a mut low_entity>,
    High: Option<&'a mut high_entity>,
}

pub struct GameState<'a> {
    WorldArena: memory_arena,
    world: Option<&'a mut world<'a>>,

    CameraFollowingEntityIndex: u32,
    CameraP: tile_map_position,

    PlayerIndexForController: [u32; 5], //GameInput::default().controllers.len() IS THE LENGTH, DOUBLE CHECK TO MATCH

    LowEntityCount: u32,
    LowEntities: [low_entity; 4096],

    HighEntityCount: u32,
    HighEntities_: [high_entity; 256],

    Backdrop: loaded_bitmap,
    Shadow: loaded_bitmap,
    HeroBitmaps: [hero_bitmaps; 4],
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
    CAlpha: f32,
) {
    RealX -= AlignX as f32;
    RealY -= AlignY as f32;
    let mut MinX = RoundReal32ToInt32(RealX);
    let mut MinY = RoundReal32ToInt32(RealY);
    let mut MaxX = MinX + Bitmap.Width;
    let mut MaxY = MinY + Bitmap.Height;

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
                let mut A = ((*Source >> 24) & 0xFF) as f32 / 255.0;
                A *= CAlpha;

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
//NOT IN SYNC BECAUSE OF _ROTL INTRINSIC, SAME FUNCTIONALITY THOUGH ATM
// SEE EPISODE 46 , 1:38:00
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

fn GetController(Input: &mut GameInput, controller_index: u32) -> &mut GameControllerInput {
    let result = &mut Input.controllers[controller_index as usize];
    return result;
}

fn GetLowEntity<'a>(GameState: &'a mut GameState, Index: u32) -> Option<&'a mut low_entity> {
    let mut Entity = None;
    if (Index > 0) && (Index < (GameState.LowEntityCount)) {
        Entity = Some(&mut GameState.LowEntities[usize::try_from(Index).unwrap()]);
    }

    return Entity;
}

//MAY BE BUGGED??
// CHANGE TO USE RAW POINTERS? BECAUSE OF POINTER ARITHMETIC. GOING TO TRY USING REGULAR INDEXING INTO ARRAY ATM
fn MakeEntityHighFrequency<'a>(
    GameState: &'a mut GameState,
    LowIndex: u32,
) -> Option<&'a mut high_entity> {
    let mut EntityHigh = None;

    let mut EntityLow = &mut GameState.LowEntities[LowIndex as usize];
    if EntityLow.HighEntityIndex != 0 {
        EntityHigh = Some(&mut GameState.HighEntities_[EntityLow.HighEntityIndex as usize]);
    } else {
        if GameState.HighEntityCount < (GameState.HighEntities_.len()).try_into().unwrap() {
            let HighIndex = GameState.HighEntityCount;
            GameState.HighEntityCount += 1;
            let Diff = Subtract(
                GameState.world.as_ref().unwrap().TileMap.as_ref().unwrap(),
                &EntityLow.P,
                &GameState.CameraP,
            );
            EntityLow.HighEntityIndex = HighIndex;

            let mut temp = &mut GameState.HighEntities_[HighIndex as usize];
            temp.P = Diff.dXY;
            temp.dP = v2::default();
            temp.AbsTileZ = EntityLow.P.AbsTileZ;
            temp.FacingDirection = 0;
            temp.LowEntityIndex = LowIndex;

            EntityHigh = Some(temp);
        } else {
            panic!();
        }
    }

    return EntityHigh;
}

//might not be 'a , because its a reference to an array on the gamestate struct itself!
fn GetHighEntity<'a>(GameState: &'a mut GameState, LowIndex: u32) -> entity<'a> {
    let mut result = entity::default();

    if (LowIndex > 0) && (LowIndex < GameState.LowEntityCount) {
        result.LowIndex = LowIndex;

        unsafe {
            let GameState = GameState as *mut GameState;
            result.High = MakeEntityHighFrequency(&mut *GameState, LowIndex);
        }
        result.Low = Some(&mut GameState.LowEntities[LowIndex as usize]);
    }
    return result;
}

fn MakeEntityLowFrequency(GameState: &mut GameState, LowIndex: u32) {
    let EntityLow = &mut GameState.LowEntities[LowIndex as usize];
    let HighIndex = EntityLow.HighEntityIndex;
    if HighIndex != 0 {
        let LastHighIndex = GameState.HighEntityCount - 1;
        if HighIndex != LastHighIndex {
            let high_entities = &mut GameState.HighEntities_;

            let LastEntity = &mut high_entities[LastHighIndex as usize];
            GameState.LowEntities[LastEntity.LowEntityIndex as usize].HighEntityIndex = HighIndex;

            let stored_last_entity = *LastEntity; //borrowing issues

            let high_entities = &mut GameState.HighEntities_;
            let DelEntity = &mut high_entities[HighIndex as usize];

            *DelEntity = stored_last_entity;
        }
        GameState.HighEntityCount -= 1;
        let EntityLow = &mut GameState.LowEntities[LowIndex as usize];
        EntityLow.HighEntityIndex = 0;
    }
}

fn OffsetAndCheckFrequencyByArea(GameState: &mut GameState, Offset: v2, CameraBounds: rectangle2) {
    let mut EntityIndex = 1;
    while EntityIndex < GameState.HighEntityCount {
        let High = &mut GameState.HighEntities_[EntityIndex as usize];
        let LowEntityIndex = High.LowEntityIndex; // borrowing issues

        High.P += Offset;
        if IsInRectangle(CameraBounds, High.P) {
            EntityIndex += 1;
        } else {
            MakeEntityLowFrequency(GameState, LowEntityIndex);
        }
    }
}
fn AddLowEntity(GameState: &mut GameState, Type: entity_type) -> u32 {
    let EntityIndex = GameState.LowEntityCount;
    GameState.LowEntityCount += 1;
    GameState.LowEntities[EntityIndex as usize] = low_entity {
        Type: Type,
        P: tile_map_position::default(),
        Width: 0.0,
        Height: 0.0,
        Collides: false,
        dAbsTileZ: 0,

        HighEntityIndex: 0,
    };
    //dbg!(GameState.LowEntities[EntityIndex as usize]);
    return EntityIndex;
}

fn AddWall(GameState: &mut GameState, AbsTileX: u32, AbsTileY: u32, AbsTileZ: u32) -> u32 {
    let EntityIndex = AddLowEntity(GameState, entity_type::EntityType_Wall);
    let height = GameState
        .world
        .as_ref()
        .unwrap()
        .TileMap
        .as_ref()
        .unwrap()
        .TileSideInMeters;
    let EntityLow = GetLowEntity(GameState, EntityIndex).unwrap();
    EntityLow.P.AbsTileX = AbsTileX;
    EntityLow.P.AbsTileY = AbsTileY;
    EntityLow.P.AbsTileZ = AbsTileZ;
    EntityLow.Height = height; //have to move up, borrowing issues
    EntityLow.Width = EntityLow.Height;
    EntityLow.Collides = true;

    return EntityIndex;
}

fn AddPlayer(GameState: &mut GameState) -> u32 {
    let EntityIndex = AddLowEntity(GameState, entity_type::EntityType_Hero);
    let EntityLow = GetLowEntity(GameState, EntityIndex).unwrap();
    EntityLow.P.AbsTileX = 1;
    EntityLow.P.AbsTileY = 3;
    EntityLow.P.Offset_.X = 0.0;
    EntityLow.P.Offset_.Y = 0.0;
    EntityLow.Height = 0.5; // 1.4f;
    EntityLow.Width = 1.0;
    EntityLow.Collides = true;

    if GameState.CameraFollowingEntityIndex == 0 {
        GameState.CameraFollowingEntityIndex = EntityIndex;
    }

    return EntityIndex;
}

fn TestWall(
    WallX: f32,
    RelX: f32,
    RelY: f32,
    PlayerDeltaX: f32,
    PlayerDeltaY: f32,
    tMin: &mut f32,
    MinY: f32,
    MaxY: f32,
) -> bool {
    let mut hit = false;
    let tEpsilon = 0.001;
    if PlayerDeltaX != 0.0 {
        let tResult = (WallX - RelX) / PlayerDeltaX;
        let Y = RelY + tResult * PlayerDeltaY;
        if (tResult >= 0.0) && (*tMin > tResult) {
            if (Y >= MinY) && (Y <= MaxY) {
                *tMin = 0.0f32.max(tResult - tEpsilon);
                hit = true;
            }
        }
    }
    return hit;
}

//original takes gamestate opposed to tilemap but should probably pass in tilemap if it doesn't utilzie the gamestate struct fields
fn MovePlayer(GameState: &mut GameState, Entity: entity, dt: f32, mut ddP: v2) {
    let TileMap = GameState.world.as_mut().unwrap().TileMap.as_mut().unwrap();

    let ddPLength = LengthSq(ddP);
    if ddPLength > 1.0 {
        ddP *= 1.0 / ddPLength.sqrt();
    }

    let PlayerSpeed = 50.0; // m/s^2
    ddP *= PlayerSpeed;

    // TODO(casey): ODE here!

    /* let mut MinTileX = Minimum(OldPlayerP.AbsTileX, NewPlayerP.AbsTileX);
    let mut MinTileY = Minimum(OldPlayerP.AbsTileY, NewPlayerP.AbsTileY);
    let mut MaxTileX = Maximum(OldPlayerP.AbsTileX, NewPlayerP.AbsTileX);
    let mut MaxTileY = Maximum(OldPlayerP.AbsTileY, NewPlayerP.AbsTileY);

    let EntityTileWidth = (Entity.Width / TileMap.TileSideInMeters).ceil() as u32;
    let EntityTileHeight = (Entity.Height / TileMap.TileSideInMeters).ceil() as u32;

    MinTileX -= EntityTileWidth;
    MinTileY -= EntityTileHeight;
    MaxTileX += EntityTileWidth;
    MaxTileY += EntityTileHeight;

    let AbsTileZ = Entity.P.AbsTileZ; */

    match Entity {
        entity {
            LowIndex,
            Low: Some(EntityLow),
            High: Some(EntityHigh),
        } => {
            ddP += -8.0 * EntityHigh.dP;
            let OldPlayerP = EntityHigh.P;
            let mut PlayerDelta = 0.5 * ddP * Square(dt) + EntityHigh.dP * dt;
            EntityHigh.dP = ddP * dt + EntityHigh.dP;
            let NewPlayerP = OldPlayerP + PlayerDelta;

            let mut iteration = 0;
            while iteration < 4 {
                let mut tMin = 1.0;
                let mut WallNormal = v2::default();
                let mut HitHighEntityIndex = 0;

                let DesiredPosition = EntityHigh.P + PlayerDelta;
                let mut TestHighEntityIndex = 1;
                while TestHighEntityIndex < GameState.HighEntityCount {
                    if TestHighEntityIndex != EntityLow.HighEntityIndex
                    //destructor struct instead of single option
                    {
                        let mut TestEntity = entity::default();

                        TestEntity.High =
                            Some(&mut GameState.HighEntities_[TestHighEntityIndex as usize]);
                        TestEntity.LowIndex = TestEntity.High.as_ref().unwrap().LowEntityIndex;
                        TestEntity.Low =
                            Some(&mut GameState.LowEntities[TestEntity.LowIndex as usize]);

                        if let Some(TestEntityLow) = TestEntity.Low {
                            if TestEntityLow.Collides {
                                let DiameterW = TestEntityLow.Width + EntityLow.Width;
                                let DiameterH = TestEntityLow.Height + EntityLow.Height;

                                let MinCorner = -0.5
                                    * v2 {
                                        X: DiameterW,
                                        Y: DiameterH,
                                    };
                                let MaxCorner = 0.5
                                    * v2 {
                                        X: DiameterW,
                                        Y: DiameterH,
                                    };

                                let Rel = EntityHigh.P - TestEntity.High.as_ref().unwrap().P;

                                if TestWall(
                                    MinCorner.X,
                                    Rel.X,
                                    Rel.Y,
                                    PlayerDelta.X,
                                    PlayerDelta.Y,
                                    &mut tMin,
                                    MinCorner.Y,
                                    MaxCorner.Y,
                                ) {
                                    WallNormal = v2 { X: -1.0, Y: 0.0 };
                                    HitHighEntityIndex = TestHighEntityIndex;
                                }

                                if TestWall(
                                    MaxCorner.X,
                                    Rel.X,
                                    Rel.Y,
                                    PlayerDelta.X,
                                    PlayerDelta.Y,
                                    &mut tMin,
                                    MinCorner.Y,
                                    MaxCorner.Y,
                                ) {
                                    WallNormal = v2 { X: 1.0, Y: 0.0 };
                                    HitHighEntityIndex = TestHighEntityIndex;
                                }

                                if TestWall(
                                    MinCorner.Y,
                                    Rel.Y,
                                    Rel.X,
                                    PlayerDelta.Y,
                                    PlayerDelta.X,
                                    &mut tMin,
                                    MinCorner.X,
                                    MaxCorner.X,
                                ) {
                                    WallNormal = v2 { X: 0.0, Y: -1.0 };
                                    HitHighEntityIndex = TestHighEntityIndex;
                                }

                                if TestWall(
                                    MaxCorner.Y,
                                    Rel.Y,
                                    Rel.X,
                                    PlayerDelta.Y,
                                    PlayerDelta.X,
                                    &mut tMin,
                                    MinCorner.X,
                                    MaxCorner.X,
                                ) {
                                    WallNormal = v2 { X: 0.0, Y: 1.0 };
                                    HitHighEntityIndex = TestHighEntityIndex;
                                }
                            }
                        }
                    }
                    TestHighEntityIndex += 1;
                }

                EntityHigh.P += tMin * PlayerDelta;
                if HitHighEntityIndex != 0 {
                    EntityHigh.dP =
                        EntityHigh.dP - 1.0 * Inner(EntityHigh.dP, WallNormal) * WallNormal;
                    PlayerDelta = DesiredPosition - EntityHigh.P;
                    PlayerDelta = PlayerDelta - 1.0 * Inner(PlayerDelta, WallNormal) * WallNormal;

                    let HitHigh = &GameState.HighEntities_[HitHighEntityIndex as usize];
                    let HitLow = &GameState.LowEntities[HitHigh.LowEntityIndex as usize];
                    EntityHigh.AbsTileZ += HitLow.dAbsTileZ as u32; //MIGHT BE WRONG DEFAULT CONVERSION?
                } else {
                    break;
                }
                iteration += 1;
            }

            // TODO(casey): Change to using the acceleration vector
            if (EntityHigh.dP.X == 0.0) && (EntityHigh.dP.Y == 0.0) {
                // NOTE(casey): Leave FacingDirection whatever it was
            } else if (EntityHigh.dP.X).abs() > (EntityHigh.dP.Y).abs() {
                if EntityHigh.dP.X > 0.0 {
                    EntityHigh.FacingDirection = 0;
                } else {
                    EntityHigh.FacingDirection = 2;
                }
            } else {
                if EntityHigh.dP.Y > 0.0 {
                    EntityHigh.FacingDirection = 1;
                } else {
                    EntityHigh.FacingDirection = 3;
                }
            }

            EntityLow.P = MapIntoTileSpace(
                GameState.world.as_mut().unwrap().TileMap.as_mut().unwrap(),
                GameState.CameraP,
                EntityHigh.P,
            );
        }
        _ => {}
    }
}

fn SetCamera(GameState: &mut GameState, NewCameraP: tile_map_position) {
    let TileMap = GameState.world.as_mut().unwrap().TileMap.as_mut().unwrap();

    let dCameraP = Subtract(TileMap, &NewCameraP, &GameState.CameraP);
    GameState.CameraP = NewCameraP;

    // TODO(casey): I am totally picking these numbers randomly!
    let TileSpanX = 17 * 3;
    let TileSpanY = 9 * 3;
    let CameraBounds = RectCenterDim(
        v2::default(),
        TileMap.TileSideInMeters
            * v2 {
                X: TileSpanX as f32,
                Y: TileSpanY as f32,
            },
    );
    let EntityOffsetForFrame = -dCameraP.dXY;
    OffsetAndCheckFrequencyByArea(GameState, EntityOffsetForFrame, CameraBounds);

    let MinTileX = NewCameraP.AbsTileX - TileSpanX / 2;
    let MaxTileX = NewCameraP.AbsTileX + TileSpanX / 2;
    let MinTileY = NewCameraP.AbsTileY - TileSpanY / 2;
    let MaxTileY = NewCameraP.AbsTileY + TileSpanY / 2;

    let mut EntityIndex = 1;
    while EntityIndex < GameState.LowEntityCount {
        let Low = &mut GameState.LowEntities[EntityIndex as usize];
        if Low.HighEntityIndex == 0 {
            if (Low.P.AbsTileZ == NewCameraP.AbsTileZ)
                && (Low.P.AbsTileX >= MinTileX)
                && (Low.P.AbsTileX <= MaxTileX)
                && (Low.P.AbsTileY <= MinTileY)
                && (Low.P.AbsTileY >= MaxTileY)
            {
                MakeEntityHighFrequency(GameState, EntityIndex);
            }
        }
        EntityIndex += 1;
    }
}

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

            //reserved for null entity
            AddLowEntity(game_state, entity_type::EntityType_Null);
            game_state.HighEntityCount = 1;

            game_state.Backdrop = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_background.bmp",
            );

            game_state.Shadow = DEBUGLoadBMP(
                thread,
                memory.debug_platform_read_entire_file,
                "test/test_hero_shadow.bmp",
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

            // TILEMAP INITIALIZATION HERE
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
                    let game_state = &mut *(memory.permanent_storage as *mut GameState);
                    //let World = game_state.world.as_mut().unwrap();
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

            /*             let mut ScreenX = std::u32::MAX / 2;
            let mut ScreenY = std::u32::MAX / 2; */
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
            for ScreenIndex in 0..2
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
                } /*  else {
                                     RandomNumberIndex += 1;
                                     RandomChoice = RandomNumberTable[RandomNumberIndex] % 3;
                                 }
                  */
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

                        if TileValue == 2 {
                            AddWall(game_state, AbsTileX, AbsTileY, AbsTileZ);
                        }
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

            let mut NewCameraP = tile_map_position::default();
            NewCameraP.AbsTileX = 17 / 2;
            NewCameraP.AbsTileY = 9 / 2;
            let game_state = &mut *(memory.permanent_storage as *mut GameState);
            SetCamera(game_state, NewCameraP);

            memory.is_initalized = true as bool32;
        }

        let game_state = &mut *(memory.permanent_storage as *mut GameState);
        let World = game_state.world.as_mut().unwrap();
        let TileMap = World.TileMap.as_ref().unwrap();

        let TileSideInPixels = 60;
        let MetersToPixels = TileSideInPixels as f32 / TileMap.TileSideInMeters as f32;

        let LowerLeftX = -(TileSideInPixels / 2) as f32;
        let LowerLeftY = buffer.height as f32;

        for controller_index in 0..input.controllers.len() {
            let controller = GetController(input, controller_index.try_into().unwrap());
            let game_state = &mut *(memory.permanent_storage as *mut GameState);
            let LowIndex = game_state.PlayerIndexForController[controller_index as usize];

            if LowIndex == 0 {
                if controller.start().ended_down != 0 {
                    let EntityIndex = AddPlayer(game_state);
                    game_state.PlayerIndexForController[controller_index as usize] = EntityIndex;
                }
            } else {
                let mut ControllingEntity = GetHighEntity(game_state, LowIndex);

                let mut ddP = v2::default();

                if controller.is_analog != 0 {
                    ddP = v2 {
                        X: controller.stick_average_x,
                        Y: controller.stick_average_y,
                    }
                } else {
                    // NOTE(casey): Use digital movement tuning
                    if controller.move_up().ended_down != 0 {
                        ddP.Y = 1.0;
                    }
                    if controller.move_down().ended_down != 0 {
                        ddP.Y = -1.0;
                    }
                    if controller.move_left().ended_down != 0 {
                        ddP.X = -1.0;
                    }
                    if controller.move_right().ended_down != 0 {
                        ddP.X = 1.0;
                    }
                }
                //BEFORE
                if controller.action_up().ended_down != 0 {
                    ControllingEntity.High.as_mut().unwrap().dZ = 3.0;
                }

                {
                    let game_state = &mut *(memory.permanent_storage as *mut GameState);

                    //LOOP MOVED INTO HERE? error :use of moved value: `ControllingEntity`
                    MovePlayer(game_state, ControllingEntity, input.dtForFrame, ddP);
                }
            }
        }

        let game_state = &mut *(memory.permanent_storage as *mut GameState);
        let EntityOffsetForFrame = v2::default();
        let CameraFollowingEntity =
            GetHighEntity(game_state, game_state.CameraFollowingEntityIndex);

        if CameraFollowingEntity.High.is_some() {
            match CameraFollowingEntity {
                entity {
                    LowIndex: _,
                    Low: Some(Low),
                    High: Some(High),
                } => {
                    let game_state = &mut *(memory.permanent_storage as *mut GameState);
                    let mut NewCameraP = game_state.CameraP;

                    NewCameraP.AbsTileZ = Low.P.AbsTileZ;

                    if High.P.X > (9.0 * TileMap.TileSideInMeters) {
                        NewCameraP.AbsTileX += 17;
                    }
                    if High.P.X < -(9.0 * TileMap.TileSideInMeters) {
                        NewCameraP.AbsTileX -= 17;
                    }
                    if High.P.Y > (5.0 * TileMap.TileSideInMeters) {
                        NewCameraP.AbsTileY += 9;
                    }
                    if High.P.Y < -(5.0 * TileMap.TileSideInMeters) {
                        NewCameraP.AbsTileY -= 9;
                    }

                    SetCamera(game_state, NewCameraP);
                }
                _ => {}
            }
            /* #else
                    if(CameraFollowingEntity.High.P.X > (1.0f*TileMap.TileSideInMeters))
                    {
                        NewCameraP.AbsTileX += 1;
                    }
                    if(CameraFollowingEntity.High.P.X < -(1.0f*TileMap.TileSideInMeters))
                    {
                        NewCameraP.AbsTileX -= 1;
                    }
                    if(CameraFollowingEntity.High.P.Y > (1.0f*TileMap.TileSideInMeters))
                    {
                        NewCameraP.AbsTileY += 1;
                    }
                    if(CameraFollowingEntity.High.P.Y < -(1.0f*TileMap.TileSideInMeters))
                    {
                        NewCameraP.AbsTileY -= 1;
                    }
            #endif
                     */
            // TODO(casey): Map new entities in and old entities out!!!
            // TODO(casey): Mapping tiles and stairs into the entity set!
        }

        //
        // NOTE(casey): Render
        //
        DrawBitmap(buffer, &game_state.Backdrop, 0.0, 0.0, 0, 0, 1.0);

        let ScreenCenterX = 0.5 * buffer.width as f32;
        let ScreenCenterY = 0.5 * buffer.height as f32;

        /*         for RelRow in -10..10
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
                        X: ScreenCenterX - MetersToPixels * game_state.CameraP.Offset_.X
                            + (RelColumn as f32) * TileSideInPixels as f32,
                        Y: ScreenCenterY + MetersToPixels * game_state.CameraP.Offset_.Y
                            - (RelRow as f32) * TileSideInPixels as f32,
                    };
                    let Min = Cen - 0.9 * TileSide;
                    let Max = Cen + 0.9 * TileSide;
                    /*  v2 Min = Cen - TileSide;
                    v2 Max = Cen + TileSide; */
                    DrawRectangle(buffer, Min, Max, Gray, Gray, Gray);
                }
            }
        } */
        for HighEntityIndex in 1..game_state.HighEntityCount {
            let mut HighEntity = &mut game_state.HighEntities_[HighEntityIndex as usize];
            let LowEntity = &mut game_state.LowEntities[HighEntity.LowEntityIndex as usize];

            HighEntity.P += EntityOffsetForFrame;

            let dt = input.dtForFrame;
            let ddZ = -9.8;
            HighEntity.Z = 0.5 * ddZ * Square(dt) + HighEntity.dZ * dt + HighEntity.Z;
            HighEntity.dZ = ddZ * dt + HighEntity.dZ;
            if HighEntity.Z < 0.0 {
                HighEntity.Z = 0.0;
            }
            let mut CAlpha = 1.0 - 0.5 * HighEntity.Z;
            if CAlpha < 0.0 {
                CAlpha = 0.0;
            }

            let PlayerR = 1.0;
            let PlayerG = 1.0;
            let PlayerB = 0.0;
            let PlayerGroundPointX = ScreenCenterX + MetersToPixels * HighEntity.P.X;
            let PlayerGroundPointY = ScreenCenterY - MetersToPixels * HighEntity.P.Y;
            let Z = -MetersToPixels * HighEntity.Z;
            let PlayerLeftTop = v2 {
                X: PlayerGroundPointX - 0.5 * MetersToPixels * LowEntity.Width,
                Y: PlayerGroundPointY - 0.5 * MetersToPixels * LowEntity.Height,
            };
            let EntityWidthHeight = v2 {
                X: LowEntity.Width,
                Y: LowEntity.Height,
            };

            if let entity_type::EntityType_Hero = LowEntity.Type {
                let HeroBitmaps = &game_state.HeroBitmaps[HighEntity.FacingDirection as usize];
                DrawBitmap(
                    buffer,
                    &game_state.Shadow,
                    PlayerGroundPointX,
                    PlayerGroundPointY,
                    HeroBitmaps.AlignX,
                    HeroBitmaps.AlignY,
                    CAlpha,
                );
                DrawBitmap(
                    buffer,
                    &HeroBitmaps.Torso,
                    PlayerGroundPointX,
                    PlayerGroundPointY + Z,
                    HeroBitmaps.AlignX,
                    HeroBitmaps.AlignY,
                    1.0,
                );
                DrawBitmap(
                    buffer,
                    &HeroBitmaps.Cape,
                    PlayerGroundPointX,
                    PlayerGroundPointY + Z,
                    HeroBitmaps.AlignX,
                    HeroBitmaps.AlignY,
                    1.0,
                );
                DrawBitmap(
                    buffer,
                    &HeroBitmaps.Head,
                    PlayerGroundPointX,
                    PlayerGroundPointY + Z,
                    HeroBitmaps.AlignX,
                    HeroBitmaps.AlignY,
                    1.0,
                );
            } else {
                DrawRectangle(
                    buffer,
                    PlayerLeftTop,
                    PlayerLeftTop + MetersToPixels * EntityWidthHeight,
                    PlayerR,
                    PlayerG,
                    PlayerB,
                );
            }
        }
    }
}

fn DrawRectangle(Buffer: &mut GameOffScreenBuffer, vMin: v2, vMax: v2, R: f32, G: f32, B: f32) {
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
    unsafe {
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

#[no_mangle]
pub unsafe extern "C" fn GameGetSoundSamples(
    thread: &thread_context,
    Memory: &mut GameMemory,
    SoundBuffer: &mut game_sound_output_buffer,
) {
    let GameState = Memory.permanent_storage as *mut GameState;
    GameOutputSound(GameState, SoundBuffer, 400);
}
