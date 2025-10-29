use crate::poly::{DispPoly, Poly};
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
    fn into_parts(self) -> (f64, f64) {
        match self {
            Pole::Real(re) => (re, 0.0),
            Pole::Complex(re, im) => (re, im),
        }
    }

    pub fn magnitude(&self) -> f64 {
        match self {
            Pole::Real(r) => r.abs(),
            Pole::Complex(re, im) => (re.powi(2) + im.powi(2)).sqrt(),
        }
    }
}

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
    pub fn poles(&self) -> Poles {
        self.denominator
            .reciprocal()
            .roots()
            .into_iter()
            .map(Pole::from)
            .collect::<Vec<_>>()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::{
        poly::Poly,
        sf::{Pole, Poles, SystemFunction},
    };

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
}
