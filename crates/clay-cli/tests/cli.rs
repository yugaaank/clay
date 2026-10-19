#[path = "../src/test_support.rs"]
mod test_support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use test_support::TempDir;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_clay"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status: {}; stdout: {}; stderr: {}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    );
}

#[test]
fn binary_reports_help_version_and_argument_errors() {
    let output = cli().arg("--help").output().expect("run help");
    assert_success(&output);
    for command in ["new", "dev", "build", "--framework-path", "--manifest-path"] {
        assert!(text(&output.stdout).contains(command));
    }
    let output = cli()
        .args(["new", "--help"])
        .output()
        .expect("run new help");
    assert_success(&output);
    assert!(text(&output.stdout).contains("https://github.com/yugaaank/clay.git"));
    assert!(text(&output.stdout).contains("v0.3.0"));
    let output = cli().arg("--version").output().expect("run version");
    assert_success(&output);
    assert!(text(&output.stdout).starts_with("clay "));
    for args in [vec!["not-a-command"], vec!["new"], vec!["build", "--oops"]] {
        let output = cli().args(&args).output().expect("run invalid command");
        assert!(!output.status.success(), "invalid arguments: {args:?}");
        assert!(text(&output.stderr).contains("error:"));
    }
}

fn stub_framework(parent: &Path) -> PathBuf {
    let framework = parent.join("framework checkout");
    fs::create_dir(&framework).expect("create stub framework");
    fs::write(
        framework.join("Cargo.toml"),
        "[package]\nname = \"clay\"\nversion = \"0.1.0\"\n",
    )
    .expect("stub manifest");
    framework
}

#[test]
fn invalid_and_existing_destinations_fail_without_invoking_cargo() {
    let temp = TempDir::new();
    let framework = stub_framework(temp.path());
    let invalid = temp.path().join("bad name");
    let output = cli()
        .arg("new")
        .arg(&invalid)
        .arg("--framework-path")
        .arg(&framework)
        .env("PATH", temp.path().join("no-cargo-here"))
        .output()
        .expect("run CLI");
    assert!(!output.status.success());
    assert!(text(&output.stderr).contains("invalid Rust package name"));
    assert!(!invalid.exists());

    let existing = temp.path().join("existing-app");
    fs::create_dir(&existing).expect("existing directory");
    fs::write(existing.join("sentinel"), "keep me").expect("sentinel");
    let output = cli()
        .arg("new")
        .arg(&existing)
        .arg("--framework-path")
        .arg(&framework)
        .env("PATH", temp.path().join("no-cargo-here"))
        .output()
        .expect("run CLI");
    assert!(!output.status.success());
    assert!(text(&output.stderr).contains("already exists"));
    assert_eq!(
        fs::read_to_string(existing.join("sentinel")).expect("sentinel preserved"),
        "keep me"
    );
    assert!(!existing.join("Cargo.toml").exists());
}

#[test]
fn missing_cargo_keeps_generated_files_and_explains_the_retry() {
    let temp = TempDir::new();
    let framework = stub_framework(temp.path());
    let destination = temp.path().join("my-app");
    let output = cli()
        .arg("new")
        .arg(&destination)
        .arg("--framework-path")
        .arg(&framework)
        .env("PATH", temp.path().join("no-cargo-here"))
        .output()
        .expect("run CLI");
    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(stderr.contains("could not run cargo generate-lockfile"));
    assert!(stderr.contains("Project files were kept"));
    assert!(stderr.contains("cargo generate-lockfile"));
    assert!(stderr.contains("from that directory to retry"));
    assert!(destination.join("src/main.rs").is_file());
    assert!(destination.join("Cargo.toml").is_file());
}

#[test]
fn new_resolves_lockfile_and_dev_build_forward_to_direct_cargo_children() {
    let temp = TempDir::new();
    let framework = stub_framework(temp.path());
    let tools = temp.path().join("fake cargo tools");
    fs::create_dir(&tools).expect("create tools directory");
    let fake_cargo = tools.join(format!("cargo{}", std::env::consts::EXE_SUFFIX));
    let output = Command::new("rustc")
        .arg("--edition=2024")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/fake_cargo.rs"))
        .arg("-o")
        .arg(&fake_cargo)
        .output()
        .expect("compile local fake Cargo (no dependencies or network)");
    assert_success(&output);

    let log = temp.path().join("cargo-arguments");
    let command = || {
        let mut command = cli();
        command
            .env("PATH", &tools)
            .env("GPUI_CLI_TEST_LOG", &log)
            .env("GPUI_CLI_TEST_CARGO_EXIT", "0")
            .current_dir(temp.path());
        command
    };

    let failed = temp.path().join("failed-app");
    let output = command()
        .arg("new")
        .arg(&failed)
        .arg("--framework-path")
        .arg(&framework)
        .env("GPUI_CLI_TEST_CARGO_EXIT", "42")
        .output()
        .expect("exercise lock resolution failure");
    assert!(!output.status.success());
    assert!(text(&output.stderr).contains("cargo generate-lockfile failed"));
    assert!(text(&output.stderr).contains("Project files were kept"));
    assert!(text(&output.stderr).contains("from that directory to retry"));
    assert!(failed.join("Cargo.toml").is_file());
    assert!(!failed.join("Cargo.lock").exists());

    // Use relative paths from an unrelated working directory; installed CLI assets
    // and lock resolution must not depend on the source checkout being the cwd.
    let destination = temp.path().join("parent with spaces/my-app");
    let output = command()
        .args(["new", "parent with spaces/my-app", "--framework-path"])
        .arg(framework.file_name().expect("framework directory name"))
        .output()
        .expect("generate project with fake Cargo");
    assert_success(&output);
    assert!(destination.join("Cargo.lock").is_file());
    assert!(destination.join("src/counter.rs").is_file());
    assert_eq!(
        fs::read_to_string(&log).expect("lock command arguments"),
        format!(
            "generate-lockfile\n--manifest-path\n{}",
            destination.join("Cargo.toml").display()
        )
    );
    assert_eq!(
        PathBuf::from(fs::read_to_string(log.with_extension("cwd")).expect("Cargo cwd")),
        fs::canonicalize(&destination).expect("generated directory")
    );

    let public_destination = temp.path().join("public-app");
    let output = command()
        .arg("new")
        .arg(&public_destination)
        .output()
        .expect("generate project with public source");
    assert_success(&output);
    let public_manifest =
        fs::read_to_string(public_destination.join("Cargo.toml")).expect("public-source manifest");
    assert!(
        public_manifest.contains(
            "clay = { git = \"https://github.com/yugaaank/clay.git\", tag = \"v0.3.0\" }"
        )
    );
    assert!(!public_manifest.contains("path ="));

    let manifest = destination.join("Cargo.toml");
    let literal = "a ; $(touch SHOULD_NOT_EXIST) \\ 雪";
    let output = command()
        .arg("dev")
        .arg("--manifest-path")
        .arg(&manifest)
        .args(["--", "--title", literal])
        .env("GPUI_CLI_TEST_CARGO_EXIT", "7")
        .output()
        .expect("forward dev arguments");
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(
        fs::read_to_string(&log).expect("dev command arguments"),
        format!(
            "run\n--locked\n--manifest-path\n{}\n--\n--title\n{literal}",
            manifest.display()
        )
    );
    assert!(!temp.path().join("SHOULD_NOT_EXIST").exists());

    let output = command()
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("forward build arguments");
    assert_success(&output);
    assert_eq!(
        fs::read_to_string(&log).expect("build command arguments"),
        format!(
            "build\n--release\n--locked\n--manifest-path\n{}",
            manifest.display()
        )
    );
}
