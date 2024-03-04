# Developer setup

In all cases a "full" (i.e. kernel + GNU userspace + systemd) Linux system is assumed, the distro itself should not matter.
WSL2 should probably work, but has not been extensively tested.

## Pipeline software

1. Install [Nix](https://nixos.org/) using the [Determinate Installer](https://github.com/DeterminateSystems/nix-installer#usage).
2. Install [direnv](https://direnv.net/docs/installation.html).
3. `git clone https://github.com/STFC-ICD-Research-and-Design/supermusr-data-pipeline`
4. `cd supermusr-data-pipeline`
5. `direnv allow`

You should now be set up to build the various components of the data pipeline.
A list of useful commands to get started:

- `cargo build`: build library/binary (in debug mode by default) or all if run in the root of the repository
- `cargo test`: run unit tests for a library/binary or all if run in the root of the repository
- `treefmt`: format all code
- `nix flake show`: list the things that can be built via Nix
- `nix build .#<something>`: build something via Nix

## Kafka

### Development broker

1. Follow the [Redpanda quickstart](https://docs.redpanda.com/current/get-started/quick-start/) for a single broker.
