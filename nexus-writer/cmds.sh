cargo run --bin trace-to-events -- \
    --broker localhost:19092 --group trace-to-events \
    --trace-topic Traces --event-topic Events \
    constant-phase-discriminator --threshold-trigger=-40,1,0 &

cargo run --bin nexus-writer -- \
    --broker localhost:19092 --consumer-group nexus-writer \
    --control-topic Controls --frame-event-topic FrameEvents \
    --file-name output/Saves &

cargo run --bin digitiser-aggregator -- \
    --broker localhost:19092 --group digitiser-aggregator \
    --input-topic Events --output-topic FrameEvents \
    -d1 &

sleep 1
echo "Simulating messages"

cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-start --instrument-name MUSR
cargo run --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample --digitizer-id=1
cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-stop
cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-start --instrument-name MUSR
cargo run --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample --digitizer-id=1
cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-stop

echo "Waiting five seconds for trace-to-events and nexus-writer to complete"
sleep 5

pkill trace-to-event
pkill nexus-writer
pkill digitiser-aggregator
echo "Script complete"