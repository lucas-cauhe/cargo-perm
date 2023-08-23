use log::*;
// payload result wrapper
pub type PayloadResult<R> = Result<R, String>;

// trait Payload which must be implemented by all available payloads
// permits a payload to be injectable to some given code in the line specified
pub trait Payload {
    // takes a bunch of string lines and a line number and injects the payload implementing it
    // into the string starting from <line_no>
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
        Ok("hello".to_string())
    }
}
