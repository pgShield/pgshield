use lib_engine::Engine;
use tokio;

#[tokio::main]
async fn main() {
    let engine = Engine::new().await.unwrap();
    engine.start();
}