# Release Process

TerraKit components are versioned and released independently.

The release model is designed around one rule:

> A version increase on a persistent component branch marks that component as ready to promote and release.

`main` remains the canonical integrated state from which releases are published.


## Components

Current versioned components are:

| Component | Version Source | Tag Format |
| --- | --- | --- |
| TerraKit Core | root `Cargo.toml` | `terrakit-vX.Y.Z` |
| Console | `interfaces/terrakit-console/Cargo.toml` | `terrakit-console-vX.Y.Z` |
| Godot | `interfaces/terrakit-godot/addon/addons/terrakit/plugin.cfg` | `terrakit-godot-vX.Y.Z` |

The ABI also has its own compatibility version, which is not required to match the TerraKit product version.


## Why Independent Versions?

The components evolve at different rates.

A Godot packaging fix should not require a TerraKit core release.

A Console feature should not force the Godot plugin to change version.

Independent versioning keeps release history meaningful.


## Development State

While development is ongoing, changes accumulate on the component's persistent branch.

Example:

```text
feature/godot/editor-graph ─┐
fix/godot/windows-load ─────┼──► godot
chore/godot/ci-cache ───────┘
```

No release is created simply because feature work lands on `godot`.


## Marking a Release

When the component is ready, create a version branch.

Example:

```bash
git switch godot
git pull --ff-only origin godot
git switch -c version/godot/0.2.0
```

Update:

```text
interfaces/terrakit-godot/addon/addons/terrakit/plugin.cfg
```

to the intended release version.

Then open:

```text
version/godot/0.2.0 ─────► godot
```

After required checks pass and the version change is merged, the component branch now advertises a version newer than the corresponding version on `main`.


## Promotion

Promotion automation compares the persistent branch against `main`.

For a versioned component, promotion requires:

1. the persistent branch contains the current `main` history;
2. divergence stays inside the component's ownership boundary;
3. the component version is valid stable `X.Y.Z` SemVer;
4. the component version is greater than the version on `main`;
5. the intended release tag does not already exist.

If those conditions are satisfied, automation opens or reuses:

```text
component ─────► main
```

and enables auto-merge.

Branch protection still applies.

Automation does not bypass required checks.


## Canonical State

A release is published only after the component's versioned state reaches `main`.

This prevents producing a release from code that has not entered the canonical integrated repository state.

Conceptually:

```text
version branch
      │
      ▼
component branch
      │
      ▼
     main
      │
      ▼
   release
```


## Release State

The release workflow determines the component version from its authoritative version source.

Version values are normalized before comparison so source syntax does not affect SemVer validation.

For example, a Godot config value such as:

```ini
version="0.2.0"
```

is treated as:

```text
0.2.0
```

for comparison and tag creation.


## Existing Tags

A release must not overwrite an existing component tag.

If the tag for the current version already exists, the workflow should not publish a duplicate release.

If a tag exists without the expected matching release, the workflow should fail rather than silently reusing ambiguous state.


## TerraKit Core Release

Core release tags use:

```text
terrakit-vX.Y.Z
```

Example:

```text
terrakit-v0.0.3
```

A core release should validate:

- root workspace quality checks;
- supported platform native builds;
- C ABI build;
- required exported symbols;
- packaging;
- checksums.

The TerraKit core release may be marked as the repository's latest release.


## Console Release

Console tags use:

```text
terrakit-console-vX.Y.Z
```

Example:

```text
terrakit-console-v0.2.0
```

The Console release should build supported platform executables and package them independently from the core release version.

Console release artifacts should identify the Console version explicitly.


## Godot Release

Godot tags use:

```text
terrakit-godot-vX.Y.Z
```

Example:

```text
terrakit-godot-v0.2.0
```

The Godot release should build the supported native GDExtension artifacts and package the addon in a form suitable for installation.

Godot release builds should use release-mode native compilation rather than the faster debug-only compatibility build used in normal pull-request CI.


## Checksums

Published binary artifacts should include checksums.

A typical release pair is:

```text
artifact.zip
artifact.zip.sha256
```


## Release Notes

Release notes should be scoped to the component being released.

A Godot release should not present unrelated Console changes as if they are part of the Godot release.

Where possible, generated release notes should use the previous tag for the same component as the comparison boundary.


## Latest Release

Because multiple products share one GitHub repository, not every interface release should replace the repository-wide "Latest" designation.

The core TerraKit release can act as the primary repository release.

Interface releases may be published without becoming the repository's global latest release.


## Maintenance Changes

Maintenance is not a versioned product.

Changes such as:

```text
.github/workflows/_component-promote.yml
docs/**
.devcontainer/**
README.md
```

may promote:

```text
maintenance ─────► main
```

without a product version increase.

After the merge, component release workflows should observe that their versions have not changed and therefore publish nothing.


## Compatibility

Product versions and ABI versions are separate.

An interface should validate the ABI contract it consumes rather than assuming a matching product version is sufficient.

For example:

```text
Godot plugin 0.3.0
```

may legitimately consume:

```text
TerraKit ABI 0.1.x
```

if that ABI range is documented as compatible.


## Breaking Changes

Prefer staged compatibility changes.

For example:

```text
1. Add ABI v2 capability.
2. Keep the old ABI behavior available.
3. Update Console.
4. Update Godot.
5. Release compatible consumers.
6. Remove deprecated behavior in a later breaking release.
```

This lets independently versioned components migrate without requiring one atomic repository-wide release.


## Failed Releases

If a release workflow fails before publication:

1. fix the underlying problem through the owning component branch;
2. keep the intended version if no tag/release was published;
3. rerun through the normal promotion/release path.

If a release tag or public release was already published, do not silently retarget or overwrite it.

Create a new patch version instead.


## Release Checklist

Before increasing a component version:

- [ ] Intended features and fixes are already on the component branch.
- [ ] The component contains current `main`.
- [ ] Required CI is green.
- [ ] Public documentation is current.
- [ ] Breaking changes are documented.
- [ ] The new version is greater than the version on `main`.
- [ ] The intended tag does not already exist.
- [ ] Release packaging is expected to succeed on supported platforms.


## Related Documentation

- [Git Workflow](git-workflow.md)
- [Testing](testing.md)
- [C ABI](../architecture/abi.md)
