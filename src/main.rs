use anyhow::{Context, Result};
use fxhash::FxHashMap;
use std::{
    cell::OnceCell,
    fmt::Write as _,
    path::{Path, PathBuf},
};

fn main() {
    if let Err(error) = main_inner() {
        eprintln!("pathwrap error: {error:?}");
        std::process::exit(2);
    }
}

const SUFFIX: &str = "-pathwrap";

fn main_inner() -> Result<()> {
    let mut args = std::env::args_os();

    env_logger::Builder::from_env("PATHWRAP_LOG").init();

    // Find out the command we're wrapping
    let Some(argv0) = args.next() else {
        print_usage();
        anyhow::bail!("unable to get argv[0]");
    };
    log::debug!("argv[0] = {argv0:?}");

    let inner_exe = (|| {
        // argv0 mode
        let exe = Path::new(&argv0);
        let exe_name = exe
            .file_stem()
            .with_context(|| {
                format!(
                    "executable name (argv[0]) lacks file name: {}",
                    exe.display()
                )
            })?
            .to_str()
            .context("executable name (argv[0]) is not a Unicode string")?;
        log::debug!("exe_name = {exe_name:?}");

        let inner_exe_name = exe_name.strip_suffix(SUFFIX).with_context(|| {
            format!("executable name '{exe_name}' does not end with '{SUFFIX}'")
        })?;
        let mut inner_exe = exe.with_file_name(inner_exe_name);
        if let Some(extension) = exe.extension() {
            inner_exe = inner_exe.with_extension(extension);
        }
        Ok(inner_exe)
    })();

    let inner_exe = inner_exe.or_else(|error: anyhow::Error| {
        // argv1 mode
        log::debug!("Unable to figure out the wrapped command from argv[0]: {error:?}");
        log::debug!("Checking argv[1] now...");
        let argv1 = args.next().context("unable to get argv[1]")?;

        Ok(argv1.into())
    });

    let inner_exe = inner_exe.map_err(|error: anyhow::Error| {
        print_usage();
        error
    })?;

    log::debug!("inner_exe = {inner_exe:?}");

    // Process command arguments
    let mut temp_file_set = TempFileSet::default();
    let new_args: Vec<_> = args
        .map(|arg| {
            log::debug!("Processing argument: {arg:?}");

            // Ignore flags
            if arg.to_str().is_some_and(|arg| arg.starts_with('-')) {
                log::debug!(
                    "The argument starts with '-', ignoring it because it's probably a flag"
                );
                return Ok(arg);
            }

            // TODO: Handle `@`-files, which rustc may use to circumvent the OS
            // limitations in a command-line argument length
            // <https://github.com/rust-lang/rust/blob/378a43a06510f3e3a49c69c8de71745e6a884048/compiler/rustc_codegen_ssa/src/back/link.rs#L1606>

            // Substitute the file path
            temp_file_set
                .wrap(Path::new(&arg))
                .map(Into::into)
                .with_context(|| format!("unable to process argument {arg:?}"))
        })
        .inspect(|arg| {
            let Ok(arg) = arg else { return };
            log::debug!("Processed argument: {arg:?}");
        })
        .collect::<Result<_>>()?;

    // Run the wrapped command
    let mut command = std::process::Command::new(inner_exe);
    command.args(new_args);
    let status = command
        .status()
        .with_context(|| format!("failed to spawn command {command:?}"))?;
    let code = status
        .code()
        .with_context(|| format!("command exited with {status}"))?;

    // Clean up the temporary files
    // TODO: Somehow do this even if the command is interrupted
    drop(temp_file_set);

    std::process::exit(code);
}

fn print_usage() {
    print!(
        "\
pathwrap - a wrapper program to substitute long paths in command-line arguments

USAGE: {argv1_cmd:>17} COMMAND ARGS... (argv1 mode)
       {argv0_cmd:>17} ARGS...         (argv0 mode)
",
        argv0_cmd = format!("COMMAND{SUFFIX}"),
        argv1_cmd = "pathwrap",
    );
}

#[derive(Default)]
struct TempFileSet {
    dir: OnceCell<tempfile::TempDir>,
    entries: FxHashMap<String, Vec<Entry>>,
    linked_dirs: FxHashMap<PathBuf, PathBuf>,
}

struct Entry {}

impl TempFileSet {
    fn wrap(&mut self, path: &Path) -> Result<PathBuf> {
        if path
            .to_str()
            .is_some_and(|path| path.is_ascii() && path.len() < 250)
        {
            // This path is short enough and all-ASCII
            log::debug!("Not wrapping path {path:?} because it looks short enough");
            return Ok(path.to_owned());
        }

        let Some(file_name) = path.file_name() else {
            log::debug!("Not wrapping path {path:?} because it lacks a file name portion");
            return Ok(path.to_owned());
        };

        let Some(parent) = path.parent() else {
            log::debug!("Not wrapping path {path:?} because its parent directory is unclear");
            return Ok(path.to_owned());
        };
        let parent = match std::fs::canonicalize(parent) {
            Ok(path) => path,
            Err(error) => {
                log::debug!(
                    "Not wrapping path {path:?} because canonicalization of \
                    parent directory path failed: {error}"
                );
                return Ok(path.to_owned());
            }
        };

        log::debug!("Wrapping path {path:?}");

        // Link its parent directory
        let new_parent = if let Some(new_parent) = self.linked_dirs.get(&parent) {
            new_parent
        } else {
            let new_parent = self
                .link_dir(&parent)
                .with_context(|| format!("failed to link parent directory of {path:?}"))?;
            self.linked_dirs
                .entry(parent.to_owned())
                .or_insert(new_parent)
        };

        // Determine the replacement file name
        Ok(new_parent.join(file_name))
    }

    fn link_dir(&mut self, path: &Path) -> Result<PathBuf> {
        // Determine the temporary directory name
        let Some(name) = path.file_name() else {
            // No file name
            return Ok(path.to_owned());
        };
        let mut name = name.to_string_lossy().into_owned();
        if !name.is_ascii() {
            let mut new_name = String::with_capacity(name.len() * 2);
            for ch in name.chars() {
                if ch.is_ascii() && ch != '~' {
                    new_name.push(ch);
                } else {
                    write!(new_name, "x{:02X}", ch as u32).unwrap();
                }
            }
            name = new_name;
        }
        name.truncate(20);

        let entries_of_name = self.entries.entry(name.clone()).or_default();
        if !entries_of_name.is_empty() {
            // `~` is removed from `name` above, so this is guaranteed to
            // generate unique file names
            write!(name, "~{}", entries_of_name.len()).unwrap();
        }

        // Determine the temporary directory path
        if self.dir.get().is_none() {
            _ = self
                .dir
                .set(tempfile::tempdir().context("failed to create a temporary directory")?);
        }
        let new_path = self.dir.get().unwrap().path().join(name);

        log::debug!("Creating a symbolic link {new_path:?} -> {path:?}");
        std::os::windows::fs::symlink_dir(path, &new_path).with_context(|| {
            format!("failed to create a symbolic link from {new_path:?} to {path:?}")
        })?;

        entries_of_name.push(Entry {});

        Ok(new_path)
    }
}
