Python bindings
===============

LoPhat computes persistence pairings over the field with two elements.
Build and install the extension before building this reference::

    maturin develop --generate-stubs

The API documentation below is read from the installed extension's docstrings.
Type stubs are generated from the same bindings. For implementation details,
see `the repository <https://github.com/tomchaplin/lophat>`_.

.. automodule:: lophat

.. autofunction:: lophat.compute_pairings

.. autofunction:: lophat.compute_pairings_with_reps

.. autoclass:: lophat.LoPhatOptions
   :members:

``compute_pairings`` returns a :py:class:`dict` mapping each birth index to
its death index, or ``None`` for an essential feature. For example,
``{0: None, 1: 2}`` describes one essential feature and one finite interval.
Use ``diagram[birth]`` for lookup and ``diagram.items()`` for iteration.
An empty matrix returns ``{}``. The dictionary is independent of other results.

``compute_pairings_with_reps`` returns ``(diagram, representatives)``. Both
:py:class:`dict` objects have the same birth keys. ``representatives[birth]``
is a list of nonzero basis indices forming that feature's representative cycle
over the field with two elements. Match intervals and cycles by key. An empty
matrix returns ``({}, {})``.
