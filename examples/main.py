from lophat import compute_pairings, compute_pairings_with_reps, LoPhatOptions

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

# Can pass in matrix either as List[...] or Iterator[...]

dgm_list = compute_pairings(matrix)
dgm_iter = compute_pairings(iter(matrix))

# Can optionally provide a LoPhatOptions
# Don't maintain V, use 4 threads, assume matrix is square,
# ensure each thread gets at least 2 colums at a time,
# turn off clearing optimisation
opts = LoPhatOptions(
    maintain_v=False, num_threads=4, column_height=None, min_chunk_len=2, clearing=False
)
# Don't anti-transpose matrix before computing pairings
dgm_custom = compute_pairings(matrix, anti_transpose=False, options=opts)

print("Iterator:")
print(dgm_iter)

print("\nList:")
print(dgm_list)

print("\nCustom:")
print(dgm_custom)

print("\nWith representatives:")
dgm_with_reps = compute_pairings_with_reps(matrix)
print("Paired: ", end="")
print(dgm_with_reps.paired)
print("Reps: ", end="")
print(dgm_with_reps.paired_reps)
print("Unpaired: ", end="")
print(dgm_with_reps.unpaired)
print("Reps: ", end="")
print(dgm_with_reps.unpaired_reps)

assert dgm_iter == dgm_custom
assert dgm_iter == dgm_list
assert dgm_iter.paired == set(dgm_with_reps.paired)
assert dgm_iter.unpaired == set(dgm_with_reps.unpaired)
