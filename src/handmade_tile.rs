use crate::*;
use std::convert::TryInto;
#[derive(Default, Clone)] // does this really need to implement clone?
pub struct tile_map_position {
    // NOTE(casey): These are fixed point tile locations.  The high
    // bits are the tile chunk index, and the low bits are the tile
    // index in the chunk.
    pub AbsTileX: u32,
    pub AbsTileY: u32,
    pub AbsTileZ: u32,

    // TODO(casey): Should these be from the center of a tile?
    // TODO(casey): Rename to offset X and Y
    pub TileRelX: f32,
    pub TileRelY: f32,
}

#[derive(Default)]
pub struct tile_chunk_position {
    pub TileChunkX: u32,
    pub TileChunkY: u32,
    pub TileChunkZ: u32,

    pub RelTileX: u32,
    pub RelTileY: u32,
}

pub struct tile_chunk<'a> {
    // TODO(casey): Real structure for a tile!
    pub Tiles: Option<&'a mut u32>,
}

#[derive(Default)]
pub struct tile_map<'a> {
    pub ChunkShift: u32,
    pub ChunkMask: u32,
    pub ChunkDim: u32,

    pub TileSideInMeters: f32,

    // TODO(casey): REAL sparseness so anywhere in the world can be
    // represented without the giant pointer array.
    pub TileChunkCountX: u32,
    pub TileChunkCountY: u32,
    pub TileChunkCountZ: u32,
    pub TileChunks: Option<&'a mut tile_chunk<'a>>,
}

pub unsafe fn RecanonicalizeCoord(TileMap: &tile_map, mut Tile: *mut u32, TileRel: *mut f32) {
    // TODO(casey): Need to do something that doesn't use the divide/multiply method
    // for recanonicalizing because this can end up rounding back on to the tile
    // you just came from.
    // NOTE(casey): TileMap is assumed to be toroidal topology, if you
    // step off one end you come back on the other!
    let Offset = RoundReal32ToInt32(*TileRel / TileMap.TileSideInMeters);
    *Tile = Offset.try_into().unwrap();
    Tile = Tile.offset(1);
    //*Tile += Offset;
    *TileRel = *TileRel - Offset as f32 * TileMap.TileSideInMeters;
    //*TileRel -= Offset * TileMap.TileSideInMeters;

    // TODO(casey): Fix floating point math so this can be < ?
    //Assert(*TileRel >= -0.5f*TileMap.TileSideInMeters);
    //Assert(*TileRel <= 0.5f*TileMap.TileSideInMeters);
}

pub unsafe fn RecanonicalizePosition(TileMap: &tile_map, Pos: tile_map_position) -> tile_map_position {
    let mut result = Pos;

    RecanonicalizeCoord(TileMap, &mut result.AbsTileX, &mut result.TileRelX);
    RecanonicalizeCoord(TileMap, &mut result.AbsTileY, &mut result.TileRelY);

    return result;
}

//inline tile_chunk *
pub unsafe fn GetTileChunk<'a>(
    TileMap: &'a tile_map<'a>,
    TileChunkX: u32,
    TileChunkY: u32,
    TileChunkZ: u32,
) -> &'a mut tile_chunk<'a> {
   //let mut TileChunk= 0 as *mut tile_chunk; 
    let mut TileChunk = (&TileMap.TileChunks).unwrap();

    if (TileChunkX >= 0)
        && (TileChunkX < TileMap.TileChunkCountX)
        && (TileChunkY >= 0)
        && (TileChunkY < TileMap.TileChunkCountY)
        && (TileChunkZ >= 0)
        && (TileChunkZ < TileMap.TileChunkCountZ)
    {
        match TileMap.TileChunks {
            Some(chunk) => {
                let temp = (chunk as *mut tile_chunk).offset(
                    (TileChunkZ * TileMap.TileChunkCountY * TileMap.TileChunkCountX
                        + TileChunkY * TileMap.TileChunkCountX
                        + TileChunkX)
                        .try_into()
                        .unwrap(),
                );
                unsafe {
                    TileChunk = &mut *temp;
                }
            }
            _ => {}
        }
        /*     TileChunk = &TileMap.TileChunks.offset(
        TileChunkZ*TileMap.TileChunkCountY*TileMap.TileChunkCountX +
        TileChunkY*TileMap.TileChunkCountX +
        TileChunkX); */
    }

    return &mut *TileChunk;
}

pub unsafe fn GetTileValueUnchecked(
    TileMap: &tile_map,
    TileChunk: &mut tile_chunk,
    TileX: u32,
    TileY: u32,
) -> u32 {
    /*   Assert(TileChunk);
    Assert(TileX < TileMap.ChunkDim);
    Assert(TileY < TileMap.ChunkDim);
     */
    let TileChunkValue = (TileChunk.Tiles.unwrap() as *mut u32)
        .offset((TileY * TileMap.ChunkDim + TileX).try_into().unwrap());
    return *TileChunkValue;
}

