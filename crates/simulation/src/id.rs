//! Канонические идентификаторы ресурсов вида `namespace:path`.

/// Пространство имён по умолчанию (встроенные блоки/айтемы игры).
pub const DEFAULT_NAMESPACE: &str = "one_of_two";

/// Канонический идентификатор ресурса: `namespace:path`.
///
/// Стабильная идентичность сущностей (блоки, будущие айтемы). Числовые хэндлы
/// (например [`crate::block::BlockId`]) непрозрачны и нестабильны между
/// сессиями; `ResourceId` — то, что сохраняется и передаётся по сети.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    /// Создаёт идентификатор из пространства имён и пути.
    ///
    /// В debug-сборке паникует, если `namespace` или `path` пусты или содержат `:`.
    #[must_use]
    pub fn new(namespace: &str, path: &str) -> Self {
        debug_assert!(!namespace.is_empty(), "namespace must not be empty");
        debug_assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(!namespace.contains(':'), "namespace must not contain ':'");
        debug_assert!(!path.contains(':'), "path must not contain ':'");
        Self(format!("{namespace}:{path}"))
    }

    /// Идентификатор в пространстве имён по умолчанию.
    #[must_use]
    pub fn builtin(path: &str) -> Self {
        Self::new(DEFAULT_NAMESPACE, path)
    }

    /// Пространство имён (часть до первого `:`).
    ///
    /// # Panics
    ///
    /// Не паникует: [`ResourceId`] всегда строится через [`ResourceId::new`],
    /// которая гарантирует наличие разделителя `:`.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0
            .split_once(':')
            .expect("ResourceId is always namespace:path")
            .0
    }

    /// Путь (часть после первого `:`).
    ///
    /// # Panics
    ///
    /// Не паникует: [`ResourceId`] всегда строится через [`ResourceId::new`],
    /// которая гарантирует наличие разделителя `:`.
    #[must_use]
    pub fn path(&self) -> &str {
        self.0
            .split_once(':')
            .expect("ResourceId is always namespace:path")
            .1
    }

    /// Полный идентификатор как `namespace:path`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_id_format() {
        let id = ResourceId::builtin("stone");
        assert_eq!(id.namespace(), DEFAULT_NAMESPACE);
        assert_eq!(id.path(), "stone");
        assert_eq!(id.as_str(), "one_of_two:stone");
        assert_eq!(id.to_string(), "one_of_two:stone");
    }

    #[test]
    fn equality_and_hash() {
        assert_eq!(ResourceId::new("a", "b"), ResourceId::new("a", "b"));
        assert_ne!(ResourceId::new("a", "b"), ResourceId::new("a", "c"));
        assert_ne!(ResourceId::new("a", "b"), ResourceId::new("c", "b"));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "namespace must not be empty")]
    fn empty_namespace_panics() {
        let _ = ResourceId::new("", "x");
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "path must not be empty")]
    fn empty_path_panics() {
        let _ = ResourceId::new("ns", "");
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "namespace must not contain ':'")]
    fn colon_in_namespace_panics() {
        let _ = ResourceId::new("a:b", "c");
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "path must not contain ':'")]
    fn colon_in_path_panics() {
        let _ = ResourceId::new("a", "b:c");
    }
}
