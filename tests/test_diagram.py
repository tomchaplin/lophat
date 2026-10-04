"""Check diagram equality and exceptions for unsupported comparisons."""

import operator
from collections.abc import Callable

import pytest
from lophat import (
	PersistenceDiagram,
	PersistenceDiagramWithReps,
	compute_pairings,
	compute_pairings_with_reps,
)


@pytest.mark.parametrize(
	("additional_columns", "unchanged_attribute"),
	[
		([(0, []), (1, [0, 3])], "unpaired"),
		([(0, [])], "paired"),
	],
)
def test_equality_compares_both_sets(
	additional_columns: list[tuple[int, list[int]]],
	unchanged_attribute: str,
) -> None:
	matrix = [(0, []), (0, []), (1, [0, 1])]
	diagram = compute_pairings(matrix)
	assert diagram == compute_pairings(matrix)
	other = compute_pairings(matrix + additional_columns)
	assert getattr(diagram, unchanged_attribute) == getattr(other, unchanged_attribute)
	assert not operator.eq(diagram, other)


@pytest.mark.parametrize(
	("compute", "attributes"),
	[
		(compute_pairings, ("paired", "unpaired")),
		(compute_pairings_with_reps, ("paired", "unpaired", "paired_reps", "unpaired_reps")),
	],
)
def test_diagram_attributes_are_read_only(
	compute: Callable[..., PersistenceDiagram | PersistenceDiagramWithReps],
	attributes: tuple[str, ...],
) -> None:
	diagram = compute([(0, []), (0, []), (1, [0, 1])])
	for attribute in attributes:
		expected = getattr(diagram, attribute)
		with pytest.raises(AttributeError, match=attribute):
			setattr(diagram, attribute, expected)
		assert getattr(diagram, attribute) == expected
		returned = getattr(diagram, attribute)
		if attribute.endswith("_reps"):
			returned[0].clear()
		else:
			returned.clear()
		assert getattr(diagram, attribute) == expected


@pytest.mark.parametrize(
	"compare", [operator.ne, operator.lt, operator.le, operator.gt, operator.ge]
)
def test_unsupported_comparisons_raise_not_implemented(
	compare: Callable[[PersistenceDiagram, PersistenceDiagram], bool],
) -> None:
	diagram = compute_pairings([])
	other = compute_pairings([])
	with pytest.raises(NotImplementedError, match="Only equality comparisons are supported"):
		compare(diagram, other)
