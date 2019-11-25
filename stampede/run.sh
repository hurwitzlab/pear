#!/bin/bash

set -u

IMG="/work/05066/imicrobe/singularity/pear-0.9.11.img"

if [[ ! -e "$IMG" ]]; then
    echo "Missing Singularity image \"$IMG\""
    exit 1
fi

singularity exec $IMG run_pear "$@" -o "pear-out"

echo "Comments to kyclark@email.arizona.edu"
