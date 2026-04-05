// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, time::Instant};

use plotters::prelude::*;
use rand::Rng;

/// Approach A: Convert &str to String, then use HashMap::get()
fn search_approach_a(
    map: &HashMap<(String, String), u32>,
    key: (&str, &str),
) -> Option<u32> {
    map.get(&(key.0.to_string(), key.1.to_string())).copied()
}

/// Approach B: Iterate over HashMap, compare keys using as_str()
fn search_approach_b(
    map: &HashMap<(String, String), u32>,
    key: (&str, &str),
) -> Option<u32> {
    map.iter()
        .find(|((k1, k2), _)| k1.as_str() == key.0 && k2.as_str() == key.1)
        .map(|(_, v)| *v)
}

/// Generate a random string of given length
fn random_string(rng: &mut impl Rng, len: usize) -> String {
    (0..len)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect()
}

/// Benchmark a search function over multiple queries, returns average time in
/// nanoseconds
fn benchmark<F>(
    map: &HashMap<(String, String), u32>,
    search_fn: F,
    queries: &[(&str, &str)],
) -> f64
where
    F: Fn(&HashMap<(String, String), u32>, (&str, &str)) -> Option<u32>,
{
    let iterations = queries.len();
    let total_start = Instant::now();
    for q in queries {
        let _ = search_fn(map, *q);
    }
    let total_elapsed = total_start.elapsed().as_nanos() as f64;
    total_elapsed / iterations as f64
}

fn main() {
    let mut rng = rand::thread_rng();

    // Different HashMap sizes for X-axis
    let sizes: Vec<usize> = (100..=10_000).step_by(100).collect();

    let mut results_a: Vec<f64> = Vec::new();
    let mut results_b: Vec<f64> = Vec::new();

    for &size in &sizes {
        // Build HashMap
        let mut map: HashMap<(String, String), u32> =
            HashMap::with_capacity(size);
        for i in 0..size {
            let k1 = random_string(&mut rng, 10);
            let k2 = random_string(&mut rng, 10);
            map.insert((k1, k2), i as u32);
        }

        // Collect existing keys for queries (to ensure hits)
        let keys: Vec<(String, String)> =
            map.keys().take(100).cloned().collect();
        let queries: Vec<(&str, &str)> = keys
            .iter()
            .map(|(k1, k2)| (k1.as_str(), k2.as_str()))
            .collect();

        // Benchmark both approaches
        let time_a = benchmark(&map, search_approach_a, &queries);
        let time_b = benchmark(&map, search_approach_b, &queries);

        results_a.push(time_a);
        results_b.push(time_b);

        println!(
            "size={}, approach_A={:.2} ns/lookup, approach_B={:.2} ns/lookup",
            size, time_a, time_b
        );
    }

    // Generate plot
    let root =
        BitMapBackend::new("benchmark.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let max_y = results_b.iter().cloned().fold(f64::MIN, f64::max) * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "HashMap Lookup: Approach A vs Approach B",
            ("sans-serif", 30),
        )
        .margin(10)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(0u32..*sizes.last().unwrap() as u32, 0f64..max_y)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("HashMap Size (number of entries)")
        .y_desc("Average Lookup Time (ns)")
        .x_label_style(("sans-serif", 15))
        .y_label_style(("sans-serif", 15))
        .draw()
        .unwrap();

    // Approach A: Blue line
    chart
        .draw_series(LineSeries::new(
            sizes
                .iter()
                .zip(results_a.iter())
                .map(|(&x, &y)| (x as u32, y)),
            &BLUE,
        ))
        .unwrap()
        .label("A: to_string() + get() [O(1)]")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Approach B: Red line
    chart
        .draw_series(LineSeries::new(
            sizes
                .iter()
                .zip(results_b.iter())
                .map(|(&x, &y)| (x as u32, y)),
            &RED,
        ))
        .unwrap()
        .label("B: iter() + as_str() [O(n)]")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();

    println!("\nPlot saved to benchmark.png");
}
