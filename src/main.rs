// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, time::Instant};

use plotters::prelude::*;
use rand::{prelude::SliceRandom, Rng};

/// Approach get: Convert &str to String, then use HashMap::get()
fn search_approach_get(
    map: &HashMap<(String, String), u32>,
    key: (&str, &str),
) -> Option<u32> {
    map.get(&(key.0.to_string(), key.1.to_string())).copied()
}

/// Approach iter: Iterate over HashMap, compare keys using as_str()
fn search_approach_iter(
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

    let mut results_get: Vec<f64> = Vec::new();
    let mut results_iter: Vec<f64> = Vec::new();

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
        // Randomly sample 100 keys from all keys
        let all_keys: Vec<(String, String)> = map.keys().cloned().collect();
        let keys: Vec<(String, String)> = all_keys
            .choose_multiple(&mut rng, 100.min(all_keys.len()))
            .cloned()
            .collect();
        let queries: Vec<(&str, &str)> = keys
            .iter()
            .map(|(k1, k2)| (k1.as_str(), k2.as_str()))
            .collect();

        let time_get = benchmark(&map, search_approach_get, &queries);

        // Build a fresh HashMap for Approach iter to avoid CPU cache advantage
        let mut map_b: HashMap<(String, String), u32> =
            HashMap::with_capacity(size);
        for i in 0..size {
            let k1 = random_string(&mut rng, 10);
            let k2 = random_string(&mut rng, 10);
            map_b.insert((k1, k2), i as u32);
        }

        // Collect existing keys for queries (to ensure hits)
        // Randomly sample 100 keys from all keys
        let all_keys: Vec<(String, String)> = map_b.keys().cloned().collect();
        let keys: Vec<(String, String)> = all_keys
            .choose_multiple(&mut rng, 100.min(all_keys.len()))
            .cloned()
            .collect();
        let queries: Vec<(&str, &str)> = keys
            .iter()
            .map(|(k1, k2)| (k1.as_str(), k2.as_str()))
            .collect();
        let time_iter = benchmark(&map_b, search_approach_iter, &queries);

        results_get.push(time_get);
        results_iter.push(time_iter);

        println!(
            "size={}, approach_get={:.2} ns/lookup, approach_iter={:.2} \
             ns/lookup",
            size, time_get, time_iter
        );
    }

    // Generate plot
    let root =
        BitMapBackend::new("benchmark.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let max_y = results_iter.iter().cloned().fold(f64::MIN, f64::max) * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption("HashMap Lookup: get() vs iter()", ("sans-serif", 30))
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

    // Approach get: Blue line
    chart
        .draw_series(LineSeries::new(
            sizes
                .iter()
                .zip(results_get.iter())
                .map(|(&x, &y)| (x as u32, y)),
            &BLUE,
        ))
        .unwrap()
        .label("get() + to_string() [O(1)]")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    // Approach iter: Red line
    chart
        .draw_series(LineSeries::new(
            sizes
                .iter()
                .zip(results_iter.iter())
                .map(|(&x, &y)| (x as u32, y)),
            &RED,
        ))
        .unwrap()
        .label("iter() + as_str() [O(n)]")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();

    println!("\nPlot saved to benchmark.png");
}
