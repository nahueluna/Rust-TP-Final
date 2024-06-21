#!/bin/bash
cargo contract build "$@"
cargo contract build --manifest-path "../contrato_reportes/Cargo.toml" "$@"
