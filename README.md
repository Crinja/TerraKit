# TerraKit

[![TerraKit CI](https://github.com/Crinja/TerraKit/actions/workflows/terrakit-ci.yml/badge.svg)](https://github.com/Crinja/TerraKit/actions/workflows/terrakit-ci.yml)
[![Console CI](https://github.com/Crinja/TerraKit/actions/workflows/console-ci.yml/badge.svg)](https://github.com/Crinja/TerraKit/actions/workflows/console-ci.yml)
[![Godot CI](https://github.com/Crinja/TerraKit/actions/workflows/godot-ci.yml/badge.svg)](https://github.com/Crinja/TerraKit/actions/workflows/godot-ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/Crinja/TerraKit?filter=terrakit-v*&label=TerraKit)](https://github.com/Crinja/TerraKit/releases)

TerraKit is an engine-agnostic procedural world generation framework.

Instead of embedding world-generation logic inside a game engine, TerraKit runs generation through a reusable native pipeline and exposes the result through stable interfaces. The same generation system can therefore be consumed by game engines, tools, headless servers, automated pipelines, and custom clients without making the generation implementation dependent on any one of them.

TerraKit is designed around a simple idea:

> **World generation should be a reusable system that engines interface with, not a subsystem that engines own.**


## Why TerraKit?

World-generation systems are often tightly coupled to the engine or application that consumes them. That can make generation logic difficult to reuse, test independently, execute headlessly, move between engines, or expose to server-side workflows.

TerraKit separates **generation** from **presentation**.

A TerraKit pipeline can be used to:

- run procedural generation independently of a game engine;
- compose generation from reusable stages;
- define stage inputs, outputs, and parameters;
- construct generation graphs from external interfaces;
- expose native functionality through a C-compatible ABI;
- support multiple engine integrations without duplicating generation logic;
- run in desktop, tooling, server, or headless environments;
- keep world-generation architecture reusable across projects and clients.

This makes TerraKit suitable for workflows where the same generation stack needs to be shared by multiple consumers or survive changes in the engine surrounding it.


## Architecture

At a high level, TerraKit separates the generation runtime from the clients that consume it.

```text
                         TerraKit
                            │
              ┌─────────────┴─────────────┐
              │                           │
        Generation Stages            Pipeline Runtime
              │                           │
              └─────────────┬─────────────┘
                            │
                       TerraKit Core
                            │
                          C ABI
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
       Console            Unity            Future
      Interface         Interface         Interfaces
          │                 │                 │
      Headless /          Engine           Engines,
      Tooling Use       Integration        Tools, Servers
```

### Core

The TerraKit core owns the runtime infrastructure used to execute generation pipelines, coordinate stages, exchange data, and expose generated results.

### Generation Stages

Generation functionality is implemented as composable stages.

Stages can define:

- inputs;
- outputs;
- configurable parameters;
- processing behavior.

This allows interfaces to build stages without any specific knowledge of what they are or how they work.

### C ABI

TerraKit exposes native functionality through a C-compatible ABI.

The ABI acts as the interoperability boundary between the TerraKit runtime and external consumers, allowing integrations to use TerraKit through FFI without requiring the consumer to be written in Rust.

### Interfaces

Interfaces adapt TerraKit to a particular environment.

They do not own the generation system itself. Their job is to expose TerraKit in a way that is natural for the client they integrate with.

Current interfaces include:

- **Console** — headless use, testing and automation
- **Godot** — native Godot integration using GDExtension.

Additional interfaces can be added without moving the generation implementation into the client.


## Quick Start

### Requirements

For core development:

- Git
- Rust 1.87 or newer
- Make

Some interfaces have additional requirements. See their documentation for details.

### Run the Core Checks

```bash
make ci
```

### Build TerraKit

```bash
cargo build --workspace
```

### Build the Native C ABI

```bash
cargo build -p terrakit-c-api --release
```

### Run the Console Interface

The console interface is maintained as a standalone Cargo workspace:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  -- mesh-demo
```

For an optimized build:

```bash
cargo run \
  --manifest-path interfaces/terrakit-console/Cargo.toml \
  --release \
  -- mesh-demo
```

## Interfaces

TerraKit interfaces are versioned and developed independently from the core.

| Interface | Purpose |
| --- | --- |
| C ABI | Native interoperability boundary |
| Console | Headless, testing, automation, and reference interface |
| Godot | Godot GDExtension integration |
| Additional interfaces | Other engines, tools, servers, and clients |

Interface versions do not need to match the TerraKit core version.

An interface should depend on the compatibility contract it consumes rather than assuming that matching product versions imply compatibility.


## Repository Structure

```text
TerraKit/
├── engine/                  # TerraKit runtime and native core crates
├── generation/              # Generation stages and related functionality
├── interfaces/
│   ├── terrakit-console/    # Standalone console interface
│   └── terrakit-godot/      # Godot integration
├── tools/                   # Repository and release tooling
├── docs/                    # Project documentation
├── .github/
│   └── workflows/           # CI, promotion, release, and repository automation
├── Cargo.toml               # Core workspace definition
├── rust-toolchain.toml      # Rust toolchain configuration
└── Makefile                 # Common development commands
```

The root Cargo workspace intentionally contains the TerraKit core, generation crates, and repository tooling.

Interfaces are treated as separate integration products and may maintain their own build, versioning, and release lifecycle.


## Versioning

TerraKit components are versioned independently.

| Component | Version Source | Tag Format |
| --- | --- | --- |
| TerraKit Core | Root `Cargo.toml` | `terrakit-vX.Y.Z` |
| Console | `interfaces/terrakit-console/Cargo.toml` | `terrakit-console-vX.Y.Z` |
| Godot | Godot `plugin.cfg` | `terrakit-godot-vX.Y.Z` |

A version increase on a component branch marks that component as ready for promotion and release.

For example:

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

Component releases are independent. A Godot release does not require the Godot plugin version to match the TerraKit core version, and other interfaces follow the same model.


## Development Model

TerraKit uses persistent component branches with short-lived development branches.

```text
feature/tk/*
fix/tk/*
refactor/tk/*
chore/tk/*
version/tk/*
        │
        ▼
       tk
        │
        ▼
       main
```

The same structure is used for `console` and `godot`.

Shared repository infrastructure is owned by the persistent `maintenance` branch:

```text
feature/maintenance/*
fix/maintenance/*
refactor/maintenance/*
chore/maintenance/*
        │
        ▼
   maintenance
        │
        ▼
       main
```

`main` represents the canonical integrated repository state. Development is performed through the appropriate component or maintenance branch rather than directly on `main`.

Pull requests are validated for both branch routing and file ownership.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development and pull-request workflow.


## Documentation

Project documentation lives under [`docs/`](docs/).

Recommended entry points:

- [Architecture Overview](docs/architecture/overview.md)
- [Pipeline Model](docs/architecture/pipeline.md)
- [Stage Development](docs/architecture/stages.md)
- [C ABI](docs/architecture/abi.md)
- [Building TerraKit](docs/development/building.md)
- [Testing](docs/development/testing.md)
- [Release Process](docs/development/releases.md)
- [Git Workflow](docs/development/git-workflow.md)
- [Contributing](CONTRIBUTING.md)


## Design Goals

TerraKit is being developed around several long-term design goals.

### Engine Independence

Generation logic should not depend on Unity, Godot, Unreal, or any other client environment.

### Reusability

A generation pipeline should be reusable across clients, projects, tools, and server environments.

### Composability

Generation should be expressible as a graph of reusable stages rather than a single monolithic generator.

### Headless Operation

Generation should be usable without a rendering engine or graphical client.

### Stable Interoperability Boundaries

External consumers should interact with TerraKit through documented interfaces rather than relying on internal Rust implementation details.

### Extensibility

New stages and interfaces should be addable without requiring unrelated parts of the system to be redesigned.

### Testability

The generation runtime should be independently testable without requiring a game engine to host it.


## Use Cases

### Engine Integrations

A game engine can request generated world data from TerraKit while remaining responsible for rendering, scene construction, physics, and engine-specific representation.

### Headless Generation

Servers and command-line clients can execute the same generation pipelines without launching an engine.

### Shared Generation Infrastructure

Multiple clients can consume the same generation architecture rather than maintaining separate implementations.

### Tooling

Editors, visual graph tools, exporters, validators, and automated pipelines can interact with TerraKit through the same underlying runtime.

### Server-Driven Worlds

A server-side process can generate or update world data independently of the client application, allowing generation infrastructure to evolve separately from the rendering client where the integration architecture permits it.


## Building Interfaces

Interfaces are intentionally separated from the root Cargo workspace.

### Console

```bash
cargo build \
  --manifest-path interfaces/terrakit-console/Cargo.toml
```

### Godot

The Godot interface uses GDExtension and SCons.

Its native extension and TerraKit ABI library are built as part of the Godot interface workflow.

See the Godot interface documentation for platform-specific instructions.


## CI and Releases

TerraKit validates components independently.

The repository currently maintains separate gates for:

- TerraKit Core;
- Console;
- Godot;
- shared repository maintenance;
- pull-request routing;
- promotion into `main`.

Component releases are produced from the canonical state on `main`.

Release tags use component-specific prefixes, allowing the repository to contain independent release histories without conflating their versions.

See [`docs/development/releases.md`](docs/development/releases.md) for the full release model.


## Contributing

Before opening a pull request, read [CONTRIBUTING.md](CONTRIBUTING.md).

TerraKit uses component-owned branches, so temporary development branches follow this form:

```text
<change-type>/<component>/<description>
```

Examples:

```text
feature/tk/plugin-registry
fix/console/windows-paths
refactor/godot/native-loader
chore/godot/ci-cache
version/godot/0.2.0
chore/maintenance/update-actions
```

Supported change types are:

- `feature`
- `fix`
- `refactor`
- `chore`
- `version` for versioned components

Pull requests should target the persistent branch belonging to the component being changed.
