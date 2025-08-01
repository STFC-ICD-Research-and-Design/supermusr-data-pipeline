#[derive(Default, Clone)]
pub(crate) struct Pair<D: Default> {
    pub(crate) time: D,
    pub(crate) intensity: D,
}

#[derive(Default, Clone)]
pub(crate) struct Bound {
    pub(crate) min: f64,
    pub(crate) max: f64,
}

impl Bound {
    pub(crate) fn from<D: Default + Ord + Into<f64>, I: Iterator<Item = D> + Clone>(
        buffer: f64,
        data: I,
    ) -> Bound {
        let min: f64 = data.clone().min().unwrap_or_default().into();
        let max: f64 = buffer * (data.max().unwrap_or_default().into());
        Bound { min, max }
    }

    fn mid_point(&self) -> f64 {
        (self.max + self.min) / 2.0
    }

    pub(crate) fn range(&self) -> f64 {
        self.max - self.min
    }

    fn transform(&self, zoom_factor: f64, delta: f64) -> Self {
        Self {
            min: (self.min - self.mid_point()) / zoom_factor + delta,
            max: (self.max - self.mid_point()) / zoom_factor + delta,
        }
    }
}

pub(crate) type Bounds = Pair<Bound>;

impl Bounds {
    pub(crate) fn mid_point(&self) -> Point {
        Point {
            time: self.time.mid_point(),
            intensity: self.intensity.mid_point(),
        }
    }

    pub(crate) fn transform(&self, zoom_factor: f64, delta: &Point) -> Self {
        Self {
            time: self.time.transform(zoom_factor, delta.time),
            intensity: self.intensity.transform(zoom_factor, delta.intensity),
        }
    }
}

pub(crate) type Point = Pair<f64>;

impl From<Point> for (f64, f64) {
    fn from(value: Point) -> Self {
        (value.time, value.intensity)
    }
}
