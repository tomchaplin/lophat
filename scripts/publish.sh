#!/bin/bash
sudo systemctl start docker
sudo docker run \
    --env-file .env \
    --rm \
    -v $(pwd):/io \
    ghcr.io/pyo3/maturin \
    publish -f --skip-existing
