TARGET = alacritty

UNAME_S := $(shell uname -s 2>/dev/null || echo Unknown)
ifeq ($(OS),Windows_NT)
	HOST_OS = Windows
else ifeq ($(UNAME_S),Darwin)
	HOST_OS = macOS
else ifeq ($(UNAME_S),Linux)
	HOST_OS = Linux
else ifneq (,$(findstring MINGW,$(UNAME_S)))
	HOST_OS = Windows
else ifneq (,$(findstring MSYS,$(UNAME_S)))
	HOST_OS = Windows
else
	HOST_OS = Unknown
endif

ASSETS_DIR = extra
RELEASE_DIR = target/release
MANPAGE = $(ASSETS_DIR)/man/alacritty.1.scd
MANPAGE-MSG = $(ASSETS_DIR)/man/alacritty-msg.1.scd
MANPAGE-CONFIG = $(ASSETS_DIR)/man/alacritty.5.scd
MANPAGE-CONFIG-BINDINGS = $(ASSETS_DIR)/man/alacritty-bindings.5.scd
TERMINFO = $(ASSETS_DIR)/alacritty.info
COMPLETIONS_DIR = $(ASSETS_DIR)/completions
COMPLETIONS = $(COMPLETIONS_DIR)/_alacritty \
	$(COMPLETIONS_DIR)/alacritty.bash \
	$(COMPLETIONS_DIR)/alacritty.fish

APP_NAME = Alacritty.app
APP_TEMPLATE = $(ASSETS_DIR)/osx/$(APP_NAME)
APP_DIR = $(RELEASE_DIR)/osx
APP_BINARY = $(RELEASE_DIR)/$(TARGET)
APP_BINARY_DIR = $(APP_DIR)/$(APP_NAME)/Contents/MacOS
APP_EXTRAS_DIR = $(APP_DIR)/$(APP_NAME)/Contents/Resources
APP_COMPLETIONS_DIR = $(APP_EXTRAS_DIR)/completions

DMG_NAME = Alacritty.dmg
DMG_DIR = $(RELEASE_DIR)/osx

ifeq ($(HOST_OS),Windows)
	HOST_BINARY_RELEASE = $(RELEASE_DIR)/$(TARGET).exe
	HOST_BINARY_DEBUG = target/debug/$(TARGET).exe
	HOST_SOCKET = ./alacritty-agent.sock
	AGENT_DAEMON_LOG = ./alacritty-agent-daemon.log
else
	HOST_BINARY_RELEASE = $(RELEASE_DIR)/$(TARGET)
	HOST_BINARY_DEBUG = target/debug/$(TARGET)
	HOST_SOCKET = /tmp/alacritty-agent.sock
	AGENT_DAEMON_LOG = /tmp/alacritty-agent-daemon.log
endif

SOCKET ?= $(HOST_SOCKET)
AGENT_DAEMON_PID ?= .alacritty-agent-daemon.pid
WINDOWS_MSVC_TARGET ?= x86_64-pc-windows-msvc
WINDOWS_GNU_TARGET ?= x86_64-pc-windows-gnu
WINDOWS_BINARY_MSVC = target/$(WINDOWS_MSVC_TARGET)/release/$(TARGET).exe
WINDOWS_BINARY_GNU = target/$(WINDOWS_GNU_TARGET)/release/$(TARGET).exe

vpath $(TARGET) $(RELEASE_DIR)
vpath $(APP_NAME) $(APP_DIR)
vpath $(DMG_NAME) $(APP_DIR)

all: help

help: ## Print this help message
	@grep -E '^[a-zA-Z._-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

host-info: ## Print the detected host OS and binary paths
	@echo "HOST_OS=$(HOST_OS)"
	@echo "HOST_BINARY_DEBUG=$(HOST_BINARY_DEBUG)"
	@echo "HOST_BINARY_RELEASE=$(HOST_BINARY_RELEASE)"
	@echo "SOCKET=$(SOCKET)"
	@echo "AGENT_DAEMON_LOG=$(AGENT_DAEMON_LOG)"
	@echo "AGENT_DAEMON_PID=$(AGENT_DAEMON_PID)"

windows-targets-install: ## Install Rust std targets for Windows checks/builds
	@rustup target add $(WINDOWS_MSVC_TARGET) $(WINDOWS_GNU_TARGET)

