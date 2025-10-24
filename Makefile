CARGO := cargo

APP_NAME := senarai
RELEASE_DIR := target/release
RELEASE_BINARY := $(RELEASE_DIR)/$(APP_NAME)

MAGENTA := $(shell tput -T xterm setaf 5)
BLUE    := $(shell tput -T xterm setaf 4)
RESET   := $(shell tput -T xterm sgr0)
BOLD    := $(shell tput -T xterm bold)

define log_info
	@echo "$(BOLD)$(BLUE)    ✔ $1$(RESET)"
endef

define log_important
	@echo "$(BOLD)$(MAGENTA)===> $1$(RESET)"
endef

.PHONY: all setup build run clean

all: setup build

setup:
	$(call log_important,Setting up dependencies)
	@$(CARGO) fetch --verbose
	$(call log_info,Dependencies are up to date)

build:
	$(call log_important,Building application for release)
	@$(CARGO) build --release --verbose
	$(call log_info,Build complete: $(RELEASE_BINARY))
	@cp config.yaml $(RELEASE_DIR)/config.yaml
	$(call log_info,Copied config.yaml to $(RELEASE_DIR)/config.yaml)

run:
	@if [ ! -f $(RELEASE_BINARY) ]; then \
		make build; \
	fi
	$(call log_important,Running application)
	@$(RELEASE_BINARY)

clean:
	$(call log_important,Cleaning up build artifacts)
	@$(CARGO) clean --verbose
	$(call log_info,Cleanup complete)
