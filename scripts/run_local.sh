#!/usr/bin/bash
# ======================================================================
# Example usage: DATA=data/wikitext-2-raw source scripts/run_local.sh
# ======================================================================

DATA_PATH="${DATA:-data}"

RUST_BACKTRACE=1 ./target/release/rusty-dawg \
    --train-path "$DATA_PATH/$1/wiki.train.raw" \
    --test-path "$DATA_PATH/$1/wiki.valid.raw" \
    --save-path "" \
    --results-path "" \
    --n-eval 0 \
    --nodes-ratio 1.25 \
    --edges-ratio 2.20 \
    --tokenizer gpt2 \
    --utype u16
    # --cdawg \
    # --train-vec-path "/Users/willm/Desktop/train.vec"
    # --disk-path "/tmp/$1-dawg" \
