use std::path::Path;

fn main() {
    let schema_dir = Path::new("../schemas/");
    let target_dir = Path::new("./src/generated");

    println!("cargo:rerun-if-changed={}", schema_dir.display());

    flatc_rust::run(flatc_rust::Args {
        inputs: &[
            schema_dir
                .join("aev1_frame_assembled_event_v1.fbs")
                .as_path(),
            schema_dir
                .join("dat1_digitizer_analog_trace_v1.fbs")
                .as_path(),
            schema_dir.join("dev1_digitizer_event_v1.fbs").as_path(),
            schema_dir.join("frame_metadata_v1.fbs").as_path(),
            schema_dir.join("hst1_histogram_v1.fbs").as_path(),
            schema_dir.join("ecs_6s4t_run_stop.fbs").as_path(),
            schema_dir.join("ecs_df12_det_spec_map.fbs").as_path(),
            schema_dir.join("ecs_pl72_run_start.fbs").as_path(),
        ],
        out_dir: target_dir,
        ..Default::default()
    })
    .expect("flatc");
}
