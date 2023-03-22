from lophat import compute_pairings, LoPhatOptions

# Note that I don't tell lophat what dimension my columns are
matrix = [
    [],
    [],
    [],
    [],
    [0, 1],
    [0, 2],
    [1, 2],
    [0, 3],
    [1, 3],
    [2, 3],
    [4, 7, 8],
    [5, 7, 9],
    [6, 8, 9],
    [4, 5, 6],
]

# Can pass in matrix either as List[List[int]] or Iterator[List[int]]

dgm_iter = compute_pairings(matrix)
dgm_list = compute_pairings(iter(matrix))

# Can optionally provide a LoPhatOptions
# Don't maintain V, use 4 threads, assume matrix is square,
# ensure each thread gets at least 2 colums at a time
opts = LoPhatOptions(False, 4, None, 2)
dgm_custom = compute_pairings(matrix, opts)


print("Iterator:")
print(dgm_iter)

print("List:")
print(dgm_list)

print("Custom:")
print(dgm_custom)

assert dgm_iter == dgm_custom
assert dgm_iter == dgm_list
