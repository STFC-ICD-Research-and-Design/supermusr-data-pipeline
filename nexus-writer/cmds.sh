cargo run --release --bin trace-to-events -- \
    --broker localhost:19092 --group trace-to-events \
    --trace-topic Traces --event-topic Events \
    constant-phase-discriminator --threshold-trigger=-40,1,0 &

cargo run --release --bin nexus-writer -- \
    --broker localhost:19092 --consumer-group nexus-writer \
    --control-topic Controls --frame-event-topic FrameEvents \
    --cache-run-ttl-ms=1000 \
    --file-name output/Saves &

cargo run --release --bin digitiser-aggregator -- \
    --broker localhost:19092 --group digitiser-aggregator \
    --input-topic Events --output-topic FrameEvents \
    --frame-ttl-ms 1500 \
    -d1 -d2 &

sleep 0.25
echo "Simulating messages"

cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test1 \
    --time "2024-02-12 11:48:00.000000z" \
    run-start --instrument-name MUSR

cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces \
    --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces \
    --number-of-trace-events 5 --random-sample \
    --digitizer-id=1 --frame-number 1 --channel-id-shift=0 --frame-interval-ms=20 \
    --time "2024-02-12 11:48:00.000000z"
cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces \
    --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces \
    --number-of-trace-events 5 --random-sample \
    --digitizer-id=2 --frame-number 1 --channel-id-shift=4 --frame-interval-ms=20  \
    --time "2024-02-12 11:48:00.000000z"
    
cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test1 \
    --time "2024-02-12 11:48:01.000000z" \
    run-stop

cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test2 \
    --time "2024-02-12 11:48:01.000000z" \
    run-start --instrument-name MUSR

cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces \
    --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces \
    --number-of-trace-events 2 --random-sample \
    --digitizer-id=1 --frame-number 1 --frame-interval-ms=200  \
    --time "2024-02-12 11:48:01.000010z"

cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test2 \
    --time "2024-02-12 11:48:02.101000z" \
    run-stop

echo "Waiting five seconds for trace-to-events and nexus-writer to complete"
sleep 8

pkill trace-to-event
pkill nexus-writer
pkill digitiser-aggre
echo "Script complete"