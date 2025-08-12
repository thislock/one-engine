use cgmath::Vector3;

type Vec3 = Vector3<f32>;

pub const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

struct Rotation {
  front: Vec3,
  pitch: f32,
  yaw: f32,
}

impl Rotation {
  const fn default() -> Self {
    Rotation {
      front: Vec3::new(0.0, 0.0, 0.0),
      pitch: 0.0,
      yaw: 0.0,
    }
  }
}

trait Object3D {
  const OBJECT_UP: Vec3 = WORLD_UP;
  const OBJECT_RIGHT: Vec3 = Vec3::new(1.0, 0.0, 0.0);

  const DEFAULT_ROTATION: Rotation = Rotation::default();
  const DEFAULT_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);

  fn get_pos() -> Vec3;
  fn get_rot() -> Rotation;
}
