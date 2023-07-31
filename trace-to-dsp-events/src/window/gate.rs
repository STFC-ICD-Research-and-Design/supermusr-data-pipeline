use crate::Real;

use super::Window;



#[derive(Default)]
pub struct Gate {
    threshold: Real,
    value: Option<Real>,
}
impl Gate {
    pub fn new(threshold : f64) -> Self {
        if threshold <= 0. {
            panic!("Threshold must be positive");
        }
        Gate { threshold, ..Default::default() }
    }
}
impl Window for Gate {
    type InputType = Real;
    type OutputType = Real;

    fn push(&mut self, value : Real) -> bool {
        match self.value {
            Some(old_value) => {
                if (old_value - value).abs() >= self.threshold {
                    self.value = Some(value);
                }
            },
            None => {
                self.value = Some(value);
            }
        }
        true
    }
    fn stats(&self) -> Option<Real> {
        self.value
    }
}




#[cfg(test)]
mod tests {
    use crate::processing;

    use super::*;
    use super::super::WindowFilter;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    #[should_panic]
    fn test_gate_zero_threshold() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        data.iter().enumerate().map(processing::make_enumerate_real)
            .window(Gate::new(0.));
    }
    
    #[test]
    #[should_panic]
    fn test_gate_negative_threshold() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        data.iter().enumerate().map(processing::make_enumerate_real)
        .window(Gate::new(-5.0));
    }

    #[test]
    fn test_gate_no_data() {
        let data = [];
        assert!(
            data.iter().enumerate().map(processing::make_enumerate_real)
                .window(Gate::new(5.0))
                .next()
                .is_none()
        );
    }
    #[test]
    fn test_gate() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        assert!(
            data.iter().enumerate().map(processing::make_enumerate_real)
                .window(Gate::new(4.0))
                .next()
                .is_some()
        );
    }
    #[test]
    fn test_gate_accurate() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        let mut itr = data.iter().enumerate().map(processing::make_enumerate_real)
            .window(Gate::new(3.0));

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,0.);
        assert_eq!(value,4.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,1.);
        assert_eq!(value,4.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,2.);
        assert_eq!(value,4.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,3.);
        assert_eq!(value,4.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,4.);
        assert_eq!(value,4.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,5.);
        assert_eq!(value,1.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,6.);
        assert_eq!(value,5.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,7.);
        assert_eq!(value,5.);

        let (i,value) = itr.next().unwrap();
        assert_eq!(i,8.);
        assert_eq!(value,2.);

        assert!(itr.next().is_none());
    }

    
}