#![allow(allow_bad_style)]
use crate::handmade_math::v2;
use crate::memory_arena;
use crate::PushStruct;
use core::ptr::null_mut;
use std::convert::TryInto;
#[derive(Default)]
pub struct world_difference {
    pub dXY: v2,
    pub dZ: f32,
}
#[derive(Default, Copy, Clone)]
pub struct world_position {
    // TODO(casey): Puzzler!  How can we get rid of abstile* here,
    // and still allow references to entities to be able to figure
    // out _where they are_ (or rather, which world_chunk they are
    // in?)
    pub ChunkX: i32,
    pub ChunkY: i32,
    pub ChunkZ: i32,

    // NOTE(casey): These are the offsets from the chunk center
    pub Offset_: v2,
}

// TODO(casey): Could make this just tile_chunk and then allow multiple tile chunks per X/Y/Z
//NO WAY TO CLONE IF NEXT HOLDED A MUTABLE REFERENCE. YOU CANNOT CLONE MUTABLE REFERENCES
#[derive(Clone, Copy)]
pub struct world_entity_block {
    pub EntityCount: u32,
    pub LowEntityIndex: [u32; 16],
    pub Next: Option<*mut world_entity_block>,
}

pub struct world_chunk {
    pub ChunkX: i32,
    pub ChunkY: i32,
    pub ChunkZ: i32,

    pub FirstBlock: Option<world_entity_block>,
    pub NextInHash: Option<*mut world_chunk>,
}

pub struct world {
    pub TileSideInMeters: f32,
    pub ChunkSideInMeters: f32,

    pub FirstFree: Option<*mut world_entity_block>,

    pub ChunkHash: [world_chunk; 4096],
}

static TILE_CHUNK_SAFE_MARGIN: i32 = (std::i32::MAX / 64);
static TILE_CHUNK_UNINITIALIZED: i32 = std::i32::MAX;

static TILES_PER_CHUNK: u32 = 16;

pub fn IsCanonical(World: &world, TileRel: f32) -> bool {
    // TODO(casey): Fix floating point math so this can be exact?
    let result =
        (TileRel >= -0.5 * World.ChunkSideInMeters) && (TileRel <= 0.5 * World.ChunkSideInMeters);

    return result;
}

pub fn IsCanonical_v2(World: &world, Offset: v2) -> bool {
    let result = IsCanonical(World, Offset.X) && IsCanonical(World, Offset.Y);

    return result;
}

pub fn AreInSameChunk(World: &world, A: &world_position, B: &world_position) -> bool {
    let result = (A.ChunkX == B.ChunkX) && (A.ChunkY == B.ChunkY) && (A.ChunkZ == B.ChunkZ);

    return result;
}

//ORIGINAL FUNCTION PASSED IN ENTIRE WORLD STRUCT INSTEAD OF CHUNKHASH
pub fn GetWorldChunk(
    ChunkHash: &mut [world_chunk],
    ChunkX: i32,
    ChunkY: i32,
    ChunkZ: i32,
    Arena: Option<*mut memory_arena>,
    //CHANGING THIS TO world_chunk<'b> FIXES THE PROBLEM. IF YOU DONT USE MULTIPLE LIFETIMES, THIS FUNCTION WILL TAKE AH OLD OF THE &'A MUT BORROW FOR THE ENTIRE EXISTANCE OF SCOPE IN OTHER FUNCTIONS
) -> *mut world_chunk {
    let HashValue = 19 * ChunkX + 7 * ChunkY + 3 * ChunkZ;
    let HashSlot = HashValue & (ChunkHash.len() - 1) as i32;

    let len = ChunkHash.len().try_into().unwrap();
    let mut Chunk = &mut ChunkHash[HashSlot as usize];
    let valid_arena = Arena.is_some();

    unsafe {
        loop {
            if (ChunkX == Chunk.ChunkX) && (ChunkY == Chunk.ChunkY) && (ChunkZ == Chunk.ChunkZ) {
                break;
            }
            if valid_arena
                && (Chunk.ChunkX != TILE_CHUNK_UNINITIALIZED)
                && (!Chunk.NextInHash.is_some())
            {
                //handmade_world.rs(78, 18): lifetime `'a` defined here
                //mutable borrow starts here in previous iteration of loop
                //requires that `Arena` is borrowed for `'a`
                //no way but to take arena as a *mut :
                // see https://users.rust-lang.org/t/mutable-borrow-starts-here-in-previous-iteration-of-loop/26145/4?u=jest
                unsafe {
                    Chunk.NextInHash = Some(PushStruct::<world_chunk>(&mut *Arena.unwrap()));
                }
                Chunk = &mut *(Chunk.NextInHash.unwrap());
                Chunk.ChunkX = TILE_CHUNK_UNINITIALIZED;
            }
            if valid_arena && (Chunk.ChunkX == TILE_CHUNK_UNINITIALIZED) {
                Chunk.ChunkX = ChunkX;
                Chunk.ChunkY = ChunkY;
                Chunk.ChunkZ = ChunkZ;

                Chunk.NextInHash = None;
                break;
            }

            if HashSlot < len {
                break;
            }
            Chunk = &mut *Chunk.NextInHash.unwrap();
        }
    }
    return Chunk as *mut world_chunk;
}

