DESTDIR := env_var_or_default("DESTDIR", "/")

default:
    @just --list

run:
    cargo build && sudo -E target/debug/impala

build:
    cargo run --release --locked

install:
    install -Dm 755 target/release/impala -t {{DESTDIR}}/usr/bin/
    install -Dm 755 desktop -t {{DESTDIR}}/usr/share/applications/
