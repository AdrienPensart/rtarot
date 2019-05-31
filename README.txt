French Tarot

Implemented with :
- rules : http://www.fftarot.fr/assets/documents/R-RO201206.pdf
- traductions : https://en.wikipedia.org/wiki/French_tarot

Usage :
- coverage :
RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin
RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo tarpaulin -v
- test : cargo test -- --nocapture
- clippy : cargo clippy --all-targets --all-features -- -D warnings

Todo :
- type games : defense, attack, petit hunt, full assets
- add colors
- game managing
    - flag on/off
    - cut game
    - random reunion
- duplicate
