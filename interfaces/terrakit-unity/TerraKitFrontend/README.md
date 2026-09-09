# TerraKit Unity Frontend

TerraKit Unity Frontend provides a Unity Graph Editor and runtime integration for generating terrain through the TerraKit native backend.

## Requirements

- Unity `6000.5.6f1`
- Universal Render Pipeline (URP)
- TerraKit native backend library

Support for additional platforms can be added by providing compatible native libraries.

## Native Backend Library

The Unity frontend communicates with the TerraKit backend through a native C API library.

Pre-built native libraries are available from the TerraKit GitHub Releases page.

For local Unity integration, the required native library should be placed under:

```text
Assets/Plugins/TerraKit/
```

The current release provides pre-built binaries for:
macOS arm64
Windows x86_64
Linux x86_64

## Quick Start

### 1. Check the Backend

Open the Unity project and select:

```text
Tools > TerraKit > Check Backend Connection
```

A successful check displays the ABI version, library version, and three built-in backend stages.

### 2. Create or Open a Backend Graph

Open:

Tools > TerraKit > Graph Editor

Open an existing backend-compatible graph or create a new one.

A backend-compatible graph should contain executable backend stages, such as:

Flat Height (Backend)
    -> Noise Height (Backend)
    -> Heightfield Mesh (Backend)

Validate and Compile the graph before generation.

### 3. Generate Terrain

Open:

```text
Tools > TerraKit > Backend Generator
```

Choose the graph and start with:

| Setting | Value |
| --- | ---: |
| Seed | `12345` |
| Region X / Y | `0 / 0` |
| LOD | `0` |
| Cell Width / Height | `16 / 16` |
| Base Spacing | `1 / 1` |

Use:

- **Generate Single Region Preview** for one terrain region.
- **Generate 3 x 3 Region Preview** to inspect neighbouring regions and seams.

Changing the Seed changes the terrain shape, but does not change the estimated vertex or triangle count.

### 4. Save Output

After generating terrain, use:

- **Save Single Mesh Asset**
- **Save 3 x 3 Meshes + Prefab**

The default output folder is:

```text
Assets/TerraKitGenerated
```

## Troubleshooting

### Backend connection fails

Confirm that a compatible TerraKit native backend library exists under:

```text
Assets/Plugins/TerraKit/
```

Then run **Check Backend Connection** again.

### Generate buttons are disabled

Confirm that:

- A backend-compatible graph is selected.
- Seed is a valid whole number.
- LOD is between `0` and `30`.
- Cell dimensions are between `1` and `4096`.
- Base Spacing values are greater than zero.

### The graph compiles but cannot generate terrain

Use **Backend Demo**, or confirm that the graph contains only nodes marked **(Backend)**.