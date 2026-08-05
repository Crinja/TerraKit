.PHONY: fmt fmt-check lint test doc doc-check package-check check ci release-check demo clean c-build c-build-release c-header c-header-check c-symbol-check c-test c-example c-package c-package-release c-archive c-archive-release ensure-cbindgen

CBINDGEN_VERSION ?= 0.29.0
CBINDGEN ?= cbindgen
CARGO_BUILD_FLAGS ?=
CARGO_TARGET_DIR ?= target
TARGET_TRIPLE ?= native
TARGET_DIR ?= $(CARGO_TARGET_DIR)/debug
XTASK = cargo run --quiet -p terrakit-xtask --
TERRAKIT_DLL = $(TARGET_DIR)/terrakit.dll
TERRAKIT_IMPORT_WIN = $(TARGET_DIR)/terrakit.dll.lib
TERRAKIT_SO = $(TARGET_DIR)/libterrakit.so
TERRAKIT_DYLIB = $(TARGET_DIR)/libterrakit.dylib
TERRAKIT_STATIC_WIN = $(TARGET_DIR)/terrakit.lib
TERRAKIT_STATIC_UNIX = $(TARGET_DIR)/libterrakit.a
C_EXAMPLE_BASENAME := build/c/generate_heightfield
REQUIRED_C_SYMBOLS := tk_get_abi_version tk_stage_registry_create_builtin tk_pipeline_assembler_finish tk_runtime_generate_2d tk_generation_result_get_mesh
ENSURE_CBINDGEN = $(XTASK) ensure-cbindgen --version "$(CBINDGEN_VERSION)" --binary "$(CBINDGEN)"
RUN_C_EXAMPLE = $(XTASK) run-with-library-path --target-dir "$(TARGET_DIR)" -- "$(C_EXAMPLE_EXE)"
DOC_CHECK = $(XTASK) doc-check
MKDIR_P = $(XTASK) mkdir
PACKAGE_C_API = $(XTASK) package-c-api --target-dir "$(TARGET_DIR)" --target-triple "$(TARGET_TRIPLE)"
ARCHIVE_C_API = $(XTASK) archive-c-package --target-triple "$(TARGET_TRIPLE)"

ifeq ($(OS),Windows_NT)
C_EXAMPLE_EXE := $(C_EXAMPLE_BASENAME).exe
C_EXAMPLE_LIB := $(TERRAKIT_DLL)
else
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
C_EXAMPLE_LIB := $(TERRAKIT_DYLIB)
else
C_EXAMPLE_LIB := $(TERRAKIT_SO)
endif
C_EXAMPLE_EXE := $(C_EXAMPLE_BASENAME)
endif

RM_RF ?= rm -rf

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features

doc:
	cargo doc --workspace --all-features --no-deps

doc-check:
	$(DOC_CHECK)

package-check:
	$(XTASK) check-release-manifests

check: fmt-check lint test doc-check

ci: check c-test package-check

release-check: ci

demo:
	cargo run -p terrakit-console

c-build:
	cargo build -p terrakit-c-api --all-features $(CARGO_BUILD_FLAGS)

c-build-release: CARGO_BUILD_FLAGS=--release
c-build-release: TARGET_DIR=$(CARGO_TARGET_DIR)/release
c-build-release: c-build

ensure-cbindgen:
	@$(ENSURE_CBINDGEN)

c-header: ensure-cbindgen
	$(CBINDGEN) --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h

c-header-check: ensure-cbindgen
	$(CBINDGEN) --verify --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h

c-symbol-check: c-build
	$(XTASK) c-symbol-check --target-dir "$(TARGET_DIR)" --symbols "$(REQUIRED_C_SYMBOLS)"

c-example: c-build
	$(MKDIR_P) build/c
	gcc -std=c11 -Wall -Wextra -Ibindings/c/include bindings/c/examples/generate_heightfield.c $(C_EXAMPLE_LIB) -o $(C_EXAMPLE_EXE)
	$(RUN_C_EXAMPLE)
	g++ -std=c++17 -Ibindings/c/include -c bindings/c/examples/header_smoke.cpp -o build/c/header_cpp.o

c-test: c-build c-header-check c-symbol-check
	cargo test -p terrakit-c-api --all-features
	$(MAKE) c-example

c-package: c-build c-header-check
	$(PACKAGE_C_API)

c-package-release: CARGO_BUILD_FLAGS=--release
c-package-release: TARGET_DIR=$(CARGO_TARGET_DIR)/release
c-package-release: c-package

c-archive: c-package
	$(ARCHIVE_C_API)

c-archive-release: CARGO_BUILD_FLAGS=--release
c-archive-release: TARGET_DIR=$(CARGO_TARGET_DIR)/release
c-archive-release: c-archive

clean:
	cargo clean
	$(RM_RF) build dist
