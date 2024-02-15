echo "Terminating any still active background programs"
pkill trace-to-event
pkill nexus-writer
pkill digitiser-aggre

echo "Executing trace-to-events"
cargo run --release --bin trace-to-events -- \
    --broker localhost:19092 --group trace-to-events \
    --trace-topic Traces --event-topic Events \
    constant-phase-discriminator --threshold-trigger=-40,1,0 &

# Issue
# If frames are incomplete, then the aggregator is waiting 500ms for each frame.
# The writer doesn't know this is happening, so will write the (incomplete) file
# after --cache-run-ttl-ms. At which point will the 
echo "Executing nexus-writer"
RUST_LOG=off cargo run --release --bin nexus-writer -- \
    --broker localhost:19092 --consumer-group nexus-writer --observability-address "127.0.0.1:9091" \
    --control-topic Controls --frame-event-topic FrameEvents \
    --cache-run-ttl-ms 4000 \
    --file-name output/Saves &

echo "Executing aggregator"
cargo run --release --bin digitiser-aggregator -- \
    --broker localhost:19092 --group digitiser-aggregator \
    --input-topic Events --output-topic FrameEvents \
    --frame-ttl-ms 500 \
    -d1 -d2 -d3 -d4 -d5 -d6 -d7 -d8 &



echo "Starting trace-reader"
for ((frame=1; ;frame++)); do

    NOW=$(date +"%Y-%m-%dT%H:%M:%S.%Nz")

    for d in 1 2 3 4 5 6 7 8; do
        cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces \
            --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces \
            --number-of-trace-events 1 --random-sample \
            --digitizer-id $d --frame-number $frame \
            --channel-id-shift  $(($d*4 - 4)) \
            --time $NOW  &> /dev/null
    done
    echo "Frame $frame"
done