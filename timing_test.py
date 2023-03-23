import random
import math
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
import pandas as pd
from lophat import compute_pairings, LoPhatOptions
from gudhi import RipsComplex
import time
import pickle

random.seed(42)
np.random.seed(42)

N = 250
N_nice = 20
max_diagram_dim = 1
jitter_strength = 0.05
truncation = math.sqrt(2)


def get_jitterer_circle_point(phase, jit):
    random_phase = random.random() * jit
    return [
        0.7 * math.cos(2 * math.pi * (phase + random_phase)),
        0.7 * math.sin(2 * math.pi * (phase + random_phase)),
    ]


nice_points = np.array(
    [get_jitterer_circle_point(i / N_nice, jitter_strength) for i in range(N_nice)]
)

random_points = np.random.rand(N, 2) * 2 - 1

pts = np.vstack((nice_points, random_points))


rcomp = RipsComplex(points=pts, max_edge_length=truncation)


# Build simplex tree
simplex_tree = rcomp.create_simplex_tree(max_dimension=max_diagram_dim + 1)
# Build second simplex tree with index as filtration value
s_tree2 = simplex_tree.copy()
entrance_times = []
dimensions = []
for idx, f_val in enumerate(simplex_tree.get_filtration()):
    s_tree2.assign_filtration(f_val[0], idx)
    entrance_times.append(f_val[1])
    dimensions.append(len(f_val[0]) - 1)


def get_sparse_boundary(smplx):
    return (
        len(smplx),
        sorted([int(face_idx) for _, face_idx in s_tree2.get_boundaries(smplx)]),
    )


chunk_sizes = [
    10,
    100,
    500,
    1000,
    5000,
    10000,
    20000,
    50000,
]

data = []
N_runs = 50

print("Starting runs")
clearing_opts = LoPhatOptions(min_chunk_len=10000, clearing=True)
no_clearing_opts = LoPhatOptions(min_chunk_len=10000, clearing=False)

matrix = [get_sparse_boundary(f_val[0]) for f_val in s_tree2.get_filtration()]
tic1 = time.time()
diagram_clr = compute_pairings(matrix, clearing_opts)
tic2 = time.time()
print(tic2 - tic1)

matrix = [get_sparse_boundary(f_val[0]) for f_val in s_tree2.get_filtration()]
tic3 = time.time()
diagram_clr = compute_pairings(matrix, no_clearing_opts)
tic4 = time.time()
print(tic4 - tic3)
