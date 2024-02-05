use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Distribution {
  Constant(f64),
  Uniform {
    min: f64,
    max: f64,
  },
  Normal {
    mean: f64,
    sd: f64,
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Pulse {
  weight: f64,
  intensity: Distribution,
  peak: Distribution
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Simulation {
  time_bins: Time,
  voltage: Interval<Intensity>,
  voltage_tranformation: Transformation,
  sample_rate: u32,
  trace_messages: Vec<TraceMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TraceMessage {
  digitizer_ids: Vec<DigitizerID>,
  frames: Vec<FrameNumber>,
  pulses: Option<Vec<usize>>,
  noises: Option<Vec<usize>>,
  channels: usize,
  num_pulses: Distribution,
  pulse_weights: Option<Vec<f64>>,
  timestamp: "now",
  frame_delay_ns: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Settings {
    simulation: Simulation,
    pulses: Vec<Pulse>,
    noises: Vec<Noise>
}
json::derive!("
{
  "simulation": {
    "time-bins": 30000,
    "voltage": {"min": 0, "max": 80},
    "voltage_transform": {"scale": -1, "translate": 80 },
    "sample_rate": 100000000,
    "trace_messages": [
      {"digitizer_ids": [0], "frames": [5], "pulses": "any", "noises": "any", "channels": 8, "num_pulses": 500, "timestamp": "now", "frame_delay_ns": 0 }
    ],
  },
  "pulses": [
    { "type": "gaussian",
      "peak": 40,
      "mean": { "uniform": {"min": 5000,   "max": 20000 } },
      "sd": { "uniform": {"min": 10, "max": 200 } }
    }
  ],
  "noises": [
    { "type": "random",
      "intensity": 1,
    }
  ]
}
");
