test_1:
	cargo run --quiet -- "Hel[^\\s]+" "*.txt"

test_2:
	cargo run --quiet -- "Hel" "*.txt"

test_3:
	cargo run --quiet -- "Re[^\\s]+" "src/*.rs"

build:
	cargo build

.PHONY: test_1 test_2 test_3 build