//! A direct executable fixture: no real Cargo resolution, shell, or network.
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let log = PathBuf::from(std::env::var_os("GPUI_CLI_TEST_LOG").expect("test log path"));
    fs::write(
        &log,
        args.iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .expect("record arguments");
    fs::write(
        log.with_extension("cwd"),
        std::env::current_dir()
            .expect("current directory")
            .to_string_lossy()
            .as_bytes(),
    )
    .expect("record working directory");

    let code: u8 = std::env::var("GPUI_CLI_TEST_CARGO_EXIT")
        .expect("test exit code")
        .parse()
        .expect("numeric exit code");
    if code == 0 && args.first().is_some_and(|arg| arg == "generate-lockfile") {
        assert_eq!(args[1], "--manifest-path");
        fs::write(
            PathBuf::from(&args[2]).with_file_name("Cargo.lock"),
            "# Test fixture lockfile; not a real dependency resolution.\nversion = 4\n",
        )
        .expect("write fake lockfile");
    }
    ExitCode::from(code)
}
