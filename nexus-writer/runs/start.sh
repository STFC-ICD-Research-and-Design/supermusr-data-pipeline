RunName=MyTest

cargo run --release --bin run-simulator -- --broker localhost:19092 --topic Controls \
    --run-name $RunName \
    run-start --instrument-name MuSR