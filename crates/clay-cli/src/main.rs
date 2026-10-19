mod args;
mod cargo;
mod project;

#[cfg(test)]
mod test_support;

use std::process::ExitCode;

use args::Action;

fn main() -> ExitCode {
    match args::parse(std::env::args_os().skip(1)).and_then(execute) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(action: Action) -> Result<ExitCode, String> {
    match action {
        Action::Help(topic) => {
            print!("{}", topic.text());
            Ok(ExitCode::SUCCESS)
        }
        Action::Version => {
            println!("clay {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        Action::New {
            path,
            framework_path,
        } => {
            let framework = framework_path.map(project::FrameworkSource::LocalPath);
            let source = framework.unwrap_or(project::FrameworkSource::PublicGit);
            let project = project::write_project(&path, source)?;
            let mut command = cargo::lockfile_command(&project.directory);
            let retry = || {
                format!(
                    "Project files were kept at {}.\n\
                     Fix the dependency-resolution error, then run `cargo generate-lockfile` \
                     from that directory to retry before using dev/build.",
                    project.directory.display()
                )
            };
            let status = command.status().map_err(|error| {
                format!(
                    "could not run cargo generate-lockfile: {error}\n{}",
                    retry()
                )
            })?;
            if !status.success() {
                return Err(format!(
                    "cargo generate-lockfile failed ({status}).\n{}",
                    retry()
                ));
            }
            println!(
                "Created `{}` at {}.\n\
                 From that directory, run `clay dev` or `clay build`.",
                project.package_name,
                project.directory.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        Action::Dev {
            manifest_path,
            app_args,
        } => cargo::run(cargo::dev_command(manifest_path.as_deref(), &app_args)),
        Action::Build { manifest_path } => {
            cargo::run(cargo::build_command(manifest_path.as_deref()))
        }
    }
}
