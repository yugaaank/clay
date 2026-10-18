use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Help(HelpTopic),
    Version,
    New {
        path: PathBuf,
        framework_path: Option<PathBuf>,
    },
    Dev {
        manifest_path: Option<PathBuf>,
        app_args: Vec<OsString>,
    },
    Build {
        manifest_path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Main,
    New,
    Dev,
    Build,
}

impl HelpTopic {
    pub fn text(self) -> &'static str {
        match self {
            Self::Main => {
                "GPUI Framework application CLI\n\n\
                 Usage: gpui-framework <COMMAND>\n\n\
                 Commands:\n  \
                   new <path> [--framework-path <path>]  Create an application and Cargo.lock\n  \
                   dev [--manifest-path <Cargo.toml>] [-- <app args>]\n    \
                     Run with cargo run --locked\n  \
                   build [--manifest-path <Cargo.toml>]\n    \
                     Build with cargo build --release --locked\n\n\
                 Options:\n  \
                   -h, --help     Print help\n  \
                   -V, --version  Print version\n"
            }
            Self::New => {
                "Usage: gpui-framework new <path> [--framework-path <path>]\n\n\
                 Create a standalone counter application and resolve its Cargo.lock.\n\
                 The destination must not exist. Its final component is the Rust package name\n\
                 (ASCII letters, digits, '-' or '_'; starts with a letter or '_'; no keywords).\n\
                 The framework defaults to the source checkout used to build this CLI.\n\
                 Use --framework-path to select another framework crate containing Cargo.toml.\n\
                 Use `new -- <path>` for a path beginning with '-'.\n"
            }
            Self::Dev => {
                "Usage: gpui-framework dev [--manifest-path <Cargo.toml>] [-- <app args>]\n\n\
                 Runs cargo run --locked. Arguments after -- are passed to the application.\n\
                 Without --manifest-path, Cargo finds the manifest from the current directory.\n"
            }
            Self::Build => {
                "Usage: gpui-framework build [--manifest-path <Cargo.toml>]\n\n\
                 Runs cargo build --release --locked.\n\
                 Without --manifest-path, Cargo finds the manifest from the current directory.\n"
            }
        }
    }
}

pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Action, String> {
    let mut args = arguments.into_iter();
    let Some(command) = args.next() else {
        return Ok(Action::Help(HelpTopic::Main));
    };
    match command.to_str() {
        Some("-h" | "--help") => {
            no_more_args(args)?;
            Ok(Action::Help(HelpTopic::Main))
        }
        Some("-V" | "--version") => {
            no_more_args(args)?;
            Ok(Action::Version)
        }
        Some("new") => parse_new(args),
        Some("dev") => parse_cargo(args, true),
        Some("build") => parse_cargo(args, false),
        _ => Err(format!(
            "unknown command `{}`; use `gpui-framework --help`",
            command.to_string_lossy()
        )),
    }
}

fn no_more_args(mut args: impl Iterator<Item = OsString>) -> Result<(), String> {
    match args.next() {
        None => Ok(()),
        Some(arg) => Err(format!("unexpected argument `{}`", arg.to_string_lossy())),
    }
}

fn option_path(
    option: &str,
    target: &mut Option<PathBuf>,
    args: &mut impl Iterator<Item = OsString>,
) -> Result<(), String> {
    if target.is_some() {
        return Err(format!("{option} may only be specified once"));
    }
    let value = args
        .next()
        .filter(|value| !value.is_empty() && !value.to_string_lossy().starts_with('-'))
        .ok_or_else(|| format!("{option} requires a path"))?;
    *target = Some(value.into());
    Ok(())
}

fn parse_new(mut args: impl Iterator<Item = OsString>) -> Result<Action, String> {
    let mut path = None;
    let mut framework_path = None;
    let mut positional_only = false;
    while let Some(arg) = args.next() {
        if !positional_only {
            match arg.to_str() {
                Some("-h" | "--help") => return Ok(Action::Help(HelpTopic::New)),
                Some("--framework-path") => {
                    option_path("--framework-path", &mut framework_path, &mut args)?;
                    continue;
                }
                Some("--") => {
                    positional_only = true;
                    continue;
                }
                _ if arg.to_string_lossy().starts_with('-') => {
                    return Err(format!("unknown new option `{}`", arg.to_string_lossy()));
                }
                _ => {}
            }
        }
        if path.replace(PathBuf::from(&arg)).is_some() {
            return Err("new accepts exactly one destination path".into());
        }
    }
    let path = path
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| {
            "new requires a destination path; use `gpui-framework new --help`".to_owned()
        })?;
    Ok(Action::New {
        path,
        framework_path,
    })
}

