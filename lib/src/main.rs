use sm::{
    self,
    opt::{opt_over_line, range},
};
use std::cmp::Ordering;

fn main() {
    // let k1 = 10.;
    // let k2 = 10.;
    for k3 in [1., 3., 10., 30.] {
        // for k1 in [30.] {
        // for k1 in [100.] {
        // for k1 in [300.] {
        // for k1 in [10., 30., 100., 300.] {
        //
        let objective = |k4| {
            let sf = sm::angle_plus_prop_model(k3, k4);
            // eprintln!("{k2} => {sf}");
            let poles = sf.poles();
            let dominant = poles.dominant();
            // dbg!(dominant);
            dominant.magnitude()
        };

        let best = opt_over_line(
            objective,
            range(-k3 * 50., k3 * 50., (1000. * k3.abs() / 0.1).ceil() as u32),
            |m1, m2| matches!(m1.total_cmp(&m2), Ordering::Less),
        );
        dbg!((k3, best));
        let sf = sm::angle_plus_prop_model(k3, best.1);
        eprintln!("{sf}");
    }

    // for k in [1.0, 10.0, 0.1] {
    //     let sf = sm::wall_follower_model(k, 0.1, 0.1);
    //     let poles = sf.poles();
    //     let dominant = poles.dominant();
    //     dbg!(dominant);

    //     match dominant {
    //         sm::sf::Pole::Real(_) => unreachable!(),
    //         sm::sf::Pole::Complex(re, im) => {
    //             let omega = im.atan2(re).abs();
    //             // if omega < 0.0 {
    //             //     omega += 2.0 * PI
    //             // };
    //             let period = 2.0 * PI / omega;
    //             println!("k={k}, period={period}");
    //         }
    //     }
    // }
}
