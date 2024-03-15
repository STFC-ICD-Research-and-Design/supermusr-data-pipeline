# kafka-daq-report

A simple TUI tool that listens on the trace topic and reports in a table view the following for each DAQ that is seen to be sending messages:

- Number of messages received
- Timestamp of first message received since starting kafka-daq-stats
- Timestamp of last message received
- Frame number of the last message received
- Number of channels present in the last message received
- A flag indicating if the number of channels has ever changed
- Number of samples in the first channel of the last message received
- A flag indicating if the number of samples is not identical in each channel
- A flag indicating if the number of samples has ever changed

## Running in Podman

Please see the appropriate [documentation](https://podman.io/docs/installation) for installing Podman on your system.

### Installation on Windows

When using a Windows system, please see [here](https://github.com/containers/podman/blob/main/docs/tutorials/podman-for-windows.md). Note that the Windows installation requires WSL.

- [Download](https://github.com/containers/podman/releases) the latest release for Windows, and follow the installation instructions.
- In Powershell run `podman machine init` to download and set up virtual machine.
- When the above process is complete run `podman machine start`.
- To shut down the Podman VM run `podman machine stop`

Some issues may be caused by running Podman in rootless mode. To change to rootful mode run `podman machine set --rootful`. If a restart is required run `podman machine stop`, then `podman machine start`.

## Running

To run using Podman, execute the following command, substituting the broker, trace-topic, and group arguments as appropriate.

```shell
podman run --rm -it \
    ghcr.io/stfc-icd-research-and-design/supermusr-kafka-daq-report:main \
    --broker 130.246.55.29:9090 \
    --trace-topic daq-traces-in  \
    --group vis-3
```
