use crate::Real;

use crate::window::{smoothing_window::SmoothingWindow, Window};

use crate::tracedata::{SNRSign, Stats};

#[derive(Default, Clone)]
pub struct NoiseSmoothingWindow {
    threshold: Real,
    _influence: Real, //  Maybe we should do something with this?
    position: Real,
    window: SmoothingWindow,
}
impl NoiseSmoothingWindow {
    pub fn new(size: usize, threshold: Real, _influence: Real) -> Self {
        NoiseSmoothingWindow {
            threshold,
            _influence,
            window: SmoothingWindow::new(size),
            ..Default::default()
        }
    }
}
impl Window for NoiseSmoothingWindow {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value: Real) -> bool {
        if let Some(mut stats) = self.window.stats() {
            //let old_value = stats.value;
            stats.value = value - self.position;
            if SNRSign::Zero == stats.signal_over_noise_sign(self.threshold) {
                self.window.push(stats.value)
            } else {
                self.position = value - stats.mean;
                true
            }
        } else {
            self.window.push(value)
        }
    }
    fn stats(&self) -> Option<Self::OutputType> {
        let mut stats = self.window.stats()?;
        stats.shift(self.position);
        Some(stats)
    }
    fn apply_time_shift(&self, time: Real) -> Real {
        self.window.apply_time_shift(time)
    }
}

#[cfg(test)]
mod tests {
    use crate::processing;

    use super::super::iter::WindowFilter;
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    #[should_panic]
    fn test_window_size_zero() {
        let data = [4.0, 3.0, 2.0, 5.0, 6.0, 1.0, 5.0, 7.0, 2.0, 4.0];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(0, 1., 0.));
    }

    #[test]
    #[should_panic]
    fn test_window_size_one() {
        let data = [4.0, 3.0, 2.0, 5.0, 6.0, 1.0, 5.0, 7.0, 2.0, 4.0];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(1, 1., 0.));
    }

    #[test]
    fn test_window_size_two() {
        let data = [4.0, 3.0, 2.0, 5.0, 6.0, 1.0, 5.0, 7.0, 2.0, 4.0];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(2, 1., 0.))
            .next()
            .is_some());
    }

    #[test]
    fn test_no_data() {
        let data = [];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(3, 1., 0.))
            .next()
            .is_none());
    }
    #[test]
    fn test_insufficient_data() {
        let data = [4.0, 3.0];
        assert!(data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(3, 1., 0.))
            .next()
            .is_none());
    }
    #[test]
    fn test_minimal() {
        let data = [4.0, 3.0];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(2, 1., 0.))
            .next()
            .unwrap();
        println!("{i}, {stats}");
        assert_eq!(i, 1.);
        assert_eq!(stats.value, 3.);
        assert_approx_eq!(stats.mean, 7. / 2.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 7. / 2.).powi(2) + (3. as Real - 7. / 2.).powi(2)) / (2. - 1.)
        );
    }
    #[test]
    fn test_three_data() {
        let data = [4.0, 3.0, 1.0];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(2, 1., 0.))
            .skip(1)
            .next()
            .unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 1.5);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 3.5).powi(2) + (3. as Real - 3.5).powi(2)) / (2. - 1.)
        );
    }

    #[test]
    fn test_three_data_three_window() {
        let data = [4.0, 3.0, 1.0];
        let (i, stats) = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(3, 1., 0.))
            .next()
            .unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 8. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 8. / 3.).powi(2)
                + (3. as Real - 8. / 3.).powi(2)
                + (1. as Real - 8. / 3.).powi(2))
                / (3. - 1.)
        );
    }

    #[test]
    fn test_five_data_three_window() {
        let data = [4.0, 3.0, 1.0, 5.0, 3.0];
        let mut itr = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(NoiseSmoothingWindow::new(3, 1., 0.));
        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 2.);
        assert_eq!(stats.value, 1.);
        assert_approx_eq!(stats.mean, 8. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 8. / 3.).powi(2)
                + (3. as Real - 8. / 3.).powi(2)
                + (1. as Real - 8. / 3.).powi(2))
                / (3. - 1.)
        );

        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 3.);
        assert_eq!(stats.value, 5.);
        assert_approx_eq!(stats.mean, 8. / 3. + 4.);
        assert_approx_eq!(
            stats.variance,
            ((4. as Real - 8. / 3.).powi(2)
                + (3. as Real - 8. / 3.).powi(2)
                + (1. as Real - 8. / 3.).powi(2))
                / (3. - 1.)
        );

        let (i, stats) = itr.next().unwrap();
        assert_eq!(i, 4.);
        assert_eq!(stats.value, 3.);
        assert_approx_eq!(stats.mean, 7. / 3.);
        assert_approx_eq!(
            stats.variance,
            ((1. as Real - 9. / 3.).powi(2)
                + (5. as Real - 9. / 3.).powi(2)
                + (3. as Real - 9. / 3.).powi(2))
                / (3. - 1.)
        );
    }
    /*
    #[test]
    fn test_variance_accuracy() {
        use rand::random;
        let data: Vec<Intensity> = (0..1000).map(|_| random::<Intensity>()).collect();

        for window_size in 2..100 {
            let smoothing_window = NoiseSmoothingWindow::new(window_size, 1., 0.);
            let mut itr = data
                .iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(smoothing_window);
            while let Some(stat) = itr.next() {
                assert_approx_eq!(stat.1.mean, itr.get_window().test_mean());
                assert_approx_eq!(stat.1.variance, itr.get_window().test_variance());
            }
        }
    }

    #[test]
    fn test_variance_zero_on_constant_sequence() {
        use rand::random;
        let num = random::<Intensity>();
        let data: Vec<Intensity> = vec![num; 1000];

        for window_size in 2..100 {
            let smoothing_window = NoiseSmoothingWindow::new(window_size, 1., 0.);
            let mut itr = data
                .iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(smoothing_window);
            while let Some(stat) = itr.next() {
                assert_eq!(stat.1.mean, num as Real);
                assert_eq!(stat.1.variance, 0.);
            }
        }
    } */
}
