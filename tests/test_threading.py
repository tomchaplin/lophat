"""Exercise independent computations and shared Python objects concurrently."""

import gc
import os
import subprocess
import sys
import sysconfig
from collections.abc import Callable, Generator
from concurrent.futures import ThreadPoolExecutor
from threading import Barrier, Event, Thread

import pytest
from lophat import LoPhatOptions, compute_pairings, compute_pairings_with_reps

type AnnotatedColumn = tuple[int, list[int]]
type Diagram = dict[int, int | None]
type DiagramWithReps = tuple[Diagram, dict[int, list[int]]]

MATRIX = [
	(0, []),
	(0, []),
	(0, []),
	(1, [0, 1]),
	(1, [0, 2]),
	(1, [1, 2]),
	(2, [3, 4, 5]),
]
EXPECTED = {0: None, 1: 3, 2: 4, 5: 6}


def assert_pairings(result: Diagram | DiagramWithReps) -> None:
	diagram = result[0] if isinstance(result, tuple) else result
	assert diagram == EXPECTED


def assert_cycle(representative: list[int]) -> None:
	boundary = set()
	for column in representative:
		boundary.symmetric_difference_update(MATRIX[column][1])
	assert not boundary


@pytest.mark.skipif(
	sysconfig.get_config_var("Py_GIL_DISABLED") != 1,
	reason="requires a free-threaded interpreter",
)
def test_import_keeps_gil_disabled() -> None:
	# Do not force -X gil=0: that would hide an incorrect module declaration.
	env = os.environ.copy()
	env.pop("PYTHON_GIL", None)
	subprocess.run(
		[
			sys.executable,
			"-W",
			"error::RuntimeWarning",
			"-c",
			(
				"import sys; assert not sys._is_gil_enabled(); "
				"import lophat; assert not sys._is_gil_enabled()"
			),
		],
		env=env,
		check=True,
		timeout=30,
	)


@pytest.mark.parametrize("num_threads", [1, 2])
@pytest.mark.parametrize("clearing", [False, True])
@pytest.mark.parametrize("anti_transpose", [False, True])
def test_concurrent_pairings(num_threads: int, *, clearing: bool, anti_transpose: bool) -> None:
	options = LoPhatOptions(num_threads=num_threads, clearing=clearing)
	barrier = Barrier(4)

	def worker(index: int) -> None:
		barrier.wait(timeout=10)
		for _ in range(6):
			# Share unchanged list inputs; give each call its own iterator.
			matrix = MATRIX if index % 2 == 0 else iter(MATRIX)
			assert_pairings(compute_pairings(matrix, anti_transpose, options))

	with ThreadPoolExecutor(max_workers=4) as executor:
		list(executor.map(worker, range(4)))


@pytest.mark.parametrize("num_threads", [1, 2])
@pytest.mark.parametrize("clearing", [False, True])
def test_concurrent_representatives(num_threads: int, *, clearing: bool) -> None:
	options = LoPhatOptions(num_threads=num_threads, clearing=clearing)
	barrier = Barrier(4)

	def worker(index: int) -> None:
		barrier.wait(timeout=10)
		for _ in range(6):
			matrix = MATRIX if index % 2 == 0 else iter(MATRIX)
			diagram, representatives = compute_pairings_with_reps(matrix, options)
			assert_pairings(diagram)
			assert representatives.keys() == diagram.keys()
			for birth, representative in representatives.items():
				assert max(representative) == birth
				assert_cycle(representative)

	with ThreadPoolExecutor(max_workers=4) as executor:
		list(executor.map(worker, range(4)))
	assert options.maintain_v is False


@pytest.mark.parametrize("compute", [compute_pairings, compute_pairings_with_reps])
def test_options_are_snapshotted_before_iteration(compute: Callable) -> None:
	options = LoPhatOptions(num_threads=1, column_height=len(MATRIX))

	def columns() -> Generator[AnnotatedColumn]:
		# A later mutation must neither conflict with a retained borrow nor
		# change the options already copied into the computation.
		options.column_height = 0
		yield from MATRIX

	assert_pairings(compute(columns(), options=options))
	assert options.column_height == 0


def test_computations_during_garbage_collection() -> None:
	options = LoPhatOptions(num_threads=2)
	collecting = Event()
	stop = Event()

	def collect() -> None:
		while not stop.is_set():
			gc.collect()
			collecting.set()
			stop.wait(0.001)

	collector = Thread(target=collect)
	collector.start()
	try:
		assert collecting.wait(timeout=10)
		with ThreadPoolExecutor(max_workers=4) as executor:
			results = list(
				executor.map(
					lambda _: compute_pairings_with_reps(iter(MATRIX), options),
					range(24),
				),
			)
		for result in results:
			assert_pairings(result)
	finally:
		stop.set()
		collector.join(timeout=10)
		assert not collector.is_alive()


@pytest.mark.parametrize("compute", [compute_pairings, compute_pairings_with_reps])
def test_shared_diagram_reads(compute: Callable) -> None:
	diagram = compute(MATRIX, options=LoPhatOptions(num_threads=1))
	expected = compute(MATRIX, options=LoPhatOptions(num_threads=1))
	barrier = Barrier(4)

	def worker(_: int) -> None:
		barrier.wait(timeout=10)
		for _ in range(100):
			assert_pairings(diagram)
			assert diagram == expected
			repr(diagram)

	with ThreadPoolExecutor(max_workers=4) as executor:
		list(executor.map(worker, range(4)))
	assert_pairings(diagram)


def test_shared_options_reads_and_writes() -> None:
	options = LoPhatOptions(num_threads=1)
	barrier = Barrier(4)

	def worker(index: int) -> int:
		barrier.wait(timeout=10)
		successes = 0
		for iteration in range(100):
			try:
				if index == 0:
					options.maintain_v = bool(iteration % 2)
				else:
					compute = compute_pairings if index % 2 else compute_pairings_with_reps
					assert_pairings(compute(iter(MATRIX), options=options))
				successes += 1
			except RuntimeError as error:
				assert "borrow" in str(error).lower()  # ruff: ignore[PT017]
		return successes

	with ThreadPoolExecutor(max_workers=4) as executor:
		successes = list(executor.map(worker, range(4)))
	assert all(count > 0 for count in successes)
