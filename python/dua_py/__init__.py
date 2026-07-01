"""Python interface for dua's Rust traversal tree."""

from __future__ import annotations

from dataclasses import dataclass
from os import PathLike, fsdecode, fspath
from typing import Any, Iterable

from . import _native

__version__ = "0.1.0"


@dataclass(frozen=True)
class Entry:
    """A filesystem entry produced by dua's traversal graph."""

    name: str
    path: str
    size: int
    mtime: float
    entry_count: int | None
    metadata_io_error: bool
    is_dir: bool
    children: tuple["Entry", ...]

    @classmethod
    def from_mapping(cls, value: dict[str, Any]) -> "Entry":
        return cls(
            name=value["name"],
            path=value["path"],
            size=value["size"],
            mtime=value["mtime"],
            entry_count=value["entry_count"],
            metadata_io_error=value["metadata_io_error"],
            is_dir=value["is_dir"],
            children=tuple(cls.from_mapping(child) for child in value["children"]),
        )


def scan(
    paths: str | PathLike[str] | Iterable[str | PathLike[str]],
    *,
    threads: int | None = None,
    apparent_size: bool = False,
    count_hard_links: bool = False,
    cross_filesystems: bool = True,
    ignore_dirs: Iterable[str | PathLike[str]] | None = None,
    sort: bool = True,
    use_root_path: bool = True,
) -> tuple[Entry, ...]:
    """Scan one or more paths and return the top-level dua traversal entries."""

    native_entries = _native.scan(
        _normalize_paths(paths),
        threads=threads,
        apparent_size=apparent_size,
        count_hard_links=count_hard_links,
        cross_filesystems=cross_filesystems,
        ignore_dirs=None if ignore_dirs is None else _normalize_paths(ignore_dirs),
        sort=sort,
        use_root_path=use_root_path,
    )
    return tuple(Entry.from_mapping(entry) for entry in native_entries)


def walk(
    path: str | PathLike[str],
    *,
    threads: int | None = None,
    apparent_size: bool = False,
    count_hard_links: bool = False,
    cross_filesystems: bool = True,
    ignore_dirs: Iterable[str | PathLike[str]] | None = None,
    sort: bool = True,
    use_root_path: bool = True,
) -> Entry:
    """Scan a single path and return its root entry."""

    entries = scan(
        [path],
        threads=threads,
        apparent_size=apparent_size,
        count_hard_links=count_hard_links,
        cross_filesystems=cross_filesystems,
        ignore_dirs=ignore_dirs,
        sort=sort,
        use_root_path=use_root_path,
    )
    if len(entries) != 1:
        raise RuntimeError(f"expected one traversal root, got {len(entries)}")
    return entries[0]


def _normalize_paths(
    paths: str | PathLike[str] | Iterable[str | PathLike[str]],
) -> list[str]:
    if isinstance(paths, (str, bytes, PathLike)):
        return [fsdecode(fspath(paths))]
    return [fsdecode(fspath(path)) for path in paths]


__all__ = ["Entry", "__version__", "scan", "walk"]
