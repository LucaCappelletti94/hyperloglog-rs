//! Submodule providing the Ramer-Douglas-Peucker algorithm for line simplification.
use rayon::prelude::*;

/// A point in 2D space
pub trait Point {
    fn distance_to(&self, other: &Self) -> f64 {
        ((self.x() - other.x()).powi(2) + (self.y() - other.y()).powi(2)).sqrt()
    }

    /// Get the x-coordinate of the point
    fn x(&self) -> f64;

    /// Get the y-coordinate of the point
    fn y(&self) -> f64;

    fn perpendicular_distance_to_line(&self, line_start: &Self, line_end: &Self) -> f64 {
        let numerator = ((line_end.y() - line_start.y()) * self.x()
            - (line_end.x() - line_start.x()) * self.y()
            + line_end.x() * line_start.y()
            - line_end.y() * line_start.x())
            .abs();
        let denominator = line_start.distance_to(line_end);
        numerator / denominator
    }   
}

/// Recursive function for the Ramer-Douglas-Peucker algorithm
///
/// # Arguments
/// * `points` - The list of points to simplify
/// * `tolerance` - The maximum distance from the simplified line
pub fn rdp<X: Send + Sync + Point + Copy>(points: &[X], tolerance: f64) -> Vec<X> {
    if points.len() < 2 {
        return points.to_vec();
    }

    // Find the point with the maximum distance from the line segment connecting the first and last points
    let (index, max_distance) = points
        .par_iter()
        .enumerate()
        .skip(1)
        .take(points.len() - 2)
        .map(|(i, point)| {
            (
                i,
                point.perpendicular_distance_to_line(&points[0], &points[points.len() - 1]),
            )
        })
        .reduce(
            || (0, 0.0),
            |(max_index, max_dist), (i, dist)| {
                if dist > max_dist {
                    (i, dist)
                } else {
                    (max_index, max_dist)
                }
            },
        );

    // If the maximum distance is greater than the tolerance, recursively simplify
    if max_distance > tolerance {
        let mut result1 = rdp(&points[..=index], tolerance);
        let mut result2 = rdp(&points[index..], tolerance);

        // Combine the results, removing the duplicate point at index
        result1.pop();
        result1.append(&mut result2);

        result1
    } else {
        // If no point is farther than the tolerance, return just the endpoints
        vec![points[0], points[points.len() - 1]]
    }
}
