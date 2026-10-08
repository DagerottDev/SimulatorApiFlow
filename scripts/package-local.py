#!/usr/bin/env python3
"""Build a relocatable, source-tree-independent localhost bundle."""

import argparse
import json
import os
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(*command):
    subprocess.run(command, cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="Create the bundle in this empty directory")
    options = parser.parse_args()
    output = options.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error(f"Output directory is not empty: {output}")

    run("npm", "run", "build", "--prefix", "apps/desktop")
    run("cargo", "build", "--release", "-p", "simulator-api-flow-server", "-p", "simulator-api-flow-script-worker")
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=ROOT))
    target = Path(metadata["target_directory"]) / "release"
    suffix = ".exe" if os.name == "nt" else ""

    output.mkdir(parents=True, exist_ok=True)
    (output / "bin").mkdir()
    shutil.copy2(target / f"simulator-api-flow-server{suffix}", output / "bin")
    shutil.copy2(target / f"simulator-api-flow-script-worker{suffix}", output / "bin")
    shutil.copytree(ROOT / "apps/desktop/dist", output / "ui", dirs_exist_ok=True)
    shutil.copytree(ROOT / "sidecars/mitm-addon", output / "sidecars/mitm-addon", dirs_exist_ok=True, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    (output / "cli").mkdir()
    shutil.copy2(ROOT / "scripts/saf-cli.py", output / "cli")
    shutil.copy2(ROOT / "scripts/mas-cli.py", output / "cli")
    shutil.copy2(ROOT / "README.md", output)
    shutil.copy2(ROOT / "LICENSE", output)
    shutil.copy2(ROOT / "docs/PLATFORM_SUPPORT.md", output)
    (output / "start.sh").write_text('''#!/bin/sh\nset -eu\nROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nMAS_UI_DIR="$ROOT/ui"\nMAS_ADDON_PATH="$ROOT/sidecars/mitm-addon/saf_bridge.py"\nexport MAS_UI_DIR MAS_ADDON_PATH\nexec "$ROOT/bin/simulator-api-flow-server" "$@"\n''')
    (output / "start.sh").chmod(0o755)
    (output / "start.ps1").write_text('''$ErrorActionPreference = 'Stop'\n$env:MAS_UI_DIR = Join-Path $PSScriptRoot 'ui'\n$env:MAS_ADDON_PATH = Join-Path $PSScriptRoot 'sidecars/mitm-addon/saf_bridge.py'\n& (Join-Path $PSScriptRoot 'bin/simulator-api-flow-server.exe') @args\nexit $LASTEXITCODE\n''')
    (output / "cli" / "README.txt").write_text('Run with Python 3: python saf-cli.py health (send {} on stdin) or python saf-cli.py --mcp.\n')
    print(f"Portable bundle created at {output}")


if __name__ == "__main__":
    main()
