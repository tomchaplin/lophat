from typing import Iterator, List, Set, Tuple


def compute_pairings(
    matrix: List[Tuple[int, List[int]]] | Iterator[Tuple[int, List[int]]],
    anti_transpose: bool = True,
    options: LoPhatOptions | None = None,
) -> PersistenceDiagram:
    """
    Decomposes the input matrix, using the lockfree algorithm.

    :param matrix: The boundary matrix, provided in sparse column format. Each column is a tuple of (dimension, boundary) where boundary is the list of non-zero indices.
    :param anti_transpose: Whether to anti-transpose the matrix first. Best left True with clearing on. Set to False if input matrix non-square.
    :param options: Options to control the R=DV decomposition algorithm.
    :returns: The persistence pairings read off from the R=DV decomposition.
    """


def compute_pairings_with_reps(
    matrix: List[Tuple[int, List[int]]] | Iterator[Tuple[int, List[int]]],
    options: LoPhatOptions | None = None,
) -> PersistenceDiagramWithReps:
    """
    Decomposes the input matrix, using the lockfree algorithm.
    Additionally returns representatives of the pairings found.
    Note that options will be overwritten to ensure that V is maintained in the decomposition.

    :param matrix: The boundary matrix, provided in sparse column format. Each column is a tuple of (dimension, boundary) where boundary is the list of non-zero indices.
    :param options: Options to control the R=DV decomposition algorithm.
    :returns: The persistence pairings read off from the R=DV decomposition.
    """


class LoPhatOptions:
    """
    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.

    :param maintain_v: Whether to maintain_v during decompositon, usually best left False.
    :param num_threads: Max number of threads to use. Set at 0 to use all threads.
    :param column_height: Optional hint to height of columns. If None, assumed that matrix is square.
    :param min_chunk_len: When splitting work, don't reduce chunks to smaller than this size.
    :param clearing: Whether to employ the clearing optimisation. Cannot use if input non-square.
    """

    def __init__(
        self,
        maintain_v: bool = False,
        num_threads: int = 0,
        column_height: int | None = None,
        min_chunk_len: int = 1,
        clearing: bool = True,
    ) -> None:
        ...


class PersistenceDiagram:
    """
    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.

    :param unpaired: The set of input column indices that were not paired in the R=DV decomposition.
    :param paired: The set of (birth, death) pairs of column indices that were paired in the R=DV decomposition.
    """

    unpaired: Set[int]
    paired: Set[Tuple[int, int]]


class PersistenceDiagramWithReps:
    """
    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.
    For each (paired or unpaired) feature, a representative is stored in the same index in the corresponding list of representatives.

    :param unpaired: The list of input column indices that were not paired in the R=DV decomposition.
    :param unpaired_reps: A list of representatives for each of the unpaired features.
    :param paired: The list of (birth, death) pairs of column indices that were paired in the R=DV decomposition.
    :param paired_reps: A list of representatives for each of the paired features.
    """

    unpaired: List[int]
    unpaired_reps: List[List[int]]
    paired: List[Tuple[int, int]]
    paired_reps: List[List[int]]
