# Testing

TerraKit uses layered validation.

Core logic should be testable independently from engine integrations, while interfaces add their own compatibility and platform checks.


## Testing Goals

Tests should catch failures at the lowest appropriate layer.

```text
unit tests
    │
    ▼
component tests
    │
    ▼
ABI / integration tests
    │
    ▼
interface compatibility
    │
    ▼
platform CI
```

A generation algorithm should not require Godot to test its basic correctness.

Likewise, a Godot loading issue belongs in interface validation rather than core unit tests.


## Core Validation

Run the canonical root check:

```bash
make ci
```

The underlying Rust checks typically include:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Core changes should pass all of these before merge.


## Formatting

Run:

```bash
cargo fmt --all
```

Check without modifying files:

```bash
cargo fmt --all --check
```

Formatting failures should be fixed locally rather than ignored in CI.


## Clippy

Run:

```bash
cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  -- -D warnings
```

Warnings are treated as errors in CI.

If a lint must be suppressed, use the narrowest practical suppression and document why.


## Unit Tests

Run all root workspace tests:

```bash
cargo test --workspace --all-features
```

For a specific package:

```bash
cargo test -p <package>
```

For a specific test:

```bash
cargo test <test-name>
```


## Generation Tests

Generation code should be tested at the stage or algorithm level where possible.

Useful tests include:

- deterministic output;
- parameter boundaries;
- invalid inputs;
- edge-size inputs;
- empty inputs;
- repeat execution;
- known geometric or numeric invariants.

Avoid tests that assert huge raw outputs when a smaller invariant can prove correctness more robustly.


## Pipeline Tests

Pipeline tests should validate behavior across stage boundaries.

Important cases include:

- valid connections;
- invalid connections;
- missing required inputs;
- parameter propagation;
- execution order;
- stage failure propagation;
- graph validation;
- deterministic repeated execution.


## ABI Tests

ABI tests should focus on the public interoperability contract.

They should verify:

- required symbols are exported;
- supported ABI version information is correct;
- null or invalid inputs fail safely;
- handles can be created and destroyed correctly;
- data can cross the boundary without lifetime violations;
- errors are surfaced predictably.

Where possible, tests should exercise the ABI as an external caller would rather than directly calling internal Rust implementation functions.


## Console Tests

The Console is a standalone workspace.

Formatting:

```bash
cargo fmt \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --check
```

Clippy:

```bash
cargo clippy \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --all-targets \
  --all-features \
  -- -D warnings
```

Tests:

```bash
cargo test \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --all-features
```

Integration/demo run:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --release \
  -- mesh-demo
```

The Console is useful as an engine-independent integration consumer.

A successful Console test provides evidence that TerraKit behavior is not accidentally dependent on an engine interface.


## Godot Tests

Godot CI primarily validates that the interface still builds against the current native TerraKit runtime.

Current pull-request compatibility validation should prefer fast debug builds.

Typical checks include:

```bash
cargo build -p terrakit-c-api --all-features
```

then from `interfaces/terrakit-godot`:

```bash
scons \
  platform=<platform> \
  target=template_debug \
  arch=x86_64 \
  terrakit_profile=debug
```

Release builds belong in the release workflow unless a release-specific issue is being tested.


## Godot Editor Smoke Test

For local development, a useful smoke test is:

1. install or copy the addon into a Godot project;
2. ensure the GDExtension native library is present;
3. enable the plugin;
4. confirm the native bridge class is registered;
5. verify the ABI version can be read;
6. check the editor output for loading errors.

A headless Godot editor invocation can also expose parser and loading failures:

```bash
godot --headless --editor --path <project-path> --quit
```

Use the console executable on Windows if needed to ensure error output is visible.


## Platform Testing

A native project can pass on one operating system and fail on another because of:

- library naming;
- dynamic loader behavior;
- path separators;
- calling conventions;
- compiler differences;
- linker behavior;
- architecture differences.

For this reason, release and interface CI should validate supported platforms independently.


## Regression Tests

Every bug fix should include a regression test where the failure can be reproduced reasonably in automated code.

A regression test should:

1. fail before the fix;
2. pass after the fix;
3. target the behavior that was actually broken.

Avoid regression tests that depend on incidental implementation details.


## Deterministic Test Data

Prefer fixed seeds and small deterministic fixtures.

Randomized property testing can be useful, but failures should report enough information to reproduce the exact case.

If fuzzing or randomized tests are added later, record the failing seed.


## Performance Tests

Performance benchmarks should be separate from correctness tests.

Correctness tests should not fail because a shared CI runner happened to be slower.

Benchmarks are useful for measuring:

- stage execution;
- allocation behavior;
- pipeline overhead;
- serialization;
- FFI cost;
- large world-region generation.

Performance changes should be compared against a meaningful baseline.


## Test Ownership

Tests should live with the component that owns the behavior.

Examples:

```text
generation algorithm
    → generation tests

pipeline execution
    → core tests

C ABI contract
    → ABI tests

Console parsing
    → Console tests

Godot library loading
    → Godot interface tests
```

This mirrors the repository ownership model.


## CI Gates

The repository uses component-level gates.

Typical gates include:

- TerraKit Gate;
- Console Gate;
- Godot Gate;
- Maintenance Gate;
- PR Routing Gate;
- Main Scope Gate.

A required gate should summarize the result of the jobs belonging to that component so branch protection can depend on a stable check name.


## Before Opening a PR

Run the checks relevant to your component.

At minimum:

### TerraKit

```bash
make ci
```

### Console

```bash
cargo fmt --manifest-path interfaces/terrakit-console/Cargo.toml --check
cargo clippy --manifest-path interfaces/terrakit-console/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path interfaces/terrakit-console/Cargo.toml --all-features
```

### Godot

Build the native ABI and the Godot extension for your local platform.

### Maintenance

Validate workflow and configuration changes locally where practical.


## Related Documentation

- [Building TerraKit](building.md)
- [Git Workflow](git-workflow.md)
- [Release Process](releases.md)
- [Stage Development](../architecture/stages.md)
