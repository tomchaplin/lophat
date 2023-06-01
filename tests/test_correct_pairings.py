from lophat import compute_pairings, compute_pairings_with_reps

def test_2_simplex():
    matrix = [
        (0, []),
        (0, []),
        (0, []),
        (1, [0, 1]),
        (1, [0, 2]),
        (1, [1, 2]),
        (2, [3, 4, 5]),
    ]
    correct_paired = {(1, 3), (2, 4), (5, 6)}
    correct_unpaired = {0}
    dgm = compute_pairings(matrix);
    assert dgm.paired == correct_paired
    assert dgm.unpaired == correct_unpaired
    dgm_with_reps = compute_pairings_with_reps(matrix)
    assert set(dgm_with_reps.paired) == correct_paired
    assert set(dgm_with_reps.unpaired) == correct_unpaired 