pub fn SetTileValueUnchecked(
    TileMap: &tile_map,
    TileChunk: &tile_chunk,
    TileX: u32,
    TileY: u32,
    TileValue: u32,
) {
    /*     Assert(TileChunk);
    Assert(TileX < TileMap.ChunkDim);
    Assert(TileY < TileMap.ChunkDim) */

    unsafe {
        *(TileChunk.Tiles.unwrap() as *mut u32)
            .offset((TileY * TileMap.ChunkDim + TileX).try_into().unwrap()) = TileValue;
    }
}

pub fn GetTileValue_(
    TileMap: Option<&tile_map>,
    TileChunk: Option<&mut tile_chunk>,
    TestTileX: u32,
    TestTileY: u32,
) -> u32 {
    let TileChunkValue = 0;

    if TileChunk.is_some() && TileChunk.unwrap().Tiles.is_some() {
        TileChunkValue =
            GetTileValueUnchecked(TileMap.unwrap(), TileChunk.unwrap(), TestTileX, TestTileY);
    }

    return (TileChunkValue);
}

pub fn SetTileValue_(
    TileMap: Option<&tile_map>,
    TileChunk: Option<&tile_chunk>,
    TestTileX: u32,
    TestTileY: u32,
    TileValue: u32,
) {
    if TileChunk.is_some() && TileChunk.unwrap().Tiles.is_some() {
        // can remove if check since .unwrap() does  the check
        SetTileValueUnchecked(
            TileMap.unwrap(),
            TileChunk.unwrap(),
            TestTileX,
            TestTileY,
            TileValue,
        );
    }
}

pub fn GetChunkPositionFor(
    TileMap: &tile_map,
    AbsTileX: u32,
    AbsTileY: u32,
    AbsTileZ: u32,
) -> tile_chunk_position {
    let mut result = tile_chunk_position::default();

    result.TileChunkX = AbsTileX >> TileMap.ChunkShift;
    result.TileChunkY = AbsTileY >> TileMap.ChunkShift;
    result.TileChunkZ = AbsTileZ;
    result.RelTileX = AbsTileX & TileMap.ChunkMask;
    result.RelTileY = AbsTileY & TileMap.ChunkMask;

    return result;
}

pub unsafe fn GetTileValue<'a>(
    TileMap: &'a tile_map<'a>,
    AbsTileX: u32,
    AbsTileY: u32,
    AbsTileZ: u32,
) -> u32 {
    let Empty = false;

    let ChunkPos = GetChunkPositionFor(TileMap, AbsTileX, AbsTileY, AbsTileZ);
    let TileChunk = GetTileChunk(
        TileMap,
        ChunkPos.TileChunkX,
        ChunkPos.TileChunkY,
        ChunkPos.TileChunkZ,
    );
    let TileChunkValue = GetTileValue_(
        Some(TileMap),
        Some(TileChunk),
        ChunkPos.RelTileX,
        ChunkPos.RelTileY,
    );

    return TileChunkValue;
}

pub unsafe fn IsTileMapPointEmpty<'a>(TileMap: &'a tile_map<'a>, CanPos: tile_map_position) -> bool {
    let TileChunkValue = GetTileValue(TileMap, CanPos.AbsTileX, CanPos.AbsTileY, CanPos.AbsTileZ);
    let Empty = TileChunkValue == 1;

    return Empty;
}

// function overloading
pub unsafe fn SetTileValue<'a>(
    Arena: &mut memory_arena,
    TileMap: &'a tile_map<'a>,
    AbsTileX: u32,
    AbsTileY: u32,
    AbsTileZ: u32,
    TileValue: u32,
) {
    let ChunkPos = GetChunkPositionFor(TileMap, AbsTileX, AbsTileY, AbsTileZ);
    let mut TileChunk = GetTileChunk(
        TileMap,
        ChunkPos.TileChunkX,
        ChunkPos.TileChunkY,
        ChunkPos.TileChunkZ,
    );

    //Assert(TileChunk);
    if TileChunk.Tiles.is_none() {
        let TileCount = TileMap.ChunkDim * TileMap.ChunkDim;
        TileChunk.Tiles = Some(&mut (*PushArray::<u32>(Arena, TileCount)));
        for TileIndex in 0..TileCount
        /*   (let TileIndex = 0;
        TileIndex < TileCount;
        ++TileIndex) */
        {
            *(TileChunk.Tiles.unwrap() as *mut u32).offset(TileIndex.try_into().unwrap()) = 1;
        }
    }

    SetTileValue_(
        Some(TileMap),
        Some(TileChunk),
        ChunkPos.RelTileX,
        ChunkPos.RelTileY,
        TileValue,
    );
}
