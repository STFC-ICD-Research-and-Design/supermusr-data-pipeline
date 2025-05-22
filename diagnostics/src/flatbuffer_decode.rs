use std::io::BufRead;
use tracing::debug;

pub(crate) async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let stdin = std::io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let line = line?;

        let bytes = hex_to_bytes(&line)?;
        debug!("len = {}", bytes.len());

        super::decode_and_print(&bytes);
    }

    Ok(())
}

fn hex_to_bytes(hex: &str) -> anyhow::Result<Vec<u8>> {
    let hex = hex.trim();

    if !hex.contains(" ") {
        anyhow::bail!("Byte hex should be space delimited");
    }

    hex.split_whitespace()
        .map(|byte| {
            u8::from_str_radix(byte, 16).map_err(|_| anyhow::anyhow!("Invalid hex byte: {}", byte))
        })
        .collect()
}
