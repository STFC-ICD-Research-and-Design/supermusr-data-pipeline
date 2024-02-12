cargo run --release --bin trace-to-events -- \
    --broker localhost:19092 --group trace-to-events \
    --trace-topic Traces --event-topic Events \
    constant-phase-discriminator --threshold-trigger=-40,1,0 &

#cargo run --release --bin nexus-writer -- \
#    --broker localhost:19092 --consumer-group nexus-writer \
#    --control-topic Controls --frame-event-topic FrameEvents \
#    --file-name output/Saves &

cargo run --release --bin digitiser-aggregator -- \
    --broker localhost:19092 --group digitiser-aggregator \
    --input-topic Events --output-topic FrameEvents \
    --frame-ttl-ms 2000 \
    -d1 -d2 &

sleep 1
echo "Simulating messages"

cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test1 run-start --instrument-name MUSR
cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample --digitizer-id=1 --channel-id-shift=0
cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample --digitizer-id=2 --channel-id-shift=4
sleep 1
cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test1 run-stop
cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test2 run-start --instrument-name MUSR
cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample --digitizer-id=1
sleep 1
cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test2 run-stop

echo "Waiting five seconds for trace-to-events and nexus-writer to complete"
sleep 5

pkill trace-to-event
pkill nexus-writer
pkill digitiser-aggre
echo "Script complete"