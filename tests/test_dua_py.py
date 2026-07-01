from __future__ import annotations

from pathlib import Path

import pytest

import dua_py


def test_walk_returns_nested_entries(tmp_path: Path) -> None:
    (tmp_path / "alpha.txt").write_bytes(b"alpha")
    nested = tmp_path / "nested"
    nested.mkdir()
    (nested / "beta.bin").write_bytes(b"betabeta")

    root = dua_py.walk(tmp_path, apparent_size=True, sort=True)

    assert root.is_dir
    assert root.path == str(tmp_path)
    assert root.entry_count and root.entry_count >= 4

    children = {Path(child.name).name: child for child in root.children}
    assert children["alpha.txt"].size == 5
    assert children["alpha.txt"].children == ()
    assert children["nested"].is_dir

    nested_children = {Path(child.name).name: child for child in children["nested"].children}
    assert nested_children["beta.bin"].size == 8


def test_scan_rejects_empty_paths() -> None:
    with pytest.raises(ValueError, match="paths must not be empty"):
        dua_py.scan([])
