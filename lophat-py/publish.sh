#!/bin/bash
sudo docker run --env-file .maturin_env --rm -v $(pwd):/io ghcr.io/pyo3/maturin publish
