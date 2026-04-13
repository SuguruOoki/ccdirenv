use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let raw_args: Vec<std::ffi::OsString> = env::args_os().collect();
    let invoked_as = PathBuf::from(raw_args.first().cloned().unwrap_or_default())
        .file_name()
        .and_then(|s| s.to_str().map(str::to_owned))
        .unwrap_or_default();

    if invoked_as == "claude" {
        let Err(e) = ccdirenv::shim::run(raw_args);
        eprintln!("ccdirenv shim: {e}");
        return ExitCode::from(127);
    }

    match ccdirenv::manager::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ccdirenv: {e}");
            ExitCode::from(1)
        }
    }
}
