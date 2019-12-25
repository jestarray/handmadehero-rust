use crate::*;
use std::convert::TryInto;

#[derive(Default, Clone, Copy, Debug)] // must implement copy

pub struct tile_map_position {
    // NOTE(casey): These are fixed point tile locations.  The high
    // bits are the tile chunk index, and the low bits are the tile
    // index in the chunk.
    pub AbsTileX: u32,
    pub AbsTileY: u32,
    pub AbsTileZ: u32,

    // TODO(casey): Should these be from the center of a tile?
    pub Offset: v2,
}

#[derive(Default)]
pub struct tile_chunk_position {
    pub TileChunkX: u32,
    pub TileChunkY: u32,
    pub TileChunkZ: u32,

    pub RelTileX: u32,
    pub RelTileY: u32,
}

pub struct tile_chunk {
    // TODO(casey): Real structure for a tile!
    pub Tiles: *mut u32,
}

pub struct tile_map {
    pub ChunkShift: u32,
    pub ChunkMask: u32,
    pub ChunkDim: u32,

    pub TileSideInMeters: f32,

    // TODO(casey): REAL sparseness so anywhere in the world can be
    // represented without the giant pointer array.
    pub TileChunkCountX: u32,
    pub TileChunkCountY: u32,
    pub TileChunkCountZ: u32,
    pub TileChunks: *mut tile_chunk,
}
impl Default for tile_map {
    fn default() -> Self {
        tile_map {
            ChunkShift: 0,
            ChunkMask: 0,
            ChunkDim: 0,
            TileSideInMeters: 0.0,
            TileChunkCountX: 0,
            TileChunkCountY: 0,
            TileChunkCountZ: 0,
            TileChunks: 0 as *mut tile_chunk,
        }
    }
}

pub fn RecanonicalizeCoord(TileMap: &tile_map, Tile: &mut u32, TileRel: &mut f32) {
    // TODO(casey): Need to do something that doesn't use the divide/multiply method
    // for recanonicalizing because this can end up rounding back on to the tile
    // you just came from.
    // NOTE(casey): TileMap is assumed to be toroidal topology, if you
    // step off one end you come back on the other!
    let Offset = RoundReal32ToInt32(*TileRel / TileMap.TileSideInMeters);
    *Tile = *Tile + Offset as u32; //overflows
    *TileRel -= Offset as f32 * TileMap.TileSideInMeters;
    //*TileRel -= Offset * TileMap.TileSideInMeters;
    // TODO(casey): Fix floating point math so this can be < ?
    //Assert(*TileRel >= -0.5f*TileMap.TileSideInMeters);
    //Assert(*TileRel <= 0.5f*TileMap.TileSideInMeters);
}

pub fn RecanonicalizePosition(TileMap: &tile_map, Pos: tile_map_position) -> tile_map_position {
    let mut result = Pos;

    RecanonicalizeCoord(TileMap, &mut result.AbsTileX, &mut result.Offset.X);
    RecanonicalizeCoord(TileMap, &mut result.AbsTileY, &mut result.Offset.Y);

    return result;
}
pub unsafe fn GetTileChunk(
    TileMap: &tile_map,
    TileChunkX: u32,
    TileChunkY: u32,
    TileChunkZ: u32,
) -> *mut tile_chunk {
    //let mut TileChunk= 0 as *mut tile_chunk;
    let mut TileChunk = (*TileMap).TileChunks;
    if (TileChunkX >= 0)
        && (TileChunkX < TileMap.TileChunkCountX)
        && (TileChunkY >= 0)
        && (TileChunkY < TileMap.TileChunkCountY)
        && (TileChunkZ >= 0)
        && (TileChunkZ < TileMap.TileChunkCountZ)
    {
        let temp = TileChunk.offset(
            (TileChunkZ * TileMap.TileChunkCountY * TileMap.TileChunkCountX
                + TileChunkY * TileMap.TileChunkCountX
                + TileChunkX)
                .try_into()
                .unwrap(),
        );
        TileChunk = &mut *temp;
    }
    /*     TileChunk = &TileMap.TileChunks.offset(
    TileChunkZ*TileMap.TileChunkCountY*TileMap.TileChunkCountX +
    TileChunkY*TileMap.TileChunkCountX +
    TileChunkX); */

    return &mut *TileChunk;
}

pub unsafe fn GetTileValueUnchecked(
    TileMap: &tile_map,
    TileChunk: *mut tile_chunk,
    TileX: u32,
    TileY: u32,
) -> u32 {
    /*   Assert(TileChunk);
    Assert(TileX < TileMap.ChunkDim);
    Assert(TileY < TileMap.ChunkDim);
     */
    let TileChunkValue = *(*TileChunk)
        .Tiles
        .offset((TileY * TileMap.ChunkDim + TileX).try_into().unwrap());
    /*  let TileChunkValue = (*(*(TileChunk)).Tiles as *mut u32)
    .offset((TileY * (*TileMap).ChunkDim + TileX).try_into().unwrap()); */
    return TileChunkValue;
}

