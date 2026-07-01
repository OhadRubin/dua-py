# dua-py

A minimal Python interface around the Rust traversal engine from
[dua-cli](https://github.com/Byron/dua-cli).

`dua` represents a traversal as an in-memory
`StableGraph<EntryData, (), Directed>` with `TreeIndex` node IDs. This package
uses PyO3 and maturin to run that traversal, integrate the emitted events into
the graph, and return nested Python objects that keep the same tree shape.

## Install

```bash
pip install dua-py
```

## What gets bundled

`dua-py` uses the Rust library API from `dua-cli`; it does not shell out to the
`dua` command-line executable.

When you install a PyPI wheel, the compiled Rust extension includes the
`dua-cli` library code and its Rust dependencies. You do not need to install the
standalone `dua` binary separately.

PyPI wheels are currently built for macOS and Linux. Windows wheels are not
published because `dua-cli` currently requires an unstable Rust feature for its
Windows library build.

When pip falls back to the source distribution, the `dua-cli` source is not
vendored into this repository. The build uses `Cargo.toml` and `Cargo.lock`, so
the machine installing from source needs Rust/Cargo and access to crates.io to
fetch and compile the Rust dependencies.

## Install for development

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin pytest
maturin develop
pytest
```

## Usage

```python
from pathlib import Path

import dua_py

root = dua_py.walk(Path("."))
print(root.path, root.size, root.entry_count)

for child in root.children:
    print(child.name, child.size)
```

For multiple traversal roots, use `scan()`:

```python
roots = dua_py.scan(["src", "tests"], apparent_size=True)
```

## API

### `dua_py.walk(path, **options) -> Entry`

Scan a single filesystem path and return one `Entry`.

### `dua_py.scan(paths, **options) -> tuple[Entry, ...]`

Scan one or more filesystem paths and return the top-level traversal entries.

Options:

- `threads: int | None = None` - number of walker threads. `None` uses the
  available CPU parallelism.
- `apparent_size: bool = False` - use metadata length instead of allocated disk
  usage.
- `count_hard_links: bool = False` - count every hard-link occurrence
  independently.
- `cross_filesystems: bool = True` - allow traversal to cross filesystem
  boundaries.
- `ignore_dirs: Iterable[path] | None = None` - directories to skip.
- `sort: bool = True` - sort entries alphabetically while walking and returning
  children.
- `use_root_path: bool = True` - keep the root path as the root entry name.

### `Entry`

`Entry` is an immutable dataclass:

```python
@dataclass(frozen=True)
class Entry:
    name: str
    path: str
    size: int
    mtime: float
    entry_count: int | None
    metadata_io_error: bool
    is_dir: bool
    children: tuple[Entry, ...]
```

`size` is reported in bytes. `mtime` is seconds since the Unix epoch. For
directories, `size` and `entry_count` are recursively aggregated by `dua`.
