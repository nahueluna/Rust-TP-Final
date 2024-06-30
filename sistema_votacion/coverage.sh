#!/bin/bash
cp lib.rs lib_bk.rs
sed -i 's/#\[ink(message)\]/\/\/&/' lib.rs
sed -i 's/impl SistemaVotacion {/&\n#\[ink(message)\]\npub fn dummy(\&self) {}/' lib.rs
cargo tarpaulin --target-dir target/tarpaulin/artifacts --skip-clean
mv lib_bk.rs lib.rs
