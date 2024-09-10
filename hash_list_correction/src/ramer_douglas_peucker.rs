//! Submodule providing the Ramer-Douglas-Peucker algorithm for line simplification.

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    x: f64,
    y: f64,
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

impl Point {
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Get the x-coordinate of the point
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Get the y-coordinate of the point
    pub fn y(&self) -> f64 {
        self.y
    }
}

// Compute the perpendicular distance from point 'p' to the line segment from 'start' to 'end'
fn perpendicular_distance(point: &Point, start: &Point, end: &Point) -> f64 {
    let line_length = start.distance_to(end);
    if line_length == 0.0 {
        return point.distance_to(start);
    }

    // Compute the projection of 'p' onto the line segment
    let t = ((point.x - start.x) * (end.x - start.x) + (point.y - start.y) * (end.y - start.y))
        / line_length.powi(2);

    // If the projection is outside the line segment, return the distance to the closest endpoint
    if t < 0.0 {
        point.distance_to(start)
    } else if t > 1.0 {
        point.distance_to(end)
    } else {
        // Otherwise, return the distance to the line
        let projection = Point {
            x: start.x + t * (end.x - start.x),
            y: start.y + t * (end.y - start.y),
        };
        point.distance_to(&projection)
    }
}

/// Recursive function for the Ramer-Douglas-Peucker algorithm
///
/// # Arguments
/// * `points` - The list of points to simplify
/// * `tolerance` - The maximum distance from the simplified line
/// * `max_points` - The maximum number of points to return
pub fn rdp(points: &[Point], tolerance: f64, max_points: usize) -> Vec<Point> {
    if points.len() < 2 {
        return points.to_vec();
    }

    // Find the point with the maximum distance from the line segment connecting the first and last points
    let (index, max_distance) = points
        .iter()
        .enumerate()
        .skip(1)
        .take(points.len() - 2)
        .map(|(i, point)| {
            (
                i,
                perpendicular_distance(point, &points[0], &points[points.len() - 1]),
            )
        })
        .fold((0, 0.0), |(max_index, max_dist), (i, dist)| {
            if dist > max_dist {
                (i, dist)
            } else {
                (max_index, max_dist)
            }
        });

    // If the maximum distance is greater than the tolerance, recursively simplify
    if max_distance > tolerance && max_points > 2 {
        let mut result1 = rdp(&points[..=index], tolerance, max_points / 2);
        let mut result2 = rdp(&points[index..], tolerance, max_points / 2);

        // Combine the results, removing the duplicate point at index
        result1.pop();
        result1.append(&mut result2);

        result1
    } else {
        // If no point is farther than the tolerance, return just the endpoints
        vec![points[0], points[points.len() - 1]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_simplification() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
            Point { x: 3.0, y: 0.0 },
            Point { x: 4.0, y: 0.0 },
        ];

        let tolerance = 0.001;
        let simplified = rdp(&points, tolerance, 10);

        // With a low tolerance, no simplification should occur
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified, [points[0], points[points.len() - 1]]);
    }

    #[test]
    fn test_full_simplification() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
            Point { x: 3.0, y: 0.0 },
            Point { x: 4.0, y: 0.0 },
        ];

        let tolerance = 1.0;
        let simplified = rdp(&points, tolerance, 10);

        // All points are on a straight line, so the algorithm should return just the endpoints
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified, vec![points[0], points[points.len() - 1]]);
    }

    #[test]
    fn test_partial_simplification() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.1 },
            Point { x: 2.0, y: -0.1 },
            Point { x: 3.0, y: 5.0 },
            Point { x: 4.0, y: 6.0 },
            Point { x: 5.0, y: 7.0 },
        ];

        let tolerance = 1.0;
        let simplified = rdp(&points, tolerance, 10);

        assert_eq!(simplified.len(), 4);
        assert_eq!(simplified, vec![points[0], points[2], points[3], points[5]]);
    }

    #[test]
    fn test_high_tolerance() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 2.0 },
            Point { x: 2.0, y: 1.0 },
            Point { x: 3.0, y: 4.0 },
            Point { x: 4.0, y: 3.0 },
            Point { x: 5.0, y: 6.0 },
        ];

        let tolerance = 5.0;
        let simplified = rdp(&points, tolerance, 10);

        // With a high tolerance, the algorithm should return just the first and last points
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified, vec![points[0], points[points.len() - 1]]);
    }

    #[test]
    fn test_single_point() {
        let points = vec![Point { x: 0.0, y: 0.0 }];

        let tolerance = 0.1;
        let simplified = rdp(&points, tolerance, 10);

        // A single point cannot be simplified
        assert_eq!(simplified.len(), 1);
        assert_eq!(simplified, points);
    }

    #[test]
    fn test_two_points() {
        let points = vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }];

        let tolerance = 0.1;
        let simplified = rdp(&points, tolerance, 10);

        // Two points cannot be simplified further
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified, points);
    }

    #[test]
    fn test_identical_points() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 0.0 },
            Point { x: 0.0, y: 0.0 },
        ];

        let tolerance = 0.1;
        let simplified = rdp(&points, tolerance, 10);

        // If all points are the same, the algorithm should return just one point
        assert_eq!(simplified.len(), 2);
        assert_eq!(simplified, vec![points[0], points[0]]);
    }
}
