mod bounds;
mod svg;

use crate::messages::{DigitiserMetadata, DigitiserTrace};
use miette::IntoDiagnostic;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};
use strum::{Display, EnumIter, EnumString};
use supermusr_common::Channel;

pub(crate) use bounds::{Bound, Bounds, Point};
pub(crate) use svg::SvgSaver;

#[derive(Clone, EnumString, Display, EnumIter)]
pub(crate) enum FileFormat {
    #[strum(to_string = "svg")]
    Svg,
}

impl FileFormat {
    pub(crate) fn build_path(
        self,
        path: &Path,
        metadata: &DigitiserMetadata,
        channel: Channel,
    ) -> miette::Result<PathBuf> {
        let mut path_buf = path.to_owned();
        path_buf.push(metadata.timestamp.to_rfc3339());
        create_dir_all(&path_buf).into_diagnostic()?;
        path_buf.push(channel.to_string());

        if path_buf.set_extension(self.to_string()) {
            Ok(path_buf)
        } else {
            Err(miette::miette!(
                "Could not set file extension {} to {:?}",
                self.to_string(),
                path_buf
            ))
        }
    }
}

pub(crate) trait GraphSaver: Default {
    fn save_as_svg(
        trace: &DigitiserTrace,
        channels: Vec<Channel>,
        path: PathBuf,
        size: (u32, u32),
        bounds: Bounds,
    ) -> Result<(), miette::Error>;
}
