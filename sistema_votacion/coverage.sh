#!/bin/bash
cargo tarpaulin --target-dir target/tarpaulin/artifacts --skip-clean "$@"
