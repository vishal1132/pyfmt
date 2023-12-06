alias b:=build
alias m:=move

build:
    @cargo build --release

fmt:
    @cargo fmt

move: build
    @mv target/release/pyfmt ~/.bin