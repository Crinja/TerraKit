#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import os
import re
import zipfile
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


def github_output(name: str, value: str) -> None:
    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        return

    with open(output_path, "a", encoding="utf-8") as output:
        output.write(f"{name}={value}\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Archive a packaged C ABI directory.")
    parser.add_argument("--target-triple", default="native")
    parser.add_argument("--package-dir")
    args = parser.parse_args()

    package_dir = (
        Path(args.package_dir).resolve()
        if args.package_dir
        else ROOT / "dist" / "c" / args.target_triple
    )

    if not package_dir.is_dir():
        raise RuntimeError(f"Package directory does not exist: {package_dir}")

    version = workspace_version()
    archive_dir = ROOT / "dist" / "archives"
    archive_dir.mkdir(parents=True, exist_ok=True)

    archive_name = f"terrakit-c-api-{version}-{args.target_triple}.zip"
    archive_path = archive_dir / archive_name
    top_level = f"terrakit-c-api-{version}-{args.target_triple}"

    if archive_path.exists():
        archive_path.unlink()

    files = sorted(path for path in package_dir.rglob("*") if path.is_file())
    if not files:
        raise RuntimeError(f"No files found to archive under {package_dir}")

    with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in files:
            archive.write(path, Path(top_level) / path.relative_to(package_dir))

    digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    checksum_path = archive_path.with_suffix(archive_path.suffix + ".sha256")
    checksum_path.write_text(f"{digest}  {archive_name}\n", encoding="utf-8")

    github_output("archive", str(archive_path))
    github_output("checksum", str(checksum_path))

    print(archive_path)
    print(checksum_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
