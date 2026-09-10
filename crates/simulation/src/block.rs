//! Блоки: числовые хэндлы и их свойства.

use hashbrown::HashMap;

use crate::id::ResourceId;

/// Числовой идентификатор блока — непрозрачный рантайм-хэндл, назначаемый
/// реестром при регистрации. Действителен только в рамках сессии: не сохранять
/// и не передавать по сети. Стабильная идентичность — [`ResourceId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(u16);

impl BlockId {
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Свойства типа блока.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Канонический идентификатор (например, `one_of_two:stone`).
    pub id: ResourceId,
    /// Есть ли коллизия.
    pub solid: bool,
    /// Перекрывает ли соседние грани (culling при мешинге).
    pub opaque: bool,
}

impl Block {
    #[must_use]
    pub fn new(id: ResourceId, solid: bool, opaque: bool) -> Self {
        Self { id, solid, opaque }
    }
}

/// Встроенные блоки — единственный источник правды.
fn builtin_blocks() -> Vec<Block> {
    vec![
        Block::new(ResourceId::builtin("grass"), true, true),
        Block::new(ResourceId::builtin("dirt"), true, true),
        Block::new(ResourceId::builtin("stone"), true, true),
        Block::new(ResourceId::builtin("sand"), true, true),
        Block::new(ResourceId::builtin("water"), false, false),
        Block::new(ResourceId::builtin("wood"), true, true),
        Block::new(ResourceId::builtin("leaves"), true, false),
        Block::new(ResourceId::builtin("planks"), true, true),
    ]
}

/// Разрешённые идентификаторы встроенных блоков.
///
/// Создаётся один раз из реестра; остальной код берёт [`BlockId`] через поля,
/// не обращаясь к строковым именам.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinBlocks {
    pub air: BlockId,
    pub grass: BlockId,
    pub dirt: BlockId,
    pub stone: BlockId,
    pub sand: BlockId,
    pub water: BlockId,
    pub wood: BlockId,
    pub leaves: BlockId,
    pub planks: BlockId,
}

/// Реестр блоков.
#[derive(Debug, Clone)]
pub struct BlockRegistry {
    blocks: Vec<Block>,
    by_id: HashMap<ResourceId, BlockId>,
}

