#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
use boa_engine::{
    Context, JsError, JsNativeError, JsResult, JsValue, Module, NativeFunction,
    Source,
    builtins::promise::PromiseState,
    js_string,
    object::{ObjectInitializer, builtins::JsArray},
    property::Attribute,
};
use boa_parser::Source as ParserSource;
use include_dir::{Dir, include_dir};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

static BOA_HOST_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_boa_host_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&BOA_HOST_DATA_DIR, key)
}

thread_local! {
    static CAPTURED_STDOUT: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}

pub fn start_capturing_stdout() {
    CAPTURED_STDOUT.with(|captured| {
        *captured.borrow_mut() = Some(Vec::new());
    });
}

pub fn stop_capturing_stdout() -> Option<Vec<String>> {
    CAPTURED_STDOUT.with(|captured| captured.borrow_mut().take())
}

fn rust_print_stdout(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    if let Some(s) = args.first().and_then(|v| v.as_string()) {
        let text = s.to_std_string_escaped();
        let intercepted = CAPTURED_STDOUT.with(|captured| {
            if let Some(ref mut vec) = *captured.borrow_mut() {
                vec.push(text.clone());
                true
            } else {
                false
            }
        });
        if !intercepted {
            print!("{}", text);
        }
    }
    Ok(JsValue::undefined())
}

fn rust_read_file(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let content = std::fs::read_to_string(&path_str).map_err(|e| {
        JsNativeError::typ()
            .with_message(format!("failed to read file '{}': {}", path_str, e))
    })?;

    let path = Path::new(&path_str);
    let display_path = if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path)
    } else {
        path
    };
    info_fmt!("JS Host: Reading '{}'", display_path.display());

    Ok(JsValue::from(js_string!(content)))
}

fn rust_write_file(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let data = args
        .get(1)
        .ok_or_else(|| JsNativeError::typ().with_message("missing data"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("data must be a string")
        })?
        .to_std_string_escaped();

    if let Some(parent) = Path::new(&path_str).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("failed to create dir for write: {}", e))
        })?;
    }

    std::fs::write(&path_str, data).map_err(|e| {
        JsNativeError::typ()
            .with_message(format!("failed to write file '{}': {}", path_str, e))
    })?;

    let path = Path::new(&path_str);
    let display_path = if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path)
    } else {
        path
    };
    info_fmt!("JS Host: Writing '{}'", display_path.display());

    Ok(JsValue::undefined())
}

fn rust_file_exists(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let exists = Path::new(&path_str).exists();
    info_fmt!("JS Host: Check exists '{}' -> {}", path_str, exists);
    Ok(JsValue::from(exists))
}

fn rust_read_dir(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let entries = std::fs::read_dir(&path_str).map_err(|e| {
        JsNativeError::typ()
            .with_message(format!("failed to read dir '{}': {}", path_str, e))
    })?;

    let mut js_entries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            JsNativeError::typ()
                .with_message(format!("failed to read entry: {}", e))
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let js_name = js_string!(name);
        let js_entry = ObjectInitializer::new(context)
            .property(js_string!("name"), js_name.clone(), Attribute::all())
            .property(js_name.clone(), js_name, Attribute::all())
            .function(
                NativeFunction::from_copy_closure(move |_, _, _| {
                    Ok(JsValue::from(is_file))
                }),
                js_string!("isFile"),
                0,
            )
            .function(
                NativeFunction::from_copy_closure(move |_, _, _| {
                    Ok(JsValue::from(is_dir))
                }),
                js_string!("isDirectory"),
                0,
            )
            .function(
                NativeFunction::from_copy_closure(move |_, _, _| {
                    Ok(JsValue::from(false))
                }),
                js_string!("isSymbolicLink"),
                0,
            )
            .build();
        js_entries.push(JsValue::from(js_entry));
    }

    let array = JsArray::from_iter(js_entries, context);
    Ok(JsValue::from(array))
}

