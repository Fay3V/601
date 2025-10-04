use sm::sm::StateMachine;
use sm::sm_course::{Delay, Wire};
use std::ops::Add;
use std::ops::Mul;

fn main() {
    let mut factorial = Delay::new(1)
        .feedbackOp(Wire::default(), i32::add)
        .cascade(Delay::new(1).feedbackOp(Wire::default(), i32::mul));
    // for v in factorial.iter(std::iter::repeat(1)) {
    //     println!("{v}");
    // }
    for v in std::iter::repeat(1).filter_map(factorial.start()) {
        println!("{v}");
    }
}
