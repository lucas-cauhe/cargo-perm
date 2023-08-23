
use std::io::Write;
// ghost function
// takes a string, creates a rust program with it and compiles it returning its result
pub fn compile_mock_integration(src_program: &str, target_crate: &str, target_file: &str) -> CompilationStatus {
  //create file with <src_program>'s content 
  let new_file_path = "/tmp/mock_program.rs".to_string();
  let _ = std::fs::File::create(&new_file_path).unwrap().write(src_program.as_bytes());
  // run shell script
  let output = std::process::Command::new("/bin/bash")
      .args(["shell-scripts/compile_mock_integration.sh", target_crate, target_file, &new_file_path])
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .output()
      .unwrap();
  // convert output to CompilationStatus
  if output.stdout.len() > 0 {
      CompilationStatus::Correct(src_program.to_string(), target_file.to_string())
  }
  else {
      CompilationStatus::Flaw(String::from_utf8(output.stderr).unwrap())
  }
}

pub enum CompilationStatus {
    Correct(String, String), // contains the resulting program that compiled and the target_file
    Flaw(String) // contains the error message
}
