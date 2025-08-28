use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const DEFAULT_PATH: &'static str = "./assets";

pub enum FileType {
  Image,
  Obj,
  Shader,
}

fn bytes_to_str(bytes: Vec<u8>) -> io::Result<String> {
  const ERR_MSG: &str = "failed to convert bytes into utf8";
  Ok(String::from_utf8(bytes).expect(ERR_MSG))
}

fn load_file_string(filetype: FileType, filename: &str) -> io::Result<String> {
  let shader_bytes = load_file_bytes(filetype, filename)?;
  return Ok(bytes_to_str(shader_bytes)?);
}

// ********************** SHADERS **************************** //
pub fn load_shader_str(filename: &str) -> io::Result<String> {
  return Ok(load_file_string(FileType::Shader, filename)?);
}

// ********************** OBJ FILES **************************** //
pub fn load_obj_str(filename: &str) -> io::Result<String> {
  return Ok(load_file_string(FileType::Obj, filename)?);
}

// ********************** IMAGE FILES **************************** //
pub fn load_image_bytes(filename: &str) -> io::Result<Vec<u8>> {
  return load_file_bytes(FileType::Image, filename);
}

pub fn load_file_bytes(filetype: FileType, filename: &str) -> io::Result<Vec<u8>> {
  let path_str = get_file_path(filetype, filename);
  let path = Path::new(&path_str);
  ensure_directory_exists(path)?;
  return read_file_as_raw(path);
}

fn add_directory(path: &mut String, concat: &str) {
  path.push('/');
  path.push_str(concat);
}

fn ensure_directory_exists(path: &Path) -> io::Result<()> {
  if let Some(parent) = path.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent)?;
    }
  }
  Ok(())
}

fn get_file_path(filetype: FileType, filename: &str) -> String {
  let mut path = String::new();
  path.push_str(DEFAULT_PATH);
  match filetype {
    FileType::Image => add_directory(&mut path, "images"),
    FileType::Obj => add_directory(&mut path, "obj"),
    FileType::Shader => add_directory(&mut path, "shaders"),
  }
  add_directory(&mut path, filename);
  path
}

fn read_file_as_raw(path: &Path) -> io::Result<Vec<u8>> {
  let mut file = File::open(path)?;
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer)?;
  return Ok(buffer);
}
