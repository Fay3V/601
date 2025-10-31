use crate::{
    poly::{DispPoly, Poly},
    sm::StateMachine,
};
#[cfg(feature = "poles")]
use faer::complex::Complex;

use std::{fmt::Display, ops::Deref};

#[derive(Debug)]
pub struct Poles(Vec<Pole>);

impl From<Vec<Pole>> for Poles {
    fn from(value: Vec<Pole>) -> Self {
        Self(value)
    }
}

impl Deref for Poles {
    type Target = Vec<Pole>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Poles {
    pub fn magnitudes(&self) -> Vec<f64> {
        self.0.iter().map(Pole::magnitude).collect()
    }
    pub fn dominant(&self) -> Pole {
        let dominant = self
            .0
            .iter()
            .map(|p| (p, p.magnitude()))
            .max_by(|(_, m1), (_, m2)| m1.total_cmp(m2))
            .expect("cannot find a dominant");
        *dominant.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Pole {
    Real(f64),
    Complex(f64, f64),
}

impl Pole {
    pub fn magnitude(&self) -> f64 {
        match self {
            Pole::Real(r) => r.abs(),
            Pole::Complex(re, im) => (re.powi(2) + im.powi(2)).sqrt(),
        }
    }
}

#[cfg(feature = "poles")]
impl From<Complex<f64>> for Pole {
    fn from(value: Complex<f64>) -> Self {
        if value.im == 0.0 {
            Pole::Real(value.re)
        } else {
            Pole::Complex(value.re, value.im)
        }
    }
}

pub struct SystemFunction {
    numerator: Poly,
    denominator: Poly,
}

impl SystemFunction {
    pub fn new(numerator: Poly, denominator: Poly) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn cascade(self, other: SystemFunction) -> Self {
        Self {
            numerator: self.numerator * other.numerator,
            denominator: self.denominator * other.denominator,
        }
    }

    pub fn feedback_sub(self, other: Option<SystemFunction>) -> Self {
        let SystemFunction {
            numerator: n1,
            denominator: d1,
        } = self;

        let (n2, d2) = match other {
            Some(other) => (other.numerator, other.denominator),
            None => (Poly::new([1.0]), Poly::new([1.0])),
        };

        Self {
            numerator: n1.clone() * d2.clone(),
            denominator: (d1 * d2) + (n1 * n2),
        }
    }

    pub fn feedback_add(self, other: Option<SystemFunction>) -> Self {
        let SystemFunction {
            numerator: n1,
            denominator: d1,
        } = self;
        let (n2, d2) = match other {
            Some(other) => (other.numerator, other.denominator),
            None => (Poly::new([1.0]), Poly::new([1.0])),
        };

        Self {
            numerator: n1.clone() * d2.clone(),
            denominator: (d1 * d2) - (n1 * n2),
        }
    }
}

impl Display for SystemFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SF({} / {})",
            DispPoly::<'R'>(&self.numerator),
            DispPoly::<'R'>(&self.denominator)
        )
    }
}
impl SystemFunction {
    #[cfg(feature = "poles")]
    pub fn poles(&self) -> Poles {
        self.denominator
            .reciprocal()
            .roots()
            .into_iter()
            .map(Pole::from)
            .collect::<Vec<_>>()
            .into()
    }
    pub fn into_sm(
        self,
        prev_inputs: Option<Vec<f64>>,
        prev_outputs: Option<Vec<f64>>,
    ) -> impl StateMachine<f64, f64> {
        LTSIM::from_sf(self, prev_inputs, prev_outputs)
    }
}

pub fn gain(k: f64) -> SystemFunction {
    SystemFunction {
        numerator: Poly::new([k]),
        denominator: Poly::new([1.0]),
    }
}

pub fn delay() -> SystemFunction {
    SystemFunction {
        numerator: Poly::new([1.0, 0.0]),
        denominator: Poly::new([1.0]),
    }
}

