use miette::IntoDiagnostic;
use std::io::BufRead;
use tracing::debug;

pub(crate) async fn run() -> miette::Result<()> {
    tracing_subscriber::fmt::init();

    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let line = line.into_diagnostic()?;

        let bytes = hex_to_bytes(&line)?;
        debug!("len = {}", bytes.len());

        super::decode_and_print(&bytes);
    }

    Ok(())
}

fn hex_to_bytes(hex: &str) -> miette::Result<Vec<u8>> {
    let hex = hex.trim();

    if !hex.contains(" ") {
        Err(miette::miette!("Byte hex should be space delimited"))
    } else {
        hex.split_whitespace()
            .map(|byte| {
                u8::from_str_radix(byte, 16)
                    .map_err(|_| miette::miette!("Invalid hex byte: {}", byte))
            })
            .collect()
    }
}
