
cargo run --release --bin simulator -- \
    --broker localhost:19092 \
    --trace-topic Traces \
    json --path simulator/data.json

cargo run --release --bin trace-to-events -- \
    --broker localhost:19092 \
    --trace-topic Traces --event-topic Events \
    --group trace-to-events \
    --save-file myoutput \
    constant-phase-discriminator --threshold-trigger=-40,1,0
