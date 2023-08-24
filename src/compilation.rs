
use log::*;
// ghost function
// takes a string, creates a rust program with it and compiles it returning its result
pub fn compile_mock_integration(src_program: &str, target_crate: &str, target_file: &str) -> CompilationStatus {
  // run shell script
  info!("Running: \nshell-scripts/compile_mock_integration.sh {target_crate} {target_file} {src_program}");
  let output = std::process::Command::new("/bin/bash")
      .arg("-c")
      .arg(&format!("shell-scripts/compile_mock_integration.sh \"{target_crate}\" \"{target_file}\" \"{src_program}\""))
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .output()
      .unwrap();
  // convert output to CompilationStatus
  info!("Command stdout: {:?}", String::from_utf8(output.stdout.clone()).unwrap());
  info!("Command stderr: {:?}", String::from_utf8(output.stderr.clone()).unwrap());
  if output.stdout.len() > 0 {
      CompilationStatus::Correct(src_program.to_string(), target_file.to_string())
  }
  else {
      CompilationStatus::Flaw(String::from_utf8(output.stderr).unwrap())
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompilationStatus {
    Correct(String, String), // contains the resulting program that compiled and the target_file
    Flaw(String) // contains the error message
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compilation_works() {
        env_logger::init();
        // This src_program should compile correctly (guessing game example)
        let src_program = "$HOME/Documents/cargo-perm/tests/compilation_test_pass.txt";
        let target_crate = "index.crates.io-6f17d22bba15001f/num-bigint-0.1.44";
        let target_file = "src/bigint.rs";
        if let CompilationStatus::Correct(_, _) = compile_mock_integration(src_program, target_crate, target_file) {
            assert!(true);
        } else {
            assert!(false);
        }
        // This src_program should not compile
        let src_program = "$HOME/Documents/cargo-perm/tests/compilation_test_fail.txt";
        if let CompilationStatus::Flaw(_) = compile_mock_integration(src_program, target_crate, target_file) {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
