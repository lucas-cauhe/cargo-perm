use std::path::Path;
use std::process::ExitStatus;
// stores method name and its line number
pub struct FileMethod {
    pub name: String,
    pub line_no: usize
}

// stores file names, method names and line numbers
pub struct FileOutput {
    pub name: String,
    pub methods: Vec<FileMethod>
}
impl FileOutput {
    pub fn nth_method_sl(&self, method_no: usize) -> Result<&usize, String> {
       match self.methods.get(method_no) {
           Some(f_method) => Ok(&f_method.line_no),
           None => Err("Bad method number".to_string())
       }
    }
}


pub struct Vaoutput {
   pub files: Vec<(String, FileOutput)> // crate name + file output
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
    pub fn deserialize(src: &str) -> Self {
        let mut vaoutput = Vaoutput {files: Vec::new()};
        let splitted_by_files = src.split(':').collect::<Vec<&str>>()[1..].to_vec();
        for file in splitted_by_files {
            let lines = file.split('\n').collect::<Vec<&str>>();
            let (crate_name, file_name) = (lines[0].trim().split(' ').nth(0).unwrap(), lines[0].trim().split(' ').nth(1).unwrap());
            let mut f_out = FileOutput {name: file_name.to_string(), methods: Vec::new()};
            for line in lines[1..].to_vec() {
                let (method_name, line_no) = (line.trim().split(' ').nth(1).unwrap(), line.trim().split(' ').nth(2).unwrap().parse::<usize>().unwrap()); 
                let f_method = FileMethod {name: method_name.to_string(), line_no};
                f_out.methods.push(f_method);
            }
            vaoutput.files.push((crate_name.to_string(), f_out));
        }
        vaoutput
    }
}

pub fn vanalyze(path: &Path, username: &str) -> Result<Vaoutput, String> {
    // prepare for running vanalyze script
  let output = std::process::Command::new("/bin/bash")
      .args(["shell-scripts/vanalyzer/run.sh", path.to_str().unwrap(), username])
      .stdout(std::process::Stdio::piped())
      .output()
      .unwrap();
   // receive output and deserialize it into a Vaoutput
   if ExitStatus::code(&output.status).unwrap() == 1 {
       return Err(format!("Command failed: {:?}", &String::from_utf8(output.stdout).unwrap()))
   }
   Ok(Vaoutput::deserialize(&String::from_utf8(output.stdout).unwrap()))
}