pub fn SetTileValueUnchecked(
    TileMap: *mut tile_map,
    TileChunk: *mut tile_chunk,
    TileX: u32,
    TileY: u32,
    TileValue: u32,
) {
    /*     Assert(TileChunk);
    Assert(TileX < TileMap.ChunkDim);
    Assert(TileY < TileMap.ChunkDim) */

    unsafe {
        *((*TileChunk).Tiles as *mut u32)
            .offset((TileY * (*TileMap).ChunkDim + TileX).try_into().unwrap()) = TileValue;
    }
}

pub fn GetTileValue_(
    TileMap: &tile_map,
    TileChunk: *mut tile_chunk,
    TestTileX: u32,
    TestTileY: u32,
) -> u32 {
    let mut TileChunkValue = 0;
    unsafe {
        if TileChunk != null_mut() && (*TileChunk).Tiles != null_mut() {
            TileChunkValue = GetTileValueUnchecked(TileMap, TileChunk, TestTileX, TestTileY);
        }

        return TileChunkValue;
    }
}

pub fn SetTileValue_(
    TileMap: *mut tile_map,
    TileChunk: *mut tile_chunk,
    TestTileX: u32,
    TestTileY: u32,
    TileValue: u32,
) {
    unsafe {
        if TileChunk != null_mut() && (*TileChunk).Tiles != null_mut() {
            // can remove if check since .unwrap() does  the check
            SetTileValueUnchecked(TileMap, TileChunk, TestTileX, TestTileY, TileValue);
        }
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

pub fn GetTileValue(TileMap: &tile_map, AbsTileX: u32, AbsTileY: u32, AbsTileZ: u32) -> u32 {
    let ChunkPos = GetChunkPositionFor(TileMap, AbsTileX, AbsTileY, AbsTileZ);
    unsafe {
        let TileChunk = GetTileChunk(
            TileMap,
            ChunkPos.TileChunkX,
            ChunkPos.TileChunkY,
            ChunkPos.TileChunkZ,
        );
        let TileChunkValue =
            GetTileValue_(TileMap, TileChunk, ChunkPos.RelTileX, ChunkPos.RelTileY);

        return TileChunkValue;
    }
}
pub fn GetTileValue_P(TileMap: &tile_map, Pos: tile_map_position) -> u32 {
    let TileChunkValue = GetTileValue(TileMap, Pos.AbsTileX, Pos.AbsTileY, Pos.AbsTileZ);

    return TileChunkValue;
}
pub fn IsTileValueEmpty(TileValue: u32) -> bool {
    let Empty = (TileValue == 1) || (TileValue == 3) || (TileValue == 4);

    return Empty;
}
pub fn IsTileMapPointEmpty(TileMap: &tile_map, CanPos: tile_map_position) -> bool {
    let TileChunkValue = GetTileValue(TileMap, CanPos.AbsTileX, CanPos.AbsTileY, CanPos.AbsTileZ);
    let empty = IsTileValueEmpty(TileChunkValue);
    return empty;
}

// function overloading
pub unsafe fn SetTileValue(
    Arena: *mut memory_arena,
    TileMap: &mut tile_map,
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
    if !((*TileChunk).Tiles != null_mut()) {
        let TileCount = (*TileMap).ChunkDim * (*TileMap).ChunkDim;
        (*TileChunk).Tiles = &mut (*PushArray::<u32>(&mut *Arena, TileCount));
        for TileIndex in 0..TileCount
        /*   (let TileIndex = 0;
        TileIndex < TileCount;
        ++TileIndex) */
        {
            *((*TileChunk).Tiles as *mut u32).offset(TileIndex.try_into().unwrap()) = 1;
        }
    } else {
    }

    SetTileValue_(
        TileMap,
        TileChunk,
        ChunkPos.RelTileX,
        ChunkPos.RelTileY,
        TileValue,
    );
}
#[derive(Default)]
pub struct tile_map_difference {
    pub dXY: v2,
    pub dZ: f32,
}

pub fn Subtract(
    TileMap: &tile_map,
    A: &tile_map_position,
    B: &tile_map_position,
) -> tile_map_difference {
    let mut result = tile_map_difference::default();

    let dTileXY = v2 {
        X: A.AbsTileX as f32 - B.AbsTileX as f32,
        Y: A.AbsTileY as f32 - B.AbsTileY as f32,
    };
    let dTileZ = A.AbsTileZ as f32 - B.AbsTileZ as f32;

    result.dXY = TileMap.TileSideInMeters * dTileXY + (A.Offset - B.Offset);
    // TODO(casey): Think about what we want to do about Z
    result.dZ = TileMap.TileSideInMeters * dTileZ;

    return result;
}

pub fn AreOnSameTile(A: &tile_map_position, B: &tile_map_position) -> bool {
    let result =
        (A.AbsTileX == B.AbsTileX) && (A.AbsTileY == B.AbsTileY) && (A.AbsTileZ == B.AbsTileZ);

    return result;
}

pub fn CenteredTilePoint(AbsTileX: u32, AbsTileY: u32, AbsTileZ: u32) -> tile_map_position {
    let mut result = tile_map_position::default();

    result.AbsTileX = AbsTileX;
    result.AbsTileY = AbsTileY;
    result.AbsTileZ = AbsTileZ;

    return result;
}