fn rust_file_size(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let size = std::fs::metadata(&path_str).map(|m| m.len()).unwrap_or(0);
    Ok(JsValue::from(js_string!(size.to_string())))
}

fn rust_file_mtime(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let mtime_str = std::fs::metadata(&path_str)
        .and_then(|m| m.modified())
        .and_then(|t| {
            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map_err(std::io::Error::other)
        })
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string());

    Ok(JsValue::from(js_string!(mtime_str)))
}

fn rust_is_file(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let is_file = Path::new(&path_str).is_file();
    info_fmt!("JS Host: Check is_file '{}' -> {}", path_str, is_file);
    Ok(JsValue::from(is_file))
}

fn rust_mkdir_sync(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    std::fs::create_dir_all(&path_str).map_err(|e| {
        JsNativeError::typ()
            .with_message(format!("failed to mkdir '{}': {}", path_str, e))
    })?;
    Ok(JsValue::undefined())
}

fn rust_utimes_sync(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn rust_unlink_sync(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let path_str = args
        .first()
        .ok_or_else(|| JsNativeError::typ().with_message("missing path"))?
        .as_string()
        .ok_or_else(|| {
            JsNativeError::typ().with_message("path must be a string")
        })?
        .to_std_string_escaped();

    let path = Path::new(&path_str);
    let display_path = if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path)
    } else {
        path
    };
    info_fmt!("JS Host: Unlinking '{}'", display_path.display());

    let _ = std::fs::remove_file(&path_str);
    Ok(JsValue::undefined())
}

fn rust_path_dirname(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let p = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let parent = Path::new(&p).parent().unwrap_or_else(|| Path::new(""));
    let parent_str = parent.to_string_lossy().into_owned().replace('\\', "/");
    Ok(JsValue::from(js_string!(parent_str)))
}

fn rust_path_basename(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let p = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let basename = Path::new(&p)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new(""));
    let basename_str = basename.to_string_lossy().into_owned();
    Ok(JsValue::from(js_string!(basename_str)))
}

fn rust_path_join(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let mut path_buf = PathBuf::new();
    for arg in args {
        if let Some(s) = arg.as_string() {
            path_buf.push(s.to_std_string_escaped());
        }
    }
    let res = path_buf.to_string_lossy().into_owned().replace('\\', "/");
    Ok(JsValue::from(js_string!(res)))
}

fn rust_path_resolve(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let mut resolved = PathBuf::new();
    for arg in args {
        if let Some(s) = arg.as_string() {
            let p = s.to_std_string_escaped();
            if Path::new(&p).is_absolute() {
                resolved = PathBuf::from(p);
            } else {
                resolved.push(p);
            }
        }
    }
    if !resolved.is_absolute()
        && let Ok(cwd) = std::env::current_dir()
    {
        resolved = cwd.join(resolved);
    }
    let mut normalized = PathBuf::new();
    for component in resolved.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::Normal(c) => {
                normalized.push(c);
            }
            Component::RootDir => {
                normalized.push("/");
            }
            Component::Prefix(p) => {
                normalized.push(p.as_os_str());
            }
        }
    }
    let res = normalized.to_string_lossy().into_owned().replace('\\', "/");
    Ok(JsValue::from(js_string!(res)))
}

fn rust_cwd(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
        .unwrap_or_else(|_| "/".to_string());
    Ok(JsValue::from(js_string!(cwd)))
}

