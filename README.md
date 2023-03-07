<div align="center">

<h1>LoPHAT</h1>

<b>Lo</b>ckfree <b>P</b>ersistent <b>H</b>omology <b>A</b>lgorithm <b>T</b>oolbox

</div>

## Overview

LoPHAT is a Rust library implementing the lockfree algorithm for computing persistent homology (PH), introduced in [[1]](#1).
Python bindings are provided via PyO3, with an interface familiar to PHAT's.

The goal of this library is to make the algorithm accessible to those wishing to compute PH of arbitrary filtered chain complexes.
In particular, LoPHAT is **not** specialised to compute PH of common filtrations or even filtered simplicial complexes.
As such, you should expect LoPHAT to under-perform as compared to [giotto-ph [2]](#2) or [oineus  [3]](#3), both of which use the algorithm of [[1]](#1).

> **Warning**
> LoPHAT is currently in alpha.
> The implementation is not optimised, the API is not fixed and no tests have been written.
> Use at your own risk.

## References

<a id="1">[1]</a> Morozov, Dmitriy, and Arnur Nigmetov.
"Towards lockfree persistent homology."
Proceedings of the 32nd ACM Symposium on Parallelism in Algorithms and Architectures. 2020.

<a id="2">[2]</a> Pérez, Julián Burella, et al.
"giotto-ph: A python library for high-performance computation of persistent homology of Vietoris-Rips filtrations."
arXiv preprint [arXiv:2107.05412](https://arxiv.org/abs/2107.05412) (2021).
[GitHub](https://github.com/giotto-ai/giotto-ph)

<a id="3">[3]</a> Nigmetov, Arnur, Morozov, Dmitriy, and USDOE.
Oineus v1.0. Computer software.
[https://www.osti.gov//servlets/purl/1774764](https://www.osti.gov//servlets/purl/1774764). USDOE. 1 Apr. 2021.
Web. [doi:10.11578/dc.20210407.1](https://doi.org/10.11578/dc.20210407.1). [GitHub](https://github.com/anigmetov/oineus)