check-windows-msvc: ## Type-check alacritty for Windows MSVC target
	@rustup target list --installed | grep -qx "$(WINDOWS_MSVC_TARGET)" || (echo "Missing target: $(WINDOWS_MSVC_TARGET). Run: make windows-targets-install"; exit 1)
	@cargo check -p alacritty --target $(WINDOWS_MSVC_TARGET)

check-windows-gnu: ## Type-check alacritty for Windows GNU target
	@rustup target list --installed | grep -qx "$(WINDOWS_GNU_TARGET)" || (echo "Missing target: $(WINDOWS_GNU_TARGET). Run: make windows-targets-install"; exit 1)
	@cargo check -p alacritty --target $(WINDOWS_GNU_TARGET)

check-windows: check-windows-msvc check-windows-gnu ## Run both Windows target checks

build-windows-host: ## Build release binary on native Windows host
ifeq ($(HOST_OS),Windows)
	@cargo build -p alacritty --release
	@echo "Built $(HOST_BINARY_RELEASE)"
else
	@echo "build-windows-host requires running make on a Windows machine."
	@echo "Current HOST_OS=$(HOST_OS). Use 'make check-windows' here, and CI/release for Windows artifacts."
	@exit 1
endif

build-host: ## Build the best host-native release artifact for this OS
ifeq ($(HOST_OS),macOS)
	@$(MAKE) binary
else
	@cargo build -p alacritty --release
endif

build-host-debug: ## Build a host-native debug binary
	@cargo build -p alacritty

run-host: build-host ## Run host-native release build
	@$(HOST_BINARY_RELEASE)

run-host-debug: build-host-debug ## Run host-native debug build
	@$(HOST_BINARY_DEBUG)

agent-daemon: build-host ## Start daemon mode for IPC-based feature testing (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-daemon target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) --daemon --socket "$(SOCKET)"
endif

agent-daemon-bg: build-host ## Start daemon in background and write PID file (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-daemon-bg target is Unix-only."
else
	@nohup $(HOST_BINARY_RELEASE) --daemon --socket "$(SOCKET)" >"$(AGENT_DAEMON_LOG)" 2>&1 & echo $$! > "$(AGENT_DAEMON_PID)"
	@echo "Daemon PID $$(cat "$(AGENT_DAEMON_PID)")"
	@echo "Daemon log: $(AGENT_DAEMON_LOG)"
	@for i in 1 2 3 4 5; do \
		if [ -S "$(SOCKET)" ]; then \
			echo "Socket is ready: $(SOCKET)"; \
			exit 0; \
		fi; \
		sleep 1; \
	done; \
	echo "Socket not found at $(SOCKET). Check $(AGENT_DAEMON_LOG)."; \
	exit 1
endif

agent-daemon-stop: ## Stop background daemon started by agent-daemon-bg (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-daemon-stop target is Unix-only."
else
	@if [ -f "$(AGENT_DAEMON_PID)" ]; then \
		kill "$$(cat "$(AGENT_DAEMON_PID)")" >/dev/null 2>&1 || true; \
		rm -f "$(AGENT_DAEMON_PID)"; \
		echo "Daemon stop requested."; \
	else \
		echo "No PID file found at $(AGENT_DAEMON_PID)"; \
	fi
endif

agent-new-window: build-host ## Open a new terminal window through IPC (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-new-window target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" create-window
endif

agent-test-identify: build-host ## Run v2 identify API call (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-test-identify target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method identify
endif

agent-test-tree: build-host ## Run v2 system.tree API call (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-test-tree target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method system.tree
endif

agent-test-notify: build-host ## Create/list/read a notification through v2 API (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-test-notify target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method notification.create --params '{"workspace_id":"workspace:1","title":"Agent done","body":"Review output"}'
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method notification.list
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method notification.mark_read --params '{"id":0}'
endif

agent-test-split: build-host ## Run split state API call and show updated tree (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-test-split target is Unix-only."
else
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method surface.split --params '{"workspace_id":"workspace:1","target_surface_id":"surface:1","direction":"right"}'
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method system.tree
endif

agent-test-webview: build-host ## Exercise webview state APIs (Unix)
ifeq ($(HOST_OS),Windows)
	@echo "agent-test-webview target is Unix-only."
