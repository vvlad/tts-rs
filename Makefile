
TARGETS := target/release/say \
  target/release/tts-service \
  target/release/speak-selection

all: $(TARGETS) 

target/release/say: src/bin/say.rs
	cargo build --release --bin say 

target/release/tts-service: 
	cargo build --release --bin tts-service 

target/release/speak-selection: src/bin/speak-selection.rs
	cargo build --release --bin speak-selection
clean:
	rm -f $(TARGETS)

install: all
	sudo install $(TARGETS) /usr/local/bin/
	install misc/tts.service ~/.config/systemd/user
