# Changelog

## 0.13.0 (unreleased)

Changes since 0.12.0.

- Add `Decomposition::diagram_with_reps` for Rust callers, returning the diagram
  alongside a result containing representative cycles keyed by birth. Finite
  intervals use R at death, while essential intervals use V at birth. Preserve
  the diagram on V availability errors and document the decomposition's
  uniform V availability invariant.
- Use `Decomposition::diagram_with_reps` in `compute_pairings_with_reps`,
  preserving the Python API's empty-input result and options behavior.
- Document representative selection, its mathematical assumptions, and error
  behavior, with a Rust usage example. Test cycle lifetimes across algorithms,
  column types, clearing settings, and threading configurations; cover Python
  empty-input results and both supplied `maintain_v` settings.

## 0.12.0

Changes since 0.11.0.

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
- Replace `PersistenceDiagram`'s paired/unpaired sets with a
  `HashMap<usize, ExtendedUsize>` mapping each birth to `Finite(death)` or
  `Infinity`. Rust users can access map operations through `Deref` and
  `DerefMut`, construct diagrams by collecting entries, and reindex endpoints
  with `map_idxs`. `Decomposition::diagram()` uses the shared extraction logic.
- Return a plain `dict[int, int | None]` from Python `compute_pairings`, replacing
  the `PersistenceDiagram` Python class. Use `diagram[birth]` and
  `diagram.items()` instead of `.paired` and `.unpaired`; `None` marks essential
  features. Both owned and borrowed Rust diagrams and extended indices support
  conversion when `python-module` is enabled, with generated return annotations.
- Remove `PersistenceDiagramWithReps`. `compute_pairings_with_reps` now returns
  `(PersistenceDiagram, HashMap<usize, Vec<usize>>)`, converted to two Python
  dictionaries. Both maps use birth indices as keys; match an interval to its
  representative with that key instead of aligned paired/unpaired lists.
  Empty input returns `({}, {})`.

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
- Set the correct module name on `LoPhatOptions`.
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
  dictionary conversion, representatives keyed by birth, options snapshots, and
  concurrent calls, including free-threaded Python.
- Replace the Python CI workflow with uv and Maturin builds that test installed
  wheels and verify generated stubs on Linux and Windows x64/ARM64 and macOS
  ARM64. Select stable CPython versions from `requires-python`, including supported
  free-threaded builds. Run on pushes, pull requests, and manual dispatch; for
  published releases targeting `main`, upload the tested wheels and publish to
  PyPI only after all builds and tests succeed.
- Configure locked release builds and stub generation in `pyproject.toml`.
- Add a Rust workflow that runs tests on pushes, pull requests, manual dispatch,
  and published releases. Publish the crate to crates.io only after tests pass
  for a published release targeting `main`. Correct the crate keywords to
  satisfy crates.io's character and length requirements.
- Add Abhinav Natarajan to the license and documentation author credits.
