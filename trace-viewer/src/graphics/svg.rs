use super::{Bounds, GraphSaver};
use crate::messages::{DigitiserTrace, EventList, Trace};
use miette::IntoDiagnostic;
use plotters::{
    chart::{ChartBuilder, ChartContext},
    coord::{Shift, types::RangedCoordf64},
    prelude::{Cartesian2d, Circle, DrawingArea, IntoDrawingArea, PathElement, SVGBackend},
    series::{LineSeries, PointSeries},
    style::{BLACK, BLUE, IntoFont, ShapeStyle, WHITE},
};
use std::path::PathBuf;
use supermusr_common::Channel;
use tracing::instrument;

type MyDrawingArea<'a> = DrawingArea<SVGBackend<'a>, Shift>;
type MyChartContext<'a> =
    ChartContext<'a, SVGBackend<'a>, Cartesian2d<RangedCoordf64, RangedCoordf64>>;

trait MyBuilder<'a>: Sized {
    fn build_trace_graph(root: &MyDrawingArea<'a>, bounds: Bounds) -> miette::Result<Self>;
    fn draw_eventlist_to_chart(
        &mut self,
        eventlist: &EventList,
        label: &str,
    ) -> Result<(), miette::Error>;
    fn draw_trace_to_chart(&mut self, trace: &Trace, label: &str) -> Result<(), miette::Error>;
}

#[derive(Default)]
pub(crate) struct SvgSaver {}

impl<'a> MyBuilder<'a> for MyChartContext<'a> {
    #[instrument(skip_all, level = "debug")]
    fn build_trace_graph(
        root: &MyDrawingArea<'a>,
        bounds: Bounds,
    ) -> miette::Result<MyChartContext<'a>> {
        let mut chart = ChartBuilder::on(root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .margin(5)
            .caption("Trace", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(
                bounds.time.min..bounds.time.max,
                bounds.intensity.min..bounds.intensity.max,
            )
            .into_diagnostic()?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()
            .into_diagnostic()?;

        Ok(chart)
    }

    #[instrument(skip_all, level = "debug")]
    fn draw_eventlist_to_chart(
        &mut self,
        eventlist: &EventList,
        label: &str,
    ) -> Result<(), miette::Error> {
        let data = eventlist
            .iter()
            .map(|el| (el.time as f64, el.intensity as f64));

        let ps: PointSeries<_, _, Circle<_, _>, _> =
            PointSeries::new(data, 4, ShapeStyle::from(&BLACK));

        self.draw_series(ps)
            .into_diagnostic()?
            .label(label)
            .legend(|(x, y)| Circle::new((x, y), 4, BLACK));
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    fn draw_trace_to_chart(&mut self, trace: &Trace, label: &str) -> Result<(), miette::Error> {
        let data = trace
            .iter()
            .cloned()
            .enumerate()
            .map(|(x, y)| (x as f64, y as f64));

        self.draw_series(LineSeries::new(data, &BLUE))
            .into_diagnostic()?
            .label(label)
            .legend(|(x, y)| PathElement::new(vec![(x - 10, y), (x + 10, y)], BLUE));
        Ok(())
    }
}

impl GraphSaver for SvgSaver {
    fn save_as_svg(
        trace: &DigitiserTrace,
        channels: Vec<Channel>,
        path: PathBuf,
        (width, height): (u32, u32),
        bounds: Bounds,
    ) -> Result<(), miette::Error> {
        let root = SVGBackend::new(&path, (width, height)).into_drawing_area();

        root.fill(&WHITE).into_diagnostic()?;

        let mut chart = MyChartContext::build_trace_graph(&root, bounds)?;

        for c in channels {
            chart.draw_trace_to_chart(&trace.traces[&c], &format!("trace[{c}]"))?;
            if let Some(eventlists) = &trace.events {
                if let Some(eventlist) = eventlists.get(&c) {
                    chart.draw_eventlist_to_chart(eventlist, &format!("event[{c}]"))?;
                }
            }
        }

        chart
            .configure_series_labels()
            .background_style(WHITE)
            .draw()
            .into_diagnostic()?;

        root.present().into_diagnostic()?;
        Ok(())
    }
}
