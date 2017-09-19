#!/bin/bash

#SBATCH -A iPlant-Collabs
#SBATCH -t 24:00:00
#SBATCH -N 1
#SBATCH -n 1
#SBATCH -J mash
#SBATCH -p normal

set -u

IN_DIR=""
OUT_DIR="$PWD/pear-out"
IMG="pear-0.9.10.img"

function lc() {
  wc -l "$1" | cut -d ' ' -f 1
}

function HELP() {
  printf "Usage:\n  %s -q QUERY -o OUT_DIR\n\n" "$(basename "$0")"

  echo "Required arguments:"
  echo " -d IN_DIR (input FASTQ)"
  echo ""
  echo "Options (default in parentheses):"
  echo " -o OUT_DIR ($OUT_DIR)"
  echo ""
  exit 0
}

if [[ $# -eq 0 ]]; then
  HELP
fi

while getopts :d:o:h OPT; do
  case $OPT in
    d)
      IN_DIR="$OPTARG"
      ;;
    h)
      HELP
      ;;
    o)
      OUT_DIR="$OPTARG"
      ;;
    :)
      echo "Error: Option -$OPTARG requires an argument."
      exit 1
      ;;
    \?)
      echo "Error: Invalid option: -${OPTARG:-""}"
      exit 1
  esac
done

PARAM="$$.pear.param"
singularity exec "$IMG" run_pear.py -p "$IMG" -d "$IN_DIR" -o "$OUT_DIR" > "$PARAM"

export LAUNCHER_DIR="$HOME/src/launcher"
export LAUNCHER_PLUGIN_DIR="$LAUNCHER_DIR/plugins"
export LAUNCHER_WORKDIR="$PWD"
export LAUNCHER_RMI="SLURM"
export LAUNCHER_SCHED="interleaved"
export LAUNCHER_JOB_FILE="$PARAM"

NJOBS=$(lc "$PARAM")
[[ $NJOBS -gt 4 ]] && export LAUNCHER_PPN=4

echo "Launching $NJOBS jobs"
"$LAUNCHER_DIR/paramrun"
echo "Launcher finished"

echo "Done, see OUT_DIR \"$OUT_DIR\""
echo "Comments to kyclark@email.arizona.edu"