struct LTSIM {
    c_coeffs: Vec<f64>,
    d_coeffs: Vec<f64>,
    prev_inputs: Vec<f64>,
    prev_outputs: Vec<f64>,
}

impl LTSIM {
    fn new(
        c_coeffs: Vec<f64>,
        d_coeffs: Vec<f64>,
        prev_inputs: Option<Vec<f64>>,
        prev_outputs: Option<Vec<f64>>,
    ) -> Self {
        let j = d_coeffs.len();
        let k = c_coeffs.len();
        let prev_inputs = prev_inputs
            .map(|mut v| {
                if v.len() >= j {
                    v.drain(j..);
                }
                v.push(0.0);
                v
            })
            .unwrap_or_else(|| vec![0.0; j]);
        let prev_outputs = prev_outputs
            .map(|mut v| {
                if v.len() >= k {
                    v.drain(k..);
                }
                v
            })
            .unwrap_or_else(|| vec![0.0; k]);
        dbg!(&c_coeffs);
        dbg!(&d_coeffs);
        Self {
            c_coeffs,
            d_coeffs,
            prev_inputs,
            prev_outputs,
        }
    }

    fn from_sf(
        sf: SystemFunction,
        prev_inputs: Option<Vec<f64>>,
        prev_outputs: Option<Vec<f64>>,
    ) -> Self {
        println!("{sf}");
        let SystemFunction {
            numerator,
            denominator,
        } = sf;
        let mut c_coeffs = denominator.coeffs();
        c_coeffs.pop();
        c_coeffs.reverse();
        for coeff in &mut c_coeffs {
            *coeff *= -1.0;
        }
        let mut d_coeffs = numerator.coeffs();
        d_coeffs.reverse();
        Self::new(c_coeffs, d_coeffs, prev_inputs, prev_outputs)
    }
}

#[track_caller]
fn dot_product(v1: &[f64], v2: &[f64]) -> f64 {
    assert_eq!(v1.len(), v2.len());
    v1.iter().zip(v2.iter()).map(|(v1, v2)| v1 * v2).sum()
}

impl StateMachine<f64, f64> for LTSIM {
    type State = (Vec<f64>, Vec<f64>);

    fn start_state(&self) -> Self::State {
        (self.prev_inputs.clone(), self.prev_outputs.clone())
    }

    fn next_values(
        &self,
        (mut inputs, mut outputs): Self::State,
        input: Option<f64>,
    ) -> (Self::State, Option<f64>) {
        let inputs_len = inputs.len();
        inputs.rotate_left(inputs_len - 1);
        unsafe { *inputs.get_unchecked_mut(0) = input.expect("no input") };
        let mut output = dot_product(&outputs, &self.c_coeffs);
        output += dot_product(&inputs, &self.d_coeffs);

        let outputs_len = outputs.len();
        outputs.rotate_left(outputs_len - 1);
        unsafe { *outputs.get_unchecked_mut(0) = output };

        ((inputs, outputs), Some(output))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::{
        poly::Poly,
        sf::{LTSIM, Pole, Poles, SystemFunction},
    };

    impl Pole {
        fn into_parts(self) -> (f64, f64) {
            match self {
                Pole::Real(re) => (re, 0.0),
                Pole::Complex(re, im) => (re, im),
            }
        }
    }

    fn sort_poles(poles: &mut Poles) {
        poles.0.sort_by(|p1, p2| {
            let (r1, i1) = p1.into_parts();
            let (r2, i2) = p2.into_parts();
            match r1.total_cmp(&r2) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => i1.total_cmp(&i2),
            }
        });
    }

    fn float_assert_eq(left_val: f64, right_val: f64) {
        if !((left_val - right_val).abs() <= 10e-9) {
            panic!(
                r#"assertion `left == right` failed
left: {left_val:?}
right: {right_val:?}"#
            );
        }
    }

