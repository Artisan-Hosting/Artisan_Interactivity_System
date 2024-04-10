# Define variables
DEST_DIR = /usr/local/bin

# Define targets
.PHONY: all clean test build install uninstall post-clean

# Default target
all: stop test build install

# Clean target
clean:
	@cargo clean

# Test target
test:
	@cargo test --all-features

# Build target
build:
	@cargo build --release

# Stop services 
stop:
	@systemctl stop ais

# Install target
install:
	@mkdir -p $(DEST_DIR)
	@cp -v target/release/ais_client /usr/local/bin/ais # ais_client is ais
	@cp -v target/release/ais_credentials /usr/local/bin/ais_credentials
	@cp -v target/release/ais_clone /usr/local/bin/ais_clone
	@cp -v target/release/ais_welcome /usr/local/bin/ais_welcome
	@cp -v target/release/ais_first_run /usr/local/bin/ais_first_run
	@systemctl start ais
# Restart the service

post-clean:
	@cargo clean

# Remove installed executables
uninstall:
	@rm -f $(DEST_DIR)/ais
	@rm -f $(DEST_DIR)/ais_credentials
	@rm -f $(DEST_DIR)/ais_clone
	@rm -f $(DEST_DIR)/ais_welcome
	@rm -f $(DEST_DIR)/ais_first_run
