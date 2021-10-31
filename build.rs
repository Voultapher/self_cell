use std::env;
use std::ffi::OsString;
use std::process::Command;

fn main() {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(&rustc).arg("--version").output().unwrap();
    let version_info = String::from_utf8_lossy(&output.stdout);
    let version = version_info.split_whitespace().nth(1).unwrap();
    let pieces = version
        .split('.')
        .take(2)
        .map(|x| x.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    let old_rust = pieces[0] == 1 && pieces[1] < 51;
    if old_rust {
        println!("cargo:rustc-cfg=addr_of_mut_polyfill");
    }
}