pub fn InitializeWorld(World: &mut world, TileSideInMeters: f32) {
    World.TileSideInMeters = TileSideInMeters;
    World.ChunkSideInMeters = TILES_PER_CHUNK as f32 * TileSideInMeters;
    World.FirstFree = None;

    for ChunkIndex in 0..World.ChunkHash.len() {
        World.ChunkHash[ChunkIndex].ChunkX = TILE_CHUNK_UNINITIALIZED;
        World.ChunkHash[ChunkIndex]
            .FirstBlock
            .as_mut()
            .unwrap()
            .EntityCount = 0;
    }
}

pub fn RecanonicalizeCoord(World: &world, Tile: &mut i32, TileRel: &mut f32) {
    // TODO(casey): Need to do something that doesn't use the divide/multiply method
    // for recanonicalizing because this can end up rounding back on to the tile
    // you just came from.

    // NOTE(casey): Wrapping IS NOT ALLOWED, so all coordinates are assumed to be
    // within the safe margin!
    // TODO(casey): Assert that we are nowhere near the edges of the world.
    let Offset = (*TileRel / World.ChunkSideInMeters).round() as i32;
    *Tile += Offset;
    *TileRel -= Offset as f32 * World.ChunkSideInMeters;
}

pub fn MapIntoChunkSpace(World: &world, BasePos: world_position, Offset: v2) -> world_position {
    let mut result = BasePos;

    result.Offset_ += Offset;
    RecanonicalizeCoord(World, &mut result.ChunkX, &mut result.Offset_.X);
    RecanonicalizeCoord(World, &mut result.ChunkY, &mut result.Offset_.Y);

    return result;
}

pub fn ChunkPositionFromTilePosition(
    World: &world,
    AbsTileX: i32,
    AbsTileY: i32,
    AbsTileZ: i32,
) -> world_position {
    let mut result = world_position::default();

    result.ChunkY = (AbsTileY as u32 / TILES_PER_CHUNK).try_into().unwrap();
    result.ChunkX = (AbsTileX as u32 / TILES_PER_CHUNK).try_into().unwrap();
    result.ChunkZ = (AbsTileZ as u32 / TILES_PER_CHUNK).try_into().unwrap();

    // TODO(casey): DECIDE ON TILE ALIGNMENT IN CHUNKS!
    result.Offset_.X = (AbsTileX - (result.ChunkX as u32 * TILES_PER_CHUNK) as i32) as f32
        * World.TileSideInMeters;
    result.Offset_.Y = (AbsTileY - (result.ChunkY as u32 * TILES_PER_CHUNK) as i32) as f32
        * World.TileSideInMeters;
    // TODO(casey): Move to 3D Z!!!

    return result;
}

