//! Мир: чанки и доступ к блокам по глобальным координатам.

use hashbrown::HashMap;

use crate::block::BlockId;
use crate::chunk::Chunk;
use crate::coord::{BlockPos, ChunkPos};

/// Мир: все загруженные чанки + seed.
///
/// Чанки хранятся в быстром `HashMap` (`hashbrown`, хешер `foldhash`): доступ
/// к чанку — горячий путь, а ключи (`ChunkPos`) внутренние, поэтому
/// DoS-устойчивый `SipHash` из `std` не нужен. Порядок итерации не используется —
/// только lookup (детерминизм).
#[derive(Debug, Clone)]
pub struct World {
    seed: u64,
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    /// Пустой мир с заданным seed (без загруженных чанков).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            chunks: HashMap::new(),
        }
    }

    /// Seed мира.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Загруженный чанк (если есть).
    #[must_use]
    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// Блок по глобальным координатам.
    ///
    /// Возвращает `None`, если чанк не загружен — в отличие от «воздуха», это
    /// разные состояния: `Some(AIR)` значит «загруженный пустой блок», а `None`
    /// — «чанка нет». Вызывающий сам решает, как трактовать незагруженный чанк.
    #[must_use]
    pub fn get_block(&self, pos: BlockPos) -> Option<BlockId> {
        let (chunk_pos, local) = pos.decompose();
        self.chunks.get(&chunk_pos).map(|chunk| chunk.get(local))
    }

    /// Устанавливает блок по глобальным координатам.
    ///
    /// Возвращает `true`, если блок установлен, и `false`, если чанк не загружен
    /// (ничего не меняется). Чанки создаются только явно через [`World::insert_chunk`].
    pub fn set_block(&mut self, pos: BlockPos, block: BlockId) -> bool {
        let (chunk_pos, local) = pos.decompose();
        match self.chunks.get_mut(&chunk_pos) {
            Some(chunk) => {
                chunk.set(local, block);
                true
            }
            None => false,
        }
    }

    /// Вставляет готовый чанк (заменяет существующий).
    pub fn insert_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    /// Число загруженных чанков.
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Пуст ли мир (нет загруженных чанков).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockRegistry;
    use crate::coord::LocalPos;

    #[test]
    fn unloaded_chunk_returns_none() {
        let world = World::new(0);
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), None);
        assert_eq!(world.get_block(BlockPos::new(-100, 50, 1000)), None);
    }

    #[test]
    fn set_block_into_unloaded_chunk_fails() {
        let mut world = World::new(0);
        assert!(!world.set_block(BlockPos::new(0, 0, 0), BlockId::new(5)));
        assert_eq!(world.len(), 0);
    }

    #[test]
    fn set_then_get_round_trip_loaded_chunk() {
        let mut world = World::new(0);
        let stone = BlockId::new(5);
        world.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::empty());
        assert!(world.set_block(BlockPos::new(1, 2, 3), stone));
        assert_eq!(world.get_block(BlockPos::new(1, 2, 3)), Some(stone));
        // Соседняя клетка загруженного чанка — Some(воздух), а не None.
        assert_eq!(
            world.get_block(BlockPos::new(1, 2, 4)),
            Some(BlockRegistry::AIR)
        );
    }

    #[test]
    fn set_across_chunk_boundaries() {
        let mut world = World::new(0);
        let a = BlockId::new(5);
        let b = BlockId::new(6);
        for cp in [
            ChunkPos::new(0, 0, 0),
            ChunkPos::new(1, 0, 0),
            ChunkPos::new(-1, 0, 0),
        ] {
            world.insert_chunk(cp, Chunk::empty());
        }
        assert!(world.set_block(BlockPos::new(0, 0, 0), a));
        assert!(world.set_block(BlockPos::new(15, 0, 0), a));
        assert!(world.set_block(BlockPos::new(16, 0, 0), b)); // чанк (1,0,0)
        assert!(world.set_block(BlockPos::new(-1, 0, 0), b)); // чанк (-1,0,0)
        assert!(world.set_block(BlockPos::new(-16, 0, 0), a)); // чанк (-1,0,0)

        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), Some(a));
        assert_eq!(world.get_block(BlockPos::new(15, 0, 0)), Some(a));
        assert_eq!(world.get_block(BlockPos::new(16, 0, 0)), Some(b));
        assert_eq!(world.get_block(BlockPos::new(-1, 0, 0)), Some(b));
        assert_eq!(world.get_block(BlockPos::new(-16, 0, 0)), Some(a));
        assert_eq!(world.len(), 3);
    }

    #[test]
    fn loaded_chunk_air_cell_returns_some_air() {
        let mut world = World::new(0);
        world.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::empty());
        assert_eq!(
            world.get_block(BlockPos::new(0, 0, 0)),
            Some(BlockRegistry::AIR)
        );
    }

    #[test]
    fn set_air_into_loaded_chunk_succeeds() {
        let mut world = World::new(0);
        world.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::empty());
        assert!(world.set_block(BlockPos::new(0, 0, 0), BlockId::new(5)));
        assert!(world.set_block(BlockPos::new(0, 0, 0), BlockRegistry::AIR));
        assert_eq!(
            world.get_block(BlockPos::new(0, 0, 0)),
            Some(BlockRegistry::AIR)
        );
    }

    #[test]
    fn insert_chunk_is_readable() {
        let mut world = World::new(0);
        let mut chunk = Chunk::empty();
        chunk.set(LocalPos::new(0, 0, 0), BlockId::new(5));
        world.insert_chunk(ChunkPos::new(0, 0, 0), chunk);
        assert_eq!(
            world.get_block(BlockPos::new(0, 0, 0)),
            Some(BlockId::new(5))
        );
    }

    #[test]
    fn seed_is_preserved() {
        let world = World::new(12345);
        assert_eq!(world.seed(), 12345);
    }

    #[test]
    fn is_empty_reflects_state() {
        let mut world = World::new(0);
        assert!(world.is_empty());
        world.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::empty());
        assert!(!world.is_empty());
    }
}
