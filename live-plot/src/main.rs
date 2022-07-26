mod backend;

use self::backend::{PixelState, TextDrawingBackend};
use anyhow::Result;
use clap::Parser;
use plotters::prelude::*;
use rdkafka::{
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: String,

    #[clap(long)]
    password: String,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long)]
    trace_channel: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &args.broker)
        .set("security.protocol", "sasl_plaintext")
        .set("sasl.mechanisms", "SCRAM-SHA-256")
        .set("sasl.username", &args.username)
        .set("sasl.password", &args.password)
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&args.trace_topic])?;

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );
                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        if let Ok(data) = root_as_digitizer_analog_trace_message(payload) {
                            log::info!(
                                "Trace packet: dig. ID: {}, status: {:?}",
                                data.digitizer_id(),
                                data.status()
                            );

                            let b = TextDrawingBackend(vec![
                                PixelState::Empty;
                                backend::REQUIRED_BUFFER
                            ])
                            .into_drawing_area();

                            let voltage = data
                                .channels()
                                .unwrap()
                                .get(args.trace_channel)
                                .voltage()
                                .unwrap();

                            let len = voltage.len() as i32;
                            let min = voltage.iter().min().unwrap() as i32;
                            let max = voltage.iter().max().unwrap() as i32;

                            log::trace!("{:?}", voltage);
                            log::trace!("len: {}, min: {}, max: {}", voltage.len(), min, max);

                            let mut chart = ChartBuilder::on(&b)
                                .margin(1)
                                .set_label_area_size(
                                    LabelAreaPosition::Left,
                                    (5i32).percent_width(),
                                )
                                .set_label_area_size(
                                    LabelAreaPosition::Bottom,
                                    (10i32).percent_height(),
                                )
                                .build_cartesian_2d(0..len, min..max)?;

                            chart
                                .configure_mesh()
                                .disable_x_mesh()
                                .disable_y_mesh()
                                .draw()?;

                            chart.draw_series(LineSeries::new(
                                voltage
                                    .safe_slice()
                                    .iter()
                                    .enumerate()
                                    .map(|i| (i.0 as i32, *i.1 as i32)),
                                &RED,
                            ))?;

                            b.present()?;
                        }
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }
            }
        };
    }
}
