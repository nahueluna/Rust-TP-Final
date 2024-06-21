#!/bin/bash
cargo contract build --manifest-path "sistema_votacion/Cargo.toml" "$@"
cargo contract build --manifest-path "contrato_reportes/Cargo.toml" "$@"
