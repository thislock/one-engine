use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const DEFAULT_PATH: &'static str = "./assets";

pub enum FileType {
  Image,
  Obj,
  Shader,
  ShaderLib,
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

/// i know loading every shader file in the lib folder every time is slow,
/// but shut up it doesn't REALLY matter for this kinda thing
/// plus it means i get to keep everything pretty and **functional**
fn load_lib_files() -> io::Result<Vec<String>> {
  let lib_path = &get_path(FileType::ShaderLib);
  fs::create_dir_all(Path::new(&lib_path))?;
  let entries = fs::read_dir(lib_path)?;

  let mut lib_files = Vec::new();

  for entry in entries {
    if let Ok(valid_entry) = entry {
      let found_file = &valid_entry.file_name();
      let filename = std::ffi::OsStr::to_str(found_file);
      if let Some(valid_filename) = filename {
        let file_str = load_file_string(FileType::ShaderLib, valid_filename)?;
        lib_files.push(file_str);
      }
    }
  }

  return Ok(lib_files);
}

fn format_shader_libraries() -> io::Result<String> {
  let shader_libs = load_lib_files()?;
  let mut formated_shaders = String::new();

  for shader in shader_libs {
    formated_shaders.push_str(&shader);
    formated_shaders.push('\n');
    formated_shaders.push_str("// hi");
    formated_shaders.push('\n');
  }

  return Ok(formated_shaders);
}

pub fn load_shader_str(filename: &str) -> io::Result<String> {
  let shader_libraries = &format_shader_libraries()?;
  let shader_data = &load_file_string(FileType::Shader, filename)?;
  let mut shader = String::new();
  // literally just dumps everything in the shader lib folder at the front of the shader file
  shader.push_str(shader_libraries);
  shader.push_str(shader_data);
  return Ok(shader);
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

mod folder_names {
  pub const IMAGES: &'static str = "images";
  pub const OBJECTS: &'static str = "obj";
  pub const SHADERS: &'static str = "shaders";
  pub const SHADER_LIB: &'static str = "shader_lib";
}

fn get_path(filetype: FileType) -> String {
  let mut path = String::new();
  path.push_str(DEFAULT_PATH);
  match filetype {
    FileType::Image => add_directory(&mut path, folder_names::IMAGES),
    FileType::Obj => add_directory(&mut path, folder_names::OBJECTS),
    FileType::Shader => add_directory(&mut path, folder_names::SHADERS),
    FileType::ShaderLib => {
      add_directory(&mut path, folder_names::SHADERS);
      add_directory(&mut path, folder_names::SHADER_LIB);
    }
  }
  return path;
}

fn get_file_path(filetype: FileType, filename: &str) -> String {
  let mut path = get_path(filetype);
  add_directory(&mut path, filename);
  return path;
}

fn read_file_as_raw(path: &Path) -> io::Result<Vec<u8>> {
  let mut file = File::open(path)?;
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer)?;
  return Ok(buffer);
}
