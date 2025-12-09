//! Parameter sweep for ecosystem hyperparameter optimization
//!
//! Run with: cargo run --release --example ecosystem_sweep
//!
//! Tests combinations of parameters to find optimal configuration.
//! Uses parallel execution for faster results.

use dna::wave_field::{Ecosystem, HyperParams, analyze_stability, StabilityReport};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// Result of testing a parameter combination
#[derive(Clone, Debug)]
struct SweepResult {
    params: HyperParams,
    report: StabilityReport,
    score: f32,
}

/// Calculate a score for a stability report (lower = better)
fn calculate_score(report: &StabilityReport, target: usize) -> f32 {
    let mut score = 0.0;

    // Extinction penalty (huge)
    score += report.extinctions as f32 * 1000.0;

    // Population error
    let pop_error = (report.avg_population - target as f32).abs() / target as f32;
    score += pop_error * 100.0;

    // Ratio error (target 65% prey)
    let ratio_error = (report.avg_prey_ratio - 0.65).abs();
    score += ratio_error * 50.0;

    // Oscillation penalty
    score += (report.oscillations as f32 / 100.0).min(10.0);

    // Standard deviation penalty
    score += (report.std_dev / 1000.0).min(20.0);

    // Bonus for settling quickly
    if let Some(settling) = report.settling_time {
        score -= (3600.0 - settling as f32).max(0.0) / 1000.0;
    }

    score
}

/// Run a single parameter test
fn test_params(params: HyperParams, seed: u64, frames: u32) -> SweepResult {
    let target = 15000;
    let mut eco = Ecosystem::with_seed(960, 540, target, seed);
    eco.apply_params(params.clone());
    eco.seed_population(3000, 1500);

    let history = eco.run(frames);
    let report = analyze_stability(&history, target);
    let score = calculate_score(&report, target);

    SweepResult {
        params,
        report,
        score,
    }
}

fn main() {
    println!("=== Ecosystem Parameter Sweep ===\n");

    let start = Instant::now();

    // Parameter grid
    let kp_values = [0.005, 0.01, 0.02, 0.04];
    let ki_values = [0.0001, 0.0005, 0.001];
    let kd_values = [0.05, 0.1, 0.2];
    let bhardwaj_values = [0.4, 0.5, 0.6, 0.7];
    let energy_values = [400, 600, 800, 1000];

    let mut param_combinations: Vec<HyperParams> = Vec::new();

    for &kp in &kp_values {
        for &ki in &ki_values {
            for &kd in &kd_values {
                for &bhardwaj in &bhardwaj_values {
                    for &energy in &energy_values {
                        let mut params = HyperParams::default();
                        params.pid_kp = kp;
                        params.pid_ki = ki;
                        params.pid_kd = kd;
                        params.bhardwaj_constant = bhardwaj;
                        params.predator_energy = energy;
                        param_combinations.push(params);
                    }
                }
            }
        }
    }

    let total = param_combinations.len();
    println!("Testing {} parameter combinations...\n", total);

    // Parallel execution
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    println!("Using {} threads\n", num_threads);

    let results: Arc<Mutex<Vec<SweepResult>>> = Arc::new(Mutex::new(Vec::new()));
    let progress: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let param_queue: Arc<Mutex<Vec<HyperParams>>> = Arc::new(Mutex::new(param_combinations));

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let results = Arc::clone(&results);
            let progress = Arc::clone(&progress);
            let param_queue = Arc::clone(&param_queue);

            thread::spawn(move || {
                loop {
                    // Get next params
                    let params = {
                        let mut queue = param_queue.lock().unwrap();
                        queue.pop()
                    };

                    let params = match params {
                        Some(p) => p,
                        None => break,
                    };

                    // Test with 30-second simulation
                    let result = test_params(params, 42, 1800);

                    // Store result
                    {
                        let mut r = results.lock().unwrap();
                        r.push(result);
                    }

                    // Update progress
                    {
                        let mut p = progress.lock().unwrap();
                        *p += 1;
                        if *p % 50 == 0 {
                            println!("Progress: {}/{}", *p, total);
                        }
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Collect and sort results
    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    let elapsed = start.elapsed();

    println!("\n=== RESULTS ===");
    println!("Elapsed: {:.1}s", elapsed.as_secs_f32());
    println!("\nTop 10 configurations:\n");

    for (i, result) in results.iter().take(10).enumerate() {
        println!("#{}: Score = {:.2}", i + 1, result.score);
        println!("  PID: Kp={:.3}, Ki={:.4}, Kd={:.2}",
                 result.params.pid_kp, result.params.pid_ki, result.params.pid_kd);
        println!("  Bhardwaj: {:.1}, Energy: {}",
                 result.params.bhardwaj_constant, result.params.predator_energy);
        println!("  Results: pop={:.0}±{:.0}, ratio={:.1}%, extinctions={}",
                 result.report.avg_population, result.report.std_dev,
                 result.report.avg_prey_ratio * 100.0, result.report.extinctions);
        println!();
    }

    // Best result
    if let Some(best) = results.first() {
        println!("=== OPTIMAL PARAMETERS ===\n");
        println!("pub const OPTIMAL_PARAMS: HyperParams = HyperParams {{");
        println!("    pid_kp: {},", best.params.pid_kp);
        println!("    pid_ki: {},", best.params.pid_ki);
        println!("    pid_kd: {},", best.params.pid_kd);
        println!("    bhardwaj_constant: {},", best.params.bhardwaj_constant);
        println!("    wave_spawn_rate: {},", best.params.wave_spawn_rate);
        println!("    max_waves: {},", best.params.max_waves);
        println!("    predator_energy: {},", best.params.predator_energy);
        println!("    predator_hunt_chance: {},", best.params.predator_hunt_chance);
        println!("    sample_rate: {},", best.params.sample_rate);
        println!("    adaptive_sampling: {},", best.params.adaptive_sampling);
        println!("    enable_direct_spawn: {},", best.params.enable_direct_spawn);
        println!("    direct_spawn_rate: {},", best.params.direct_spawn_rate);
        println!("}};");
    }
}
