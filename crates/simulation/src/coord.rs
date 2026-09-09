//! Координаты мира: позиции блоков, чанков и локальные позиции внутри чанка.
//!
//! Ключевой инвариант: деление на размер чанка — только через
//! [`i32::div_euclid`] / [`i32::rem_euclid`], чтобы отрицательные координаты
//! корректно попадали в отрицательные чанки (обычные `/` и `%` обрезают к нулю).

/// Длина ребра чанка в блоках. Чанк кубический: `CHUNK_SIZE` × `CHUNK_SIZE` × `CHUNK_SIZE`.
pub const CHUNK_SIZE: i32 = 16;

/// Глобальная позиция блока в мире.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Чанк, которому принадлежит блок. Корректно для отрицательных координат.
    #[must_use]
    pub fn to_chunk_pos(self) -> ChunkPos {
        ChunkPos::new(
            self.x.div_euclid(CHUNK_SIZE),
            self.y.div_euclid(CHUNK_SIZE),
            self.z.div_euclid(CHUNK_SIZE),
        )
    }

    /// Позиция блока внутри чанка (каждая координата в `0..CHUNK_SIZE`).
    #[must_use]
    pub fn to_local_pos(self) -> LocalPos {
        LocalPos::new(to_local(self.x), to_local(self.y), to_local(self.z))
    }

    /// Раскладывает позицию на `(чанк, локальная позиция)`.
    #[must_use]
    pub fn decompose(self) -> (ChunkPos, LocalPos) {
        (self.to_chunk_pos(), self.to_local_pos())
    }

    /// Позиция блока по чанку и локальной позиции внутри него.
    #[must_use]
    pub fn from_chunk_local(chunk: ChunkPos, local: LocalPos) -> Self {
        let origin = chunk.origin();
        Self::new(
            origin.x + i32::from(local.x),
            origin.y + i32::from(local.y),
            origin.z + i32::from(local.z),
        )
    }
}

/// Координата чанка в сетке чанков.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Глобальная позиция углового блока чанка (с минимальными координатами).
    #[must_use]
    pub fn origin(self) -> BlockPos {
        BlockPos::new(
            self.x * CHUNK_SIZE,
            self.y * CHUNK_SIZE,
            self.z * CHUNK_SIZE,
        )
    }
}

/// Локальная позиция блока внутри чанка (каждая координата в `0..CHUNK_SIZE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl LocalPos {
    /// Создаёт локальную позицию. В debug-сборке проверяет, что координаты `< CHUNK_SIZE`.
    #[must_use]
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        debug_assert!(i32::from(x) < CHUNK_SIZE);
        debug_assert!(i32::from(y) < CHUNK_SIZE);
        debug_assert!(i32::from(z) < CHUNK_SIZE);
        Self { x, y, z }
    }
}

/// Остаток от деления координаты на размер чанка, всегда в `0..CHUNK_SIZE`.
fn to_local(coord: i32) -> u8 {
    u8::try_from(coord.rem_euclid(CHUNK_SIZE)).expect("remainder is always in 0..CHUNK_SIZE")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn chunk_origin() {
        assert_eq!(ChunkPos::new(0, 0, 0).origin(), BlockPos::new(0, 0, 0));
        assert_eq!(ChunkPos::new(1, 2, 3).origin(), BlockPos::new(16, 32, 48));
        assert_eq!(
            ChunkPos::new(-1, -1, -1).origin(),
            BlockPos::new(-16, -16, -16)
        );
    }

    #[test]
    fn origin_block_is_in_chunk_zero() {
        let b = BlockPos::new(0, 0, 0);
        assert_eq!(b.to_chunk_pos(), ChunkPos::new(0, 0, 0));
        assert_eq!(b.to_local_pos(), LocalPos::new(0, 0, 0));
    }

    #[test]
    fn positive_chunk_boundary() {
        let b = BlockPos::new(16, 0, 0);
        assert_eq!(b.to_chunk_pos(), ChunkPos::new(1, 0, 0));
        assert_eq!(b.to_local_pos(), LocalPos::new(0, 0, 0));
    }

    #[test]
    fn negative_coords_go_to_negative_chunk() {
        let b = BlockPos::new(-1, -1, -1);
        assert_eq!(b.to_chunk_pos(), ChunkPos::new(-1, -1, -1));
        assert_eq!(b.to_local_pos(), LocalPos::new(15, 15, 15));
    }

    #[test]
    fn negative_chunk_boundary() {
        let b = BlockPos::new(-16, -16, -16);
        assert_eq!(b.to_chunk_pos(), ChunkPos::new(-1, -1, -1));
        assert_eq!(b.to_local_pos(), LocalPos::new(0, 0, 0));

        let b = BlockPos::new(-17, 0, 0);
        assert_eq!(b.to_chunk_pos(), ChunkPos::new(-2, 0, 0));
        assert_eq!(b.to_local_pos(), LocalPos::new(15, 0, 0));
    }

    #[test]
    fn decompose_recompose_round_trip() {
        let samples = [
            (0, 0, 0),
            (1, 2, 3),
            (-1, -1, -1),
            (16, 0, 0),
            (-17, 31, -16),
            (1000, -1000, 255),
            (-1000, 0, 1000),
        ];
        for (x, y, z) in samples {
            let b = BlockPos::new(x, y, z);
            let (c, l) = b.decompose();
            assert_eq!(
                BlockPos::from_chunk_local(c, l),
                b,
                "round-trip failed for ({x}, {y}, {z})"
            );
        }
    }

    #[test]
    fn local_coords_always_in_range() {
        for i in -33..=33 {
            let b = BlockPos::new(i, i, i);
            let l = b.to_local_pos();
            assert!(i32::from(l.x) < CHUNK_SIZE);
            assert!(i32::from(l.y) < CHUNK_SIZE);
            assert!(i32::from(l.z) < CHUNK_SIZE);
        }
    }

    #[test]
    fn chunk_pos_is_usable_as_hash_key() {
        let mut set = HashSet::new();
        set.insert(ChunkPos::new(1, 0, 0));
        assert!(set.contains(&ChunkPos::new(1, 0, 0)));
        assert!(!set.contains(&ChunkPos::new(0, 0, 1)));
    }
}
