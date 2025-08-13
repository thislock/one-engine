use cgmath::{Vector3, Vector2};

pub type Vec3 = Vector3<f32>;
pub type Vec2 = Vector2<f32>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
  radians: f64, // Internal storage in radians
}

#[allow(unused)]
impl Angle {
  /// Create from radians
  #[inline(always)]
  pub fn from_radians(rad: f64) -> Self {
    Self { radians: rad }
  }

  /// Create from degrees
  #[inline(always)]
  pub fn from_degrees(deg: f64) -> Self {
    Self {
      radians: deg.to_radians(),
    }
  }

  /// Create from grads (full turn = 400 grads)
  #[inline(always)]
  pub fn from_grads(grads: f64) -> Self {
    Self {
      radians: grads * std::f64::consts::PI / 200.0,
    }
  }

  /// Create from turns (full turn = 1.0)
  #[inline(always)]
  pub fn from_turns(turns: f64) -> Self {
    Self {
      radians: turns * std::f64::consts::TAU,
    }
  }

  /// Get radians
  #[inline(always)]
  pub fn as_radians(&self) -> f64 {
    self.radians
  }

  /// Get degrees
  #[inline(always)]
  pub fn as_degrees(&self) -> f64 {
    self.radians.to_degrees()
  }

  /// Get grads
  #[inline(always)]
  pub fn as_grads(&self) -> f64 {
    self.radians * 200.0 / std::f64::consts::PI
  }

  /// Get turns
  #[inline(always)]
  pub fn as_turns(&self) -> f64 {
    self.radians / std::f64::consts::TAU
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scalar {
  pub magnitude: f64,
  pub angle: Angle,
}

impl Scalar {
  /// Creates a new scalar with magnitude and angle (in radians)
  pub fn new(magnitude: f64, angle: Angle) -> Self {
    Self { magnitude, angle }
  }

  /// Converts this scalar into a 2D vector using magnitude & angle
  pub fn to_vec2(self) -> Vec2 {
    Vec2::new(
      (self.magnitude * self.angle.as_radians().cos()) as f32,
      (self.magnitude * self.angle.as_radians().sin()) as f32,
    )
  }

  /// Converts only magnitude into `(magnitude, magnitude)`
  pub fn magnitude_vec2(self) -> Vec2 {
    Vec2::new(self.magnitude as f32, self.magnitude as f32)
  }

  /// Converts only magnitude into `(magnitude, 0.0)`
  pub fn magnitude_vec2_x(self) -> Vec2 {
    Vec2::new(self.magnitude as f32, 0.0)
  }

  /// Converts only magnitude into `(0.0, magnitude)`
  pub fn magnitude_vec2_y(self) -> Vec2 {
    Vec2::new(0.0, self.magnitude as f32)
  }
}

use std::ops::{Add, Sub, Mul, Div};

impl Add for Scalar {
  type Output = Self;
  fn add(self, rhs: Self) -> Self {
    Self {
      magnitude: self.magnitude + rhs.magnitude,
      angle: self.angle, // keep original angle
    }
  }
}

impl Sub for Scalar {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self {
    Self {
      magnitude: self.magnitude - rhs.magnitude,
      angle: self.angle,
    }
  }
}

impl Mul<f32> for Scalar {
  type Output = Self;
  fn mul(self, rhs: f32) -> Self {
    Self {
      magnitude: self.magnitude * rhs as f64,
      angle: self.angle,
    }
  }
}

impl Div<f32> for Scalar {
  type Output = Self;
  fn div(self, rhs: f32) -> Self {
    Self {
      magnitude: self.magnitude / rhs as f64,
      angle: self.angle,
    }
  }
}
