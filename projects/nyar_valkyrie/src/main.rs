use clap::Parser;
use nyar_valkyrie::Application;

#[tokio::main]
pub async fn main() {
    Application::parse().run().await
}
