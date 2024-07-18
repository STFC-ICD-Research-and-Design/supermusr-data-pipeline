use anyhow::{anyhow, Result};
use supermusr_streaming_types::ecs_al00_alarm_generated::Severity;

pub(crate) fn severity(severity: &str) -> Result<Severity> {
    match severity {
        "OK" => Ok(Severity::OK),
        "MINOR" => Ok(Severity::MINOR),
        "MAJOR" => Ok(Severity::MAJOR),
        "INVALID" => Ok(Severity::INVALID),
        _ => Err(anyhow!("Unable to parse 'alarm severity': {severity} not one of 'OK', 'MINOR', 'MAJOR', 'INVALID'"))
    }
}
