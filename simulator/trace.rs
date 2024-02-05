use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
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
#[serde(rename_all = "PascalCase")]
struct Pulse {
  weight: f64,
  intensity: Distribution,
  peak: Distribution
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Settings {
    num_pulses: Distribution,
    pulses: Vec<Pulse>
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
