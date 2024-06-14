mod lib_engine;
extern crate lib_engine;

fn main() {
    let engine = lib_engine::Engine::new().unwrap();
    engine.start();
}
