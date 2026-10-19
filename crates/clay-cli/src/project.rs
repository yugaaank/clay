use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MANIFEST: &str = include_str!("../../../templates/minimal/Cargo.toml.template");
const README: &str = include_str!("../../../templates/minimal/README.md");
const MAIN: &str = include_str!("../../../templates/minimal/src/main.rs");
const COUNTER: &str = include_str!("../../../templates/minimal/src/counter.rs");
const GITIGNORE: &str = include_str!("../../../templates/minimal/.gitignore");

pub const PUBLIC_CLAY_REPOSITORY: &str = "https://github.com/yugaaank/clay.git";
pub const PUBLIC_CLAY_TAG: &str = "v0.3.0";

#[derive(Debug, PartialEq, Eq)]
pub enum FrameworkSource {
    PublicGit,
    LocalPath(PathBuf),
}

pub struct GeneratedProject {
    pub directory: PathBuf,
    pub package_name: String,
}

/// Write only the embedded project files. Lock resolution is a separate child process.
/// Never remove files on failure: even a partially written destination belongs to the user.
pub fn write_project(
    destination: &Path,
    framework_source: FrameworkSource,
) -> Result<GeneratedProject, String> {
    match fs::symlink_metadata(destination) {
        Ok(_) => return Err(existing_destination(destination)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "cannot inspect destination {}: {error}",
                destination.display()
            ));
        }
    }

    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "destination must end in a valid UTF-8 Rust package name".to_owned())?;
    validate_package_name(name)?;
    let framework_dependency = framework_dependency(framework_source)?;
    let manifest = MANIFEST
        .replace("{{package_name}}", name)
        .replace("{{framework_dependency}}", &framework_dependency);
    let readme = README.replace("{{package_name}}", name);

    if let Some(parent) = destination.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create parent {}: {error}", parent.display()))?;
    }
    // Unlike create_dir_all(destination), this also rejects a concurrent creator.
    fs::create_dir(destination).map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            existing_destination(destination)
        } else {
            format!(
                "cannot create destination {}: {error}",
                destination.display()
            )
        }
    })?;

    let write_files = || -> io::Result<()> {
        fs::create_dir(destination.join("src"))?;
        for (relative_path, content) in [
            ("Cargo.toml", manifest.as_str()),
            ("README.md", readme.as_str()),
            (".gitignore", GITIGNORE),
            ("src/main.rs", MAIN),
            ("src/counter.rs", COUNTER),
        ] {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(relative_path))?
                .write_all(content.as_bytes())?;
        }
        Ok(())
    };
    write_files().map_err(|error| {
        format!(
            "could not finish writing project at {}: {error}. Existing files were kept",
            destination.display()
        )
    })?;
    let directory = fs::canonicalize(destination).map_err(|error| {
        format!(
            "could not resolve generated directory {}: {error}. Project files were kept",
            destination.display()
        )
    })?;
    Ok(GeneratedProject {
        directory,
        package_name: name.to_owned(),
    })
}

fn framework_dependency(source: FrameworkSource) -> Result<String, String> {
    match source {
        FrameworkSource::PublicGit => Ok(format!(
            "{{ git = {}, tag = {} }}",
            toml_string(PUBLIC_CLAY_REPOSITORY),
            toml_string(PUBLIC_CLAY_TAG),
        )),
        FrameworkSource::LocalPath(path) => {
            let directory = validate_framework_path(&path)?;
            let path = directory.to_str().ok_or_else(|| {
                "framework path must be valid UTF-8 so it can be stored in Cargo.toml".to_owned()
            })?;
            Ok(format!("{{ path = {} }}", toml_string(path)))
        }
    }
}

fn existing_destination(destination: &Path) -> String {
    format!(
        "destination {} already exists; choose a new path (nothing was overwritten)",
        destination.display()
    )
}

fn validate_framework_path(path: &Path) -> Result<PathBuf, String> {
    let directory = fs::canonicalize(path)
        .map_err(|error| format!("cannot resolve framework path {}: {error}", path.display()))?;
    if !directory.is_dir() || !directory.join("Cargo.toml").is_file() {
        return Err(format!(
            "framework path {} must be a crate directory containing Cargo.toml",
            path.display()
        ));
    }
    Ok(directory)
}

