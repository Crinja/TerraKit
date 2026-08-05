.PHONY: fmt fmt-check lint test doc doc-check package-check check ci release-check demo clean c-build c-build-release c-header c-header-check c-symbol-check c-test c-example c-package c-package-release c-archive c-archive-release ensure-cbindgen

CBINDGEN_VERSION ?= 0.29.0
CBINDGEN ?= cbindgen
CARGO_BUILD_FLAGS ?=
TARGET_TRIPLE ?= native
TARGET_DIR ?= target/debug
TERRAKIT_DLL = $(TARGET_DIR)/terrakit.dll
TERRAKIT_IMPORT_WIN = $(TARGET_DIR)/terrakit.dll.lib
TERRAKIT_SO = $(TARGET_DIR)/libterrakit.so
TERRAKIT_DYLIB = $(TARGET_DIR)/libterrakit.dylib
TERRAKIT_STATIC_WIN = $(TARGET_DIR)/terrakit.lib
TERRAKIT_STATIC_UNIX = $(TARGET_DIR)/libterrakit.a
C_EXAMPLE_BASENAME := build/c/generate_heightfield
REQUIRED_C_SYMBOLS := tk_get_abi_version tk_stage_registry_create_builtin tk_pipeline_assembler_finish tk_runtime_generate_2d tk_generation_result_get_mesh
ENSURE_CBINDGEN = python -c "import shutil, subprocess; subprocess.check_call(['cargo', 'install', 'cbindgen', '--version', '$(CBINDGEN_VERSION)', '--locked']) if shutil.which('$(CBINDGEN)') is None else None"
RUN_C_EXAMPLE = python -c "import os, subprocess; target=os.path.abspath('$(TARGET_DIR)'); os.environ['PATH']=target+os.pathsep+os.environ.get('PATH',''); os.environ['LD_LIBRARY_PATH']=target+os.pathsep+os.environ.get('LD_LIBRARY_PATH',''); os.environ['DYLD_LIBRARY_PATH']=target+os.pathsep+os.environ.get('DYLD_LIBRARY_PATH',''); subprocess.check_call([os.path.normpath('$(C_EXAMPLE_EXE)')])"
DOC_CHECK = python -c "import os, subprocess; os.environ['RUSTDOCFLAGS']='-D warnings'; subprocess.check_call(['cargo','doc','--workspace','--all-features','--no-deps'])"
PACKAGE_C_API = python scripts/ci/package_c_api.py --target-dir "$(TARGET_DIR)" --target-triple "$(TARGET_TRIPLE)"
ARCHIVE_C_API = python scripts/ci/archive_c_package.py --target-triple "$(TARGET_TRIPLE)"

ifeq ($(OS),Windows_NT)
C_EXAMPLE_EXE := $(C_EXAMPLE_BASENAME).exe
C_EXAMPLE_LIB := $(TERRAKIT_DLL)
SYMBOL_COMMAND := objdump -p "$(TERRAKIT_DLL)"
else
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
C_EXAMPLE_LIB := $(TERRAKIT_DYLIB)
SYMBOL_COMMAND := nm -gU "$(TERRAKIT_DYLIB)"
else
C_EXAMPLE_LIB := $(TERRAKIT_SO)
SYMBOL_COMMAND := nm -D --defined-only "$(TERRAKIT_SO)"
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
	python scripts/ci/check_release_manifests.py

check: fmt-check lint test doc-check

ci: check c-test package-check

release-check: ci

demo:
	cargo run -p terrakit-console

c-build:
	cargo build -p terrakit-c-api --all-features $(CARGO_BUILD_FLAGS)

c-build-release: CARGO_BUILD_FLAGS=--release
c-build-release: TARGET_DIR=target/release
c-build-release: c-build

ensure-cbindgen:
	@$(ENSURE_CBINDGEN)

c-header: ensure-cbindgen
	$(CBINDGEN) --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h

c-header-check: ensure-cbindgen
	$(CBINDGEN) --verify --config engine/terrakit-c-api/cbindgen.toml --crate terrakit-c-api --output bindings/c/include/terrakit.h

c-symbol-check: c-build
	python -c "import subprocess, sys; symbols='$(REQUIRED_C_SYMBOLS)'.split(); output=subprocess.check_output(r'''$(SYMBOL_COMMAND)''', shell=True, text=True, errors='ignore'); missing=[symbol for symbol in symbols if symbol not in output]; print('exported symbols OK: ' + ', '.join(symbols) if not missing else 'missing exported symbols: ' + ', '.join(missing)); sys.exit(1 if missing else 0)"

c-example: c-build
	python -c "import os; os.makedirs('build/c', exist_ok=True)"
	gcc -std=c11 -Wall -Wextra -Ibindings/c/include bindings/c/examples/generate_heightfield.c $(C_EXAMPLE_LIB) -o $(C_EXAMPLE_EXE)
	$(RUN_C_EXAMPLE)
	g++ -std=c++17 -Ibindings/c/include -c bindings/c/examples/header_smoke.cpp -o build/c/header_cpp.o

c-test: c-build c-header-check c-symbol-check
	cargo test -p terrakit-c-api --all-features
	$(MAKE) c-example

c-package: c-build c-header-check
	$(PACKAGE_C_API)

c-package-release: CARGO_BUILD_FLAGS=--release
c-package-release: TARGET_DIR=target/release
c-package-release: c-package

c-archive: c-package
	$(ARCHIVE_C_API)

c-archive-release: CARGO_BUILD_FLAGS=--release
c-archive-release: TARGET_DIR=target/release
c-archive-release: c-archive

clean:
	cargo clean
	$(RM_RF) build dist
