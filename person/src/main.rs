use paper::{engine, gpu::object::{ObjectBuilder}, EngineRuntime};

async fn init_objects(e: &mut engine::Engine) -> anyhow::Result<()> {
  e.texture_bundle
    .add_texture_from_file(&e.drivers, "test_bake.png", "test")?;

  let diffuse = e.texture_bundle.get_texture_bind("test");

  let object = ObjectBuilder::new()
    .load_meshes_from_objfile(&e.texture_bundle, &e.drivers, "test_bake_table.obj")?
    .add_diffuse_texture(diffuse.clone())
    .build();

  e.render_task
    .add_object(object, &e.drivers, &e.data_bindgroups)
    .await?;

  Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut engine = EngineRuntime::new_engine().await?;

  init_objects(&mut engine.engine).await?;

  return engine.run_engine().await;
}
