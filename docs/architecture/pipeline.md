# Pipeline Model

TerraKit generation is structured as a pipeline of connected stages.

The pipeline model exists so generation can be composed, inspected, tested, and driven by external interfaces without coupling the generation implementation to a particular engine.


## Concept

A pipeline is a graph of stage instances connected through declared inputs and outputs.

```text
        ┌──────────────┐
        │ Stage A      │
        │ Output: data │
        └──────┬───────┘
               │
               ▼
        ┌──────────────┐
        │ Stage B      │
        │ Input: data  │
        │ Output: mesh │
        └──────┬───────┘
               │
               ▼
        ┌──────────────┐
        │ Stage C      │
        │ Input: mesh  │
        └──────────────┘
```

The specific stages in a graph may change, but the runtime should be able to reason about them through their exposed contracts.


## Responsibilities

The pipeline runtime is responsible for:

- receiving a graph definition;
- validating stage connections;
- resolving inputs and outputs;
- supplying stage parameters;
- scheduling stage execution;
- propagating generated data;
- surfacing validation or execution errors;
- returning results to the consumer.

The pipeline runtime is not responsible for rendering output in a host engine.


## Stage Instances

A stage definition describes a type of operation.

A stage instance represents one configured use of that stage within a pipeline.

For example, the same stage type could appear multiple times with different parameter values:

```text
Noise Stage
  frequency = 0.01
        │
        ▼
      ...

Noise Stage
  frequency = 0.20
        │
        ▼
      ...
```

Each instance therefore requires:

- a stage type identifier;
- instance identity;
- parameter values;
- input connections;
- output connections.


## Inputs and Outputs

Stages expose named or otherwise identifiable inputs and outputs.

A valid connection requires an output from one stage to be acceptable as an input to another.

Conceptually:

```text
Stage A.output ─────► Stage B.input
```

The runtime should reject invalid connections before or during execution with a clear error rather than relying on undefined behavior.

Connection compatibility may consider properties such as:

- data kind;
- element type;
- shape or dimensionality;
- semantic role;
- optional versus required input.


## Parameters

Parameters configure stage behavior without requiring a new stage implementation.

Examples of parameters may include:

```text
frequency
seed
scale
resolution
threshold
octaves
```

Interfaces should be able to inspect parameter definitions so they can present appropriate controls without hard-coding every stage.

For example, an engine editor may dynamically generate a control panel from stage metadata.


## Execution

At a conceptual level, pipeline execution proceeds as follows:

```text
pipeline definition
      │
      ▼
validation
      │
      ▼
dependency resolution
      │
      ▼
stage execution
      │
      ▼
data propagation
      │
      ▼
result
```

The runtime may execute independent stages in parallel where the implementation permits it.

Parallelism is an implementation concern and should not change the semantic meaning of a valid pipeline.


## Determinism

Where a stage is intended to be deterministic, identical:

- input data;
- parameters;
- seed values;
- relevant runtime configuration;

should produce equivalent output.

Stages that intentionally depend on external or nondeterministic state should document that behavior.

Deterministic generation is especially useful for:

- testing;
- remote generation;
- reproducible worlds;
- cache keys;
- debugging;
- multiplayer or server-authoritative systems.


## Validation

A pipeline should be validated before expensive execution where practical.

Validation may include:

- missing required inputs;
- incompatible connections;
- unknown stage identifiers;
- invalid parameter values;
- cycles where cycles are unsupported;
- unavailable stage implementations;
- invalid output requests.

Validation errors should identify the affected stage and connection where possible.


## Errors

Pipeline errors should be treated as data that can be surfaced through interfaces.

An interface should not need to infer failures from crashes or missing output.

Error information should eventually be rich enough to answer:

- which stage failed;
- which operation failed;
- why it failed;
- whether the graph itself is invalid;
- whether the failure is recoverable.

The ABI should expose errors in a language-neutral form.


## Interface Responsibilities

An interface may:

- create stage instances;
- set parameters;
- connect stages;
- request execution;
- retrieve results;
- convert results into native client types;
- present errors.

An interface should not:

- duplicate core pipeline execution;
- implement private scheduling rules;
- reinterpret stage dependency semantics;
- embed generation algorithms that belong in reusable stages.


## Testing the Pipeline

Pipeline tests should cover:

- valid graph construction;
- invalid graph rejection;
- parameter propagation;
- correct input/output routing;
- deterministic behavior where expected;
- stage failure propagation;
- repeated execution;
- resource cleanup.

Tests that can run without an engine should remain in the core or generation layers.


## Related Documentation

- [Architecture Overview](overview.md)
- [Stage Development](stages.md)
- [C ABI](abi.md)
- [Testing](../development/testing.md)
