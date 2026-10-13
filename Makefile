install: dev lock

dev: # install editable project + dev/docs/test dependencies
	uv sync --verbose --all-groups --no-progress

upgrade: # update project dependencies
	uv sync -U --all-groups --verbose --no-progress

lock: # update lockfiles
	uv lock
	uv export --format pylock.toml --all-groups -o pylock.toml --quiet

wheel: lock # build the project wheel
	uv build --no-progress --verbose --wheel

sdist: # build source distribution
	uv build --no-progress --verbose --sdist

tests: dev # run the test suite
	uv run pytest

docs:
	$(MAKE) -C docs html

all: install wheel sdist docs

clean:
	$(MAKE) -C docs clean

.PHONY: install dev upgrade lock wheel sdist tests clean
