# Contributing to TerraKit

TerraKit is developed as a monorepo with independently owned components. Changes should be made through the branch that owns the component rather than directly against `main`.

This document describes the branch model, pull-request routing, ownership boundaries, validation requirements, and release flow.


## Development Model

TerraKit uses four persistent development branches:

| Branch | Responsibility |
| --- | --- |
| `tk` | TerraKit core, generation, and TerraKit-owned workflows |
| `console` | Console interface and Console-owned workflows |
| `godot` | Godot interface and Godot-owned workflows |
| `maintenance` | Shared repository infrastructure, documentation, and maintenance |

The `main` branch represents the canonical integrated repository state.

Development does not happen directly on `main`. Changes are developed on short-lived branches, merged into the appropriate persistent branch, and then promoted into `main` after validation.


## Branch Naming

Temporary branches use:

```text
<change-type>/<component>/<description>
```

Supported change types are:

- `feature` — new functionality;
- `fix` — bug fixes;
- `refactor` — internal restructuring without intended behavior changes;
- `chore` — maintenance or non-feature work within a component;
- `version` — marks a versioned component as ready for promotion and release.

Version branches are only used for versioned components:

- `tk`;
- `console`;
- `godot`.

`maintenance` is not versioned and does not use `version/maintenance/*`.

### Examples

```text
feature/tk/plugin-registry
fix/tk/pipeline-validation
refactor/tk/stage-discovery
chore/tk/update-ci

feature/console/json-output
fix/console/windows-paths
chore/console/update-ci
version/console/0.2.0

feature/godot/editor-graph
fix/godot/windows-build
refactor/godot/native-loader
chore/godot/ci-cache
version/godot/0.2.0

chore/maintenance/update-actions
fix/maintenance/release-workflow
```


## Pull Request Routing

Pull requests must target the persistent branch belonging to their component.

```text
feature/tk/*       ───────► tk
fix/tk/*           ───────► tk
refactor/tk/*      ───────► tk
chore/tk/*         ───────► tk
version/tk/*       ───────► tk

feature/console/*  ───────► console
fix/console/*      ───────► console
refactor/console/* ───────► console
chore/console/*    ───────► console
version/console/*  ───────► console

feature/godot/*    ───────► godot
fix/godot/*        ───────► godot
refactor/godot/*   ───────► godot
chore/godot/*      ───────► godot
version/godot/*    ───────► godot

feature/maintenance/*  ───► maintenance
fix/maintenance/*      ───► maintenance
refactor/maintenance/* ───► maintenance
chore/maintenance/*    ───► maintenance
```

Persistent branches promote into `main`:

```text
tk          ───────► main
console     ───────► main
godot       ───────► main
maintenance ───────► main
```

The reverse direction is used for synchronization:

```text
main ───────► tk
main ───────► console
main ───────► godot
main ───────► maintenance
```

Repository automation validates these routes.


## Component Ownership

Each persistent branch may only diverge from `main` within the paths it owns.

### TerraKit

The `tk` branch owns TerraKit implementation code and TerraKit-specific workflows.

Typical owned paths include:

```text
engine/**
generation/**
tools/xtask/**
.github/workflows/terrakit-*.yml
```

TerraKit changes must not directly modify interface-owned implementation files or shared maintenance infrastructure.

### Console

The `console` branch owns:

```text
interfaces/terrakit-console/**
.github/workflows/console-*.yml
```

### Godot

The `godot` branch owns:

```text
interfaces/terrakit-godot/**
.github/workflows/godot-*.yml
```

### Maintenance

The `maintenance` branch owns shared repository infrastructure and documentation.

Typical owned paths include:

```text
.github/workflows/_*.yml
.github/workflows/main-scope.yml
.github/workflows/pr-routing.yml
.devcontainer/**
tools/**
docs/**
README.md
CONTRIBUTING.md
.gitignore
.gitattributes
rust-toolchain.toml
```

Maintenance is not intended as a bypass for component ownership. Changes to TerraKit, Console, or Godot implementation code should still be made through the corresponding component branch.


## Creating a Development Branch

Start from the persistent branch that owns the change.

### TerraKit

```bash
git switch tk
git pull --ff-only origin tk
git switch -c feature/tk/example
```

### Console

```bash
git switch console
git pull --ff-only origin console
git switch -c feature/console/example
```

### Godot

```bash
git switch godot
git pull --ff-only origin godot
git switch -c feature/godot/example
```

### Maintenance

```bash
git switch maintenance
git pull --ff-only origin maintenance
git switch -c chore/maintenance/example
```

Push the branch with:

```bash
git push -u origin <branch-name>
```

Then open a pull request into the corresponding persistent branch.


## Commit Messages

Use concise commit messages that describe the change. Conventional-style prefixes are encouraged.

```text
feat(core): add stage registry
fix(abi): validate null handles
refactor(pipeline): simplify execution state
feat(console): add graph inspection command
fix(godot): correct Windows library path
chore(ci): update GitHub Actions
docs: document release process
```

Avoid mixing unrelated changes into the same commit or pull request.


## Validation

Pull requests are validated automatically. Depending on the target branch, checks may include:

- branch routing;
- component ownership;
- formatting;
- linting;
- tests;
- platform builds;
- native ABI builds;
- interface compatibility builds;
- repository infrastructure validation.

A pull request should not be merged while required checks are failing.


## TerraKit Core Checks

From the repository root:

```bash
make ci
```

You can also run the Rust workspace checks directly:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Build the native C ABI with:

```bash
cargo build -p terrakit-c-api
```

For an optimized build:

```bash
cargo build -p terrakit-c-api --release
```


## Console Checks

The Console is a standalone Cargo workspace.

```bash
cargo fmt \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --check

cargo clippy \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --all-targets \
  --all-features \
  -- -D warnings

cargo test \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --all-features
```

Run the integration demo with:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --release \
  -- mesh-demo
```


## Godot Checks

The Godot interface uses GDExtension and SCons.

Its CI validates the TerraKit native ABI and Godot extension on supported platforms.

A typical local debug build includes:

```bash
cargo build -p terrakit-c-api --all-features
```

followed from `interfaces/terrakit-godot/` by:

```bash
scons \
  platform=<platform> \
  target=template_debug \
  arch=x86_64 \
  terrakit_profile=debug
```

Platform-specific setup may be required. See the Godot interface documentation for the complete build process.


## Synchronization with `main`

Persistent branches are synchronized from `main` through pull requests:

```text
main ───────► tk
main ───────► console
main ───────► godot
main ───────► maintenance
```

These synchronization pull requests bring the canonical integrated repository state back into each component branch.

A component should contain the current `main` state before promotion.

Merge commits may cause a persistent branch to appear one or more commits ahead of `main` even when there is no meaningful file divergence. This is expected. Repository automation evaluates actual content divergence and component versions rather than relying only on GitHub's ahead/behind count.


## Versioning and Promotion

TerraKit components are versioned independently.

| Component | Version Source | Tag |
| --- | --- | --- |
| TerraKit | root `Cargo.toml` | `terrakit-vX.Y.Z` |
| Console | `interfaces/terrakit-console/Cargo.toml` | `terrakit-console-vX.Y.Z` |
| Godot | `interfaces/terrakit-godot/addon/addons/terrakit/plugin.cfg` | `terrakit-godot-vX.Y.Z` |

A component remains on its persistent branch while development is ongoing.

When the component is ready for release, create a version branch from that persistent branch. For example:

```bash
git switch godot
git pull --ff-only origin godot
git switch -c version/godot/0.2.0
```

Update the component version, commit it, and open a pull request back into `godot`.

After the version change lands:

```text
version/godot/0.2.0
        │
        ▼
      godot
        │
        │ version > main
        ▼
      main
        │
        ▼
terrakit-godot-v0.2.0
```

Promotion into `main` and release creation are handled by repository automation after required checks succeed.

Maintenance changes do not require a product version increase.


## Release Boundaries

A release should contain only changes owned by the component being promoted.

A Console version bump should not be used to carry Godot changes. A Godot version bump should not be used to carry TerraKit core changes. Shared repository infrastructure should be promoted through `maintenance`.

This preserves independently understandable release histories for each component.


## Cross-Component Changes

Some changes may require coordinated modifications across multiple components.

Prefer compatibility transitions where possible. For example, when evolving the ABI:

1. add the new capability while preserving the existing interface;
2. allow consumers to migrate independently;
3. update Console and engine integrations;
4. remove deprecated behavior in a later breaking release.

This avoids forcing otherwise independent components into a single atomic change.

When a change genuinely cannot be separated, coordinate it explicitly rather than bypassing branch ownership rules.


## Pull Request Guidelines

Before opening a pull request:

- ensure the branch follows the repository naming convention;
- target the correct persistent branch;
- keep the change within that component's ownership boundary;
- run relevant local checks;
- keep commits focused;
- update documentation when behavior or public interfaces change.

A useful pull request description should explain:

- what changed;
- why the change is needed;
- any public API or ABI impact;
- how the change was tested;
- whether follow-up work is required.


## Documentation Changes

Documentation under `docs/` is shared repository infrastructure and is normally changed through `maintenance`.

Documentation that belongs directly to an independently maintained interface may live alongside that interface and follow that interface's ownership rules.

When behavior changes, update documentation in the same development cycle rather than leaving known documentation drift.


## Breaking Changes

Breaking changes should be deliberate and clearly documented.

For changes to public APIs, ABI contracts, stage behavior, or interface compatibility:

- explain the compatibility impact in the pull request;
- update relevant documentation;
- provide migration guidance where practical;
- use an appropriate version increase;
- avoid unrelated breaking changes in the same release.

TerraKit is currently pre-1.0, so interfaces may still evolve, but changes should remain explicit and reviewable.


## Summary

The normal contribution path is:

```text
temporary branch
      │
      ▼
component branch
      │
      ▼
     main
```

For example:

```text
feature/godot/editor-graph
        │
        ▼
      godot
        │
        ▼
      main
```

For shared repository infrastructure:

```text
chore/maintenance/update-actions
        │
        ▼
   maintenance
        │
        ▼
      main
```

Keep changes within their component boundary, target the correct persistent branch, and let the repository automation handle integration and release promotion.
