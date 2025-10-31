use std::f64::consts::PI;

use sm;

fn main() {
    for k in [1.0, 10.0, 0.1] {
        let sf = sm::wall_follower_model(k, 0.1, 0.1);
        let poles = sf.poles();
        let dominant = poles.dominant();
        dbg!(dominant);

        match dominant {
            sm::sf::Pole::Real(_) => unreachable!(),
            sm::sf::Pole::Complex(re, im) => {
                let omega = im.atan2(re).abs();
                // if omega < 0.0 {
                //     omega += 2.0 * PI
                // };
                let period = 2.0 * PI / omega;
                println!("k={k}, period={period}");
            }
        }
    }
}
