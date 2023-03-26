from lophat import compute_pairings, LoPhatOptions

# Note that I have to tell lophat what dimension my columns are
# This information is used for the clearing optimisation
matrix = [
    (0, []),
    (0, []),
    (0, []),
    (0, []),
    (1, [0, 1]),
    (1, [0, 2]),
    (1, [1, 2]),
    (1, [0, 3]),
    (1, [1, 3]),
    (1, [2, 3]),
    (2, [4, 7, 8]),
    (2, [5, 7, 9]),
    (2, [6, 8, 9]),
    (2, [4, 5, 6]),
]

# Can pass in matrix either as List[List[int]] or Iterator[List[int]]

dgm_iter = compute_pairings(matrix, anti_transpose=False)
dgm_list = compute_pairings(iter(matrix), anti_transpose=False)

# Can optionally provide a LoPhatOptions
# Don't maintain V, use 4 threads, assume matrix is square,
# ensure each thread gets at most 2 colums at a time
opts = LoPhatOptions(False, 4, None, 2)
dgm_custom = compute_pairings(matrix, anti_transpose=False, options=opts)

dgm_at = compute_pairings(matrix, options=opts)

print("Iterator:")
print(dgm_iter)

print("List:")
print(dgm_list)

print("Custom:")
print(dgm_custom)

print("Anti-Transpose:")
print(dgm_at)

assert dgm_iter == dgm_custom
assert dgm_iter == dgm_list
assert dgm_iter == dgm_at
