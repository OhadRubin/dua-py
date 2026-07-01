use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use dua::{
    canonicalize_ignore_dirs,
    traverse::{BackgroundTraversal, Traversal, TreeIndex},
    TraversalSorting, WalkOptions,
};
use petgraph::Direction;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyDict,
};

#[pyfunction]
#[pyo3(signature = (
    paths,
    *,
    threads = None,
    apparent_size = false,
    count_hard_links = false,
    cross_filesystems = true,
    ignore_dirs = None,
    sort = true,
    use_root_path = true,
))]
fn scan(
    py: Python<'_>,
    paths: Vec<PathBuf>,
    threads: Option<usize>,
    apparent_size: bool,
    count_hard_links: bool,
    cross_filesystems: bool,
    ignore_dirs: Option<Vec<PathBuf>>,
    sort: bool,
    use_root_path: bool,
) -> PyResult<Vec<Py<PyAny>>> {
    if paths.is_empty() {
        return Err(PyValueError::new_err("paths must not be empty"));
    }

    let traversal = run_traversal(
        paths,
        WalkOptions {
            threads: normalize_threads(threads),
            count_hard_links,
            apparent_size,
            sorting: if sort {
                TraversalSorting::AlphabeticalByFileName
            } else {
                TraversalSorting::None
            },
            cross_filesystems,
            ignore_dirs: canonicalize_ignore_dirs(&ignore_dirs.unwrap_or_default()),
        },
        use_root_path,
    )?;

    let root_indices = child_indices(&traversal, traversal.root_index, sort);
    root_indices
        .into_iter()
        .map(|idx| entry_to_dict(py, &traversal, idx, None, sort))
        .collect()
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scan, m)?)?;
    Ok(())
}

fn run_traversal(
    paths: Vec<PathBuf>,
    walk_options: WalkOptions,
    use_root_path: bool,
) -> PyResult<Traversal> {
    let mut traversal = Traversal::new();
    let mut background = BackgroundTraversal::start(
        traversal.root_index,
        &walk_options,
        paths,
        false,
        use_root_path,
    )
    .map_err(to_runtime_error)?;

    loop {
        let event = background.event_rx.recv().map_err(|_| {
            PyRuntimeError::new_err("dua traversal worker stopped before finishing")
        })?;
        if let Some(done) = background.integrate_traversal_event(&mut traversal, event) {
            if done {
                return Ok(traversal);
            }
        }
    }
}

fn entry_to_dict(
    py: Python<'_>,
    traversal: &Traversal,
    idx: TreeIndex,
    parent_path: Option<&Path>,
    sort: bool,
) -> PyResult<Py<PyAny>> {
    let data = traversal
        .tree
        .node_weight(idx)
        .ok_or_else(|| PyRuntimeError::new_err("dua traversal graph referenced a missing node"))?;
    let path = entry_path(parent_path, &data.name);
    let children = child_indices(traversal, idx, sort)
        .into_iter()
        .map(|child_idx| entry_to_dict(py, traversal, child_idx, Some(&path), sort))
        .collect::<PyResult<Vec<_>>>()?;
    let is_dir = data.is_dir || !children.is_empty() || path.is_dir();

    let dict = PyDict::new(py);
    dict.set_item("name", path_to_string(&data.name))?;
    dict.set_item("path", path_to_string(&path))?;
    dict.set_item("size", data.size)?;
    dict.set_item("mtime", system_time_to_seconds(data.mtime))?;
    dict.set_item("entry_count", data.entry_count)?;
    dict.set_item("metadata_io_error", data.metadata_io_error)?;
    dict.set_item("is_dir", is_dir)?;
    dict.set_item("children", children)?;
    Ok(dict.into())
}

fn child_indices(traversal: &Traversal, idx: TreeIndex, sort: bool) -> Vec<TreeIndex> {
    let mut children = traversal
        .tree
        .neighbors_directed(idx, Direction::Outgoing)
        .collect::<Vec<_>>();
    if sort {
        children.sort_by(|left, right| {
            let left_name = traversal
                .tree
                .node_weight(*left)
                .map(|data| data.name.as_os_str());
            let right_name = traversal
                .tree
                .node_weight(*right)
                .map(|data| data.name.as_os_str());
            left_name.cmp(&right_name)
        });
    }
    children
}

fn entry_path(parent: Option<&Path>, name: &Path) -> PathBuf {
    match parent {
        Some(parent) if !name.is_absolute() => parent.join(name),
        _ => name.to_path_buf(),
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn system_time_to_seconds(time: SystemTime) -> f64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs_f64(),
        Err(err) => -err.duration().as_secs_f64(),
    }
}

fn normalize_threads(threads: Option<usize>) -> usize {
    match threads {
        Some(threads) if threads > 0 => threads,
        _ => std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
    }
}

fn to_runtime_error(err: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}
