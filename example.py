from lophat import compute_pairings, compute_pairings_serial

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
# Specify that we want to use 4 threads
dgm_par = compute_pairings(iter(matrix), num_threads=4)
dgm_serial = compute_pairings_serial(iter(matrix))

print("Parallel:")
print(dgm_par.unpaired)
print(dgm_par.paired)

print("Serial:")
print(dgm_serial.unpaired)
print(dgm_serial.paired)

assert dgm_par.unpaired == dgm_serial.unpaired
assert dgm_par.paired == dgm_serial.paired
