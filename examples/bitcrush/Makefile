PATCH_NAME ?= patch
TIMESTAMP := $(shell date +"%Y%m%d_%H%M%S")
BUILD_DIR := build
TARGET_DIR := target/thumbv7em-none-eabihf/release

all: $(BUILD_DIR)/$(PATCH_NAME)_$(TIMESTAMP).endl

$(TARGET_DIR)/bitcrush: FORCE
	cargo build --release

$(BUILD_DIR)/$(PATCH_NAME)_$(TIMESTAMP).endl: $(TARGET_DIR)/bitcrush
	@mkdir -p $(BUILD_DIR)
	arm-none-eabi-objcopy -O binary $< $@
	@echo "Built: $@"

clean:
	cargo clean
	rm -rf $(BUILD_DIR)

FORCE:

.PHONY: all clean FORCE
