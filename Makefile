
BUILD_DIR := target/release/examples/

all: say tts-service 

say:
	cargo build --release --example say 
	cp $(BUILD_DIR)/say .
tts-service: 
	cargo build --release --example server 
	cp $(BUILD_DIR)/server tts-service

clean:
	rm -f say tts-service

install: all
	sudo install say /usr/local/bin/
	sudo install tts-service /usr/local/bin
	install misc/tts.service ~/.config/systemd/
