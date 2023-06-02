import tadasets
from gudhi import RipsComplex
from lophat import compute_pairings, LoPhatOptions
import numpy as np
import pytest

n_threads_range = list(range(1, 9))


def rips_bdry_matrix(pts):
    # Build rips complex
    rcomp = RipsComplex(points=pts, max_edge_length=100)
    # Build simplex tree (only want 2-skeleton)
    simplex_tree = rcomp.create_simplex_tree(max_dimension=3)
    # Build second simplex tree with index as filtration value
    s_tree2 = simplex_tree.copy()
    for idx, f_val in enumerate(simplex_tree.get_filtration()):
        s_tree2.assign_filtration(f_val[0], idx)
    # Build up matrix to pass to phimaker
    matrix = []
    for idx, f_val in enumerate(s_tree2.get_filtration()):
        smplx = f_val[0]
        sparse_bdry = [int(face_idx) for _, face_idx in s_tree2.get_boundaries(smplx)]
        if len(sparse_bdry) == 0:
            dimension = 0
        else:
            dimension = len(sparse_bdry) - 1
        annotated_col = (dimension, sorted(sparse_bdry))
        matrix.append(annotated_col)
    return matrix


def torus_boundary_matrix():
    pts = tadasets.torus(n=100, c=2, a=1)
    return pts


@pytest.fixture(params=n_threads_range)
def n_threads(request):
    return request.param


def test_torus(benchmark, n_threads):
    np.random.seed(42)
    pts = torus_boundary_matrix()
    matrix = rips_bdry_matrix(pts)
    options = LoPhatOptions(num_threads=n_threads)

    @benchmark
    def func_to_bench():
        compute_pairings(matrix, options=options)
