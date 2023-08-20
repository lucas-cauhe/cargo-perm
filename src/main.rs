
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
// Injects a payload into the <method_no>'th method in the <file_no>'th file specified in
// <va_input>
fn integrate(va_input: &Vaoutput, file_no: usize, method_no: usize, payload: &dyn Payload) -> PayloadResult<String> {

   let file_selected = va_input.nth_file(file_no)?;
   let method_starting_line = file_selected.nth_method_sl(method_no)?;
   payload.inject(
       &std::fs::read_to_string(file_selected.name).unwrap() 
       , method_starting_line
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
fn compile_mock_integration(src_program: String) -> CompilationStatus {}

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
   files: Vec<FileOutput>
}
impl Vaoutput {
    pub fn nth_file(&self, file_no: usize) -> Result<&FileOutput, String> {
        match self.files.get(file_no) {
            Some(f_out) => Ok(f_out),
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

                           let integrated_payload = integrate(va_out
                                                ,cmd_parts[1].parse::<usize>().unwrap()
                                                , cmd_parts[2].parse::<usize>().unwrap()
                                                , payload_from_str(cmd_parts[3])
                           ).unwrap();
                           comp_res = Some(
                               compile_mock_integration(integrated_payload)
                            );
                       } else {
                           error!("You must run vanalyze command before");
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
