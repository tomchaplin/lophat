"""Check birth-to-death dictionaries and representatives keyed by birth."""

import pytest
from lophat import LoPhatOptions, compute_pairings, compute_pairings_with_reps


@pytest.mark.parametrize("anti_transpose", [False, True])
@pytest.mark.parametrize(
	("matrix", "expected"),
	[
		([], {}),
		([(0, [])], {0: None}),
		([(0, []), (0, []), (1, [0, 1])], {0: None, 1: 2}),
	],
)
def test_diagrams_are_birth_to_death_dicts(
	matrix: list[tuple[int, list[int]]],
	expected: dict[int, int | None],
	*,
	anti_transpose: bool,
) -> None:
	diagram = compute_pairings(matrix, anti_transpose=anti_transpose)
	assert isinstance(diagram, dict)
	assert diagram == expected
	assert dict(diagram.items()) == expected
	for birth, death in expected.items():
		assert diagram[birth] == death
	with pytest.raises(KeyError):
		diagram[len(matrix)]


def test_dictionary_results_are_independent() -> None:
	matrix = [(0, []), (0, []), (1, [0, 1])]
	diagram = compute_pairings(matrix)
	diagram[0] = 2
	diagram.pop(1)
	assert compute_pairings(matrix) == {0: None, 1: 2}


def test_dictionary_equality_compares_births_and_deaths() -> None:
	matrix = [(0, []), (0, []), (1, [0, 1])]
	diagram = compute_pairings(matrix)
	assert diagram == compute_pairings(matrix)
	assert diagram != compute_pairings([*matrix, (0, [])])
	assert diagram != {0: None, 1: None}


@pytest.mark.parametrize("maintain_v", [False, True])
@pytest.mark.parametrize(
	("matrix", "expected", "expected_representatives"),
	[
		([], {}, {}),
		([(0, [])], {0: None}, {0: [0]}),
		([(0, []), (0, []), (1, [0, 1])], {0: None, 1: 2}, {0: [0], 1: [0, 1]}),
		([(1, []), (2, [0])], {0: 1}, {0: [0]}),
	],
)
def test_representatives_are_keyed_by_birth(
	matrix: list[tuple[int, list[int]]],
	expected: dict[int, int | None],
	expected_representatives: dict[int, list[int]],
	*,
	maintain_v: bool,
) -> None:
	options = LoPhatOptions(maintain_v=maintain_v, num_threads=1)
	result = compute_pairings_with_reps(matrix, options)
	assert isinstance(result, tuple)
	assert len(result) == 2
	diagram, representatives = result
	assert isinstance(diagram, dict)
	assert isinstance(representatives, dict)
	assert diagram == expected
	assert representatives == expected_representatives
	assert diagram.keys() == representatives.keys()
	assert options.maintain_v is maintain_v


def test_representative_results_are_independent() -> None:
	matrix = [(0, []), (0, []), (1, [0, 1])]
	diagram, representatives = compute_pairings_with_reps(matrix)
	representatives[1].clear()
	representatives.pop(0)
	assert diagram == {0: None, 1: 2}
	diagram.clear()
	assert compute_pairings_with_reps(matrix) == ({0: None, 1: 2}, {0: [0], 1: [0, 1]})
