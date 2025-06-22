use std::path::PathBuf;

use plotters::{
    chart::{ChartBuilder, ChartContext},
    coord::{types::RangedCoordf64, Shift},
    prelude::{Cartesian2d, Circle, DrawingArea, IntoDrawingArea, PathElement, SVGBackend},
    series::{LineSeries, PointSeries},
    style::{IntoFont, ShapeStyle, BLACK, BLUE, WHITE},
};
use supermusr_common::Channel;
use tracing::instrument;

use crate::{
    graphics::Bounds,
    messages::{DigitiserTrace, EventList, Trace},
    GraphSaver,
};

type MyDrawingArea<'a> = DrawingArea<SVGBackend<'a>, Shift>;
type MyChartContext<'a> =
    ChartContext<'a, SVGBackend<'a>, Cartesian2d<RangedCoordf64, RangedCoordf64>>;

trait MyBuilder<'a>: Sized {
    fn build_trace_graph(root: &MyDrawingArea<'a>, bounds: Bounds) -> anyhow::Result<Self>;
    fn draw_eventlist_to_chart(
        &mut self,
        eventlist: &EventList,
        label: &str,
    ) -> Result<(), anyhow::Error>;
    fn draw_trace_to_chart(&mut self, trace: &Trace, label: &str) -> Result<(), anyhow::Error>;
}

#[derive(Default)]
pub(crate) struct SvgSaver {}

impl<'a> MyBuilder<'a> for MyChartContext<'a> {
    #[instrument(skip_all, level = "debug")]
    fn build_trace_graph(
        root: &MyDrawingArea<'a>,
        bounds: Bounds,
    ) -> anyhow::Result<MyChartContext<'a>> {
        let mut chart = ChartBuilder::on(root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            //.right_y_label_area_size(40)
            .margin(5)
            .caption("Trace", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(
                bounds.time.min..bounds.time.max,
                bounds.intensity.min..bounds.intensity.max,
            )?;
        //.set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            //.y_desc("Log Scale")
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()?;

        Ok(chart)
    }

    #[instrument(skip_all, level = "debug")]
    fn draw_eventlist_to_chart(
        &mut self,
        eventlist: &EventList,
        label: &str,
    ) -> Result<(), anyhow::Error> {
        let data = eventlist
            .iter()
            .map(|el| (el.time as f64, el.intensity as f64));
        let ps: PointSeries<_, _, Circle<_, _>, _> =
            PointSeries::new(data, 4, ShapeStyle::from(&BLACK));
        self.draw_series(ps)?
            .label(label)
            .legend(|(x, y)| Circle::new((x, y), 4, BLACK));
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    fn draw_trace_to_chart(&mut self, trace: &Trace, label: &str) -> Result<(), anyhow::Error> {
        let data = trace
            .iter()
            .cloned()
            .enumerate()
            .map(|(x, y)| (x as f64, y as f64));

        self.draw_series(LineSeries::new(data, &BLUE))?
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
    ) -> Result<(), anyhow::Error> {
        let mut root = SVGBackend::new(&path, (width, height)).into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = MyChartContext::build_trace_graph(&mut root, bounds)?;

        for c in channels {
            chart.draw_trace_to_chart(&trace.traces[&c], &format!("trace[{c}]"))?;
            if let Some(eventlist) = &trace.events {
                chart.draw_eventlist_to_chart(&eventlist[&c], &format!("event[{c}]"))?;
            }
        }

        chart
            .configure_series_labels()
            .background_style(WHITE)
            .draw()?;

        root.present()?;
        Ok(())
    }
}
/*
pub(crate) struct BuildGraph<'b,B> where B: Backend<'b> {
    width: usize,
    height: usize,
    x_range: Range<f32>,
    y_range: Range<f32>,
    phantom: PhantomData<&'b B>,
}

impl<'b, B> BuildGraph<'b, B> where B: Backend<'b>, <B::Backend as DrawingBackend>::ErrorType: 'static {
    #[instrument(skip_all, level = "debug")]
    pub(crate) fn new(width: usize, height: usize, x_range: Range<f32>, y_range: Range<f32>) -> Self {
        Self {
            width,
            height,
            x_range,
            y_range,
            phantom: Default::default()
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) fn build_path(&self, path: &'b Path, metadata: &DigitiserMetadata, channel: Channel) -> Result<PathBuf, anyhow::Error> {
        let mut path_buf = path.to_owned();
        path_buf.push(metadata.timestamp.to_rfc3339());
        create_dir_all(&path_buf)?;
        path_buf.push(channel.to_string());

        if path_buf.set_extension(B::EXTENSION) {
            Ok(path_buf)
        } else {
            bail!("Could not set file extension {} to {:?}", B::EXTENSION, path_buf);
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) fn save_trace_graph(&self, path: &'b Path, trace: &Trace, eventlist: Option<&EventList>) -> Result<(), anyhow::Error> {
        info!("Saving to file {path:?}");
        let root = B::new(path, (self.width as u32, self.height as u32)).into_drawing_area();
        root.fill(&WHITE)?;
        self.build_trace_graph(&root, trace, eventlist)?;
        root.present()?;
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) fn build_trace_graph<'a>(&'a self, root: &'a DrawingArea<B::Backend, Shift>, trace: &Trace, eventlist: Option<&EventList>) -> Result<(), anyhow::Error>
    {
        let mut chart = ChartBuilder::on(root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            //.right_y_label_area_size(40)
            .margin(5)
            .caption("Trace", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(self.x_range.clone(), self.y_range.clone())?;
            //.set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);


        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            //.y_desc("Log Scale")
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()?;

        self.draw_trace_to_chart(&mut chart, trace)?;
        if let Some(eventlist) = eventlist {
            self.draw_eventlist_to_chart(&mut chart, eventlist)?;
        }

        chart
            .configure_series_labels()
            .background_style(WHITE)
            .draw()?;
        Ok(())
    }

    #[instrument(skip_all, level = "debug")]
    fn draw_eventlist_to_chart<'a>(&'a self, chart: &'a mut ChartContext<B::Backend, Cartesian2d<RangedCoordf32, RangedCoordf32>>, eventlist: &EventList) -> Result<(), anyhow::Error>
    {
        let data = eventlist.iter().map(|el|(el.time as f32, el.intensity as f32));
        let ps : PointSeries<_,_,Circle<_,_>,_> = PointSeries::new(data, 4, ShapeStyle::from(&BLACK));
        chart
            .draw_series(ps)?
            .label("y = Event")
            .legend(|(x, y)| Circle::new((x,y), 4, BLACK));
        Ok(())
    }
    #[instrument(skip_all, level = "debug")]
    fn draw_trace_to_chart<'a>(&'a self, chart: &'a mut ChartContext<B::Backend, Cartesian2d<RangedCoordf32, RangedCoordf32>>, trace: &Trace) -> Result<(), anyhow::Error>
    {
        let data = trace.iter().cloned().enumerate().map(|(x,y)|(x as f32, y as f32));
        chart
            .draw_series(LineSeries::new(
                data,
                &BLUE,
            ))?
            .label("y = Trace")
            .legend(|(x, y)| PathElement::new(vec![(x - 10, y), (x + 10, y)], BLUE));
        Ok(())
    }

    pub(crate) fn build_event_graph(&self, root: &DrawingArea<SVGBackend, Shift>, trace: Trace) -> Result<(), anyhow::Error> {

        Ok(())
    }
} */
