use ratatui::{text::Span, widgets::Axis};

use crate::graphics::{Bound, Bounds, Point};

/// Generate a [ratatui] axis object.
///
/// # Attributes
/// - bound: the source bound of the axis.
/// - title: the title to display.
/// - num_labels: the number of labels to generate.
fn make_axis<'a>(bound: &Bound, title: &'static str, num_labels: i32) -> Axis<'static> {
    let labels: Vec<_> = (0..num_labels)
        .map(|i| bound.range() * i as f64 / num_labels as f64 + bound.min)
        .map(|v| Span::raw(format!("{v:.3}")))
        .collect();
    Axis::default()
        .title(title)
        .bounds([bound.min, bound.max])
        .labels(labels)
}

/// Encapsulates the properties of a Tui Graph, that are independent of the data.
pub(crate) struct GraphProperties {
    /// The bounding rectangle of the raw data.
    pub(super) bounds: Bounds,
    /// The bounding rectangle of the transformed data.
    pub(super) zoomed_bounds: Bounds,
    /// The translation to apply to the data.
    ///
    /// This, along with [Self::zoom_factor] is applied to [Self::bounds] to compute [Self::zoomed_bounds].
    pub(super) view_port: Point,
    /// The scaling factor to apply to the data.
    ///
    /// This, along with [Self::view_port] is applied to [Self::bounds] to compute [Self::zoomed_bounds].
    pub(super) zoom_factor: f64,
    /// The horizontal (time) axis of the graph.
    pub(super) x_axis: Axis<'static>,
    /// The vertical (intensity) axis of the graph.
    pub(super) y_axis: Axis<'static>,
}

impl GraphProperties {
    /// [Self::move_viewport] scales the direction by this multiple of the entire tranformed bounding rectangle [Self::zoomed_bound].
    const SHIFT_COEF: f64 = 0.1;

    /// The zoom factor is capped above by this value.
    const MAX_ZOOM: f64 = 64.0;
    /// [Self::zoom_in] and [Self::zoom_out] multiply and divide [Self::zoom_factor] by this value, respectively.
    const ZOOM_COEF: f64 = 1.1;

    /// Creates a new instance with the given bounding rectangle, and identity transformation.
    pub(super) fn new(bounds: Bounds) -> Self {
        let zoomed_bounds = bounds.clone();
        let view_port = bounds.mid_point();

        let x_axis = make_axis(&bounds.time, "Time", 10);
        let y_axis = make_axis(&bounds.intensity, "Intensity", 5);
        Self {
            bounds,
            zoomed_bounds,
            view_port,
            zoom_factor: 1.0,
            x_axis,
            y_axis,
        }
    }

    /// Calculate the transformed bounding rectangle and rebuild the axes.
    fn calc_axes(&mut self) {
        self.zoomed_bounds = self.bounds.transform(self.zoom_factor, &self.view_port);

        self.x_axis = make_axis(&self.zoomed_bounds.time, "Time", 10);
        self.y_axis = make_axis(&self.zoomed_bounds.intensity, "Intensity", 5);
    }

    /// Increase the scaling factor.
    pub(crate) fn zoom_in(&mut self) {
        self.zoom_factor *= Self::ZOOM_COEF;
        if self.zoom_factor > Self::MAX_ZOOM {
            self.zoom_factor = Self::MAX_ZOOM;
        }
        self.calc_axes();
    }

    /// Decrease the scaling factor.
    pub(crate) fn zoom_out(&mut self) {
        self.zoom_factor /= Self::ZOOM_COEF;
        if self.zoom_factor < 1.0 {
            self.zoom_factor = 1.0;
        }
        self.calc_axes();
    }

    /// Translate the viewport in the given direction.
    ///
    /// To ensure intended behaviour, the values for each axis should be one of: `-1`, `0`, or `1`.
    /// Although, this is not checked, and should be ensured by the caller.
    ///
    /// # Attributes
    /// - time: the direction to shift in the time axis.
    /// - intensity: the direction to shift in the intensity axis.
    pub(crate) fn move_viewport(&mut self, time: f64, intensity: f64) {
        self.view_port.time += time * Self::SHIFT_COEF * self.zoomed_bounds.time.range();
        self.view_port.intensity +=
            intensity * Self::SHIFT_COEF * self.zoomed_bounds.intensity.range();
        self.calc_axes();
    }

    /// Returns a string with viewport and zoom factor.
    pub(crate) fn get_info(&self) -> String {
        format!(
            "({:.2}, {:.2}): {:.2}",
            self.view_port.time, self.view_port.intensity, self.zoom_factor,
        )
    }
}
