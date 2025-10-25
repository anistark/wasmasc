use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Compilation failed: {reason}")]
    CompilationFailed { reason: String },

    #[error("Compile tool not found: {tool}")]
    CompileToolNotFound { tool: String },

    #[error("Invalid project structure: {reason}")]
    InvalidProjectStructure { reason: String },

    #[error("Missing entry file. Expected one of: {candidates:?}")]
    MissingEntryFile { candidates: Vec<String> },

    #[error("Output directory creation failed: {path}")]
    OutputDirectoryCreationFailed { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSource {
    CratesIo { name: String, version: String },
    Git { url: String, branch: Option<String> },
    Local { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Builtin,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub compile_wasm: bool,
    pub compile_webapp: bool,
    pub live_reload: bool,
    pub optimization: bool,
    pub custom_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub extensions: Vec<String>,
    pub entry_files: Vec<String>,
    pub plugin_type: PluginType,
    pub source: Option<PluginSource>,
    pub dependencies: Vec<String>,
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub project_path: String,
    pub output_dir: String,
    pub optimization_level: OptimizationLevel,
    pub verbose: bool,
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub wasm_path: String,
    pub js_path: Option<String>,
    pub additional_files: Vec<String>,
    pub is_wasm_bindgen: bool,
}

pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn get_builder(&self) -> Box<dyn WasmBuilder>;
}

pub trait WasmBuilder: Send + Sync {
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn build(&self, config: &BuildConfig) -> PluginResult<BuildResult>;
    fn check_dependencies(&self) -> Vec<String>;
    fn validate_project(&self, project_path: &str) -> PluginResult<()>;
    fn clean(&self, project_path: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn clone_box(&self) -> Box<dyn WasmBuilder>;
    fn language_name(&self) -> &str;
    fn entry_file_candidates(&self) -> &[&str];
    fn supported_extensions(&self) -> &[&str];
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn is_tool_installed(tool: &str) -> bool {
        Command::new(tool)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn execute_command(
        cmd: &str,
        args: &[&str],
        cwd: &str,
        verbose: bool,
    ) -> PluginResult<Output> {
        if verbose {
            println!("Executing: {} {}", cmd, args.join(" "));
        }

        Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(PluginError::Io)
    }

    pub fn copy_to_output(src: &str, dst: &str, lang: &str) -> PluginResult<String> {
        let src_path = Path::new(src);
        if !src_path.exists() {
            return Err(PluginError::CompilationFailed {
                reason: format!("{lang} build completed but output file not found"),
            });
        }

        let filename = src_path.file_name().unwrap();
        let dst_path = Path::new(dst).join(filename);
        fs::copy(src_path, &dst_path).map_err(PluginError::Io)?;

        Ok(dst_path.to_string_lossy().to_string())
    }
}

pub struct PathResolver;

impl PathResolver {
    pub fn join_paths(base: &str, rel: &str) -> String {
        Path::new(base).join(rel).to_string_lossy().to_string()
    }

    pub fn validate_directory_exists(path: &str) -> PluginResult<()> {
        let p = Path::new(path);
        if !p.is_dir() {
            return Err(PluginError::InvalidProjectStructure {
                reason: format!("Directory does not exist: {path}"),
            });
        }
        Ok(())
    }

    pub fn ensure_output_directory(path: &str) -> PluginResult<()> {
        fs::create_dir_all(path).map_err(|_| PluginError::OutputDirectoryCreationFailed {
            path: path.to_string(),
        })
    }

    pub fn find_files_with_extension(path: &str, ext: &str) -> PluginResult<Vec<String>> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension.to_string_lossy() == ext {
                        files.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }
        Ok(files)
    }
}

#[derive(Clone)]
pub struct AscPlugin {
    info: PluginInfo,
}

impl AscPlugin {
    pub fn new() -> Self {
        let info = PluginInfo {
            name: "asc".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AssemblyScript WebAssembly compiler".to_string(),
            author: "Wasmrun Team".to_string(),
            extensions: vec!["ts".to_string()],
            entry_files: vec![
                "assembly/index.ts".to_string(),
                "index.ts".to_string(),
                "package.json".to_string(),
            ],
            plugin_type: PluginType::External,
            source: Some(PluginSource::CratesIo {
                name: "wasmasc".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            dependencies: vec![],
            capabilities: PluginCapabilities {
                compile_wasm: true,
                compile_webapp: false,
                live_reload: true,
                optimization: true,
                custom_targets: vec!["wasm".to_string()],
            },
        };

        Self { info }
    }

    fn is_asc_project(&self, project_path: &str) -> bool {
        let package_json_path = PathResolver::join_paths(project_path, "package.json");
        if let Ok(content) = fs::read_to_string(package_json_path) {
            content.contains("asc") || content.contains("@asc")
        } else {
            false
        }
    }

    fn find_entry_file(&self, project_path: &str) -> PluginResult<PathBuf> {
        let candidates = [
            "assembly/index.ts",
            "assembly/main.ts",
            "src/index.ts",
            "src/main.ts",
            "index.ts",
            "main.ts",
        ];

        for name in candidates.iter() {
            let path = Path::new(project_path).join(name);
            if path.exists() {
                return Ok(path);
            }
        }

        for dir in &["assembly", "src", "."] {
            let search_path = if *dir == "." {
                project_path.to_string()
            } else {
                PathResolver::join_paths(project_path, dir)
            };

            if let Ok(entries) = fs::read_dir(&search_path) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "ts" {
                            return Ok(entry.path());
                        }
                    }
                }
            }
        }

        Err(PluginError::MissingEntryFile {
            candidates: vec![
                "assembly/index.ts".to_string(),
                "assembly/main.ts".to_string(),
                "index.ts".to_string(),
            ],
        })
    }

    fn build_with_asc(&self, config: &BuildConfig) -> PluginResult<BuildResult> {
        let entry_path = self.find_entry_file(&config.project_path)?;
        PathResolver::ensure_output_directory(&config.output_dir)?;

        let output_name = entry_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let wasm_file = Path::new(&config.output_dir).join(format!("{output_name}.wasm"));

        println!("🔨 Building with AssemblyScript compiler...");

        let mut args = vec![
            entry_path.to_str().unwrap(),
            "--target",
            "release",
            "--outFile",
            wasm_file.to_str().unwrap(),
        ];

        match config.optimization_level {
            OptimizationLevel::Debug => args.extend(&["--debug"]),
            OptimizationLevel::Release => args.extend(&["--optimize"]),
            OptimizationLevel::Size => args.extend(&["--optimize", "--shrinkLevel", "2"]),
        }

        let output =
            CommandExecutor::execute_command("asc", &args, &config.project_path, config.verbose)?;

        if !output.status.success() {
            return Err(PluginError::CompilationFailed {
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        if !wasm_file.exists() {
            return Err(PluginError::CompilationFailed {
                reason: "WASM file was not created".to_string(),
            });
        }

        Ok(BuildResult {
            wasm_path: wasm_file.to_string_lossy().to_string(),
            js_path: None,
            additional_files: vec![],
            is_wasm_bindgen: false,
        })
    }

    fn build_with_npm(&self, config: &BuildConfig) -> PluginResult<BuildResult> {
        let package_json = PathResolver::join_paths(&config.project_path, "package.json");
        if !Path::new(&package_json).exists() {
            return Err(PluginError::CompilationFailed {
                reason: "No package.json found".to_string(),
            });
        }

        let project_path = Path::new(&config.project_path);
        let cmd = if CommandExecutor::is_tool_installed("bun")
            && project_path.join("bun.lockb").exists()
        {
            "bun"
        } else if CommandExecutor::is_tool_installed("pnpm")
            && project_path.join("pnpm-lock.yaml").exists()
        {
            "pnpm"
        } else if CommandExecutor::is_tool_installed("yarn")
            && project_path.join("yarn.lock").exists()
        {
            "yarn"
        } else if CommandExecutor::is_tool_installed("npm") {
            "npm"
        } else {
            return Err(PluginError::CompileToolNotFound {
                tool: "npm, pnpm, yarn, or bun".to_string(),
            });
        };

        println!("🔨 Building with {cmd}...");
        let args = match cmd {
            "yarn" => vec!["build"],
            "bun" => vec!["run", "build"],
            _ => vec!["run", "build"],
        };

        let output =
            CommandExecutor::execute_command(cmd, &args, &config.project_path, config.verbose)?;

        if !output.status.success() {
            return Err(PluginError::CompilationFailed {
                reason: format!(
                    "{} build failed: {}",
                    cmd,
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let search_dirs = ["build", "dist", "out", "target", "."];
        let mut wasm_files = Vec::new();

        for dir in search_dirs {
            let search_path = if dir == "." {
                config.project_path.clone()
            } else {
                PathResolver::join_paths(&config.project_path, dir)
            };

            if let Ok(files) = PathResolver::find_files_with_extension(&search_path, "wasm") {
                wasm_files.extend(files);
            }
        }

        if wasm_files.is_empty() {
            return Err(PluginError::CompilationFailed {
                reason: "No WASM file found after build".to_string(),
            });
        }

        let output_path =
            CommandExecutor::copy_to_output(&wasm_files[0], &config.output_dir, "AssemblyScript")?;

        Ok(BuildResult {
            wasm_path: output_path,
            js_path: None,
            additional_files: vec![],
            is_wasm_bindgen: false,
        })
    }
}

impl Plugin for AscPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn can_handle_project(&self, project_path: &str) -> bool {
        if self.is_asc_project(project_path) {
            return true;
        }

        let assembly_files = ["assembly/index.ts", "assembly/main.ts"];
        for file in assembly_files {
            if Path::new(&PathResolver::join_paths(project_path, file)).exists() {
                return true;
            }
        }

        false
    }

    fn get_builder(&self) -> Box<dyn WasmBuilder> {
        Box::new(AscPlugin::new())
    }
}

impl WasmBuilder for AscPlugin {
    fn supported_extensions(&self) -> &[&str] {
        &["ts"]
    }

    fn entry_file_candidates(&self) -> &[&str] {
        &[
            "assembly/index.ts",
            "assembly/main.ts",
            "package.json",
            "assembly/package.json",
        ]
    }

    fn language_name(&self) -> &str {
        "AssemblyScript"
    }

    fn check_dependencies(&self) -> Vec<String> {
        let mut missing = Vec::new();

        if !CommandExecutor::is_tool_installed("asc") {
            missing.push(
                "asc (AssemblyScript compiler - install with: npm install -g asc)".to_string(),
            );
        }

        if !CommandExecutor::is_tool_installed("node") {
            missing.push("node (Node.js runtime)".to_string());
        }

        missing
    }

    fn validate_project(&self, project_path: &str) -> PluginResult<()> {
        PathResolver::validate_directory_exists(project_path)?;
        let _ = self.find_entry_file(project_path)?;
        Ok(())
    }

    fn build(&self, config: &BuildConfig) -> PluginResult<BuildResult> {
        if Path::new(&config.project_path)
            .join("package.json")
            .exists()
        {
            match self.build_with_npm(config) {
                Ok(result) => Ok(result),
                Err(_) => {
                    if CommandExecutor::is_tool_installed("asc") {
                        self.build_with_asc(config)
                    } else {
                        Err(PluginError::CompileToolNotFound {
                            tool: "asc or npm/yarn".to_string(),
                        })
                    }
                }
            }
        } else if CommandExecutor::is_tool_installed("asc") {
            self.build_with_asc(config)
        } else {
            Err(PluginError::CompileToolNotFound {
                tool: "asc".to_string(),
            })
        }
    }

    fn can_handle_project(&self, project_path: &str) -> bool {
        if let Ok(entries) = fs::read_dir(project_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if self
                        .supported_extensions()
                        .contains(&ext.to_string_lossy().as_ref())
                    {
                        return true;
                    }
                }
            }
        }

        for candidate in self.entry_file_candidates() {
            if Path::new(project_path).join(candidate).exists() {
                return true;
            }
        }

        false
    }

    fn clean(&self, project_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = Path::new(project_path).join("build");
        if build_dir.exists() {
            fs::remove_dir_all(build_dir)?;
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn WasmBuilder> {
        Box::new(self.clone())
    }
}

impl Default for AscPlugin {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(AscPlugin::new())
}

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct BuildConfigC {
    pub project_path: *const c_char,
    pub output_dir: *const c_char,
    pub optimization_level: u8,
    pub verbose: bool,
    pub watch: bool,
}

#[repr(C)]
pub struct BuildResultC {
    pub wasm_path: *mut c_char,
    pub js_path: *mut c_char,
    pub is_wasm_bindgen: bool,
    pub success: bool,
    pub error_message: *mut c_char,
}

#[no_mangle]
pub extern "C" fn wasmasc_plugin_create() -> *mut c_void {
    let plugin = Box::new(AscPlugin::new());
    Box::into_raw(plugin) as *mut c_void
}

#[no_mangle]
pub extern "C" fn create_wasm_builder() -> *mut c_void {
    let builder = Box::new(AscPlugin::new());
    Box::into_raw(builder) as *mut c_void
}

#[no_mangle]
/// # Safety
///
/// This function takes raw pointers as arguments and dereferences them.
/// Callers must ensure that:
/// - `builder_ptr` is a valid pointer to an `AscPlugin` instance (or null)
/// - `project_path` is a valid null-terminated C string (or null)
///
/// If either pointer is null, the function returns `false`.
pub unsafe extern "C" fn wasmasc_can_handle_project(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const AscPlugin);
    let path = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    WasmBuilder::can_handle_project(builder, path)
}

#[no_mangle]
/// # Safety
///
/// This function takes raw pointers as arguments and dereferences them.
/// Callers must ensure that:
/// - `builder_ptr` is a valid pointer to an `AscPlugin` instance (or null)
/// - `config` is a valid pointer to a `BuildConfigC` struct (or null)
/// - All C string pointers in `config` are valid null-terminated strings
///
/// Returns a pointer to an owned `BuildResultC` that must be freed by the caller.
/// If either pointer is null, returns null.
pub unsafe extern "C" fn wasmasc_build(
    builder_ptr: *const c_void,
    config: *const BuildConfigC,
) -> *mut BuildResultC {
    if builder_ptr.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const AscPlugin);
    let config_c = &*config;

    let project_path = match CStr::from_ptr(config_c.project_path).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let output_dir = match CStr::from_ptr(config_c.output_dir).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let opt_level = match config_c.optimization_level {
        0 => OptimizationLevel::Debug,
        1 => OptimizationLevel::Release,
        2 => OptimizationLevel::Size,
        _ => OptimizationLevel::Release,
    };

    let build_cfg = BuildConfig {
        project_path,
        output_dir,
        optimization_level: opt_level,
        verbose: config_c.verbose,
        watch: config_c.watch,
    };

    match builder.build(&build_cfg) {
        Ok(result) => {
            let wasm_path = CString::new(result.wasm_path).unwrap_or_default();
            let js_path = result
                .js_path
                .and_then(|p| CString::new(p).ok())
                .map(|s| s.into_raw())
                .unwrap_or(ptr::null_mut());

            let result_c = Box::new(BuildResultC {
                wasm_path: wasm_path.into_raw(),
                js_path,
                is_wasm_bindgen: result.is_wasm_bindgen,
                success: true,
                error_message: ptr::null_mut(),
            });

            Box::into_raw(result_c)
        }
        Err(e) => {
            let error_msg = CString::new(format!("{e}")).unwrap_or_default();
            let result_c = Box::new(BuildResultC {
                wasm_path: ptr::null_mut(),
                js_path: ptr::null_mut(),
                is_wasm_bindgen: false,
                success: false,
                error_message: error_msg.into_raw(),
            });

            Box::into_raw(result_c)
        }
    }
}

#[no_mangle]
/// # Safety
///
/// This function takes raw pointers as arguments and dereferences them.
/// Callers must ensure that:
/// - `builder_ptr` is a valid pointer to an `AscPlugin` instance (or null)
/// - `project_path` is a valid null-terminated C string (or null)
///
/// If either pointer is null, the function returns `false`.
pub unsafe extern "C" fn wasmasc_clean(
    builder_ptr: *const c_void,
    project_path: *const c_char,
) -> bool {
    if builder_ptr.is_null() || project_path.is_null() {
        return false;
    }

    let builder = &*(builder_ptr as *const AscPlugin);
    let path = match CStr::from_ptr(project_path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    builder.clean(path).is_ok()
}

#[no_mangle]
/// # Safety
///
/// This function takes a raw pointer and dereferences it.
/// Callers must ensure that:
/// - `builder_ptr` is a valid pointer to an `AscPlugin` instance (or null)
///
/// Returns a pointer to a cloned `AscPlugin` that must be freed by the caller.
/// If the pointer is null, returns null.
pub unsafe extern "C" fn wasmasc_clone_box(builder_ptr: *const c_void) -> *mut c_void {
    if builder_ptr.is_null() {
        return ptr::null_mut();
    }

    let builder = &*(builder_ptr as *const AscPlugin);
    let cloned = builder.clone_box();
    Box::into_raw(cloned) as *mut c_void
}

#[no_mangle]
/// # Safety
///
/// This function takes a raw mutable pointer and deallocates it.
/// Callers must ensure that:
/// - `builder_ptr` is a pointer previously returned by `wasmasc_plugin_create` or `wasmasc_clone_box`
/// - The pointer is not used after this call
/// - The pointer is not freed again (double-free is undefined behavior)
///
/// If the pointer is null, this function does nothing (safe).
pub unsafe extern "C" fn wasmasc_drop(builder_ptr: *mut c_void) {
    if !builder_ptr.is_null() {
        let _ = Box::from_raw(builder_ptr as *mut AscPlugin);
    }
}

#[no_mangle]
pub static WASMASC_PLUGIN_NAME: &[u8] = b"asc\0";

#[no_mangle]
pub static WASMASC_PLUGIN_VERSION: &[u8] = env!("CARGO_PKG_VERSION").as_bytes();

#[no_mangle]
pub static WASMASC_PLUGIN_DESCRIPTION: &[u8] = b"AssemblyScript WebAssembly compiler plugin\0";

#[no_mangle]
pub static WASMASC_PLUGIN_AUTHOR: &[u8] = b"Wasmrun Team\0";

#[no_mangle]
pub static WASMASC_SUPPORTS_WASM: bool = true;

#[no_mangle]
pub static WASMASC_SUPPORTS_WEBAPP: bool = false;

#[no_mangle]
pub static WASMASC_SUPPORTS_LIVE_RELOAD: bool = true;

#[no_mangle]
pub static WASMASC_SUPPORTS_OPTIMIZATION: bool = true;
