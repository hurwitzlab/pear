#!/bin/bash

set -u

IN_DIR="$WORK/tricho/fastq"
OUT_DIR="$WORK/tricho/paired"
QUEUE="normal"
TIME="24:00:00"

sbatch -A iPlant-Collabs -N 1 -n 1 -t "$TIME" -p "$QUEUE" -J pear \
    run.sh -d "$IN_DIR" -o "$OUT_DIR"
