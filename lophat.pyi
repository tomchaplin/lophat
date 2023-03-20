from typing import Iterator, List, Set, Tuple

def compute_pairings(
    matrix: List[List[int]] | Iterator[List[int]],
    options: LoPhatOptions | None = None,
) -> PersistenceDiagram:
    """
    Computes pairings

    :param matrix: The boundary matrix, provided in sparse column format, either as a list of lists or an iterator of lists.
    :param options: Options to control the R=DV decomposition algorithm.
    """

class LoPhatOptions:
    """
    A class representing the possible options for controling the R=DV decomposition.

    :param maintain_v: Whether to maintain_v during decompositon, usually best left False.
    :param num_threads: Max number of threads to use. Set at 0 to use all threads. Set at 1 to use standard, serial algorithm.
    :param column_height: Optional hint to height of columns. If None, assumed that matrix is square.
    :param min_chunk_len: The minimum number of columns that a thread should be allowed to work on at once.
    """

    def __init__(
        self,
        maintain_v: bool = False,
        num_threads: int = 0,
        column_height: int | None = None,
        min_chunk_len: int = 0,
    ) -> None: ...

class PersistenceDiagram:
    """
    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.
    """

    unpaired: Set[int]
    paired: Set[Tuple[int, int]]