impl BlockRegistry {
    /// Воздух — отсутствие блока. Всегда id `0`.
    pub const AIR: BlockId = BlockId::new(0);
    /// Заглушка для неизвестных/повреждённых id. Всегда id `1`.
    pub const UNKNOWN: BlockId = BlockId::new(1);

    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: Vec::new(),
            by_id: HashMap::new(),
        };
        // Порядок фиксирует инварианты AIR == 0, UNKNOWN == 1.
        registry.register(Block::new(ResourceId::builtin("air"), false, false));
        registry.register(Block::new(ResourceId::builtin("unknown"), true, true));
        for block in builtin_blocks() {
            registry.register(block);
        }
        registry
    }

    /// Регистрирует блок и возвращает числовой id.
    ///
    /// # Panics
    ///
    /// В debug-сборке паникует при повторной регистрации того же id; паникует
    /// при переполнении числового пространства (`u16`).
    pub fn register(&mut self, block: Block) -> BlockId {
        debug_assert!(
            !self.by_id.contains_key(&block.id),
            "duplicate block id: {}",
            block.id
        );
        let id = BlockId::new(u16::try_from(self.blocks.len()).expect("too many blocks"));
        self.by_id.insert(block.id.clone(), id);
        self.blocks.push(block);
        id
    }

    /// Свойства по id. Неизвестный id → блок-заглушка [`BlockRegistry::UNKNOWN`].
    #[must_use]
    pub fn get(&self, id: BlockId) -> &Block {
        self.blocks
            .get(usize::from(id.raw()))
            .unwrap_or_else(|| self.get(Self::UNKNOWN))
    }

    /// Числовой id по каноническому идентификатору.
    #[must_use]
    pub fn get_id(&self, id: &ResourceId) -> Option<BlockId> {
        self.by_id.get(id).copied()
    }

    /// Разрешает встроенные блоки в типизированную структуру.
    ///
    /// # Panics
    ///
    /// Паникует, если встроенный блок отсутствует (невозможно при
    /// [`BlockRegistry::new`]).
    #[must_use]
    pub fn builtin_blocks(&self) -> BuiltinBlocks {
        BuiltinBlocks {
            air: self
                .get_id(&ResourceId::builtin("air"))
                .expect("builtin air"),
            grass: self
                .get_id(&ResourceId::builtin("grass"))
                .expect("builtin grass"),
            dirt: self
                .get_id(&ResourceId::builtin("dirt"))
                .expect("builtin dirt"),
            stone: self
                .get_id(&ResourceId::builtin("stone"))
                .expect("builtin stone"),
            sand: self
                .get_id(&ResourceId::builtin("sand"))
                .expect("builtin sand"),
            water: self
                .get_id(&ResourceId::builtin("water"))
                .expect("builtin water"),
            wood: self
                .get_id(&ResourceId::builtin("wood"))
                .expect("builtin wood"),
            leaves: self
                .get_id(&ResourceId::builtin("leaves"))
                .expect("builtin leaves"),
            planks: self
                .get_id(&ResourceId::builtin("planks"))
                .expect("builtin planks"),
        }
    }

    #[must_use]
    pub fn is_solid(&self, id: BlockId) -> bool {
        self.get(id).solid
    }

    #[must_use]
    pub fn is_opaque(&self, id: BlockId) -> bool {
        self.get(id).opaque
    }

    /// Количество зарегистрированных блоков.
    ///
    /// `is_empty` не предусмотрен намеренно: реестр по инварианту всегда
    /// содержит минимум воздух и заглушку `unknown`.
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_and_unknown_are_reserved_first() {
        let reg = BlockRegistry::new();
        assert_eq!(BlockRegistry::AIR.raw(), 0);
        assert_eq!(BlockRegistry::UNKNOWN.raw(), 1);
        assert_eq!(
            reg.get_id(&ResourceId::builtin("air")),
            Some(BlockRegistry::AIR)
        );
        assert_eq!(
            reg.get_id(&ResourceId::builtin("unknown")),
            Some(BlockRegistry::UNKNOWN)
        );
        assert_eq!(reg.get(BlockRegistry::AIR).id.path(), "air");
        assert_eq!(reg.get(BlockRegistry::UNKNOWN).id.path(), "unknown");
    }

    #[test]
    fn builtin_ids_resolve() {
        let reg = BlockRegistry::new();
        for path in [
            "grass", "dirt", "stone", "sand", "water", "wood", "leaves", "planks",
        ] {
            assert!(
                reg.get_id(&ResourceId::builtin(path)).is_some(),
                "missing builtin block: {path}"
            );
        }
    }

    #[test]
    fn solid_and_opaque_flags() {
        let reg = BlockRegistry::new();
        for path in ["grass", "dirt", "stone", "sand", "wood", "planks"] {
            let id = reg.get_id(&ResourceId::builtin(path)).unwrap();
            assert!(reg.is_solid(id), "{path} should be solid");
            assert!(reg.is_opaque(id), "{path} should be opaque");
        }
        let water = reg.get_id(&ResourceId::builtin("water")).unwrap();
        assert!(!reg.is_solid(water));
        assert!(!reg.is_opaque(water));
        let leaves = reg.get_id(&ResourceId::builtin("leaves")).unwrap();
        assert!(reg.is_solid(leaves));
        assert!(!reg.is_opaque(leaves));
    }

    #[test]
    fn builtin_blocks_resolve_to_distinct_ids() {
        let reg = BlockRegistry::new();
        let blocks = reg.builtin_blocks();
        assert_eq!(blocks.air, BlockRegistry::AIR);
        assert_eq!(reg.get(blocks.grass).id.path(), "grass");
        assert_eq!(reg.get(blocks.stone).id.path(), "stone");
        assert_eq!(reg.get(blocks.water).id.path(), "water");
    }

    #[test]
    fn unknown_id_returns_unknown_block_not_air() {
        let reg = BlockRegistry::new();
        let bogus = BlockId::new(999);
        let block = reg.get(bogus);
        assert_eq!(block.id, ResourceId::builtin("unknown"));
        assert!(block.solid);
        assert!(block.opaque);
    }

    #[test]
    fn register_custom_block_is_dense_and_reversible() {
        let mut reg = BlockRegistry::new();
        let id = reg.register(Block::new(ResourceId::new("mod", "copper"), true, true));
        assert_eq!(reg.get(id).id, ResourceId::new("mod", "copper"));
        assert_eq!(reg.get_id(&ResourceId::new("mod", "copper")), Some(id));
        assert_eq!(reg.len(), 11); // 2 reserved + 8 builtin + 1 custom
    }

    #[test]
    fn block_id_raw_round_trip() {
        assert_eq!(BlockId::new(42).raw(), 42);
    }
}