fn parse_cargo(mut args: impl Iterator<Item = OsString>, dev: bool) -> Result<Action, String> {
    let mut manifest_path = None;
    let mut app_args = Vec::new();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-h" | "--help") => {
                return Ok(Action::Help(if dev {
                    HelpTopic::Dev
                } else {
                    HelpTopic::Build
                }));
            }
            Some("--manifest-path") => {
                option_path("--manifest-path", &mut manifest_path, &mut args)?;
            }
            Some("--") if dev => {
                app_args.extend(args);
                break;
            }
            _ => {
                return Err(format!(
                    "unexpected {} argument `{}`; use `gpui-framework {} --help`",
                    if dev { "dev" } else { "build" },
                    arg.to_string_lossy(),
                    if dev { "dev" } else { "build" },
                ));
            }
        }
    }
    if dev {
        Ok(Action::Dev {
            manifest_path,
            app_args,
        })
    } else {
        Ok(Action::Build { manifest_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(args: &[&str]) -> Result<Action, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn help_and_version_are_available() {
        for args in [&[][..], &["--help"][..], &["-h"][..]] {
            assert_eq!(parsed(args), Ok(Action::Help(HelpTopic::Main)));
        }
        for (command, topic) in [
            ("new", HelpTopic::New),
            ("dev", HelpTopic::Dev),
            ("build", HelpTopic::Build),
        ] {
            assert_eq!(parsed(&[command, "--help"]), Ok(Action::Help(topic)));
        }
        assert_eq!(parsed(&["--version"]), Ok(Action::Version));
        assert_eq!(parsed(&["-V"]), Ok(Action::Version));
    }

    #[test]
    fn new_accepts_paths_as_single_arguments_and_options_in_either_order() {
        let expected = Action::New {
            path: "parent with spaces/my-app".into(),
            framework_path: Some("framework checkout/crates/framework".into()),
        };
        assert_eq!(
            parsed(&[
                "new",
                "parent with spaces/my-app",
                "--framework-path",
                "framework checkout/crates/framework",
            ]),
            Ok(expected)
        );
        assert_eq!(
            parsed(&["new", "--framework-path", "../framework", "my-app"]),
            Ok(Action::New {
                path: "my-app".into(),
                framework_path: Some("../framework".into()),
            })
        );
        assert_eq!(
            parsed(&["new", "--", "-parent/my-app"]),
            Ok(Action::New {
                path: "-parent/my-app".into(),
                framework_path: None,
            })
        );
    }

    #[test]
    fn dev_preserves_every_argument_after_separator() {
        assert_eq!(
            parsed(&[
                "dev",
                "--manifest-path",
                "a b/Cargo.toml",
                "--",
                "--help",
                "--manifest-path",
                "$(not a shell) ; 雪",
                "",
            ]),
            Ok(Action::Dev {
                manifest_path: Some("a b/Cargo.toml".into()),
                app_args: vec![
                    "--help".into(),
                    "--manifest-path".into(),
                    "$(not a shell) ; 雪".into(),
                    "".into(),
                ],
            })
        );
        assert_eq!(
            parsed(&["dev"]),
            Ok(Action::Dev {
                manifest_path: None,
                app_args: vec![],
            })
        );
        assert_eq!(
            parsed(&["build", "--manifest-path", "a b/Cargo.toml"]),
            Ok(Action::Build {
                manifest_path: Some("a b/Cargo.toml".into()),
            })
        );
    }

    #[test]
    fn invalid_and_ambiguous_arguments_are_errors() {
        for args in [
            vec!["unknown"],
            vec!["--help", "extra"],
            vec!["--version", "extra"],
            vec!["new"],
            vec!["new", ""],
            vec!["new", "one", "two"],
            vec!["new", "one", "--unknown"],
            vec!["new", "one", "--framework-path"],
            vec!["new", "one", "--framework-path", ""],
            vec![
                "new",
                "one",
                "--framework-path",
                "a",
                "--framework-path",
                "b",
            ],
            vec!["dev", "--manifest-path"],
            vec!["dev", "--manifest-path", "--help"],
            vec!["dev", "--manifest-path", "a", "--manifest-path", "b"],
            vec!["dev", "app-argument-without-separator"],
            vec!["build", "--release"],
            vec!["build", "--", "app-argument"],
        ] {
            assert!(
                parsed(&args).is_err(),
                "accepted invalid arguments: {args:?}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_manifest_and_app_arguments_are_not_lossily_converted() {
        use std::os::unix::ffi::OsStringExt;

        let manifest = OsString::from_vec(b"path-\xff/Cargo.toml".to_vec());
        let argument = OsString::from_vec(b"argument-\xff".to_vec());
        assert_eq!(
            parse([
                "dev".into(),
                "--manifest-path".into(),
                manifest.clone(),
                "--".into(),
                argument.clone(),
            ]),
            Ok(Action::Dev {
                manifest_path: Some(manifest.into()),
                app_args: vec![argument],
            })
        );
    }
}
