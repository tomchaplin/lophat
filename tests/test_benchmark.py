from collections.abc import Iterator

import pytest

pytest.importorskip(
	"pytest_benchmark",
	reason="install the benchmark dependency group",
	exc_type=ModuleNotFoundError,
)

from collections.abc import Generator

import numpy as np
import tadasets
from gudhi import RipsComplex
from pytest_benchmark.fixture import BenchmarkFixture

from lophat import LoPhatOptions, compute_pairings

type AnnotatedColumn = tuple[int, list[int]]


def rips_bdry_matrix(
	pts: np.ndarray[tuple[int, int], np.dtype[np.floating]],
) -> Generator[AnnotatedColumn]:
	# Build rips complex
	rcomp = RipsComplex(points=pts, max_edge_length=100)
	# Build simplex tree (only want 2-skeleton)
	simplex_tree = rcomp.create_simplex_tree(max_dimension=3)
	# Build second simplex tree with index as filtration value
	s_tree2 = simplex_tree.copy()
	for idx, f_val in enumerate(simplex_tree.get_filtration()):
		s_tree2.assign_filtration(f_val[0], idx)

	# Build up matrix to pass to phimaker
	def compute_annotated_col(smplx: list[int]) -> AnnotatedColumn:
		sparse_bdry = [int(face_idx) for _, face_idx in s_tree2.get_boundaries(smplx)]
		dimension = 0 if len(sparse_bdry) == 0 else len(sparse_bdry) - 1
		return (dimension, sorted(sparse_bdry))

	return (compute_annotated_col(smplx) for smplx, _ in s_tree2.get_filtration())


def torus_boundary_matrix() -> np.ndarray:
	return tadasets.torus(n=100, c=2, a=1)


pts = tadasets.torus(n=100, c=2, a=1, seed=42)
matrix = list(rips_bdry_matrix(pts))
n_threads_range = list(range(1, 9))


@pytest.fixture(params=n_threads_range)
def n_threads(request: pytest.FixtureRequest) -> int:
	return request.param


def func_to_bench(
	col_iter: Iterator[AnnotatedColumn] | list[AnnotatedColumn],
	n_threads: int,
) -> None:
	options = LoPhatOptions(num_threads=n_threads)
	compute_pairings(col_iter, options=options)


def test_torus_iter(benchmark: BenchmarkFixture, n_threads: int) -> None:
	def setup() -> tuple[tuple[Generator[AnnotatedColumn], int], dict]:
		col_iter = (col for col in matrix)
		return (col_iter, n_threads), {}

	benchmark.pedantic(func_to_bench, setup=setup)


def test_torus_vec(benchmark: BenchmarkFixture, n_threads: int) -> None:
	def setup() -> tuple[tuple[list[AnnotatedColumn], int], dict]:
		col_iter = list(matrix)
		return (col_iter, n_threads), {}

	benchmark.pedantic(func_to_bench, setup=setup)
