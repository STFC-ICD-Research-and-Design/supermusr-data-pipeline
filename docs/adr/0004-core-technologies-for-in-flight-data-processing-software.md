# 4. Core technologies for in-flight data processing software

Date: 2024-06-17

## Status

Accepted

(note that this ADR was written retrospectively)

## Context

The SuperMuSR data pipeline includes several software components that operate on live/in-flight data from the digitisers.

These software components (regardless of their function) have the following requirements:

1. Speed: they must reasonably keep up with the rate at which data is produced
2. Reliability: reasonable effort should be made to prevent known (and indeed, unknown) issues from causing a component to fail
3. Reproducibility: it should be possible to trivially recreate a component of the pipeline from a point in time
4. Scalability: where required, it should be possible to scale components to meet throughput requirements
5. Portability: where possible, components should not be tied to a specific means of deployment, orchestration or execution or execution environment

## Decision

The following tooling will be used:

- The Rust programming language (addressing 1 and sufficiently mitigating 2)
- The Nix package manager & nixpkgs (addressing 3)
- Deployment via OCI compatible container images (partly addressing 4 and addressing 5)

## Consequences

- All tools that operate on streamed data will be written in Rust
- All non-Rust non-vendored dependencies and associated tooling will be controlled via Nix (with Flakes)
   - Another benefit of this is significantly easier developer environment setup
- Nix tooling will be in place to produce minimal container images which can be used for testing and deployment of pipeline components