else
	@echo "Open:"
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method webview.open --params '{"workspace_id":"workspace:1","url":"https://alacritty.org"}'
	@echo "List:"
	@$(HOST_BINARY_RELEASE) msg --socket "$(SOCKET)" v2 --method webview.list
	@echo "Use the returned surface id to run navigate/close manually:"
	@echo "$(HOST_BINARY_RELEASE) msg --socket \"$(SOCKET)\" v2 --method webview.navigate --params '{\"id\":\"surface:10000000\",\"url\":\"https://example.com\"}'"
	@echo "$(HOST_BINARY_RELEASE) msg --socket \"$(SOCKET)\" v2 --method webview.close --params '{\"id\":\"surface:10000000\"}'"
endif

binary: $(TARGET)-native ## Build a release binary
binary-universal: $(TARGET)-universal ## Build a universal release binary
$(TARGET)-native:
ifeq ($(HOST_OS),macOS)
	MACOSX_DEPLOYMENT_TARGET="10.12" cargo build --release
else
	cargo build -p alacritty --release
endif
$(TARGET)-universal:
	MACOSX_DEPLOYMENT_TARGET="10.12" cargo build --release --target=x86_64-apple-darwin
	MACOSX_DEPLOYMENT_TARGET="10.12" cargo build --release --target=aarch64-apple-darwin
	@lipo target/{x86_64,aarch64}-apple-darwin/release/$(TARGET) -create -output $(APP_BINARY)

app: $(APP_NAME)-native ## Create an Alacritty.app
app-universal: $(APP_NAME)-universal ## Create a universal Alacritty.app
$(APP_NAME)-%: $(TARGET)-%
	@mkdir -p $(APP_BINARY_DIR)
	@mkdir -p $(APP_EXTRAS_DIR)
	@mkdir -p $(APP_COMPLETIONS_DIR)
	@scdoc < $(MANPAGE) | gzip -c > $(APP_EXTRAS_DIR)/alacritty.1.gz
	@scdoc < $(MANPAGE-MSG) | gzip -c > $(APP_EXTRAS_DIR)/alacritty-msg.1.gz
	@scdoc < $(MANPAGE-CONFIG) | gzip -c > $(APP_EXTRAS_DIR)/alacritty.5.gz
	@scdoc < $(MANPAGE-CONFIG-BINDINGS) | gzip -c > $(APP_EXTRAS_DIR)/alacritty-bindings.5.gz
	@tic -xe alacritty,alacritty-direct -o $(APP_EXTRAS_DIR) $(TERMINFO)
	@cp -fRp $(APP_TEMPLATE) $(APP_DIR)
	@cp -fp $(APP_BINARY) $(APP_BINARY_DIR)
	@cp -fp $(COMPLETIONS) $(APP_COMPLETIONS_DIR)
	@touch -r "$(APP_BINARY)" "$(APP_DIR)/$(APP_NAME)"
	@codesign --remove-signature "$(APP_DIR)/$(APP_NAME)"
	@codesign --force --deep --sign - "$(APP_DIR)/$(APP_NAME)"
	@echo "Created '$(APP_NAME)' in '$(APP_DIR)'"

dmg: $(DMG_NAME)-native ## Create an Alacritty.dmg
dmg-universal: $(DMG_NAME)-universal ## Create a universal Alacritty.dmg
$(DMG_NAME)-%: $(APP_NAME)-%
	@echo "Packing disk image..."
	@ln -sf /Applications $(DMG_DIR)/Applications
	@hdiutil create $(DMG_DIR)/$(DMG_NAME) \
		-volname "Alacritty" \
		-fs HFS+ \
		-srcfolder $(APP_DIR) \
		-ov -format UDZO
	@echo "Packed '$(APP_NAME)' in '$(APP_DIR)'"

install: $(INSTALL)-native ## Mount disk image
install-universal: $(INSTALL)-native ## Mount universal disk image
$(INSTALL)-%: $(DMG_NAME)-%
	@open $(DMG_DIR)/$(DMG_NAME)

.PHONY: agent-daemon agent-daemon-bg agent-daemon-stop agent-new-window agent-test-identify \
	agent-test-notify agent-test-split agent-test-tree agent-test-webview app binary \
	build-host build-host-debug build-windows-host check-windows check-windows-gnu \
	check-windows-msvc clean dmg host-info install run-host run-host-debug \
	windows-targets-install $(TARGET) $(TARGET)-universal

clean: ## Remove all build artifacts
	@cargo clean
