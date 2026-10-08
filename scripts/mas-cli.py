#!/usr/bin/env python3
"""Compatibility entry point; use saf-cli.py for SimulatorApiFlow."""
import runpy
from pathlib import Path

if __name__ == "__main__":
    runpy.run_path(str(Path(__file__).with_name("saf-cli.py")), run_name="__main__")
