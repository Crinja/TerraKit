#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RELEASE_PACKAGES = [
    "terrakit-core",
    "terrakit-pipeline",
    "terrakit-runtime",
    "terrakit-algorithms",
    "terrakit-builtins",
    "terrakit-c-api",
    "terrakit-console",
]


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


def cargo_metadata() -> dict:
    output = subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        text=True,
    )
    return json.loads(output)


def main() -> int:
    version = workspace_version()
    metadata = cargo_metadata()
    workspace_members = set(metadata["workspace_members"])
    packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_members
    }

    unknown_workspace_packages = sorted(set(packages) - set(RELEASE_PACKAGES))
    if unknown_workspace_packages:
        raise RuntimeError(
            "Workspace package(s) missing from release manifest checks: "
            + ", ".join(unknown_workspace_packages)
        )

    for package_name in RELEASE_PACKAGES:
        package = packages.get(package_name)
        if not package:
            raise RuntimeError(f"{package_name} is missing from cargo metadata")

        if package["version"] != version:
            raise RuntimeError(
                f"{package_name} version {package['version']} does not match "
                f"workspace version {version}"
            )

        if not package.get("description"):
            raise RuntimeError(f"{package_name} is missing a package description")

        readme = package.get("readme")
        if not readme:
            raise RuntimeError(f"{package_name} is missing package readme metadata")

        manifest_dir = Path(package["manifest_path"]).parent
        if not (manifest_dir / readme).resolve().exists():
            raise RuntimeError(f"{package_name} readme path does not exist: {readme}")

        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            if dependency.get("path") and dependency_name in packages:
                requirement = dependency.get("req") or ""
                if version not in requirement:
                    raise RuntimeError(
                        f"{package_name} dependency {dependency_name} uses "
                        f"version requirement {requirement!r}; expected {version}"
                    )

    print(
        "Release manifests are version-consistent: "
        + ", ".join(RELEASE_PACKAGES)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
