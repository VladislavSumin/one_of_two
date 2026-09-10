# Список команд: just
default:
    @just --list

# Форматировать всё
fmt:
    cargo fmt --all

# Проверить форматирование (CI)
fmt-check:
    cargo fmt --all --check

# Быстрая проверка типов всего workspace
check:
    cargo check --workspace --all-targets

# Строгий линт (CI)
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Все тесты (CI)
test:
    cargo test --workspace

# Документация (CI)
doc:
    cargo doc --workspace --no-deps

# Запустить игру
run:
    cargo run -p one_of_two

# Запустить игру в release (проверка производительности)
run-release:
    cargo run -p one_of_two --release
