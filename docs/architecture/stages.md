# Stage Development

Stages are TerraKit's unit of reusable generation behavior.

A stage should perform one coherent generation task, expose the information required to configure it, and communicate with the rest of the pipeline through explicit inputs and outputs.


## Stage Responsibilities

A stage is responsible for:

- declaring its identity;
- declaring supported inputs;
- declaring produced outputs;
- declaring configurable parameters;
- validating stage-specific configuration where appropriate;
- processing input data;
- producing output data;
- returning failures through the pipeline error model.

A stage should not depend on a particular game engine.


## Stage Boundaries

Prefer stages that represent a clear generation operation.

Good stage boundaries make graphs easier to:

- reuse;
- test;
- visualize;
- cache;
- reason about;
- replace.

A stage should not become a container for unrelated generation behavior simply to reduce the number of graph nodes.

At the same time, stages should not be split so aggressively that data is constantly converted or copied for trivial operations.


## Inputs

Inputs describe data required or accepted by a stage.

A stage should make required and optional inputs explicit.

Conceptually:

```text
Stage
├── heightfield   [required]
├── mask          [optional]
└── biome_data    [optional]
```

Inputs should be defined through TerraKit's common data contracts rather than engine-specific types.

Do not expose types such as:

```text
Godot ArrayMesh
Unity Mesh
Unreal UObject
```

from a reusable generation stage.

Use TerraKit-native or ABI-compatible representations instead.


## Outputs

Outputs represent generated values made available to downstream stages or consumers.

A stage may expose one or more outputs.

For example:

```text
Terrain Stage
├── heightfield
├── normals
└── metadata
```

Outputs should have enough type information for the pipeline to validate connections.


## Parameters

Parameters configure a stage instance.

A parameter definition should eventually provide enough metadata for an interface to present it without knowing the stage implementation.

Useful parameter metadata may include:

- name;
- stable identifier;
- value type;
- default value;
- minimum value;
- maximum value;
- allowed values;
- description;
- whether the value is required.

Example conceptual definition:

```text
frequency
type: float
default: 0.01
minimum: 0.0
description: Base sampling frequency.
```

Exact metadata structures are defined by the ABI.


## Stage Discovery

Interfaces should not need a hard-coded list of every stage.

TerraKit should expose stage discovery through a registry or equivalent runtime mechanism.

A client should be able to ask questions such as:

```text
What stages are available?
What inputs does this stage accept?
What outputs does it produce?
What parameters can be configured?
```

This is important for generic interfaces such as visual graph editors.


## Stage Identity

A stage needs an identity that can be referred to across:

- pipeline definitions;
- interfaces;
- diagnostics;
- discovery;
- serialization.

Identifiers should be treated as compatibility-sensitive once users can persist pipelines that depend on them.

Renaming a public stage identifier can become a breaking change even when the underlying algorithm is unchanged.


## Processing

A stage should treat processing as a transformation from declared inputs and configuration into declared outputs.

Conceptually:

```text
inputs + parameters + execution context
                  │
                  ▼
               stage
                  │
                  ▼
               outputs
```

Avoid hidden dependencies on global engine state.

If external context is required, it should be provided through an explicit TerraKit mechanism.


## State

Prefer stateless stages where practical.

Stateless stages are easier to:

- test;
- run concurrently;
- cache;
- retry;
- reason about.

Stateful stages are still valid when the algorithm requires them, but lifecycle and concurrency behavior should be explicit.


## Determinism

If a stage uses randomness, prefer an explicit seed or random source supplied through TerraKit.

Avoid silently reading process-global randomness when deterministic generation is expected.

A deterministic stage should produce equivalent output for equivalent:

- inputs;
- parameters;
- seed;
- relevant execution context.


## Error Handling

Do not panic or terminate the process for normal invalid stage input.

Stage errors should be propagated through TerraKit's error handling path.

Useful errors identify:

- the stage;
- the failed operation;
- the invalid parameter or input;
- the reason execution could not continue.

FFI-facing failures must be representable through the C ABI without exposing Rust-specific error types.


## Memory and Ownership

Stage output may cross the C ABI boundary.

When implementing data that can leave Rust, consider:

- who owns the allocation;
- how long returned data remains valid;
- whether the caller receives a copy or borrowed view;
- how the caller releases owned resources;
- whether the data can be safely accessed concurrently.

Do not expose raw Rust ownership assumptions as undocumented ABI behavior.


## Performance

Generation stages can become performance-critical.

Prefer measurement before optimization.

When optimizing a stage, consider:

- allocation frequency;
- large data copies;
- cache locality;
- parallel execution opportunities;
- repeated parameter parsing;
- intermediate representations.

Do not trade away deterministic behavior, safety, or interface clarity for unmeasured micro-optimizations.


## Parallelism

Stages may execute concurrently when their dependencies permit it.

Stage implementations should therefore avoid unnecessary global mutable state.

Where mutable shared state is required, synchronization behavior must be explicit.

A stage should not assume it is the only generation operation running in the process.


## Testing a Stage

A stage should be testable without a game engine.

Useful tests include:

### Construction

Confirm the stage can be created with valid configuration.

### Parameter Validation

Test:

- valid values;
- boundary values;
- invalid values;
- defaults.

### Input Validation

Test:

- required inputs;
- optional inputs;
- invalid types;
- missing data.

### Output

Check output structure and expected properties.

For deterministic stages, test known input/output relationships where practical.

### Repeated Execution

Ensure repeated execution does not leak stale state between runs unless state retention is explicitly part of the stage contract.


## Adding a New Stage

The exact crate and registration APIs may evolve, but the development process should follow this shape:

1. choose the appropriate generation module or crate;
2. define the stage implementation;
3. define inputs and outputs;
4. define parameter metadata;
5. implement processing;
6. register the stage with TerraKit's discovery mechanism;
7. add unit tests;
8. add pipeline-level integration tests where useful;
9. document public stage behavior;
10. verify the stage remains independent from engine-specific code.


## ABI Exposure

Not every internal detail of a stage should be exposed directly through the ABI.

The ABI should expose a stable description sufficient for external consumers to:

- discover the stage;
- instantiate or reference it;
- configure it;
- connect it;
- execute it through a pipeline;
- inspect results and errors.

Internal Rust types may evolve without becoming ABI commitments.


## Compatibility

Changes that may be compatibility-sensitive include:

- renaming a stage identifier;
- removing a stage;
- removing or renaming parameters;
- changing parameter meaning;
- changing input/output contracts;
- changing deterministic behavior for the same inputs.

When such changes are necessary, document them and provide migration guidance where practical.


## Related Documentation

- [Architecture Overview](overview.md)
- [Pipeline Model](pipeline.md)
- [C ABI](abi.md)
- [Testing](../development/testing.md)