pub fn Subtract(World: &world, A: &world_position, B: &world_position) -> world_difference {
    let mut result = world_difference::default();

    let dTileXY = v2 {
        X: A.ChunkX as f32 - B.ChunkX as f32,
        Y: A.ChunkY as f32 - B.ChunkY as f32,
    };
    let dTileZ = A.ChunkZ - B.ChunkZ;

    result.dXY = World.ChunkSideInMeters * dTileXY + (A.Offset_ - B.Offset_);
    // TODO(casey): Think about what we want to do about Z
    result.dZ = World.ChunkSideInMeters * dTileZ as f32;

    return result;
}

pub fn CenteredChunkPoint(ChunkX: u32, ChunkY: u32, ChunkZ: u32) -> world_position {
    let mut result = world_position::default();

    result.ChunkX = ChunkX.try_into().unwrap();
    result.ChunkY = ChunkY.try_into().unwrap();
    result.ChunkZ = ChunkZ.try_into().unwrap();

    return result;
}

pub fn ChangeEntityLocation(
    Arena: &mut memory_arena,
    World: &mut world,
    LowEntityIndex: u32,
    OldP: Option<&world_position>,
    NewP: &world_position,
) {
    if OldP.is_some() && AreInSameChunk(World, OldP.unwrap(), NewP) {
        // NOTE(casey): Leave entity where it is
    } else {
        if OldP.is_some() {
            let OldP = OldP.unwrap();
            // NOTE(casey): Pull the entity out of its old entity block
            let Chunk = GetWorldChunk(
                &mut World.ChunkHash,
                OldP.ChunkX,
                OldP.ChunkY,
                OldP.ChunkZ,
                None,
            );
            unsafe {
                if !Chunk.is_null() {
                    let Chunk = Chunk;
                    let mut NotFound = true;
                    let mut FirstBlock = (*Chunk).FirstBlock.as_mut().unwrap();

                    let mut Block = FirstBlock as *mut world_entity_block;

                    while !Block.is_null() && NotFound {
                        {
                            let mut Block = *Block;
                            let mut Index = 0;
                            while (Index < Block.EntityCount) && NotFound {
                                if Block.LowEntityIndex[Index as usize] == LowEntityIndex {
                                    FirstBlock.EntityCount -= 1;
                                    Block.LowEntityIndex[Index as usize] =
                                        FirstBlock.LowEntityIndex[FirstBlock.EntityCount as usize];
                                    if FirstBlock.EntityCount == 0 {
                                        if FirstBlock.Next.is_some() {
                                            let NextBlock = FirstBlock.Next.unwrap();
                                            *FirstBlock = *NextBlock;
                                            (*NextBlock).Next =
                                                Some(*World.FirstFree.as_mut().unwrap()
                                                    as *mut world_entity_block);
                                            World.FirstFree = Some(&mut *NextBlock);
                                        }
                                    }

                                    NotFound = false;
                                }
                                Index += 1;
                            }
                        }
                        if (*Block).Next.is_some() {
                            Block = (*Block).Next.unwrap();
                        } else {
                            Block = null_mut();
                        }
                    }
                }
            }
        }

        // NOTE(casey): Insert the entity into its new entity block
        let Chunk = GetWorldChunk(
            &mut World.ChunkHash,
            NewP.ChunkX,
            NewP.ChunkY,
            NewP.ChunkZ,
            Some(Arena),
        );
        unsafe {
            let mut Block = (*Chunk).FirstBlock.unwrap();
            if Block.EntityCount == (Block.LowEntityIndex.len()).try_into().unwrap() {
                // NOTE(casey): We're out of room, get a new block!
                let mut OldBlock = World.FirstFree;
                if OldBlock.is_some() {
                    World.FirstFree = (*OldBlock.unwrap()).Next;
                } else {
                    OldBlock = Some(PushStruct::<world_entity_block>(Arena));
                }
                let OldBlock = World.FirstFree.unwrap();
                *OldBlock = Block;
                Block.Next = Some(OldBlock);
                Block.EntityCount = 0;
            }

            Block.LowEntityIndex[Block.EntityCount as usize] = LowEntityIndex;
            Block.EntityCount += 1; //MIGHT BE WRONG
        }
    }
}
