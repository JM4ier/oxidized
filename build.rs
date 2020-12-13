extern crate chrono;
use chrono::prelude::*;

fn main() {
    println!(
        "cargo:rustc-env=BUILD_DATE={}",
        Utc::now().format("UTC %Y-%m-%d %H:%M:%S").to_string()
    );
}
