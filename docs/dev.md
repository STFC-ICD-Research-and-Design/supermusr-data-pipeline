# Dev notes

- CI validates schemas with `flatc` v22.9.29.
- [This tool](https://sequencediagram.org/) for editing sequence diagrams.

## Building and testing container images locally

This assumes you are using [Podman](https://podman.io/), in theory Docker should be very similar if not identical.

`nix flake show` can be used to see all of the available build outputs.

- `nix build .#<component>-container-image`
- `podman load < result`
- `podman run --rm -it ...`
