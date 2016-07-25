use std::process::Command;

pub fn unmangle(sym: &str) -> String {
    Command::new("c++filt")
        .arg("-n")
        .arg(sym)
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap())
        .or::<String>(Ok(String::from(sym)))
        .unwrap()
}
