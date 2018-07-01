
TARGETS := target/release/say \
  target/release/tts-service \
  target/release/speak-selection

VERSION := $(shell grep version Cargo.toml | head -1 | cut -d= -f 2 | xargs echo )

all: $(TARGETS)

target/release/say: src/bin/say.rs
	cargo build --release --bin say

target/release/tts-service: src/tts.rs src/config.rs src/dbus.rs src/lib.rs src/server.rs src/sound.rs
	cargo build --release --bin tts-service

target/release/speak-selection: src/bin/speak-selection.rs
	cargo build --release --bin speak-selection
clean:
	rm -f $(TARGETS)

install: all
	install $(TARGETS) $(HOME)/.local/bin/
	cat misc/tts.service | PREFIX="$(HOME)/.local" envsubst > ~/.config/systemd/user/tts.service
	systemctl --user daemon-reload

deb: all
	rm -rf dist/deb
	mkdir -p dist/deb/package/{usr/bin,lib/systemd/user/}
	cp $(TARGETS) dist/deb/package/usr/bin
	cat misc/tts.service | PREFIX="/usr" envsubst > dist/deb/package/lib/systemd/user/tts.service
	cp -r misc/DEBIAN dist/deb/package/
	cat Dockerfile | VERSION=$(VERSION) envsubst > dist/deb/Dockerfile
	sh ./misc/build.sh $(VERSION)