    fn pole_assert_eq(left_val: Pole, right_val: Pole) {
        let (left_re, left_im) = left_val.into_parts();
        let (right_re, right_im) = right_val.into_parts();
        float_assert_eq(left_re, right_re);
        float_assert_eq(left_im, right_im);
    }

    fn vec_float_assert_eq(left_val: &[f64], right_val: &[f64]) {
        assert_eq!(left_val.len(), right_val.len());
        for (left_val, right_val) in left_val.iter().zip(right_val) {
            float_assert_eq(*left_val, *right_val);
        }
    }

    #[test]
    fn test_1() {
        let s = SystemFunction::new(Poly::new([1.0]), Poly::new([0.63, -1.6, 1.0]));
        let mut poles = s.poles();
        sort_poles(&mut poles);
        assert_eq!(poles.len(), 2);
        pole_assert_eq(Pole::Real(0.7), poles[0]);
        pole_assert_eq(Pole::Real(0.9), poles[1]);
        let mut magnitudes = poles.magnitudes();
        magnitudes.sort_by(f64::total_cmp);
        vec_float_assert_eq(&[0.7, 0.9], magnitudes.as_slice());
        pole_assert_eq(Pole::Real(0.9), poles.dominant());
    }

    #[test]
    fn test_1_half() {
        let s = SystemFunction::new(Poly::new([1.0]), Poly::new([-0.99, 0.2, 1.0]));
        let mut poles = s.poles();
        sort_poles(&mut poles);
        assert_eq!(poles.len(), 2);
        pole_assert_eq(Pole::Real(-1.1), poles[0]);
        pole_assert_eq(Pole::Real(0.9), poles[1]);
        let mut magnitudes = poles.magnitudes();
        magnitudes.sort_by(f64::total_cmp);
        vec_float_assert_eq(&[0.9, 1.1], magnitudes.as_slice());
        pole_assert_eq(Pole::Real(-1.1), poles.dominant());
    }

    #[test]
    fn test_2() {
        let s = SystemFunction::new(Poly::new([1.0]), Poly::new([1.1, -1.9, 1.0]));
        let mut poles = s.poles();
        sort_poles(&mut poles);
        assert_eq!(poles.len(), 2);
        pole_assert_eq(Pole::Complex(0.95, -0.44440972086577957), poles[0]);
        pole_assert_eq(Pole::Complex(0.95, 0.44440972086577957), poles[1]);
        let mut magnitudes = poles.magnitudes();
        magnitudes.sort_by(f64::total_cmp);
        vec_float_assert_eq(
            &[1.0488088481701516, 1.0488088481701516],
            magnitudes.as_slice(),
        );
        // cannot assert now the dominant because two of them can be the result
    }

    #[test]
    fn test_combinaison() {
        let s1 = SystemFunction::new(Poly::new([-2.0]), Poly::new([1.0]));
        let s2 = SystemFunction::new(Poly::new([-0.1, 0.0]), Poly::new([-1.0, 1.0]));
        // println!("{s1}");
        // println!("{s2}");
        let s3 = s1.cascade(s2);
        // println!("{s3}");
        let s4 = s3.feedback_sub(None);
        // println!("{s4}");
        let poles = s4.poles();
        assert_eq!(poles.len(), 1);
        pole_assert_eq(Pole::Real(0.8), poles[0])
    }

    const EPSILON: f64 = 1e-10;

