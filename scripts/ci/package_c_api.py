#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def workspace_version() -> str:
    manifest = ROOT / "Cargo.toml"
    in_workspace_package = False

    for line in manifest.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()

        if stripped.startswith("[") and stripped.endswith("]"):
            in_workspace_package = stripped == "[workspace.package]"
            continue

        if in_workspace_package:
            match = re.match(r'version\s*=\s*"([^"]+)"', stripped)
            if match:
                return match.group(1)

    raise RuntimeError("Could not find [workspace.package] version in Cargo.toml")


def copy_file(source: Path, destination: Path) -> str:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    return destination.name


def main() -> int:
    parser = argparse.ArgumentParser(description="Package TerraKit C ABI artifacts.")
    parser.add_argument("--target-dir", default="target/debug")
    parser.add_argument("--target-triple", default="native")
    args = parser.parse_args()

    target_dir = (ROOT / args.target_dir).resolve()
    package_dir = ROOT / "dist" / "c" / args.target_triple

    if package_dir.exists():
        shutil.rmtree(package_dir)

    include_dir = package_dir / "include"
    lib_dir = package_dir / "lib"
    examples_dir = package_dir / "examples"
    docs_dir = package_dir / "docs"

    copied_artifacts: list[str] = []
    artifact_candidates = [
        target_dir / "terrakit.dll",
        target_dir / "terrakit.dll.lib",
        target_dir / "terrakit.lib",
        target_dir / "libterrakit.dll.a",
        target_dir / "libterrakit.so",
        target_dir / "libterrakit.dylib",
        target_dir / "libterrakit.a",
    ]

    for artifact in artifact_candidates:
        if artifact.exists():
            copied_artifacts.append(copy_file(artifact, lib_dir / artifact.name))

    if not copied_artifacts:
        raise RuntimeError(f"No C ABI artifacts found in {target_dir}")

    copied_examples = [
        copy_file(
            ROOT / "bindings" / "c" / "examples" / "generate_heightfield.c",
            examples_dir / "generate_heightfield.c",
        ),
        copy_file(
            ROOT / "bindings" / "c" / "examples" / "header_smoke.cpp",
            examples_dir / "header_smoke.cpp",
        ),
    ]

    copied_docs = [
        copy_file(ROOT / "bindings" / "c" / "README.md", docs_dir / "C-ABI.md"),
        copy_file(ROOT / "README.md", package_dir / "README.md"),
    ]

    copied_include = copy_file(
        ROOT / "bindings" / "c" / "include" / "terrakit.h",
        include_dir / "terrakit.h",
    )

    manifest = {
        "package": "terrakit-c-api",
        "version": workspace_version(),
        "target_triple": args.target_triple,
        "target_dir": str(target_dir),
        "include": [copied_include],
        "libraries": copied_artifacts,
        "examples": copied_examples,
        "docs": copied_docs,
    }

    (package_dir / "package-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n",
        encoding="utf-8",
    )

    print(package_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
