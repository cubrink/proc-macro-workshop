default:
    just --list

check:
    cargo +nightly expand --bin workshop > bin/expanded.rs
    cargo +nightly check --bin expanded

expand: 
    cargo expand --bin workshop

clippy:
    cargo +nightly expand --bin workshop > bin/expanded.rs
    cargo +nightly clippy --bin expanded
