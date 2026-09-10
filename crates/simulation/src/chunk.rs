//! Чанк: хранение блоков 16×16×16 через палитру.

use crate::block::{BlockId, BlockRegistry};
use crate::coord::{LocalPos, CHUNK_VOLUME};

/// Чанк мира: 16×16×16 блоков, хранящиеся через палитру.
///
/// Каждая клетка хранит индекс в [`Chunk::palette`]; индекс `0` — всегда
/// воздух. Плоский индекс клетки задаётся [`LocalPos::to_index`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    palette: Vec<BlockId>,
    data: [u8; CHUNK_VOLUME],
}

impl Chunk {
    /// Пустой чанк (весь заполнен воздухом).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            palette: vec![BlockRegistry::AIR],
            data: [0; CHUNK_VOLUME],
        }
    }

    /// Блок в локальной позиции.
    #[must_use]
    pub fn get(&self, pos: LocalPos) -> BlockId {
        let idx = usize::from(self.data[pos.to_index()]);
        self.palette[idx]
    }

    /// Устанавливает блок. Палитра растёт по мере появления новых типов.
    ///
    /// # Panics
    ///
    /// Паникует при переполнении палитры (более 256 различных типов в чанке) —
    /// на практике недостижимо.
    pub fn set(&mut self, pos: LocalPos, block: BlockId) {
        let idx = self.palette_index(block);
        self.data[pos.to_index()] = idx;
    }

    /// Палитра чанка: различные типы блоков (индекс 0 — воздух).
    #[must_use]
    pub fn palette(&self) -> &[BlockId] {
        &self.palette
    }

    /// Индекс блока в палитре: находит существующий или добавляет новый.
    fn palette_index(&mut self, block: BlockId) -> u8 {
        if let Some(idx) = self.palette.iter().position(|&b| b == block) {
            return u8::try_from(idx).expect("palette index fits in u8");
        }
        debug_assert!(self.palette.len() < 256, "chunk palette overflow");
        let idx = self.palette.len();
        self.palette.push(block);
        u8::try_from(idx).expect("palette index fits in u8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::LocalPos;

    fn pos(x: u8, y: u8, z: u8) -> LocalPos {
        LocalPos::new(x, y, z)
    }

    #[test]
    fn empty_chunk_is_all_air() {
        let chunk = Chunk::empty();
        for y in 0..16u8 {
            for z in 0..16u8 {
                for x in 0..16u8 {
                    assert_eq!(chunk.get(pos(x, y, z)), BlockRegistry::AIR);
                }
            }
        }
    }

    #[test]
    fn set_get_round_trip() {
        let mut chunk = Chunk::empty();
        let stone = BlockId::new(5);
        chunk.set(pos(3, 4, 5), stone);
        assert_eq!(chunk.get(pos(3, 4, 5)), stone);
        assert_eq!(chunk.get(pos(3, 4, 6)), BlockRegistry::AIR);
    }

    #[test]
    fn palette_does_not_grow_for_duplicates() {
        let mut chunk = Chunk::empty();
        let stone = BlockId::new(5);
        chunk.set(pos(0, 0, 0), stone);
        chunk.set(pos(1, 0, 0), stone);
        chunk.set(pos(2, 0, 0), stone);
        assert_eq!(chunk.palette().len(), 2); // air + stone
    }

    #[test]
    fn palette_grows_for_new_blocks() {
        let mut chunk = Chunk::empty();
        chunk.set(pos(0, 0, 0), BlockId::new(5));
        chunk.set(pos(1, 0, 0), BlockId::new(6));
        chunk.set(pos(2, 0, 0), BlockId::new(7));
        assert_eq!(chunk.palette().len(), 4); // air + 3
    }

    #[test]
    fn overwrite_replaces_block() {
        let mut chunk = Chunk::empty();
        chunk.set(pos(0, 0, 0), BlockId::new(5));
        chunk.set(pos(0, 0, 0), BlockId::new(6));
        assert_eq!(chunk.get(pos(0, 0, 0)), BlockId::new(6));
    }

    #[test]
    fn setting_air_keeps_palette_minimal() {
        let mut chunk = Chunk::empty();
        chunk.set(pos(0, 0, 0), BlockId::new(5));
        chunk.set(pos(0, 0, 0), BlockRegistry::AIR);
        assert_eq!(chunk.get(pos(0, 0, 0)), BlockRegistry::AIR);
        // Камень остаётся в палитре (без компактизации) — это ок.
        assert_eq!(chunk.palette().len(), 2);
    }

    #[test]
    fn full_cube_round_trip() {
        let mut chunk = Chunk::empty();
        for y in 0..16u8 {
            for z in 0..16u8 {
                for x in 0..16u8 {
                    let id = BlockId::new(u16::from((x + y + z) % 8) + 2);
                    chunk.set(pos(x, y, z), id);
                }
            }
        }
        for y in 0..16u8 {
            for z in 0..16u8 {
                for x in 0..16u8 {
                    let id = BlockId::new(u16::from((x + y + z) % 8) + 2);
                    assert_eq!(chunk.get(pos(x, y, z)), id);
                }
            }
        }
    }
}
