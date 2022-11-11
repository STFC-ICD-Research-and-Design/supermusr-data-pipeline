#!/usr/bin/env bash

# wget https://raw.githubusercontent.com/STFC-ICD-Research-and-Design/supermusr-data-pipeline/main/notebook/supermusr-plotting-frames.ipynb -O /tmp/supermusr-notebook/supermusr-plotting-frames.ipynb
wget -x https://raw.githubusercontent.com/Pasarus/supermusr-data-pipeline/add-notebook/notebook/supermusr-plotting-frames.ipynb -O /tmp/supermusr-notebook/supermusr-plotting-frames.ipynb

# shellcheck disable=SC2164
cd /tmp/supermusr-notebook/

conda_env=SUPERMUSRENV
conda_exe=$(which conda)
conda_install_dir=$(dirname "$(dirname "$conda_exe")")

$conda_exe env remove -n $conda_env
$conda_exe create -n $conda_env -y

. "$conda_install_dir/etc/profile.d/conda.sh"
conda activate $conda_env
mamba install jupyter h5py matplotlib -y

jupyter notebook