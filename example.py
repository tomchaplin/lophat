from lophat import (
    compute_pairings,
    LoPhatOptions,
)

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

# Note we pass iter(matrix)
dgm_default = compute_pairings(iter(matrix))
# Don't maintain V, use 4 threads, assume matrix is square,
# ensure each thread gets at least 2 colums at a time
opts = LoPhatOptions(False, 4, None, 2)
dgm_custom = compute_pairings(iter(matrix), opts)

dgm_no_iter = compute_pairings(matrix)

print("Default:")
print(dgm_default.unpaired)
print(dgm_default.paired)

print("Custom:")
print(dgm_custom.unpaired)
print(dgm_custom.paired)

print("No iter:")
print(dgm_no_iter.unpaired)
print(dgm_no_iter.paired)

assert dgm_default.unpaired == dgm_custom.unpaired
assert dgm_default.paired == dgm_custom.paired

assert dgm_default.unpaired == dgm_no_iter.unpaired
assert dgm_default.paired == dgm_no_iter.paired
