# Changelog

## 0.12.0 (unreleased)

Changes since 0.11.0 on `main`.

### Breaking changes and migration

- Require Python 3.12 or newer, raising the minimum from Python 3.10.
- Rename column `dimension` to `degree` throughout the Rust API and documentation.
  Replace `Column::dimension`, `Column::set_dimension`, and
  `Column::new_with_dimension` with `degree`, `set_degree`, and `new_with_degree`.
  Input tuples still contain the chain degree followed by the boundary entries.
  Serde formats that store field names now use `degree` instead of `dimension`;
  migrate existing data containing the old field name before deserializing it.
- Rename the Cargo feature `python` to `python-module`. Update explicit feature
  selections in downstream manifests and build commands.
- Change `Decomposition::has_v()` from `bool` to `Result<(), NoVMatrixError>`.
  It returns `Ok(())` when V is retained in a nonempty decomposition. Replace
  Boolean presence checks with `.is_ok()`, or match the error when the reason
  matters.
- Replace the unit struct `NoVMatrixError` with an enum containing
  `EmptyDecompositionError` and `VMatrixDiscardedError`. An empty decomposition
  reports the former regardless of the original `maintain_v` setting; a nonempty
  decomposition without V reports the latter. Update error construction and
  pattern matches accordingly.
- Make the Python attributes of `PersistenceDiagram` and
  `PersistenceDiagramWithReps` read-only. Changes to returned sets or lists do
  not update the diagram. `LoPhatOptions` attributes remain writable.

### Fixes

- Fix panics when serializing or copying an empty decomposition to
  `DecompositionFileFormat`. Empty decompositions are normalized to an empty R
  matrix and absent V (`v: None`), without querying a nonexistent first column.
  Their original `maintain_v` setting is not preserved. Nonempty decompositions,
  including those containing a zero R column, still preserve whether V was kept.
- Return `EmptyDecompositionError` from `get_v_col` on empty decompositions across
  the serial, locking, lock-free, and file-format implementations. R-column
  indexing remains bounds-checked by a panic; an empty decomposition has no valid
  R-column index.
- Consume Python matrix iterables once instead of retrying extraction after a
  partial failure. Input exceptions propagate to the caller rather than becoming
  a panic during conversion. A caller retrying a failed call must supply a fresh
  iterator.
- Snapshot Python options before consuming the matrix, so changes made during
  iteration do not affect the computation. The supplied options are not modified,
  including when computing representatives requires V to be retained.
- Export the diagram classes from `lophat` and set the correct module names on
  public Python classes.
- Raise `NotImplementedError` for unsupported comparisons between
  `PersistenceDiagram` objects instead of `PanicException`; `==` compares the
  paired and unpaired sets.
- Exclude the private Python bindings module from Rust documentation builds, so
  its Python examples are not compiled as Rust doctests.
- Select native ARM64 Python on Windows explicitly through `UV_PYTHON_ARCH`,
  preventing an x64 interpreter from being used to build and test ARM64 wheels.

### Concurrency and performance

- Support free-threaded Python without enabling the GIL when importing LoPhat.
- Detach from the Python interpreter during reduction and diagram extraction,
  allowing independent calls to compute concurrently. Input is collected before
  reduction; concurrent calls must use separate iterators and keep input columns
  unchanged while they are being read.

### Documentation and development

- Document the public Rust and Python APIs separately, with inline Rust comments
  and Python docstrings whose reStructuredText formatting is preserved by rustfmt.
- Add Python API links, examples, and separate exception sections. Clarify column
  ordering, row bounds, clearing requirements, representative cycles, empty input,
  and concurrent use. Invalid clearing inputs are not validated and may produce
  incorrect results or panic.
- Generate Python type stubs and their docstrings with Maturin instead of
  maintaining `lophat.pyi` by hand.
- Upgrade PyO3 from 0.25.1 to 0.29.3 and bit-set from 0.8 to 0.11.1. Require
  Maturin 1.15.0 or newer, and add `macro_rules_attribute` for docstring handling.
- Add Pyrefly as a development dependency and formatting configuration for
  rustfmt, Ruff, Tombi, and Lefthook.
- Move GUDHI, NumPy, tadasets, and pytest-benchmark into an optional `benchmark`
  dependency group. The core `test` group requires only pytest; benchmark tests
  are skipped when pytest-benchmark is unavailable.
- Add regression tests for empty decomposition serialization, V availability,
  diagram attributes and comparisons, options snapshots, and concurrent calls,
  including free-threaded Python.
- Replace the Python CI workflow with uv and Maturin builds that test installed
  wheels and verify generated stubs on Linux and Windows x64/ARM64 and macOS
  ARM64. Select stable CPython versions from `requires-python`, including supported
  free-threaded builds. Run on pushes, pull requests, and manual dispatch; for
  published releases targeting `main`, upload the tested wheels and publish to
  PyPI only after all builds and tests succeed.
- Configure locked release builds and stub generation in `pyproject.toml`.
- Add Abhinav Natarajan to the license and documentation author credits.
