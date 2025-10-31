use crate::io::{Point, Pose};
use std::f64::consts::PI;

static SONAR_POSES: [Pose; 8] = [
    Pose {
        pos: Point::new(0.08, 0.134),
        theta: PI / 2.0,
    },
    Pose {
        pos: Point::new(0.122, 0.118),
        theta: 5.0 * PI / 18.0,
    },
    Pose {
        pos: Point::new(0.156, 0.077),
        theta: PI / 6.0,
    },
    Pose {
        pos: Point::new(0.174, 0.0266),
        theta: PI / 18.0,
    },
    Pose {
        pos: Point::new(0.174, -0.0266),
        theta: -PI / 18.0,
    },
    Pose {
        pos: Point::new(0.156, -0.077),
        theta: -PI / 6.0,
    },
    Pose {
        pos: Point::new(0.122, -0.118),
        theta: -5.0 * PI / 18.0,
    },
    Pose {
        pos: Point::new(0.08, -0.134),
        theta: -PI / 2.0,
    },
];

const SONAR_MAX: f64 = 1.5;

pub fn get_distance_right(sonars: &[f64; 8]) -> f64 {
    let mut hits = [None; 8];
    for (hit, (spose, d)) in hits.iter_mut().zip(SONAR_POSES.iter().zip(sonars.iter())) {
        *hit = (*d < SONAR_MAX).then(|| spose.pos + Point::from_polar(*d, spose.theta));
    }
    distance_right(hits[6], hits[7])
}

fn distance_right(h0: Option<Point>, h1: Option<Point>) -> f64 {
    match (h0, h1) {
        (Some(h0), Some(h1)) => {
            let (_, lined) = line(h0, h1);
            lined.abs()
        }
        (Some(h), None) | (None, Some(h)) => h.distance_to_orig(),
        (None, None) => SONAR_MAX,
    }
}

fn line(h0: Point, h1: Point) -> (Point, f64) {
    let delta = h1 - h0;
    let mag = delta.distance_to_orig();
    let nx = delta.y / mag;
    let ny = -delta.x / mag;
    let d = nx * h0.x + ny * h0.y;
    (Point::new(nx, ny), d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 10e-9;

    #[test]
    fn test_parallel_wall_on_right_close() {
        // Robot parallel to wall 0.5m away on right
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.5, 0.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.6228804263).abs() < EPSILON,
            "Expected 0.6228804263, got {}",
            distance
        );
    }

    #[test]
    fn test_parallel_wall_on_right_closer() {
        // Robot parallel to wall 0.3m away
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.3, 0.3];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.4349903570).abs() < EPSILON,
            "Expected 0.4349903570, got {}",
            distance
        );
    }

    #[test]
    fn test_parallel_wall_on_right_far() {
        // Robot parallel to wall 1.2m away
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.2, 1.2];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.2806161772).abs() < EPSILON,
            "Expected 1.2806161772, got {}",
            distance
        );
    }

    #[test]
    fn test_only_sensor_7_detects_wall() {
        // Wall only visible to rightmost sensor
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.4];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.5399592577).abs() < EPSILON,
            "Expected 0.5399592577, got {}",
            distance
        );
    }

    #[test]
    fn test_only_sensor_6_detects_wall() {
        // Wall only visible to front-right sensor
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.6, 1.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.7690149538).abs() < EPSILON,
            "Expected 0.7690149538, got {}",
            distance
        );
    }

    #[test]
    fn test_angled_wall_approaching() {
        // Robot approaching angled wall, sensor 6 closer than 7
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.4, 0.6];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.5675487443).abs() < EPSILON,
            "Expected 0.5675487443, got {}",
            distance
        );
    }

    #[test]
    fn test_angled_wall_diverging() {
        // Robot moving away from angled wall, sensor 7 closer than 6
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.8, 0.4];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.4767210506).abs() < EPSILON,
            "Expected 0.4767210506, got {}",
            distance
        );
    }

    #[test]
    fn test_no_wall_detected() {
        // No walls within sonar range
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.5000000000).abs() < EPSILON,
            "Expected 1.5000000000, got {}",
            distance
        );
    }

    #[test]
    fn test_front_wall_only() {
        // Wall in front but not on right side
        let sonar = [0.3, 0.4, 0.5, 1.0, 1.0, 1.5, 1.5, 1.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.5000000000).abs() < EPSILON,
            "Expected 1.5000000000, got {}",
            distance
        );
    }

    #[test]
    fn test_very_close_wall() {
        // Wall very close on right (10cm)
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.1, 0.1];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.2472166397).abs() < EPSILON,
            "Expected 0.2472166397, got {}",
            distance
        );
    }

    #[test]
    fn test_left_wall_present_right_clear() {
        // Wall on left side, right side clear
        let sonar = [0.5, 0.5, 0.5, 1.5, 1.5, 1.5, 1.5, 1.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.5000000000).abs() < EPSILON,
            "Expected 1.5000000000, got {}",
            distance
        );
    }

    #[test]
    fn test_corner_scenario() {
        // Multiple walls detected, testing right side isolation
        let sonar = [0.5, 0.6, 0.8, 1.0, 1.0, 0.8, 0.6, 0.5];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.6390175828).abs() < EPSILON,
            "Expected 0.6390175828, got {}",
            distance
        );
    }

    #[test]
    fn test_slight_angle_tilted_right() {
        // Robot slightly angled toward wall
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.35, 0.45];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.5168101533).abs() < EPSILON,
            "Expected 0.5168101533, got {}",
            distance
        );
    }

    #[test]
    fn test_slight_angle_tilted_left() {
        // Robot slightly angled away from wall
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.55, 0.45];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.5892889296).abs() < EPSILON,
            "Expected 0.5892889296, got {}",
            distance
        );
    }

    #[test]
    fn test_at_max_range() {
        // Wall exactly at maximum detection range
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.49, 1.49];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.5531196357).abs() < EPSILON,
            "Expected 1.5531196357, got {}",
            distance
        );
    }

    #[test]
    fn test_beyond_max_range() {
        // Wall just beyond detection range
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.51, 1.51];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 1.5000000000).abs() < EPSILON,
            "Expected 1.5000000000, got {}",
            distance
        );
    }

    #[test]
    fn test_zero_distance() {
        // Edge case - sensor touching wall (0 distance)
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 0.0, 0.0];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.1537009827).abs() < EPSILON,
            "Expected 0.1537009827, got {}",
            distance
        );
    }

    #[test]
    fn test_asymmetric_readings() {
        // Sensor 6 much farther than sensor 7
        let sonar = [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.0, 0.2];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.2103004420).abs() < EPSILON,
            "Expected 0.2103004420, got {}",
            distance
        );
    }

    #[test]
    fn test_realistic_corridor() {
        // Robot in corridor with walls on both sides
        let sonar = [0.8, 0.75, 0.7, 0.8, 0.8, 0.7, 0.75, 0.8];
        let distance = get_distance_right(&sonar);
        assert!(
            (distance - 0.8817698608).abs() < EPSILON,
            "Expected 0.8817698608, got {}",
            distance
        );
    }
}
