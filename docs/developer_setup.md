# Developer setup

In all cases a "full" (i.e. kernel + GNU userspace + systemd) Linux system is assumed, the distro itself should not matter.
WSL2 should probably work, but has not been extensively tested.

## Pipeline software

1. Install [Nix](https://nixos.org/) using the [Determinate Installer](https://github.com/DeterminateSystems/nix-installer#usage).
2. Install [direnv](https://direnv.net/docs/installation.html).
3. Install [devenv](https://devenv.sh/).
4. `git clone https://github.com/STFC-ICD-Research-and-Design/supermusr-data-pipeline`
5. `cd supermusr-data-pipeline`
6. `direnv allow`

You should now be set up to build the various components of the data pipeline.
A list of useful commands to get started:

- `cargo build`: build library/binary (in debug mode by default) or all if run in the root of the repository
- `cargo test`: run unit tests for a library/binary or all if run in the root of the repository
- `treefmt`: format all code
- `buildah build -t supermusr-<component name>:latest --build-arg component=<component name> .` (where `<component name>` is any binary crate): build a container image for a given component

Note that if working with container images you should install Podman and Buildah via your system package manager (rather than via devenv.sh).
This ensures system dependencies for the container runtime are correctly configured.

## Kafka

### Development broker

1. Follow the [Redpanda quickstart](https://docs.redpanda.com/current/get-started/quick-start/) for a single broker.
