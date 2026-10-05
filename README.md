<div align="center">

<img src="https://github.com/tomchaplin/lophat/raw/main/docs/lophat_logo.svg" alt="lophat logo" width="300" role="img">

<h1>LoPHAT</h1>

<b>Lo</b>ckfree <b>P</b>ersistent <b>H</b>omology <b>A</b>lgorithm <b>T</b>oolbox

[![crates.io](https://img.shields.io/crates/v/lophat)](https://crates.io/crates/lophat)
[![PyPi](https://img.shields.io/pypi/v/lophat)](https://pypi.org/project/lophat/)
[![docs.rs](https://img.shields.io/docsrs/lophat?label=Docs.rs)](https://docs.rs/lophat/latest/lophat/)
[![Read the Docs](https://img.shields.io/readthedocs/lophat?label=Read%20The%20Docs)](https://lophat.readthedocs.io/en/latest/)

Try in: [Your Browser](https://lophat.tomchaplin.xyz/) • [Google Colab](https://colab.research.google.com/drive/1y0_wZfvuUZfRreYPO50mo4rBlflkMcfj?usp=sharing)

</div>

## Overview

LoPHAT is a Rust library implementing the lockfree algorithm for computing persistent homology (PH), introduced in [[1]](#1).
Python bindings are provided via PyO3, with an interface familiar to those who have used PHAT [[2]](#2).

The primary goal of this library is to make the algorithm accessible to those wishing to compute PH of ___arbitrary filtered chain complexes___.
In particular, LoPHAT is **not** specialised to compute PH of common filtrations or even filtered simplicial complexes.
As such, you should expect LoPHAT to under-perform as compared to [giotto-ph [3]](#3) or [oineus  [4]](#4), both of which use the algorithm of [[1]](#1).

The only changes from the algorithm described in [[1]](#1) are:
* We use the `pinboard` library for epoch-based memory management of the matrices.
* We store the $j^{th}$ column of $R$ and $V$ alongside each other in memory, allowing a full $R=DV$ decomposition (rather than just computing pairings).
* We additionally employ the clearing optimisation [[5]](#5) and provide methods for anti-transpotion (so as to compute persistent cohomology).
* We distribute chunks via work-stealing, using the `rayon` library.

> **Warning**
> LoPHAT is currently in beta.
> The implementation is not optimised, the API is not fixed and tests are limited.

## Usage in Rust

Install with
```shell
cargo add lophat
```
`PersistenceDiagram` wraps a `HashMap<usize, ExtendedUsize>` and exposes map operations through `Deref` and `DerefMut`.
`ExtendedUsize::Finite(index)` represents a finite death; `ExtendedUsize::Infinity` represents an essential feature.
Use `Decomposition::diagram()` or `PersistenceDiagram::from_decomposition()` to extract intervals, and `map_idxs()` to reindex their endpoints.

For usage, please consult [the Rust docs](https://docs.rs/lophat/latest/lophat/).

## Usage in Python

The Python bindings can be installed via
```shell
pip install lophat
```
If this fails, it is probably `pip` trying to install from source without a `cargo` toolchain present.
To force installing from binary run
```shell
pip install --only-binary lophat lophat
```
`compute_pairings` returns a dictionary mapping birth column indices to death column indices, with `None` for features that persist beyond the supplied filtration.
For example, `{0: None, 1: 2}` contains an essential feature born at index 0 and a feature born at index 1 that dies at index 2.
Use `diagram[birth]` for lookup and `diagram.items()` to iterate over intervals.
`compute_pairings_with_reps` returns `(diagram, representatives)`: the same birth-to-death dictionary and a second dictionary mapping each birth index to its representative cycle.
A cycle is a list of nonzero input basis indices over the field with two elements. The two dictionaries have identical keys; use a birth key to match an interval to its representative.
Empty input returns `({}, {})`.
Both functions use the lockfree algorithm of [[1]](#1).
Pass a `LoPhatOptions` object to configure the worker count, clearing, and other options; `num_threads=1` runs the same algorithm with one worker.

For more details, please consult [the Python docs](https://lophat.readthedocs.io/en/latest/).
For example usage, see `examples/main.py` or [this Google colab notebook](https://colab.research.google.com/drive/1y0_wZfvuUZfRreYPO50mo4rBlflkMcfj?usp=sharing).

## Building documentation and type stubs

With a virtual environment active and Maturin 1.15 or newer installed, build the extension and generate its type stubs with:

```shell
maturin develop --generate-stubs
```

For distributable wheels, use `maturin build --generate-stubs`. The release workflow supplies this flag automatically. Generated `.pyi` files are build artifacts; do not maintain a separate handwritten stub.

Rust APIs use ordinary inline Markdown doc comments (`///`), which rustfmt formats, including code examples. Shared APIs supply Python documentation through `#[doc = python_doc!(r#"Python reStructuredText"#)]` attributes; rustfmt skips these macro invocations to preserve their indentation.

Before PyO3 processes a shared class, `#[pymethods]` block, or inline Python module, a conditional `macro_rules_apply(strip_rust_docs!)` pass removes Rust doc comments, including those on fields and methods. Both the adapter alias and the stripping macro live in `src/utils/mod.rs`; the adapter comes from the optional `macro_rules_attribute` dependency. The stripping pass is disabled for Rustdoc, local rust-analyzer analysis, and builds without `python-module`. In those contexts, `python_doc!` expands to an empty string. Extension builds retain the Python documentation for runtime docstrings and generated stubs. The recursive stripping macro uses a crate recursion limit of 1024.

Build the Rust reference with `cargo doc --no-deps`. After installing Sphinx and `sphinx-rtd-theme` in the same Python environment as the extension, build the Python reference with:

```shell
python -m sphinx -W -b html docs docs/_build/html
```

Sphinx reads the installed extension's docstrings, so rebuild the extension after editing them.

## Publishing Rust releases

The `Rust crate` GitHub Actions workflow publishes to crates.io when a GitHub release is published with `main` as its target. It checks out the release tag and runs Rust unit tests and doctests before publishing the version recorded in `Cargo.toml`. Update that version before creating the release tag.

Configure the repository secret `CARGO_REGISTRY_TOKEN` with a crates.io API token authorized to publish `lophat`. The token is supplied only to the publishing step. Tests and package verification enable `local_thread_pool` and `serde` without requiring Python; the crate's published default features are unchanged.

## TODO

- [ ] Change options struct for each algorithm
- [ ] Decide on new Python bindings
- [ ] Increase property testing
- [ ] Write unit tests
- [ ] Write integration tests (testing V)
- [ ] Benchmark
- [ ] Abstract out matrix trait?
- [ ] Reduce memory usage when V not maintained
- [ ] Add contributing guide
- [ ] Fix locking in pivots vector
- [ ] Github actions

## References

<a id="1">[1]</a> Morozov, Dmitriy, and Arnur Nigmetov.
"Towards lockfree persistent homology."
Proceedings of the 32nd ACM Symposium on Parallelism in Algorithms and Architectures. 2020.

<a id="2">[2]</a> Bauer, Ulrich, et al.
"Phat–persistent homology algorithms toolbox." Journal of symbolic computation 78 (2017): 76-90.
[Bitbucket](https://bitbucket.org/phat-code/phat/src/master/)

<a id="3">[3]</a> Pérez, Julián Burella, et al.
"giotto-ph: A python library for high-performance computation of persistent homology of Vietoris-Rips filtrations."
arXiv preprint [arXiv:2107.05412](https://arxiv.org/abs/2107.05412) (2021).
[GitHub](https://github.com/giotto-ai/giotto-ph)

<a id="4">[4]</a> Nigmetov, Arnur, Morozov, Dmitriy, and USDOE.
Oineus v1.0. Computer software.
[https://www.osti.gov//servlets/purl/1774764](https://www.osti.gov//servlets/purl/1774764). USDOE. 1 Apr. 2021.
Web. [doi:10.11578/dc.20210407.1](https://doi.org/10.11578/dc.20210407.1). [GitHub](https://github.com/anigmetov/oineus)

<a id="5">[5]</a> Bauer, Ulrich, Michael Kerber, and Jan Reininghaus.
"Clear and compress: Computing persistent homology in chunks."
Topological Methods in Data Analysis and Visualization III: Theory, Algorithms, and Applications.
Springer International Publishing, 2014.
