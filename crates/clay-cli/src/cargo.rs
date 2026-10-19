use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitCode};

pub fn dev_command(manifest_path: Option<&Path>, app_args: &[OsString]) -> Command {
    let mut command = Command::new("cargo");
    command.args(["run", "--locked"]);
    manifest_arg(&mut command, manifest_path);
    if !app_args.is_empty() {
        command.arg("--").args(app_args);
    }
    command
}

pub fn build_command(manifest_path: Option<&Path>) -> Command {
    let mut command = Command::new("cargo");
    command.args(["build", "--release", "--locked"]);
    manifest_arg(&mut command, manifest_path);
    command
}

pub fn lockfile_command(project_directory: &Path) -> Command {
    let mut command = Command::new("cargo");
    command
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(project_directory.join("Cargo.toml"))
        .current_dir(project_directory);
    command
}

fn manifest_arg(command: &mut Command, path: Option<&Path>) {
    if let Some(path) = path {
        command.arg("--manifest-path").arg(path);
    }
}

pub fn run(mut command: Command) -> Result<ExitCode, String> {
    let status = command
        .status()
        .map_err(|error| format!("could not run cargo: {error}"))?;
    if !status.success() {
        eprintln!("error: cargo failed ({status})");
    }
    Ok(match status.code() {
        Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        None => ExitCode::FAILURE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn arguments(command: &Command) -> Vec<OsString> {
        assert_eq!(command.get_program(), OsStr::new("cargo"));
        command.get_args().map(OsString::from).collect()
    }

    #[test]
    fn dev_locks_resolution_and_keeps_manifest_and_app_arguments_separate() {
        let manifest = Path::new("path with spaces/\"雪\"/Cargo.toml");
        let app_args = vec!["--title".into(), "a ; $(literal) \\".into(), "".into()];
        assert_eq!(
            arguments(&dev_command(Some(manifest), &app_args)),
            vec![
                OsString::from("run"),
                "--locked".into(),
                "--manifest-path".into(),
                manifest.into(),
                "--".into(),
                app_args[0].clone(),
                app_args[1].clone(),
                app_args[2].clone(),
            ]
        );
        assert_eq!(arguments(&dev_command(None, &[])), ["run", "--locked"]);
    }

    #[test]
    fn build_is_release_and_locked_with_optional_manifest() {
        assert_eq!(
            arguments(&build_command(None)),
            ["build", "--release", "--locked"]
        );
        assert_eq!(
            arguments(&build_command(Some(Path::new("my app/Cargo.toml")))),
            [
                "build",
                "--release",
                "--locked",
                "--manifest-path",
                "my app/Cargo.toml",
            ]
        );
    }

    #[test]
    fn lockfile_resolution_uses_the_generated_project_as_its_working_directory() {
        let project = Path::new("/project with spaces/my-app");
        let command = lockfile_command(project);
        assert_eq!(command.get_current_dir(), Some(project));
        assert_eq!(
            arguments(&command),
            [
                "generate-lockfile",
                "--manifest-path",
                "/project with spaces/my-app/Cargo.toml",
            ]
        );
    }
}
