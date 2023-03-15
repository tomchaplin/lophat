from lophat import (
    compute_pairings,
    compute_pairings_serial,
    compute_pairings_lock_free,
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
dgm_par = compute_pairings_lock_free(iter(matrix))
dgm_serial = compute_pairings_serial(iter(matrix))
# Don't maintain V, use 4 threads, assume matrix is square
opts = LoPhatOptions(False, 4, None)
dgm_custom = compute_pairings(iter(matrix), opts)

print("Default:")
print(dgm_default.unpaired)
print(dgm_default.paired)

print("Parallel:")
print(dgm_par.unpaired)
print(dgm_par.paired)

print("Serial:")
print(dgm_serial.unpaired)
print(dgm_serial.paired)

print("Custom:")
print(dgm_custom.unpaired)
print(dgm_custom.paired)

assert dgm_par.unpaired == dgm_serial.unpaired
assert dgm_par.paired == dgm_serial.paired

assert dgm_default.unpaired == dgm_serial.unpaired
assert dgm_default.paired == dgm_serial.paired

assert dgm_custom.unpaired == dgm_serial.unpaired
assert dgm_custom.paired == dgm_serial.paired
