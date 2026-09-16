# Architecture Overview

TerraKit is an engine-agnostic procedural world generation framework.

Its central architectural rule is:

> World generation belongs to TerraKit. Rendering, scene construction, engine integration, and presentation belong to consumers.

This separation allows the same generation system to be reused by multiple clients without moving generation logic into any one engine or tool.


## High-Level Structure

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
       Console            Godot            Future
      Interface         Interface         Interfaces
          │                 │                 │
      Headless /          Engine           Engines,
      Tooling Use       Integration        Tools, Servers
```

The core runtime and generation layers are deliberately independent from the applications that consume their output.


## Architectural Layers

### TerraKit Core

The core owns the runtime behavior needed to execute generation.

Its responsibilities include:

- pipeline execution;
- stage coordination;
- data exchange between stages;
- native runtime state;
- validation of generation requests;
- infrastructure shared by all consumers.

The core should not contain behavior that exists only to satisfy one particular engine integration.

If a piece of logic exists because Godot, Unity, a custom editor, or a server needs a particular representation, that logic normally belongs in the corresponding interface rather than the core.


## Generation

Generation functionality is implemented as composable stages.

A stage describes a unit of generation work with a defined contract:

- inputs;
- outputs;
- parameters;
- execution behavior.

Stages are designed to be connected into a pipeline rather than hard-coded into one monolithic generator.

The generation layer should focus on producing data, not on deciding how that data is rendered.


## Pipeline Runtime

The pipeline runtime is responsible for connecting stage execution into a coherent generation process.

A consumer describes or constructs stage connections. TerraKit then executes that according to the contracts exposed by the participating stages.

An engine integration may expose a visual graph editor or another user-facing representation, but the execution model remains a TerraKit concern.

See [Pipeline Model](pipeline.md).


## C ABI

The C ABI is the interoperability boundary between TerraKit and external consumers.

The ABI is implemented in Rust but exposes a C-compatible interface so that clients can communicate with TerraKit through FFI.

This keeps the internal implementation language separate from the public native boundary.

The ABI should:

- expose stable, language-neutral data and operations;
- avoid leaking Rust-specific types across the boundary;
- define ownership and lifetime rules clearly;
- allow external bindings to be implemented without depending on TerraKit internals;
- evolve deliberately through explicit compatibility changes.

See [C ABI](abi.md).


## Interfaces

Interfaces adapt TerraKit to a particular consumer environment.

An interface may provide:

- engine-specific types;
- editor integration;
- language bindings;
- packaging;
- native library loading;
- data conversion between TerraKit and the host environment.

Interfaces should not reimplement world-generation logic that belongs in TerraKit.

Current interfaces include:

### Console

The Console provides a standalone, headless consumer of TerraKit.

It is useful for:

- development;
- testing;
- automation;
- demonstrations;
- validating behavior outside a game engine.

### Godot

The Godot interface exposes TerraKit through a GDExtension-based integration.

It is responsible for adapting the native TerraKit runtime to Godot rather than embedding TerraKit's generation implementation into GDScript or engine-specific code.


## Dependency Direction

The intended dependency direction is:

```text
generation
    │
    ▼
TerraKit core
    │
    ▼
C ABI
    │
    ▼
interfaces
    │
    ▼
consumers
```

Dependencies should not flow backwards from an interface into the core.

For example:

```text
Godot-specific editor behavior
        │
        └── belongs in the Godot interface

Pipeline scheduling
        │
        └── belongs in TerraKit core

Terrain generation algorithm
        │
        └── belongs in generation

Godot Mesh conversion
        │
        └── belongs in the Godot interface
```


## Data Ownership

Generated data originates in TerraKit.

Consumers may convert or copy that data into their own native structures, but those representations are not part of TerraKit's internal model.

For example:

```text
TerraKit mesh data
        │
        ▼
C ABI representation
        │
        ▼
Godot interface conversion
        │
        ▼
Godot mesh resource
```

The Godot mesh resource is a Godot concern. The generation that produced its data is a TerraKit concern.


## Headless Operation

TerraKit must remain usable without a graphical engine.

This is important for:

- automated tests;
- dedicated servers;
- remote generation services;
- build-time generation;
- offline tooling;
- batch processing;
- command-line workflows.

The Console interface is the current reference point for this style of use.


## Extensibility

The architecture is designed to grow in two independent directions.

### New Stages

New generation behavior can be added as stages without requiring every interface to understand the implementation details.

### New Interfaces

New consumers can be integrated without moving generation code into the consumer.

Examples may include:

- additional game engines;
- standalone world editors;
- server-side generation services;
- visualization tools;
- asset pipelines.


## Compatibility Boundaries

TerraKit has multiple independently versioned components.

The core version, ABI version, Console version, and engine interface versions do not need to match numerically.

Compatibility should be determined by the contract a component consumes, not by assuming identical product version numbers.

This is why the ABI is treated as a distinct architectural boundary.


## Design Principles

When making architectural changes, prefer solutions that preserve the following properties:

1. **Engine independence**  
   Core generation behavior should not depend on a host engine.

2. **Headless usability**  
   TerraKit should remain useful without a graphical client.

3. **Composable generation**  
   Generation should be expressible through reusable stages.

4. **Explicit boundaries**  
   Interface-specific behavior should remain outside the core.

5. **Language-neutral interoperability**  
   External clients should communicate through documented boundaries.

6. **Independent evolution**  
   Interfaces should be able to evolve separately from the core where compatibility allows.

7. **Testability**  
   Core generation should be testable without requiring an engine.


## Related Documentation

- [Pipeline Model](pipeline.md)
- [Stage Development](stages.md)
- [C ABI](abi.md)
- [Building TerraKit](../development/building.md)
- [Testing](../development/testing.md)
- [Git Workflow](../development/git-workflow.md)
