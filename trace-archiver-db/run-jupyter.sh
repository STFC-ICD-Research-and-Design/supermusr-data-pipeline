mkdir /tmp/supermusr-trace-archiver-db
wget /trace-archiver-db/trace-archiver-db.ipynb -O /tmp/trace-archiver-db/trace-archiver-db.ipynb

# shellcheck disable=SC2164
cd /tmp/supermusr-notebook/

conda_env=SUPERMUSRENV
conda_exe=$(which conda)
conda_install_dir=$(dirname "$(dirname "$conda_exe")")

$conda_exe env remove -n $conda_env
$conda_exe create -n $conda_env -y

. "$conda_install_dir/etc/profile.d/conda.sh"
conda activate $conda_env
mamba install jupyter matplotlib taospy -y

jupyter notebook