fn validate_package_name(name: &str) -> Result<(), String> {
    // Keep package names in Cargo's portable ASCII subset. Hyphens become underscores
    // in Rust crate identifiers, so validate that identifier against reserved words too.
    const RESERVED: &[&str] = &[
        "_",
        "as",
        "async",
        "await",
        "become",
        "box",
        "break",
        "const",
        "continue",
        "crate",
        "do",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "final",
        "fn",
        "for",
        "gen",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "macro",
        "match",
        "mod",
        "move",
        "mut",
        "override",
        "priv",
        "pub",
        "ref",
        "return",
        "Self",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "try",
        "type",
        "typeof",
        "unsafe",
        "unsized",
        "use",
        "virtual",
        "where",
        "while",
        "yield",
        "test",
        "build",
        "deps",
        "examples",
        "incremental",
    ];
    let mut chars = name.chars();
    let starts_correctly = chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_');
    let valid_characters = chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    let crate_name = name.replace('-', "_");
    let upper = name.to_ascii_uppercase();
    let windows_reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            upper
                .strip_prefix(prefix)
                .is_some_and(|tail| tail.len() == 1 && matches!(tail.as_bytes()[0], b'1'..=b'9'))
        });
    if !starts_correctly
        || !valid_characters
        || RESERVED.contains(&crate_name.as_str())
        || windows_reserved
    {
        return Err(format!(
            "invalid Rust package name `{name}`: use ASCII letters, digits, '-' or '_', \
             start with a letter or '_', and avoid reserved names"
        ));
    }
    Ok(())
}

