use paper::{engine, gpu::object::{Location, ObjectBuilder}, maths::Vec3, EngineRuntime};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut engine = EngineRuntime::new_engine().await?;

  //init_objects(&mut engine.engine).await?;

  return engine.run_engine().await;
}
