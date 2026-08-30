
use std::io::Write;

pub fn log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("d:\\program\\phira_render\\ipc.log")
    {
        let _ = writeln!(f, "{}", msg);
    }
}

pub mod client {
    use serde::Serialize;

    pub fn send<T: Serialize>(value: T) {
        println!("{}", serde_json::to_string(&value).unwrap());
    }
}
