French Tarot

Implemented with :
- rules : http://www.fftarot.fr/assets/documents/R-RO201206.pdf
- traductions : https://en.wikipedia.org/wiki/French_tarot

Usage :
- coverage :
  - cargo install cargo-tarpaulin
  - cargo tarpaulin -v
- test :
  - cargo test -- --nocapture
- linting :
  - clippy : cargo clippy --all-targets --all-features -- -D warnings

70.70% coverage, 772/1092 lines covered
