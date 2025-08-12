use paper::run_engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  return run_engine().await;
}
