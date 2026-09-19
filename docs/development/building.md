# Building TerraKit

This document covers the common local build workflows for TerraKit and its current interfaces.

TerraKit uses a monorepo, but the core and interfaces are separate build units.


## Requirements

### Core

Required:

- Git;
- Rust 1.87 or newer;
- Cargo;
- Make.

The repository includes `rust-toolchain.toml`, which defines the expected Rust toolchain for development.

### Console

The Console is a standalone Cargo workspace and requires the Rust toolchain.

### Godot

The Godot interface additionally requires:

- Python;
- SCons;
- a supported C++ toolchain;
- Godot development/runtime components required by the GDExtension build;
- repository submodules.

On Windows, a Visual Studio/MSVC toolchain is expected for the current native build.


## Clone the Repository

Clone with submodules:

```bash
git clone --recurse-submodules https://github.com/Crinja/TerraKit.git
cd TerraKit
```

If the repository was cloned without submodules:

```bash
git submodule update --init --recursive
```


## Dev Container

TerraKit includes a development container configuration.

If using VS Code with Dev Containers:

1. open the repository;
2. choose **Reopen in Container**;
3. allow the container setup to complete;
4. verify Rust and repository dependencies are available.

The dev container is intended to provide a repeatable Linux development environment.

Platform-specific release builds still run on their native CI runners where required.


## Build the Core Workspace

From the repository root:

```bash
cargo build --workspace
```

For an optimized build:

```bash
cargo build --workspace --release
```

The root Cargo workspace contains the TerraKit core, generation crates, and repository tooling.

Interfaces are intentionally not root workspace members.


## Build the C ABI

Debug:

```bash
cargo build -p terrakit-c-api
```

Release:

```bash
cargo build -p terrakit-c-api --release
```

Typical platform output names are determined by the operating system:

```text
Linux:   libterrakit.so
macOS:   libterrakit.dylib
Windows: terrakit.dll
```

Exact output paths follow Cargo's target directory and build profile.


## Generate or Validate ABI Artifacts

Repository tooling may generate or validate C ABI artifacts as part of CI or release workflows.

Do not hand-edit generated headers.

When working on ABI changes, run the repository's release or validation tooling in addition to a normal Cargo build.


## Run the Core Development Checks

The canonical repository-level check is:

```bash
make ci
```

This should be run before submitting core changes.

See [Testing](testing.md) for the individual commands.


## Build the Console

The Console is a standalone Cargo workspace.

Debug:

```bash
cargo build \
  --manifest-path interfaces/terrakit-console/Cargo.toml
```

Release:

```bash
cargo build \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --release
```

Run the current mesh demonstration:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  -- mesh-demo
```

Optimized:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --release \
  -- mesh-demo
```

Because the Console is independent from the root Cargo workspace, run Cargo commands using its manifest path.


## Build the Godot Interface

The Godot interface uses SCons and GDExtension.

First build the TerraKit C ABI for the profile the Godot extension will use.

For a debug build:

```bash
cargo build -p terrakit-c-api --all-features
```

Then:

```bash
cd interfaces/terrakit-godot
```

On Linux:

```bash
scons \
  platform=linux \
  target=template_debug \
  arch=x86_64 \
  terrakit_profile=debug
```

On Windows:

```powershell
scons `
  platform=windows `
  target=template_debug `
  arch=x86_64 `
  terrakit_profile=debug
```

For release builds, use the release Cargo profile and the corresponding Godot target:

```bash
cargo build -p terrakit-c-api --release --all-features
```

then:

```bash
scons \
  platform=<platform> \
  target=template_release \
  arch=x86_64 \
  terrakit_profile=release
```

The release workflow is the authoritative reference for currently supported release targets.


## Godot Submodules

The Godot interface may rely on `godot-cpp` or other repository submodules.

If SCons reports missing sources or headers, verify submodules:

```bash
git submodule status
git submodule update --init --recursive
```


## Build Profiles

Use debug builds during normal development.

Use release builds when:

- profiling optimized behavior;
- validating release packaging;
- testing release-only problems;
- preparing an actual component release.

CI may intentionally use debug builds for compatibility checks to keep pull-request validation fast.


## Clean Builds

Cargo:

```bash
cargo clean
```

For the Console's standalone workspace, remove or clean its configured target directory as appropriate.

For SCons:

```bash
scons -c
```

from the interface directory removes SCons build outputs known to that configuration.

A clean rebuild is useful when diagnosing:

- changed native headers;
- stale GDExtension outputs;
- profile mismatches;
- generated binding changes.


## Locked Dependencies

Where a committed `Cargo.lock` exists for a build unit, CI may use `--locked`.

If local dependency resolution differs from CI, update dependencies deliberately and commit the corresponding lockfile change where that workspace owns one.

Do not delete lockfiles simply to resolve an unrelated build problem.


## Platform Notes

### Linux

The dev container and most repository-level validation use Linux.

Native release artifacts are typically produced with the GNU Linux target.

### Windows

Windows native builds use MSVC in CI.

Ensure the Visual Studio C++ toolchain is installed when building locally.

### macOS

Core release builds may target Apple platforms independently from the Godot interface's currently configured target matrix.


## Troubleshooting

### Cargo reports that a package believes it is in another workspace

Interfaces such as the Console are separate workspaces.

Ensure the interface manifest contains the intended standalone workspace configuration and invoke Cargo with:

```bash
--manifest-path interfaces/<interface>/Cargo.toml
```

### Godot reports that a native class is missing

Confirm:

- the GDExtension native library was built;
- the `.gdextension` file points to the correct platform library;
- the library can be loaded by the operating system;
- the class was registered in the extension initialization code.


### CI builds but local native loading fails

Check architecture and build profile.

A debug interface must not accidentally point at a missing or incompatible release library, and vice versa.


## Related Documentation

- [Testing](testing.md)
- [Release Process](releases.md)
- [Architecture Overview](../architecture/overview.md)
- [Godot Interface](../../interfaces/terrakit-godot/README.md)
- [Console Interface](../../interfaces/terrakit-console/README.md)
