# Git Workflow

TerraKit uses persistent component branches with short-lived development branches.

The workflow is designed to enforce component ownership while keeping `main` as the canonical integrated state.


## Persistent Branches

| Branch | Owner |
| --- | --- |
| `main` | Canonical integrated state |
| `tk` | TerraKit core and generation |
| `console` | Console interface |
| `godot` | Godot interface |
| `maintenance` | Shared repository infrastructure |

Direct development should not occur on `main`.


## Temporary Branch Convention

Temporary branches use:

```text
<change-type>/<component>/<description>
```

Supported change types are:

```text
feature
fix
refactor
chore
version
```

`version` is only valid for versioned product components:

```text
tk
console
godot
```

Maintenance is not versioned.


## Examples

```text
feature/tk/plugin-registry
fix/tk/invalid-handle
refactor/tk/pipeline-state
chore/tk/update-ci
version/tk/0.1.0

feature/console/json-output
fix/console/windows-paths
version/console/0.2.0

feature/godot/editor-graph
fix/godot/windows-loading
chore/godot/ci-cache
version/godot/0.2.0

chore/maintenance/update-actions
fix/maintenance/routing-gate
```


## PR Routing

Temporary branches must target their persistent owner branch.

```text
*/tk/*          ─────► tk
*/console/*     ─────► console
*/godot/*       ─────► godot
*/maintenance/* ─────► maintenance
```

More precisely:

```text
feature/tk/*       ─────► tk
fix/tk/*           ─────► tk
refactor/tk/*      ─────► tk
chore/tk/*         ─────► tk
version/tk/*       ─────► tk
```

The same pattern applies to Console and Godot.

Maintenance accepts:

```text
feature/maintenance/*
fix/maintenance/*
refactor/maintenance/*
chore/maintenance/*
```

Repository automation rejects temporary branches targeting the wrong persistent branch.


## Promotion to Main

Persistent branches promote into `main`.

```text
tk          ─────► main
console     ─────► main
godot       ─────► main
maintenance ─────► main
```

Feature branches should not target `main` directly.


## Synchronization from Main

Integrated changes flow back into persistent branches:

```text
main ─────► tk
main ─────► console
main ─────► godot
main ─────► maintenance
```

This keeps each persistent branch aware of the current integrated repository state.

These syncs may create merge commits.

As a result, GitHub may show a branch as one or more commits ahead of `main` even when there is no meaningful file difference.

That is expected.

The repository's promotion logic uses content divergence and version state rather than relying only on ahead/behind counts.


## Creating a Branch

Start from the component branch.

Example for Godot:

```bash
git switch godot
git pull --ff-only origin godot
git switch -c feature/godot/editor-graph
```

Push:

```bash
git push -u origin feature/godot/editor-graph
```

Open the PR into:

```text
godot
```

---

## Component Ownership

Branches are restricted by file ownership.

### `tk`

Owns core implementation and TerraKit-specific workflows.

It must not carry interface-specific changes.

### `console`

Owns:

```text
interfaces/terrakit-console/**
.github/workflows/console-*.yml
```

### `godot`

Owns:

```text
interfaces/terrakit-godot/**
.github/workflows/godot-*.yml
```

### `maintenance`

Owns shared repository infrastructure, including:

```text
.github/workflows/_*.yml
.github/workflows/main-scope.yml
.github/workflows/pr-routing.yml
.devcontainer/**
tools/**
docs/**
README.md
CONTRIBUTING.md
```

Maintenance must not be used to bypass component ownership.


## Version Branches

A version branch is the release switch for a component.

Example:

```bash
git switch godot
git pull --ff-only origin godot
git switch -c version/godot/0.2.0
```

Update the Godot component version and open:

```text
version/godot/0.2.0 ─────► godot
```

Once merged, automation observes that the Godot version is newer than the version on `main`.

It can then open:

```text
godot ─────► main
```

After the promotion succeeds, the release workflow creates the corresponding release.


## Maintenance Promotion

Maintenance does not require a product version bump.

The flow is:

```text
chore/maintenance/update-actions
        │
        ▼
   maintenance
        │
        ▼
      main
```

The maintenance promotion is allowed only when its divergence is within maintenance-owned paths.


## Required Checks

The exact ruleset configuration may evolve, but the branch model expects stable checks such as:

```text
PR Routing Gate
TerraKit Gate
Console Gate
Godot Gate
Maintenance Gate
Main Scope Gate
```

The routing gate answers:

> Is this branch targeting the correct persistent branch?

The component scope gate answers:

> Is this branch modifying only files it owns?

The component CI gate answers:

> Does this component still build and test successfully?

The main scope gate answers:

> Is this a legitimate promotion into main?


## Merge Strategy

Persistent branch synchronization and promotion use merge commits.

This preserves ancestry between persistent branches and `main`.

Avoid rebasing or force-pushing protected persistent branches.

Short-lived development branches may be rebased before merge when permitted by repository policy.


## Force Pushes

Do not force-push:

```text
main
tk
console
godot
maintenance
```

These branches are long-lived shared references.


## Deleting Branches

Temporary branches can be deleted after merge.

Persistent branches must not be automatically deleted after PR merge.

This is important because GitHub's automatic head-branch deletion can otherwise remove a persistent branch after promotion into `main`.


## Cross-Component Changes

Prefer to structure changes so components migrate independently.

For example, when changing the ABI:

```text
add compatible ABI capability
        │
        ├──► update Console
        ├──► update Godot
        │
        ▼
remove deprecated capability later
```

If a change genuinely cannot exist in intermediate compatible states, coordinate it explicitly rather than bypassing ownership protections.


## Branch Naming Checklist

Before pushing a temporary branch, confirm:

- [ ] The first segment is a supported change type.
- [ ] The second segment is the owning component.
- [ ] The branch has a descriptive final segment.
- [ ] `version/*` is only used for `tk`, `console`, or `godot`.
- [ ] The PR will target the matching persistent branch.
- [ ] The change stays inside that component's ownership boundary.


## Related Documentation

- [Contributing](../../CONTRIBUTING.md)
- [Release Process](releases.md)
- [Testing](testing.md)
