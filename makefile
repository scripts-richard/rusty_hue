install: release move

release: ./target/release/rusty_hue
	cargo build --release

move: /usr/local/bin/lights
	sudo cp ./target/release/rusty_hue /usr/local/bin/lights
