use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arg0 = env::args_os().next().unwrap_or_default();
    let invoked_as = PathBuf::from(&arg0)
        .file_name()
        .and_then(|s| s.to_str().map(str::to_owned))
        .unwrap_or_default();

    match invoked_as.as_str() {
        "claude" => {
            eprintln!("ccdirenv: shim not yet implemented");
            ExitCode::from(1)
        }
        _ => {
            eprintln!("ccdirenv: manager not yet implemented");
            ExitCode::from(1)
        }
    }
}
