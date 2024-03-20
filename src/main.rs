mod engine;
mod types;

use crate::engine::Engine;

fn main() {
    let filename = std::env::args().nth(1);

    if filename == None {
        println!("Error occured: Missing filename argument");
        std::process::exit(1);
    }

    let mut engine = Engine::new();

    if let Err(err) = engine.read_and_process_input(&filename.unwrap()) {
        println!("Error occured: {}", err);
        std::process::exit(1);
    }

    engine.print_clients();
}
