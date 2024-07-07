pub(crate) mod active_muons;
pub(crate) mod digitiser_config;
pub(crate) mod simulation;

pub(crate) use simulation::Simulation;


const JSON_INPUT_1: &str = r#"
{
    "voltage-transformation": {"scale": 1, "translate": 0 },
    "pulses": [{
                    "pulse-type": "biexp",
                    "height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } },
                    "start":  { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                    "rise":   { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 30 } },
                    "decay":  { "random-type": "uniform", "min": { "fixed-value": 5 }, "max": { "fixed-value": 10 } }
                },
                {
                    "pulse-type": "flat",
                    "start":  { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                    "width":  { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 50 } },
                    "height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } }
                },
                {
                    "pulse-type": "triangular",
                    "start":     { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                    "width":     { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 50 } },
                    "peak_time": { "random-type": "uniform", "min": { "fixed-value": 0.25 }, "max": { "fixed-value": 0.75 } },
                    "height":    { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } }
                }],
    "traces": [
        {
            "sample-rate": 100000000,
            "pulses": [
                {"weight": 1, "attributes": {"create-from-index": 0}},
                {"weight": 1, "attributes": {"create-from-index": 1}},
                {"weight": 1, "attributes": {"create-from-index": 2}}
            ],
            "noises": [
                {
                    "attributes": { "noise-type" : "gaussian", "mean" : { "fixed-value": 0 }, "sd" : { "fixed-value": 20 } },
                    "smoothing-factor" : { "fixed-value": 0.975 },
                    "bounds" : { "min": 0, "max": 30000 }
                },
                {
                    "attributes": { "noise-type" : "gaussian", "mean" : { "fixed-value": 0 }, "sd" : { "frame-transform": { "scale": 50, "translate": 50 } } },
                    "smoothing-factor" : { "fixed-value": 0.995 },
                    "bounds" : { "min": 0, "max": 30000 }
                }
            ],
            "num-pulses": { "random-type": "constant", "value": { "fixed-value": 500 } },
            "time-bins": 30000,
            "timestamp": "now",
            "frame-delay-us": 20000
        }
    ],
    "schedule" [
        { "action": { "run-command": "run-start", "name": "MyRun", "instrument": "MuSR" } },
        { "action": { "wait_ms": 100 } },
        { "loop" : {
                
            }
        }
    ]
}
"#;

#[test]
fn test1() {
    let simulation: Simulation = serde_json::from_str(JSON_INPUT_1).unwrap();
    assert!(simulation.validate());
    assert_eq!(simulation.pulses.len(), 0);
    assert_eq!(simulation.voltage_transformation.scale, 1.0);
    assert_eq!(simulation.voltage_transformation.translate, 0.0);
}