/// A TOML basic string, including quotes. Paths are data, never TOML or shell syntax.
fn toml_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if ch.is_control() => {
                use std::fmt::Write;
                // Writing to a String cannot fail.
                let _ = write!(escaped, "\\u{:04X}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn framework_in(parent: &Path) -> PathBuf {
        let framework = parent.join("framework checkout");
        fs::create_dir(&framework).expect("create stub framework");
        fs::write(
            framework.join("Cargo.toml"),
            "[package]\nname = \"clay\"\nversion = \"0.1.0\"\n",
        )
        .expect("write stub manifest");
        framework
    }

    #[test]
    fn generates_a_complete_standalone_project_without_resolving_dependencies() {
        let temp = TempDir::new();
        let framework = framework_in(temp.path());
        let destination = temp.path().join("parent with spaces/my-app");
        let project = write_project(&destination, FrameworkSource::LocalPath(framework.clone()))
            .expect("generate files only");
        assert_eq!(project.package_name, "my-app");
        assert_eq!(
            project.directory,
            fs::canonicalize(&destination).expect("absolute destination")
        );
        let manifest = fs::read_to_string(destination.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains("name = \"my-app\""));
        assert!(manifest.contains("[workspace]\nresolver = \"2\""));
        assert!(manifest.contains(&format!(
            "clay = {{ path = {} }}",
            toml_string(framework.to_str().expect("UTF-8 test path"))
        )));
        assert!(manifest.contains("[patch.crates-io]"));
        assert!(manifest.contains("async-task = { git = \"https://github.com/zed-industries/async-task.git\", rev = \"015845a19fc31752c02367567d3b8900a406f9da\" }"));
        assert!(manifest.contains("calloop = { git = \"https://github.com/zed-industries/calloop\", rev = \"3759371fee14c40066d64777e2a36b6ffcc22590\" }"));
        assert!(!manifest.contains("{{"));
        for file in ["README.md", ".gitignore", "src/main.rs", "src/counter.rs"] {
            assert!(destination.join(file).is_file(), "missing {file}");
        }
        let readme = fs::read_to_string(destination.join("README.md")).expect("README");
        assert!(readme.starts_with("# my-app\n"));
        assert!(readme.contains("clay dev"));
        assert!(readme.contains("clay build"));
        assert!(readme.contains("cargo generate-lockfile"));
        assert!(!destination.join("Cargo.lock").exists());
    }

    #[test]
    fn generates_the_public_git_source_without_a_local_checkout() {
        let temp = TempDir::new();
        let destination = temp.path().join("my-app");
        write_project(&destination, FrameworkSource::PublicGit).expect("generate files only");
        let manifest = fs::read_to_string(destination.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains(&format!(
            "clay = {{ git = {}, tag = {} }}",
            toml_string(PUBLIC_CLAY_REPOSITORY),
            toml_string(PUBLIC_CLAY_TAG),
        )));
        assert!(!manifest.contains("{{framework_dependency}}"));
        assert!(!manifest.contains("path ="));
    }

    #[test]
    fn accepts_portable_package_names_and_rejects_invalid_or_reserved_names() {
        for name in ["my-app", "my_app", "A1", "_private", "a--b"] {
            assert!(validate_package_name(name).is_ok(), "rejected {name}");
        }
        for name in [
            "",
            "_",
            "123app",
            "my app",
            "foo.bar",
            "app!",
            "-app",
            "café",
            "fn",
            "Self",
            "self",
            "async",
            "gen",
            "test",
            "build",
            "deps",
            "examples",
            "incremental",
            "con",
            "NUL",
            "COM1",
            "lpt9",
        ] {
            assert!(validate_package_name(name).is_err(), "accepted {name}");
        }
    }

    #[test]
    fn invalid_name_does_not_create_destination_or_its_parents() {
        let temp = TempDir::new();
        let destination = temp.path().join("new parent/invalid name");
        let error = write_project(
            &destination,
            FrameworkSource::LocalPath(framework_in(temp.path())),
        )
        .err()
        .expect("invalid name must fail");
        assert!(error.contains("invalid Rust package name"));
        assert!(!temp.path().join("new parent").exists());
    }

    #[test]
    fn existing_empty_directories_and_files_are_never_reused_or_clobbered() {
        let temp = TempDir::new();
        let framework = framework_in(temp.path());
        let empty = temp.path().join("empty-app");
        fs::create_dir(&empty).expect("empty destination");
        let file = temp.path().join("file-app");
        fs::write(&file, "keep this data").expect("existing file");
        let populated = temp.path().join("populated-app");
        fs::create_dir(&populated).expect("populated destination");
        fs::write(populated.join("Cargo.toml"), "keep this manifest").expect("existing manifest");
        for destination in [&empty, &file, &populated] {
            let error = write_project(destination, FrameworkSource::LocalPath(framework.clone()))
                .err()
                .expect("existing destinations fail");
            assert!(error.contains("already exists"));
        }
        assert_eq!(fs::read_dir(empty).expect("existing directory").count(), 0);
        assert_eq!(
            fs::read_to_string(file).expect("existing file"),
            "keep this data"
        );
        assert_eq!(
            fs::read_to_string(populated.join("Cargo.toml")).expect("existing manifest"),
            "keep this manifest"
        );
    }

    #[cfg(unix)]
    #[test]
    fn dangling_and_directory_symlinks_are_existing_destinations() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let framework = framework_in(temp.path());
        for (name, target) in [
            ("dangling-app", temp.path().join("missing")),
            ("linked-app", framework.clone()),
        ] {
            let destination = temp.path().join(name);
            symlink(&target, &destination).expect("create destination symlink");
            assert!(
                write_project(&destination, FrameworkSource::LocalPath(framework.clone()),)
                    .err()
                    .expect("symlink must be rejected")
                    .contains("already exists")
            );
            assert_eq!(
                fs::read_link(&destination).expect("symlink retained"),
                target
            );
        }
        assert!(!framework.join("src").exists());
    }

    #[test]
    fn missing_framework_or_manifest_is_rejected_before_writing() {
        let temp = TempDir::new();
        let destination = temp.path().join("my-app");
        assert!(
            write_project(
                &destination,
                FrameworkSource::LocalPath(temp.path().join("missing")),
            )
            .is_err()
        );
        assert!(!destination.exists());
        assert!(
            write_project(
                &destination,
                FrameworkSource::LocalPath(temp.path().to_owned()),
            )
            .is_err()
        );
        assert!(!destination.exists());
        let framework = framework_in(temp.path());
        assert!(
            write_project(
                &destination,
                FrameworkSource::LocalPath(framework.join("Cargo.toml")),
            )
            .is_err()
        );
        assert!(!destination.exists());
    }

    #[test]
    fn toml_paths_escape_quotes_backslashes_and_controls_but_preserve_unicode() {
        assert_eq!(
            toml_string("C:\\my \"app\"\\雪\n\r\t\u{08}\u{0c}\u{00}\u{1f}\u{7f}"),
            "\"C:\\\\my \\\"app\\\"\\\\雪\\n\\r\\t\\b\\f\\u0000\\u001F\\u007F\""
        );
        assert_eq!(toml_string(""), "\"\"");
    }

    #[cfg(unix)]
    #[test]
    fn writes_a_real_framework_path_with_toml_metacharacters_as_data() {
        let temp = TempDir::new();
        let framework = temp
            .path()
            .join("framework \"quoted\" \\ 雪\n{{package_name}}");
        fs::create_dir(&framework).expect("unusual framework path");
        fs::write(framework.join("Cargo.toml"), "[package]\n").expect("manifest");
        let destination = temp.path().join("my-app");
        write_project(&destination, FrameworkSource::LocalPath(framework.clone()))
            .expect("path should be escaped");
        let manifest = fs::read_to_string(destination.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains(&toml_string(framework.to_str().expect("UTF-8"))));
        assert!(manifest.contains("{{package_name}}")); // Literal path data, not a template slot.
        assert!(manifest.contains("name = \"my-app\""));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_framework_path_is_reported_before_creating_destination() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let temp = TempDir::new();
        let framework = temp
            .path()
            .join(OsString::from_vec(b"framework-\xff".to_vec()));
        fs::create_dir(&framework).expect("non-UTF-8 framework path");
        fs::write(framework.join("Cargo.toml"), "[package]\n").expect("manifest");
        let destination = temp.path().join("my-app");
        assert!(
            write_project(&destination, FrameworkSource::LocalPath(framework))
                .err()
                .expect("TOML cannot represent a non-UTF-8 path")
                .contains("must be valid UTF-8")
        );
        assert!(!destination.exists());
    }
}
