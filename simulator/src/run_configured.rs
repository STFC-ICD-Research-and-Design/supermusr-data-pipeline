use crate::{simulation_config::Simulation, Cli, Defined};
use chrono::{DateTime, Utc};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{fs::File, time::Duration};
use supermusr_common::tracer::FutureRecordTracerExt;
use supermusr_streaming_types::flatbuffers::FlatBufferBuilder;
use tokio::task::JoinSet;
use tracing::{debug, debug_span, error, info, info_span, trace, Span};

struct SendMessageArgs<'a> {
    use_otel: bool,
    producer: FutureProducer,
    fbb: FlatBufferBuilder<'a>,
    topic: String,
    span: Span,
    key: &'static str,
}

impl<'a> SendMessageArgs<'a> {
    fn new(
        use_otel: bool,
        fbb: FlatBufferBuilder<'a>,
        producer: &FutureProducer,
        topic: &str,
        key: &'static str,
    ) -> Self {
        Self {
            use_otel,
            fbb,
            producer: producer.to_owned(),
            topic: topic.to_owned(),
            span: tracing::Span::current(),
            key,
        }
    }
}

async fn send_message(args: SendMessageArgs<'_>) {
    let span = debug_span!(parent: &args.span, "Send Message Thread");
    let _guard = span.enter();

    let future_record = FutureRecord::to(&args.topic)
        .payload(args.fbb.finished_data())
        .conditional_inject_span_into_headers(args.use_otel, &args.span)
        .key(args.key);

    let timeout = Timeout::After(Duration::from_millis(100));
    match args.producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e.0),
    };
}

pub(crate) async fn run_configured_simulation(
    use_otel: bool,
    cli: &Cli,
    producer: &FutureProducer,
    defined: Defined,
) {
    let Defined { file, repeat } = defined;

    let mut kafka_producer_thread_set = JoinSet::new();

    let obj: Simulation = serde_json::from_reader(File::open(file).unwrap()).unwrap();
    for trace in obj.traces {
        let now = Utc::now();
        for (index, (frame_index, frame)) in trace
            .frames
            .iter()
            .enumerate()
            .flat_map(|v| std::iter::repeat(v).take(repeat))
            .enumerate()
        {
            let ts = trace.create_time_stamp(&now, index);
            let templates = trace
                .create_frame_templates(frame_index, frame, &ts)
                .expect("Templates created");

            for template in templates {
                let span = info_span!(target: "otel", "Template",
                    "frame_number" = frame
                ); // Maybe add some extra fields in the future?
                let _guard = span.enter();

                if let Some(digitizer_id) = template.digitizer_id() {
                    if let Some(trace_topic) = cli.trace_topic.as_deref() {
                        let span = info_span!("Trace", digitiser_id = digitizer_id);
                        let _guard = span.enter();

                        let mut fbb = FlatBufferBuilder::new();

                        template
                            .send_trace_messages(
                                &mut fbb,
                                digitizer_id,
                                &obj.voltage_transformation,
                            )
                            .await
                            .expect("Trace messages should send.");

                        let ts: DateTime<Utc> =
                            (*template.metadata().timestamp.expect("Timestamp Exists"))
                                .try_into()
                                .expect("Convert to DateTime");
                        info!(
                            "Simulated Trace: {ts}, {0}",
                            template.metadata().frame_number
                        );

                        let send_args = SendMessageArgs::new(
                            use_otel,
                            fbb,
                            producer,
                            trace_topic,
                            "Simulated Trace",
                        );
                        kafka_producer_thread_set.spawn(send_message(send_args));
                    }

                    if let Some(event_topic) = cli.event_topic.as_deref() {
                        let span = info_span!("Digitiser Event List", digitiser_id = digitizer_id);
                        let _guard = span.enter();

                        let mut fbb = FlatBufferBuilder::new();
                        template
                            .send_digitiser_event_messages(
                                &mut fbb,
                                digitizer_id,
                                &obj.voltage_transformation,
                            )
                            .await
                            .expect("Event messages should send.");

                        let ts: DateTime<Utc> =
                            (*template.metadata().timestamp.expect("Timestamp Exists"))
                                .try_into()
                                .expect("Convert to DateTime");
                        info!(
                            "Simulated Digitiser Events List: {ts}, {0}",
                            template.metadata().frame_number
                        );

                        // Prepare the kafka message
                        let send_args = SendMessageArgs::new(
                            use_otel,
                            fbb,
                            producer,
                            event_topic,
                            "Simulated Digitiser Event",
                        );
                        kafka_producer_thread_set.spawn(send_message(send_args));
                    }
                } else if let Some(frame_event_topic) = cli.frame_event_topic.as_deref() {
                    let span = info_span!("Frame Assembled Event List");
                    let _guard = span.enter();

                    let mut fbb = FlatBufferBuilder::new();
                    template
                        .send_frame_event_messages(&mut fbb, &obj.voltage_transformation)
                        .await
                        .expect("Event messages should send.");

                    let ts: DateTime<Utc> =
                        (*template.metadata().timestamp.expect("Timestamp Exists"))
                            .try_into()
                            .expect("Convert to DateTime");
                    info!(
                        "Simulated Frame Events List: {ts}, {0}",
                        template.metadata().frame_number
                    );

                    // Prepare the kafka message
                    let send_args = SendMessageArgs::new(
                        use_otel,
                        fbb,
                        producer,
                        frame_event_topic,
                        "Simulated Frame Event",
                    );
                    kafka_producer_thread_set.spawn(send_message(send_args));
                }
            }
        }
    }

    trace!("Waiting for delivery threads to finish.");
    while let Some(result) = kafka_producer_thread_set.join_next().await {
        if let Err(e) = result {
            error!("{e}");
        }
    }
    trace!("All finished.");
}
