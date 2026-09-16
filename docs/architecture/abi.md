# C ABI

TerraKit exposes a C-compatible ABI so native clients can interact with the runtime without depending on Rust's binary interface.

The ABI is the primary interoperability boundary between TerraKit and external native consumers.


## Why a C ABI?

Rust does not provide a stable language ABI suitable for direct long-term consumption by arbitrary languages and engines.

A C-compatible ABI provides a much broader integration surface.

It can be consumed from:

- C;
- C++;
- C# through P/Invoke or another FFI layer;
- Godot GDExtension code;
- Python native bindings;
- other languages with C FFI support.


## Boundary

The intended dependency boundary is:

```text
Rust implementation
       │
       ▼
TerraKit C ABI
       │
       ▼
language / engine binding
       │
       ▼
consumer
```

Consumers should not rely on private Rust crate layouts or internal symbols.


## Header Generation

The public C header is generated from the Rust ABI definitions.

This keeps the exported header aligned with the implementation and avoids maintaining a second hand-written API definition.

The generated header is the authoritative C-facing declaration of exported ABI functions and types for a particular build.

> [!IMPORTANT] Do not manually edit generated headers.

## ABI Version

The ABI has its own compatibility version.

The ABI version is separate from:

- the TerraKit product version;
- any interface version;

An interface should validate that the ABI it loads is compatible with the interface's expectations.

Do not assume that matching TerraKit and interface product versions imply ABI compatibility.


## Compatibility

ABI changes fall into two broad categories.

### Compatible Additions

Examples may include:

- adding a new function;
- adding a new discoverable capability;
- introducing a new optional stage feature;
- adding new opaque handle types.

Compatible additions should avoid changing the meaning or binary layout of existing public contracts.

### Breaking Changes

Examples include:

- changing an exported function signature;
- removing an exported symbol;
- changing the layout of a public struct;
- changing ownership semantics;
- reinterpreting an existing enum value;
- changing the required calling convention.

Breaking changes require an explicit ABI compatibility decision.


## Opaque Handles

Where practical, complex Rust-owned state should cross the ABI as opaque handles rather than exposing implementation layout.

Conceptually:

```c
typedef struct TkPipeline TkPipeline;
```

The caller can hold a pointer or handle but does not need to know the Rust implementation structure.

This permits internal implementation changes without changing the binary representation expected by consumers.


## Ownership

Every ABI value that crosses the boundary must have clear ownership semantics.

For returned memory, the contract must answer:

- who allocated it;
- who owns it;
- how long it remains valid;
- whether the caller may retain it;
- how it must be released.

A safe pattern is to pair creation with a corresponding TerraKit destruction function.

Conceptually:

```text
tk_pipeline_create()
        │
        ▼
   caller owns handle
        │
        ▼
tk_pipeline_destroy()
```

The exact exported function names are defined by the generated header.


## Strings

Strings crossing the ABI require explicit encoding and lifetime rules.

UTF-8 should be preferred where practical.

The contract should define whether a returned string is:

- borrowed;
- caller-owned;
- TerraKit-owned;
- temporary;
- null-terminated;
- length-delimited.

> [!IMPORTANT] Never rely on Rust `String` or `&str` layout across FFI.


## Collections

Rust collections such as `Vec<T>` must not be exposed directly.

Collections crossing the ABI should use C-compatible representations, such as:

- pointer + length;
- explicit buffer structs;
- opaque iterators;
- accessor functions.

The chosen representation should make ownership and lifetime unambiguous.


## Errors

Rust panics must not unwind across the FFI boundary.

Expected failures should be represented through ABI-safe error reporting.

An error contract should allow clients to distinguish between categories such as:

- invalid argument;
- invalid handle;
- unavailable capability;
- graph validation failure;
- execution failure;
- incompatible ABI version;
- internal error.

The exact representation may evolve, but failures must not require a client to parse process crashes.


## Nullability

Pointer arguments and returned pointers must have documented nullability.

If an argument cannot be null, the exported function should validate that requirement before dereferencing where practical.

Invalid FFI input should fail predictably rather than invoking undefined behavior.


## Enums and Flags

Public enum values become compatibility-sensitive once exposed through the ABI.

When evolving enums:

- avoid silently renumbering existing values;
- prefer explicit discriminants where layout matters;
- document unknown-value behavior;
- consider whether clients should tolerate newer values.

Bitflags should similarly reserve room for future capabilities where useful.


## Struct Layout

Any struct exposed by value through the ABI must use an appropriate stable C representation.

Changing field order, field type, packing, or size can be breaking.

Prefer opaque handles when exposing internal runtime objects.

Use public C structs mainly for simple data-transfer objects whose layout is deliberately part of the contract.


## Calling Convention

Exported functions must use the C ABI.

Rust-specific calling conventions must not be exposed to consumers.

The generated header should describe the expected C-facing declarations.


## Thread Safety

The ABI contract should not imply thread safety where it is not guaranteed.

For each handle or operation, the implementation should eventually document whether:

- concurrent reads are safe;
- concurrent mutation is safe;
- calls must occur on one thread;
- ownership may move between threads.

Engine bindings may impose additional thread restrictions beyond TerraKit itself.


## Symbol Stability

Exported symbol names are part of the ABI.

Removing or renaming an exported symbol is breaking unless the old symbol remains available through a compatibility layer.

Release validation should verify that required public symbols are present in produced native libraries.


## Interface Loading

An interface that loads TerraKit dynamically should validate the runtime before using it.

A robust loading sequence is:

```text
load native library
       │
       ▼
resolve required symbols
       │
       ▼
query ABI version
       │
       ▼
validate compatibility
       │
       ▼
use TerraKit
```

If compatibility validation fails, the interface should report a useful error instead of continuing with an incompatible runtime.


## Generated Bindings

Higher-level interfaces may generate or maintain language bindings from the ABI.

Those bindings should remain thin.

They may provide:

- safer ownership wrappers;
- idiomatic naming;
- engine-specific conversion;
- automatic cleanup.

They should not duplicate generation behavior implemented by TerraKit.


## Release Requirements

A native ABI release should be validated on every supported release platform.

Release validation should include:

- successful native compilation;
- generated header consistency;
- required exported symbols;
- ABI version correctness;
- interface compatibility checks;
- packaging;
- checksums.


## Breaking ABI Changes

Prefer compatibility transitions over immediate removal.

A typical migration is:

```text
1. Add new ABI capability.
2. Keep old capability temporarily.
3. Update interfaces.
4. Release compatible consumers.
5. Deprecate old capability.
6. Remove it in an explicitly breaking ABI revision.
```

This allows independently versioned interfaces to migrate without forcing an atomic repository-wide release.


## Public Reference

This document describes ABI design and compatibility rules.

For exact exported names, signatures, constants, and layouts, use the generated C header from the corresponding TerraKit build.

That header is the symbol-level reference for the release being consumed.


## Related Documentation

- [Architecture Overview](overview.md)
- [Pipeline Model](pipeline.md)
- [Stage Development](stages.md)
- [Release Process](../development/releases.md)
