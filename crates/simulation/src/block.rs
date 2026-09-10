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

/// Фаза накопления: регистрирует блоки до «заморозки» в [`BlockRegistry`].
///
/// Единственная точка, где набор блоков изменяем. После [`Self::build`] реестр
/// неизменяем — это требование будущей генерации атласов.
#[derive(Debug, Default)]
pub struct BlockRegistryBuilder {
    blocks: Vec<Block>,
    by_id: HashMap<ResourceId, BlockId>,
}

impl BlockRegistryBuilder {
    /// Пустой аккумулятор. Встроенные блоки добавляются через
    /// [`Self::with_builtins`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// Регистрирует воздух, заглушку `unknown` и встроенные блоки.
    ///
    /// Порядок фиксирует инварианты [`BlockRegistry::AIR`] == 0,
    /// [`BlockRegistry::UNKNOWN`] == 1.
    #[must_use]
    pub fn with_builtins(mut self) -> Self {
        self.register(Block::new(ResourceId::builtin("air"), false, false));
        self.register(Block::new(ResourceId::builtin("unknown"), true, true));
        for block in builtin_blocks() {
            self.register(block);
        }
        self
    }

    /// Замораживает реестр: после этого набор блоков неизменяем.
    ///
    /// # Panics
    ///
    /// Паникует, если отсутствуют встроенные блоки (реестр без них не имеет
    /// смысла).
    #[must_use]
    pub fn build(self) -> BlockRegistry {
        debug_assert_eq!(
            self.by_id.get(&ResourceId::builtin("air")),
            Some(&BlockRegistry::AIR),
            "air must be the first registered block"
        );
        debug_assert_eq!(
            self.by_id.get(&ResourceId::builtin("unknown")),
            Some(&BlockRegistry::UNKNOWN),
            "unknown must be the second registered block"
        );
        let builtin = resolve_builtin(&self.by_id);
        BlockRegistry {
            blocks: self.blocks,
            by_id: self.by_id,
            builtin,
        }
    }
}

/// Реестр блоков. Неизменяем после создания: изменение возможно только через
/// [`BlockRegistryBuilder`] до [`BlockRegistryBuilder::build`].
#[derive(Debug, Clone)]
pub struct BlockRegistry {
    blocks: Vec<Block>,
    by_id: HashMap<ResourceId, BlockId>,
    builtin: BuiltinBlocks,
}

impl BlockRegistry {
    /// Воздух — отсутствие блока. Всегда id `0`.
    pub const AIR: BlockId = BlockId::new(0);
    /// Заглушка для неизвестных/повреждённых id. Всегда id `1`.
    pub const UNKNOWN: BlockId = BlockId::new(1);

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

    /// Встроенные блоки как типизированная структура (закэшировано при
    /// [`BlockRegistryBuilder::build`]).
    #[must_use]
    pub fn builtin_blocks(&self) -> BuiltinBlocks {
        self.builtin
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

/// Разрешает встроенные блоки в типизированную структуру по таблице
/// «канонический id → числовой id».
///
/// # Panics
///
/// Паникует, если какой-то встроенный блок не зарегистрирован.
#[must_use]
fn resolve_builtin(by_id: &HashMap<ResourceId, BlockId>) -> BuiltinBlocks {
    let id = |path: &str| {
        by_id
            .get(&ResourceId::builtin(path))
            .copied()
            .unwrap_or_else(|| panic!("builtin block not registered: {path}"))
    };
    BuiltinBlocks {
        air: id("air"),
        grass: id("grass"),
        dirt: id("dirt"),
        stone: id("stone"),
        sand: id("sand"),
        water: id("water"),
        wood: id("wood"),
        leaves: id("leaves"),
        planks: id("planks"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> BlockRegistry {
        BlockRegistryBuilder::new().with_builtins().build()
    }

    #[test]
    fn air_and_unknown_are_reserved_first() {
        let reg = registry();
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
        let reg = registry();
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
        let reg = registry();
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
        let reg = registry();
        let blocks = reg.builtin_blocks();
        assert_eq!(blocks.air, BlockRegistry::AIR);
        assert_eq!(reg.get(blocks.grass).id.path(), "grass");
        assert_eq!(reg.get(blocks.stone).id.path(), "stone");
        assert_eq!(reg.get(blocks.water).id.path(), "water");
    }

    #[test]
    fn unknown_id_returns_unknown_block_not_air() {
        let reg = registry();
        let bogus = BlockId::new(999);
        let block = reg.get(bogus);
        assert_eq!(block.id, ResourceId::builtin("unknown"));
        assert!(block.solid);
        assert!(block.opaque);
    }

    #[test]
    fn register_custom_block_is_dense_and_reversible() {
        let mut builder = BlockRegistryBuilder::new().with_builtins();
        let id = builder.register(Block::new(ResourceId::new("mod", "copper"), true, true));
        let reg = builder.build();
        assert_eq!(reg.get(id).id, ResourceId::new("mod", "copper"));
        assert_eq!(reg.get_id(&ResourceId::new("mod", "copper")), Some(id));
        assert_eq!(reg.len(), 11); // 2 reserved + 8 builtin + 1 custom
    }

    #[test]
    fn block_id_raw_round_trip() {
        assert_eq!(BlockId::new(42).raw(), 42);
    }
}
