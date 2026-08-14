use crate::{
    analysis::metrics::{CompletedMetricResult, FittingError, MetricOutput, MetricResultError},
    engine::{FlatChart, FlatSeries},
};
use plotly::{
    self, Layout, Plot, Scatter,
    common::{ErrorData, ErrorType, Line},
    layout::{Axis, ModeBar},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ChartOutputError {
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::error::Error),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Metric Result Error: {0}")]
    Metric(#[from] MetricResultError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ChartOutput {
    chart: FlatChart,
    data: Vec<Option<MetricOutput<Vec<Option<f64>>>>>,
}

impl ChartOutput {
    pub(crate) fn new(
        chart: &FlatChart,
        metrics: &[CompletedMetricResult],
    ) -> Result<Self, ChartOutputError> {
        // Get Series Output
        let data = chart
            .series
            .iter()
            .map(|series: &FlatSeries| {
                let metric = metrics.get(series.metric).expect("This should never fail");
                match metric.get_aggregate_property(series.from_bucket_block, &series.property) {
                    Ok(value) => Ok(Some(value)),
                    Err(MetricResultError::Fitting(FittingError::NoValue)) => Ok(None),
                    Err(e) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            chart: chart.clone(),
            data,
        })
    }

    pub(crate) fn load_json(path: &Path, chart_name: &str) -> Result<Self, ChartOutputError> {
        let mut path = path.to_path_buf();
        path.push(chart_name);
        path.add_extension("json");
        Ok(serde_json::from_reader(File::open(&path)?)?)
    }

    pub(crate) fn save_json(&self, path: &Path) -> Result<(), ChartOutputError> {
        let mut path = path.to_owned();
        path.push(&self.chart.settings.title);
        path.add_extension("json");
        let file = File::create(&path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub(crate) fn save_plotly(&self, path: &Path) -> Result<(), ChartOutputError> {
        let mut path = path.to_owned();
        path.push(&self.chart.settings.title);
        path.add_extension("html");
        let plot = self.build_graph();
        plot.write_html(&path);
        Ok(())
    }

    pub(crate) fn build_trace(
        &self,
        series: &FlatSeries,
        data: Option<&MetricOutput<Vec<Option<f64>>>>,
    ) -> Box<Scatter<f64, f64>> {
        let line = series.settings.line_colour.iter().fold(
            series
                .settings
                .line_style
                .iter()
                .fold(Line::new(), |line, dash| line.dash(dash.into())),
            |line, colour| line.color(colour.to_string()),
        );

        match data {
            Some(MetricOutput::Scalar(data)) => {
                let x_axis = self.chart.x_axis
                    .iter()
                    .zip(data)
                    .filter_map(|(a,b)|b.is_some().then_some(*a))
                    .collect::<Vec<_>>();
                let y_axis = data.iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                Scatter::new(x_axis, y_axis)
                    .line(line)
                    .name(&series.settings.name)
            }
            Some(MetricOutput::ScalarWithBand(value, band)) => {
                let x_axis = self.chart.x_axis
                    .iter()
                    .zip(value.iter().zip(band.iter()))
                    .filter_map(|(a,b)|(b.0.is_some() && b.1.is_some()).then_some(*a))
                    .collect::<Vec<_>>();
                let y_axis = value.iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                let band = band.iter()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                Scatter::new(x_axis, y_axis)
                    .line(line)
                    .name(&series.settings.name)
                    .error_y(ErrorData::new(ErrorType::Data).array(band))
            }
            None => Scatter::new(Default::default(), Default::default())
                .line(line)
                .name(format!("{} - values missing.", series.settings.name)),
        }
    }

    pub(crate) fn build_graph(&self) -> Plot {
        let mut plot: Plot = Plot::new();
        let layout = Layout::new()
            .title(&self.chart.settings.title)
            .mode_bar(ModeBar::new())
            .show_legend(true)
            .auto_size(true)
            .x_axis(Axis::new().title(&self.chart.settings.x_axis_label))
            .y_axis(Axis::new().title(&self.chart.settings.y_axis_label));

        plot.set_layout(layout);
        for (series, data) in Iterator::zip(self.chart.series.iter(), self.data.iter()) {
            plot.add_trace(self.build_trace(series, data.as_ref()));
        }
        plot
    }
}