    fn assert_vec_approx_eq(actual: &[f64], expected: &[f64], msg: &str) {
        assert_eq!(actual.len(), expected.len(), "{}: length mismatch", msg);
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                (a - e).abs() < EPSILON,
                "{}: element {} differs: expected {}, got {}",
                msg,
                i,
                e,
                a
            );
        }
    }

    fn create_ltsim(numerator: Vec<f64>, denominator: Vec<f64>) -> LTSIM {
        let sf = SystemFunction {
            numerator: Poly::from_vec(numerator),
            denominator: Poly::from_vec(denominator),
        };
        LTSIM::from_sf(sf, None, None)
    }

    #[test]
    fn test_simple_first_order() {
        let numerator = vec![1.0];
        let denominator = vec![1.0, 0.5];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_simple_lowpass() {
        let numerator = vec![0.5, 0.5];
        let denominator = vec![1.0, -0.5];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![0.5, 0.5];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_second_order_system() {
        let numerator = vec![1.0, 2.0, 1.0];
        let denominator = vec![1.0, -0.8, 0.15];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![0.8, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![1.0, 2.0, 1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_gain_only() {
        let numerator = vec![2.5];
        let denominator = vec![1.0];

        let ltsim = create_ltsim(numerator, denominator);

        assert!(ltsim.c_coeffs.is_empty(), "Expected empty cCoeffs");

        let expected_d = vec![2.5];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_integrator() {
        let numerator = vec![1.0];
        let denominator = vec![1.0, -1.0];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_differentiator() {
        // Discrete differentiator: H(z) = (1 - z^-1)
        let numerator = vec![1.0, -1.0];
        let denominator = vec![1.0];

        let ltsim = create_ltsim(numerator, denominator);

        assert!(ltsim.c_coeffs.is_empty(), "Expected empty cCoeffs");

        let expected_d = vec![-1.0, 1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_notch_filter() {
        // Notch filter at Fs/4: H(z) = (1 - z^-2) / (1 - 0.9z^-2)
        let numerator = vec![1.0, 0.0, -1.0];
        let denominator = vec![1.0, 0.0, -0.9];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-0.0, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![-1.0, 0.0, 1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_highpass_first_order() {
        let numerator = vec![0.5, -0.5];
        let denominator = vec![1.0, -0.5];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![-0.5, 0.5];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_resonant_filter() {
        let numerator = vec![1.0, 0.0, 1.0];
        let denominator = vec![1.0, 0.5, 0.9];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-0.5, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![1.0, 0.0, 1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_third_order_butterworth() {
        let numerator = vec![0.1, 0.3, 0.3, 0.1];
        let denominator = vec![1.0, -0.5, 0.3, -0.1];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-0.3, 0.5, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![0.1, 0.3, 0.3, 0.1];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_negative_coefficients() {
        // System with negative coefficients
        let numerator = vec![-1.0, 2.0, -1.0];
        let denominator = vec![1.0, 0.5, -0.3];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-0.5, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![-1.0, 2.0, -1.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_small_coefficients() {
        let numerator = vec![0.001, 0.002];
        let denominator = vec![1.0, 0.0001];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![0.002, 0.001];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_normalized_system() {
        // System with a0 != 1: H(z) = 2 / (2 + z^-1)
        let numerator = vec![2.0];
        let denominator = vec![2.0, 1.0];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![-2.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![2.0];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_fir_filter() {
        let numerator = vec![0.25, 0.5, 0.5, 0.25];
        let denominator = vec![1.0];

        let ltsim = create_ltsim(numerator, denominator);

        assert!(ltsim.c_coeffs.is_empty(), "Expected empty cCoeffs");

        let expected_d = vec![0.25, 0.5, 0.5, 0.25];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }

    #[test]
    fn test_high_order_system() {
        let numerator = vec![0.1, 0.2, 0.3, 0.2, 0.1];
        let denominator = vec![1.0, -0.5, 0.3, -0.1, 0.05];

        let ltsim = create_ltsim(numerator, denominator);

        let expected_c = vec![0.1, -0.3, 0.5, -1.0];
        assert_vec_approx_eq(&ltsim.c_coeffs, &expected_c, "cCoeffs");

        let expected_d = vec![0.1, 0.2, 0.3, 0.2, 0.1];
        assert_vec_approx_eq(&ltsim.d_coeffs, &expected_d, "dCoeffs");
    }
}