fn log_fn(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let output = args
        .iter()
        .map(|arg| arg.display().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", output);
    Ok(JsValue::undefined())
}

pub fn extract_process_exit_code(message: &str) -> Option<i32> {
    let (_, remainder) = message.split_once("process.exit: ")?;
    let code_text: String = remainder
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();

    if code_text.is_empty() {
        return None;
    }

    code_text.parse().ok()
}

pub fn process_exit_code_from_error(error: &anyhow::Error) -> Option<i32> {
    extract_process_exit_code(&error.to_string())
}

pub fn run_js_module_allow_success_exit(
    entry_point: &Path,
    loader_root: &Path,
    args: &[String],
) -> Result<()> {
    match run_js_module(entry_point, loader_root, args) {
        Ok(()) => Ok(()),
        Err(error) => {
            if process_exit_code_from_error(&error) == Some(0) {
                return Ok(());
            }

            Err(error)
        }
    }
}

pub fn create_context_with_bindings(
    entry_point: &Path,
    loader_root: &Path,
    args: &[String],
    bind_fs_and_cwd: bool,
) -> Result<(Context, Rc<boa_engine::module::SimpleModuleLoader>)> {
    let loader = Rc::new(
        boa_engine::module::SimpleModuleLoader::new(loader_root)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?,
    );

    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    context.runtime_limits_mut().set_recursion_limit(1_000_000);

    // Register console.log, console.warn, console.error
    let console = ObjectInitializer::new(&mut context)
        .function(NativeFunction::from_fn_ptr(log_fn), js_string!("log"), 1)
        .function(NativeFunction::from_fn_ptr(log_fn), js_string!("warn"), 1)
        .function(NativeFunction::from_fn_ptr(log_fn), js_string!("error"), 1)
        .build();
    context
        .register_global_property(
            js_string!("console"),
            console,
            Attribute::default(),
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Register our custom Rust bindings
    context
        .register_global_builtin_callable(
            js_string!("__rust_print_stdout"),
            1,
            NativeFunction::from_fn_ptr(rust_print_stdout),
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    if bind_fs_and_cwd {
        context
            .register_global_builtin_callable(
                js_string!("__rust_read_file"),
                1,
                NativeFunction::from_fn_ptr(rust_read_file),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_write_file"),
                2,
                NativeFunction::from_fn_ptr(rust_write_file),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_file_exists"),
                1,
                NativeFunction::from_fn_ptr(rust_file_exists),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_read_dir"),
                1,
                NativeFunction::from_fn_ptr(rust_read_dir),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_file_size"),
                1,
                NativeFunction::from_fn_ptr(rust_file_size),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_file_mtime"),
                1,
                NativeFunction::from_fn_ptr(rust_file_mtime),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_is_file"),
                1,
                NativeFunction::from_fn_ptr(rust_is_file),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_mkdir_sync"),
                1,
                NativeFunction::from_fn_ptr(rust_mkdir_sync),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_utimes_sync"),
                3,
                NativeFunction::from_fn_ptr(rust_utimes_sync),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_unlink_sync"),
                1,
                NativeFunction::from_fn_ptr(rust_unlink_sync),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_cwd"),
                0,
                NativeFunction::from_fn_ptr(rust_cwd),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        context
            .register_global_builtin_callable(
                js_string!("__rust_path_dirname"),
                1,
                NativeFunction::from_fn_ptr(rust_path_dirname),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_path_basename"),
                1,
                NativeFunction::from_fn_ptr(rust_path_basename),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_path_join"),
                1,
                NativeFunction::from_fn_ptr(rust_path_join),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        context
            .register_global_builtin_callable(
                js_string!("__rust_path_resolve"),
                1,
                NativeFunction::from_fn_ptr(rust_path_resolve),
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    let entry_abs = std::fs::canonicalize(entry_point)
        .unwrap_or_else(|_| entry_point.to_path_buf());
    let entry_abs_str =
        entry_abs.to_string_lossy().into_owned().replace('\\', "/");
    let dirname_str = entry_abs
        .parent()
        .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
        .unwrap_or_else(|| "/".to_string());

    context
        .register_global_property(
            js_string!("__filename"),
            js_string!(entry_abs_str.clone()),
            Attribute::default(),
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    context
        .register_global_property(
            js_string!("__dirname"),
            js_string!(dirname_str),
            Attribute::default(),
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Run the JS setup script to mock process and require.
    let setup_script = bail_if_none!(get_boa_host_data("setup.js"));
    let setup_script = setup_script.as_slice();
    let setup_script = str::from_utf8(setup_script)?;

    context
        .eval(Source::from_bytes(setup_script))
        .map_err(|e| {
            anyhow::anyhow!("Setup script evaluation failed: {}", e)
        })?;

    // Dynamically inject argv
    let mut js_argv = vec![
        JsValue::from(js_string!("boa")),
        JsValue::from(js_string!(entry_abs_str.as_str())),
    ];
    for arg in args {
        js_argv.push(JsValue::from(js_string!(arg.as_str())));
    }
    let js_argv_array = JsArray::from_iter(js_argv, &mut context);
    let process = context
        .global_object()
        .get(js_string!("process"), &mut context)
        .map_err(|e| anyhow::anyhow!("Failed to get process: {}", e))?;
    process
        .as_object()
        .unwrap()
        .set(js_string!("argv"), js_argv_array, false, &mut context)
        .map_err(|e| anyhow::anyhow!("Failed to set argv: {}", e))?;

    Ok((context, loader))
}

pub fn run_js_module(
    entry_point: &Path,
    loader_root: &Path,
    args: &[String],
) -> Result<()> {
    let (mut context, loader) =
        create_context_with_bindings(entry_point, loader_root, args, true)?;

    // Load and run the main entry module
    let entry_source =
        ParserSource::from_filepath(entry_point).map_err(|e| {
            anyhow::anyhow!(
                "Failed to open entry file '{}': {}",
                entry_point.display(),
                e
            )
        })?;
    let module = Module::parse(entry_source, None, &mut context)
        .map_err(|e| anyhow::anyhow!("Failed to parse module: {}", e))?;

    loader.insert(entry_point.to_path_buf(), module.clone());

    let promise = module.load_link_evaluate(&mut context);
    context
        .run_jobs()
        .map_err(|e| anyhow::anyhow!("Failed to run jobs: {}", e))?;

    match promise.state() {
        PromiseState::Pending => {
            Err(anyhow::anyhow!("Module remained pending"))
        }
        PromiseState::Fulfilled(_) => Ok(()),
        PromiseState::Rejected(err) => {
            let js_err = JsError::from_opaque(err);
            let erased = js_err.into_erased(&mut context);
            let mut stack_trace = String::new();
            for frame in context.stack_trace() {
                let loc = frame.position();
                let name = loc.function_name.to_std_string_escaped();
                let path = format!("{}", loc.path);
                let pos = loc
                    .position
                    .map(|p| {
                        format!(":{}:{}", p.line_number(), p.column_number())
                    })
                    .unwrap_or_default();
                stack_trace
                    .push_str(&format!("    at {} ({}{})\n", name, path, pos));
            }
            Err(anyhow::anyhow!(
                "Module evaluation rejected: {:?}\nJS Stack Trace:\n{}",
                erased,
                stack_trace
            ))
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[ctb_test]
    fn test_run_js_module_allow_success_exit_accepts_exit_zero() {
        let temp_dir = tempfile::tempdir().unwrap();
        let entry_point = temp_dir.path().join("exit_zero.js");

        std::fs::write(&entry_point, "process.exit(0);\n").unwrap();

        let result = run_js_module_allow_success_exit(
            &entry_point,
            temp_dir.path(),
            &[],
        );

        assert!(
            result.is_ok(),
            "Expected process.exit(0) to succeed: {result:?}"
        );
    }

    #[ctb_test]
    fn test_run_js_module_allow_success_exit_rejects_non_zero_exit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let entry_point = temp_dir.path().join("exit_one.js");

        std::fs::write(&entry_point, "process.exit(1);\n").unwrap();

        let result = run_js_module_allow_success_exit(
            &entry_point,
            temp_dir.path(),
            &[],
        );

        assert!(result.is_err(), "Expected process.exit(1) to fail");
    }
}
