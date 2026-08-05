# TerraKit C ABI

The committed header is `bindings/c/include/terrakit.h`. It exposes TerraKit's
embedding ABI for C-compatible hosts such as C, C++, C#, Unity native plugins,
Zig, and other FFI consumers.

## Build

From the repository root:

```sh
make c-build
make c-header-check
make c-symbol-check
make c-test
```

`make c-build` builds the Rust `terrakit-c-api` crate as `cdylib`,
`staticlib`, and `rlib`. `make c-header` regenerates the header with pinned
`cbindgen`. `make c-symbol-check` verifies required `tk_` exports. `make
c-package` writes an include/lib/example package under `dist/c/<target>`.
Use `mingw32-make` for these targets on Windows environments where plain
`make` is not installed.

## Linking

Dynamic consumers link against the platform dynamic library (`terrakit.dll`,
`libterrakit.so`, or `libterrakit.dylib`) and include `terrakit.h`. Static
consumers link the generated static library plus any platform system libraries
required by Rust for the selected target.

Define `TK_USE_SHARED` before including the header on Windows when importing
from a TerraKit DLL. The header declares every exported function with `TK_API`.

## Strings

`tk_string_view_t` is UTF-8 and not required to be NUL-terminated. Inputs are
copied when TerraKit needs to retain them. Schema strings borrow from the live
registry handle; copy them before destroying the registry.

## Handles

Every owned object is opaque and destroyed through a pointer-to-pointer destroy
function. Destroying through the same variable twice is safe because successful
destruction sets the variable to `NULL`.

Pipeline assembly consumes stage constructions when added successfully.
Finishing an assembler consumes the assembler on success or failure. Runtime
creation consumes a pipeline only on success.

## Results

Generation is synchronous. One runtime processes one request at a time.
Each successful generation returns an independent result. Result views borrow
from the result and remain valid until `tk_generation_result_destroy`.

Height fields use `index = y * width + x`. Density fields and voxel volumes use
`index = (z * height + y) * width + x`. Mesh positions are local `float3`
coordinates relative to the returned `origin`; indices are `uint32_t` triangle
indices. Absent mesh normals or texture coordinates are returned as `NULL, 0`.

## Errors

Fallible calls return `tk_status_t`. On failure, call
`tk_last_error_message_copy` to copy the thread-local diagnostic. Normal ABI
calls clear the previous diagnostic at entry; the error-copy functions do not.
