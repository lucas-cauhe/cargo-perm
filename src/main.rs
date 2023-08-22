
// Mock shell that receives commands

// Define available commands and their I/O (integration requires that vanalyzer has been run before) and the context they should be called in
// (maybe inside a thread)


// Vanalyzer command
// vanalyzer <target-project-path> <target-username>
// runs the script vanalyzer/run.sh and stores its result in a variable (use Command crate opening
// a socket to get the output and deserialize it)
// Finally show the formatted output to the user with the vulnerable files ordered with its
// vulnerable methods listed as well

// Integrate command
// integrate <file_no> <method_no> <payload>
// copy-pastes the code from <payload> into <file_no> in the file's <method_no>'th method
// produce a mock result (duplicate the file with the integrated payload) and compile it to check
// nothing's wrong

// Ok command
// ok
// inject the code into the source file

// Atacker pipeline:
// 1. Run the mock shell (cargo project)
// 2. run vanalyzer command
// 3. run integrate command
// 4. run ok command

use std::path::Path;
use log::{error, info}; 
use std::io::Write;

fn vanalyze(path: &Path, username: &str) -> Vaoutput {}

// Returns a rust program to be compiled afterwards
// Injects in the <file_selected> file at line <method_starting_line> the code for running
// <payload>
// <file_selected> has to be the absolute path to the file
fn integrate(file_selected: &String, method_starting_line: usize, payload: &dyn Payload) -> PayloadResult<String> {
   payload.inject(
       &std::fs::read_to_string(file_selected.clone()).unwrap() 
       , &method_starting_line
   )
}
// returns error if compilation was not successful
fn ok_command(comp_stat: CompilationStatus) -> Result<(), String> {
    match comp_stat {
        CompilationStatus::Correct(prog, file) /* Merge file */ => {
            std::fs::OpenOptions::new()
                .truncate(true)
                .open(file)
                .unwrap()
                .write(prog.as_bytes())
                .unwrap();
            Ok(())
        },
        CompilationStatus::Flaw(e) => Err(format!("Error while compiling: {e}"))
    }
}


// ghost function
// takes a string, creates a rust program with it and compiles it returning its result
fn compile_mock_integration(src_program: &str, target_crate: &str, target_file: &str) -> CompilationStatus {
  //create file with <src_program>'s content 
  let new_file_path = "/tmp/mock_program.rs".to_string();
  std::fs::File::create(new_file_path).unwrap().write(src_program.as_bytes());
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

enum CompilationStatus {
    Correct(String, String), // contains the resulting program that compiled and the target_file
    Flaw(String) // contains the error message
}

// payload result wrapper
type PayloadResult<R> = Result<R, String>;

// trait Payload which must be implemented by all available payloads
// permits a payload to be injectable to some given code in the line specified
pub trait Payload {
    // takes a bunch of string lines and a line number and injects the payload implementing it
    // into the string starting from <line_no>
    fn inject(&self, src: &str, line_no: &usize) -> PayloadResult<String>;
}

pub fn payload_from_str(src: &str) -> &dyn Payload {}

// stores method name and its line number
struct FileMethod {
    name: String,
    line_no: usize
}

// stores file names, method names and line numbers
struct FileOutput {
    name: String,
    methods: Vec<FileMethod>
}
impl FileOutput {
    pub fn nth_method_sl(&self, method_no: usize) -> Result<&usize, String> {
       match self.methods.get(method_no) {
           Some(f_method) => Ok(&f_method.line_no),
           None => Err("Bad method number".to_string())
       }
    }
}


struct Vaoutput {
   files: Vec<(String, FileOutput)> // crate name + file output
}
impl Vaoutput {
    pub fn nth_file(&self, file_no: usize) -> Result<&FileOutput, String> {
        match self.files.get(file_no) {
            Some((_, f_out)) => Ok(f_out),
            None => Err("Bad file number".to_string())
        }
    }
    pub fn nth_file_crate(&self, file_no: usize) -> Result<&String, String> {
        match self.files.get(file_no) {
            Some((crate_name, _)) => Ok(crate_name),
            None => Err("Bad file number".to_string())
        }
    }
}

fn main() {
    // launch reader thread with socket fd
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        loop {
            let stdin = std::io::stdin();
            let mut buf: String; 
            match stdin.read_line(&mut buf) {
                Ok(_) => tx.send(buf).unwrap(),
                Error => /* This should cause a dead end error */ drop(tx)
            }
        }
    });
    let mut va_output: Option<Vaoutput> = None;
    let mut comp_res: Option<CompilationStatus> = None;
    loop {
       match rx.try_recv().ok() {
           Some(cmd) => {
               let cmd_parts: Vec<&str> = cmd.split(' ').collect();
               match cmd_parts[0] {
                   "vanalyze" => {
                       va_output = Some(
                           vanalyze(Path::new(&cmd_parts[1]), &cmd_parts[2])
                        );
                   }
                   "integrate" => {
                       if let Some(ref va_out) = va_output {

                           let file_selected = va_out.nth_file(cmd_parts[1].parse::<usize>().unwrap()).expect("Bad file number");
                           let method_sl = file_selected.nth_method_sl(
                        cmd_parts[2].parse::<usize>().unwrap()
                               ).expect("Bad method number");
                           let integrated_payload = integrate(
                                                &file_selected.name
                                                , method_sl.clone() 
                                                , payload_from_str(cmd_parts[3])
                           ).unwrap();
                           // /home/username/.cargo/registry/src/*/crate_name/...
                           // crate's path ends at the 8th forslash
                           let (target_crate, mut target_file) = cmd_parts[1].match_indices('/').nth(7).map(|(index, _)| cmd_parts[1].split_at(index)).unwrap();
                           target_file = &target_file.replacen('/', "", 1);
                           comp_res = Some(
                               compile_mock_integration(&integrated_payload
                               , target_crate
                               , target_file)
                            );
                       } else {
                           error!("You must run vanalyze command before");
                       }
                   },
                   "exploit" => { 
                       
                        // get method starting line
                        // read file until you hit with the method
                        let reader = std::fs::read_to_string(&cmd_parts[1].to_string()).unwrap().lines();
                        let maybe_line =  reader.enumerate().filter(|(line_no, line)| line.contains(" fn ") && line.contains(cmd_parts[2]) /* may not be exhaustive but dah */).collect::<Vec<(usize, &str)>>(); 

                        if maybe_line.len() > 0 {
                            let method_line = maybe_line[0].0;
                            let integrated_payload = integrate(
                                      &cmd_parts[1].to_string() 
                                      , method_line 
                                      , payload_from_str(cmd_parts[3])
                              );
                           let target_file
                           comp_res = Some(
                               compile_mock_integration(&integrated_payload.expect("error injecting payload")
                               , 
                               , cmd_parts[1])
                            );
                        } else {
                            error!("Method not found in specified file")
                        }
                        
                   },
                   "ok" => {
                       if let Some(compiled) = comp_res {
                           ok_command(compiled);
                           info!("Payload has been successfully injected");
                       } else {
                           error!("You must run integrate command before");
                          
                       }
                   },
                   _ => error!("Unknown command")
               }
           },
           None => ()
       }
    }
}
