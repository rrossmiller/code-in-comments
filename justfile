run:
    @clear
    @cargo run test/rate_limiter.py
mult-run:
    @clear
    @cargo run test/rate_limiter.py test/main.py
dir-run:
    @clear
    @cargo run test


build:
    @cargo build --release
build-move:build
    mv target/release/comment_checker /usr/local/bin/
