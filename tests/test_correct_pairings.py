from lophat import compute_pairings, compute_pairings_with_reps


def test_2_simplex() -> None:
	matrix = [
		(0, []),
		(0, []),
		(0, []),
		(1, [0, 1]),
		(1, [0, 2]),
		(1, [1, 2]),
		(2, [3, 4, 5]),
	]
	dgm = compute_pairings(matrix)
	assert dgm == {0: None, 1: 3, 2: 4, 5: 6}
	dgm_with_reps, representatives = compute_pairings_with_reps(matrix)
	assert dgm_with_reps == dgm
	assert representatives.keys() == dgm.keys()
