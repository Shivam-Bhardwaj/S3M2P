mod engine;
mod lessons;
mod utils;
mod web;

#[tokio::main]
async fn main() {
    println!("Welcome to the Rust ML Journey: Zero to AGI!");
    println!("============================================\n");
    
    // 1. Run all lessons to generate visualizations
    println!("Generating lesson visualizations...\n");
    
    for lesson_id in 0..=11 {
        lessons::run_lesson(lesson_id);
        println!(); // Blank line between lessons
    }
    
    println!("============================================");
    println!("All lessons completed! Starting web server...\n");

    // 2. Start the Web Server
    web::start_server().await;
}
