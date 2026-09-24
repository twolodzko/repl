
test:
    cargo fmt
    cargo clippy
    cargo test

build:
    cargo build --release
    cp ./target/release/repl ./

install: test
    cargo install --path .

clean:
    rm -rf ./repl ./target
