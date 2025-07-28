
pub fn log_error(message: &str, error: anyhow::Error) {
  println!("ERROR, {}: {}", message, error);
}

pub fn log(message: &str) {
  println!("info: {}", message);
}