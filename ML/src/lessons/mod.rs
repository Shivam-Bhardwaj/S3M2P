pub mod lesson_00_rust_refresher;
pub mod lesson_01_linear_regression;
pub mod lesson_02_logistic_regression;
pub mod lesson_03_neural_networks;
pub mod lesson_04_cnn;
pub mod lesson_05_policy_networks;
pub mod lesson_06_qlearning;
pub mod lesson_07_policy_gradients;
pub mod lesson_08_mcts;
pub mod lesson_09_alphazero;
pub mod lesson_10_llm;
pub mod lesson_11_agi;

pub fn run_lesson(id: u32) {
    match id {
        0 => lesson_00_rust_refresher::run(),
        1 => lesson_01_linear_regression::run(),
        2 => lesson_02_logistic_regression::run(),
        3 => lesson_03_neural_networks::run(),
        4 => lesson_04_cnn::run(),
        5 => lesson_05_policy_networks::run(),
        6 => lesson_06_qlearning::run(),
        7 => lesson_07_policy_gradients::run(),
        8 => lesson_08_mcts::run(),
        9 => lesson_09_alphazero::run(),
        10 => lesson_10_llm::run(),
        11 => lesson_11_agi::run(),
        _ => println!("Lesson {} not yet implemented.", id),
    }
}
