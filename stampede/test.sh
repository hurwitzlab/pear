#!/bin/bash

#SBATCH -A iPlant-Collabs
#SBATCH -t 02:00:00
#SBATCH -N 1
#SBATCH -n 1
#SBATCH -J peartest
#SBATCH -p development
#SBATCH --mail-type BEGIN,END,FAIL
#SBATCH --mail-user kyclark@email.arizona.edu

./run.sh -d "$WORK/data/tricho/small"
