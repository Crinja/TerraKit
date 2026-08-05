#!/usr/bin/env python3
from __future__ import annotations

import os
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "Cargo.toml"


def workspace_version() -> str:
    in_workspace_package = False

    for line in MANIFEST.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()

        if stripped.startswith("[") and stripped.endswith("]"):
            in_workspace_package = stripped == "[workspace.package]"
            continue

        if in_workspace_package:
            match = re.match(r'version\s*=\s*"([^"]+)"', stripped)
            if match:
                return match.group(1)

    raise RuntimeError("Could not find [workspace.package] version in Cargo.toml")


def github_output(name: str, value: str) -> None:
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        return

    with open(output_path, "a", encoding="utf-8") as output:
        output.write(f"{name}={value}\n")


def main() -> int:
    ref_name = (
        sys.argv[1]
        if len(sys.argv) > 1
        else os.environ.get("GITHUB_REF_NAME")
        or os.environ.get("GITHUB_REF", "").rsplit("/", 1)[-1]
    )

    if not ref_name:
        print("No tag or ref name was provided.", file=sys.stderr)
        return 1

    tag_version = ref_name[1:] if ref_name.startswith("v") else ref_name
    manifest_version = workspace_version()

    github_output("tag", ref_name)
    github_output("version", manifest_version)

    if tag_version != manifest_version:
        print(
            f"Tag {ref_name} does not match workspace version {manifest_version}. "
            f"Use tag v{manifest_version}.",
            file=sys.stderr,
        )
        return 1

    print(f"Release tag {ref_name} matches workspace version {manifest_version}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
