Python Bindings
==================================

The current bindings are exposed in the Python lophat package.
For more information, please see `the repository <https://github.com/tomchaplin/lophat>`_.

.. py:currentmodule:: lophat

.. py:function:: compute_pairings(matrix, options= None)

    Decomposes the input matrix, using the lockfree or standard algorithm (according to options).

    :param matrix: The boundary matrix, provided in sparse column format, either as a list of lists or an iterator of lists.
    :type matrix: List[List[int]] | Iterator[List[Int]]
    :param options: Options to control the R=DV decomposition algorithm.
    :type options: LoPhatOptions
    :returns: The persistence pairings read off from the R=DV decomposition.
    :rtype: PersistenceDiagram

.. py:class:: LoPhatOptions(maintain_v = False,num_threads= 0,column_height= None,max_chunk_len= 1)

    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.

    :param maintain_v: Whether to maintain_v during decompositon, usually best left False.
    :type maintain_v: bool 
    :param num_threads: Max number of threads to use. Set at 0 to use all threads. Set at 1 to use standard, serial algorithm.
    :type num_threads: int
    :param column_height: Optional hint to height of columns. If None, assumed that matrix is square.
    :type column_height: int | None
    :param max_chunk_len: Maximum size of a chunk, given to each thread.
    :type max_chunk_len: int

.. py:class:: PersistenceDiagram()

    A class representing the persistence diagram computed by LoPHAT.
    Each column index in the input matrix appears exactly once, either in a pairing or as unpaired.

    :param unpaired: The set of input column indices that were not paired in the R=DV decomposition.
    :type unpaired: Set[int]
    :param paired: The set of (birth, death) pairs of column indices that were paired in the R=DV decomposition.
    :type paired: Set[Tuple[int, int]]

