use clap::{Parser, Subcommand};
use std::path::PathBuf;
use antimony_core::spatial::{SpatialKey, DataLayer};
use glam::Vec3;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate synthetic galaxy data (Random)
    Generate {
        #[arg(short, long, default_value = "./data")]
        output: PathBuf,
        #[arg(short, long, default_value_t = 1_000_000)]
        count: usize,
        #[arg(short, long, default_value_t = 6)]
        level: u8,
    },
    /// Ingest external star catalog (CSV) and restructure for too.foo
    Ingest {
        /// Input CSV file (e.g., HYG database)
        #[arg(short, long)]
        input: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "./data")]
        output: PathBuf,
        /// Max octree level for spatial indexing
        #[arg(short, long, default_value_t = 6)]
        level: u8,
    },
    /// Generate planet trajectories (N-body or Keplerian)
    Orbit {
        #[arg(short, long, default_value = "./data")]
        output: PathBuf,
        /// Simulation duration in years
        #[arg(short, long, default_value_t = 1000)]
        duration: u32,
    }
}

#[derive(serde::Serialize, Clone)]
struct StarData {
    x: f32,
    y: f32,
    z: f32,
    temp: f32,
    mag: f32,
}

// Example CSV structure for a catalog like HYG
#[derive(Debug, Deserialize)]
struct RawStarRecord {
    id: u64,
    x: f32,
    y: f32,
    z: f32,
    mag: Option<f32>,
    ci: Option<f32>, // Color index -> Temp
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output, count, level } => {
            generate_synthetic(output, count, level)?;
        }
        Commands::Ingest { input, output, level } => {
            ingest_catalog(input, output, level)?;
        }
        Commands::Orbit { output, duration } => {
            generate_orbits(output, duration)?;
        }
    }

    Ok(())
}

fn generate_synthetic(output: PathBuf, count: usize, level: u8) -> anyhow::Result<()> {
    println!("🚀 Initializing Synthetic Galaxy...");
    let mut rng = rand::thread_rng();
    let mut spatial_map: HashMap<SpatialKey, Vec<StarData>> = HashMap::new();

    let pb = ProgressBar::new(count as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"));

    for i in 0..count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU * 5.0);
        let dist = rng.gen_range(1.0..1000.0);
        let spread = rng.gen_range(-50.0..50.0);
        
        let x = angle.cos() * dist + rng.gen_range(-10.0..10.0);
        let y = spread * (-dist / 1000.0).exp();
        let z = angle.sin() * dist + rng.gen_range(-10.0..10.0);
        
        let pos = Vec3::new(x, y, z);
        let key = SpatialKey::from_point(pos.normalize(), level);
        
        let star = StarData {
            x: pos.x, y: pos.y, z: pos.z,
            temp: rng.gen_range(2000.0..10000.0),
            mag: rng.gen_range(-1.0..15.0),
        };

        spatial_map.entry(key).or_default().push(star);
        
        if i % 1000 == 0 { pb.inc(1000); }
    }
    pb.finish_with_message("Simulation Complete");

    write_chunks(spatial_map, output, "stars")
}

fn ingest_catalog(input: PathBuf, output: PathBuf, level: u8) -> anyhow::Result<()> {
    println!("📥 Ingesting Catalog: {:?}", input);
    println!("   Target Layout: too.foo spatial format (L{})", level);

    let mut rdr = csv::Reader::from_path(input)?;
    let mut spatial_map: HashMap<SpatialKey, Vec<StarData>> = HashMap::new();
    let mut count = 0;

    for result in rdr.deserialize() {
        let record: RawStarRecord = match result {
            Ok(rec) => rec,
            Err(_) => continue, // Skip bad rows
        };

        let pos = Vec3::new(record.x, record.y, record.z);
        // Skip stars at origin or invalid
        if pos.length_squared() < 0.001 { continue; }

        let key = SpatialKey::from_point(pos.normalize(), level);
        
        // Estimate temp from Color Index (CI)
        let temp = match record.ci {
            Some(ci) => 4600.0 * ((1.0 / (0.92 * ci + 1.7)) + (1.0 / (0.92 * ci + 0.62))),
            None => 5700.0, // Sun default
        };

        let star = StarData {
            x: record.x,
            y: record.y,
            z: record.z,
            temp,
            mag: record.mag.unwrap_or(10.0),
        };

        spatial_map.entry(key).or_default().push(star);
        count += 1;
    }

    println!("✅ Parsed {} stars", count);
    write_chunks(spatial_map, output, "stars")
}

fn generate_orbits(_output: PathBuf, duration: u32) -> anyhow::Result<()> {
    println!("🪐 Generating Planet Trajectories ({} years)...", duration);
    // Placeholder for N-body simulation logic
    // In reality, this would output a different layer, e.g., "orbits"
    // Storing splines or sampled points.
    
    println!("⚠️  N-body simulation module not yet linked to output.");
    Ok(())
}

fn write_chunks(map: HashMap<SpatialKey, Vec<StarData>>, output: PathBuf, layer: &str) -> anyhow::Result<()> {
    println!("💾 Writing spatial chunks...");
    let pb = ProgressBar::new(map.len() as u64);
    
    for (key, data) in map {
        let (face, level, x, y) = (key.face(), key.level(), key.coords().0, key.coords().1);
        
        let mut path = output.clone();
        path.push(layer);
        path.push(face.to_string());
        path.push(level.to_string());
        path.push(x.to_string());
        
        std::fs::create_dir_all(&path)?;
        path.push(format!("{}.bin", y));
        
        let encoded: Vec<u8> = bincode::serialize(&data)?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        pb.inc(1);
    }
    
    pb.finish_with_message("Write Complete");
    Ok(())
}
