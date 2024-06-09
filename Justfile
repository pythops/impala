default:
    @just --list

run:
    cargo build && sudo -E target/debug/impala
