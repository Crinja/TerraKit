# TerraKit

TerraKit is a Rust workspace for deterministic terrain generation primitives,
stage pipelines, first-party generation stages, and a synchronous headless
runtime.

## Workspace Structure

```text
engine/
  terrakit-c-api/      TerraKit's embedding ABI for C-compatible hosts.
  terrakit-core/       Core terrain data, grids, regions, seeds, and math.
  terrakit-pipeline/   Stage schemas, resource storage, assembly, and execution.
  terrakit-runtime/    Synchronous headless generation runtime and results.

generation/
  terrakit-algorithms/ Reusable deterministic algorithms.
  terrakit-builtins/   First-party stage definitions and stage implementations.

interfaces/
  terrakit-console/    Command-line mesh demonstration and OBJ export.
```

## Compiling for Other Projects

Most downstream projects should start from a TerraKit release package. Releases
include the committed C ABI header, `terrakit.h`, alongside the platform library
artifacts such as `terrakit.dll`, `libterrakit.so`, or `libterrakit.dylib`.
Add the release `include` directory to your compiler's include path, then link
or load the matching library for your target platform.

To build those artifacts from source, run the C package target from the
repository root:

```sh
make c-package
```

The package is written to `dist/c/<target>/` with `include/terrakit.h`, the
generated libraries under `lib/`, and a small C example under `examples/`.
For a release-optimized package:

```sh
make c-package-release
```

Use `mingw32-make` instead of `make` on Windows environments where plain
`make` is not installed. See `bindings/c/README.md` for C ABI linking,
ownership, and error-handling details.

## CI

Run the same local gate used by Linux CI:

```sh
make ci
```

That validates formatting, clippy, tests, docs with warnings as errors, C ABI
headers, exported symbols, native C/C++ examples, and release manifest
readiness.

## Releases

TODO

## Coordinate Convention

TerraKit uses a right-handed coordinate system. X and Z form the standard
horizontal plane, and Y is the standard upward axis. Two-dimensional region
coordinates advance world X and world Z. Adjacent point-sampled regions share
boundary positions when generated independently under the same layout.
