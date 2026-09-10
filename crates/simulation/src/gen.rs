//! Детерминированная генерация террейна по seed.

use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

use crate::block::{BlockId, BuiltinBlocks};
use crate::chunk::Chunk;
use crate::coord::{ChunkPos, LocalPos, CHUNK_SIZE_U8};

/// Уровень моря.
pub const SEA_LEVEL: i32 = 64;
/// Минимальная высота поверхности.
pub const MIN_HEIGHT: i32 = 8;
/// Максимальная высота поверхности.
pub const MAX_HEIGHT: i32 = 160;

/// Толщина слоя земли под травой.
const DIRT_DEPTH: i32 = 3;
/// Частота шума (циклов на блок).
const FREQUENCY: f64 = 0.005;
/// Октавы fBm.
const OCTAVES: usize = 4;
/// Персистентность fBm.
const PERSISTENCE: f64 = 0.5;
/// Лакунарность fBm.
const LACUNARITY: f64 = 2.0;
/// Амплитуда высоты (в блоках, от уровня моря).
const AMPLITUDE: f64 = 32.0;

/// Генератор террейна: детерминированная высотная карта по seed.
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    seed: u64,
    noise: Fbm<Perlin>,
    blocks: BuiltinBlocks,
}

impl TerrainGenerator {
    /// Создаёт генератор по seed и набору встроенных блоков.
    // `noise` принимает u32-сид; младшие биты u64 так же хороши.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn new(seed: u64, blocks: BuiltinBlocks) -> Self {
        let noise = Fbm::<Perlin>::new(seed as u32)
            .set_octaves(OCTAVES)
            .set_frequency(FREQUENCY)
            .set_persistence(PERSISTENCE)
            .set_lacunarity(LACUNARITY);

        Self {
            seed,
            noise,
            blocks,
        }
    }

    /// Seed генератора.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Высота поверхности (верхний блок земли) в колонке `(x, z)`.
    // Значение зажато в [MIN_HEIGHT, MAX_HEIGHT] ⊂ i32 — приведение безопасно.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn height_at(&self, x: i32, z: i32) -> i32 {
        let n = self.noise.get([f64::from(x), f64::from(z)]);
        let h = f64::from(SEA_LEVEL) + n * AMPLITUDE;
        h.round()
            .clamp(f64::from(MIN_HEIGHT), f64::from(MAX_HEIGHT)) as i32
    }

    /// Генерирует чанк. Детерминированно: одинаковый seed → одинаковый чанк.
    #[must_use]
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let origin = pos.origin();
        let mut chunk = Chunk::empty();
        for lz in 0..CHUNK_SIZE_U8 {
            for lx in 0..CHUNK_SIZE_U8 {
                let gx = origin.x + i32::from(lx);
                let gz = origin.z + i32::from(lz);
                let h = self.height_at(gx, gz);
                for ly in 0..CHUNK_SIZE_U8 {
                    let gy = origin.y + i32::from(ly);
                    let block = self.block_at(gy, h);
                    if block != self.blocks.air {
                        chunk.set(LocalPos::new(lx, ly, lz), block);
                    }
                }
            }
        }
        chunk
    }

    /// Блок для глобальной `y` при высоте поверхности `h`.
    fn block_at(&self, y: i32, h: i32) -> BlockId {
        if y <= h {
            if y == h {
                if h >= SEA_LEVEL {
                    self.blocks.grass
                } else {
                    self.blocks.sand
                }
            } else if y >= h - DIRT_DEPTH {
                self.blocks.dirt
            } else {
                self.blocks.stone
            }
        } else if y <= SEA_LEVEL {
            self.blocks.water
        } else {
            self.blocks.air
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockRegistry;
    use crate::coord::{BlockPos, ChunkPos};

    fn generator(seed: u64) -> TerrainGenerator {
        TerrainGenerator::new(seed, BlockRegistry::new().builtin_blocks())
    }

    #[test]
    fn same_seed_produces_identical_chunks() {
        let a = generator(42);
        let b = generator(42);
        for pos in [
            ChunkPos::new(0, 0, 0),
            ChunkPos::new(1, 2, 3),
            ChunkPos::new(-1, -1, -1),
        ] {
            assert_eq!(
                a.generate_chunk(pos),
                b.generate_chunk(pos),
                "chunk {pos:?}"
            );
        }
    }

    #[test]
    fn generator_is_deterministic_per_instance() {
        let gen = generator(7);
        let pos = ChunkPos::new(0, 0, 0);
        assert_eq!(gen.generate_chunk(pos), gen.generate_chunk(pos));
    }

    #[test]
    fn different_seeds_differ() {
        let a = generator(1);
        let b = generator(2);
        let mut any_diff = false;
        for x in -2..=2 {
            for z in -2..=2 {
                let pos = ChunkPos::new(x, 3, z);
                if a.generate_chunk(pos) != b.generate_chunk(pos) {
                    any_diff = true;
                }
            }
        }
        assert!(any_diff, "seeds 1 and 2 produced identical terrain");
    }

    #[test]
    fn height_at_is_deterministic_and_in_range() {
        let gen = generator(9);
        for x in -100..=100 {
            for z in -100..=100 {
                let h = gen.height_at(x, z);
                assert!(
                    (MIN_HEIGHT..=MAX_HEIGHT).contains(&h),
                    "height {h} out of range at ({x},{z})"
                );
                assert_eq!(h, gen.height_at(x, z));
            }
        }
    }

    #[test]
    fn column_layering_is_consistent() {
        let gen = generator(5);
        let (x, z) = (0, 0);
        let h = gen.height_at(x, z);

        let block_at = |y: i32| -> BlockId {
            let pos = BlockPos::new(x, y, z);
            gen.generate_chunk(pos.to_chunk_pos())
                .get(pos.to_local_pos())
        };

        // Поверхность и ниже — не воздух.
        assert_ne!(block_at(h), gen.blocks.air);
        assert_ne!(block_at(h - 1), gen.blocks.air);
        // Выше уровня моря — воздух.
        if h >= SEA_LEVEL {
            assert_eq!(block_at(h + 1), gen.blocks.air);
        }
        // Глубоко внизу — камень.
        assert_eq!(block_at(2), gen.blocks.stone);
    }

    #[test]
    fn seed_is_preserved() {
        assert_eq!(generator(12345).seed(), 12345);
    }
}
