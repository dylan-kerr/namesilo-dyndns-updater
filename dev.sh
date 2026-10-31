#!/bin/bash
watchexec -w src "cargo clippy --all-targets \
    && cargo llvm-cov \
    && cargo llvm-cov report --lcov --output-path lcov.info \
    && cargo run"
