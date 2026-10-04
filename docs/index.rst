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

.. autoclass:: lophat.PersistenceDiagram
   :members:

.. autoclass:: lophat.PersistenceDiagramWithReps
   :members:
