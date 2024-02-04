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

{
  "num_pulses": { "Constant": 500 },
  "pulses": [
    {
      "weight": 1,
      "intensity": 40,
      "type": {
        "gaussian": {
          mean: { "uniform": {"min": 0,   "max": 1 } },
          "sd": { "uniform": {"min": 0.5, "max": 2 } }
        }
      }
    }
  ]
}
