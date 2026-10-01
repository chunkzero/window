//! Window compiler WASM component.
//!
//! The rpp package in `plugin/` is TypeScript. It loads this component to compile the
//! TypeScript-authored project model into pack artifacts and optional Kotlin bindings.

use std::collections::BTreeMap;

use window_core::pipeline::{CompileInput, CompileOutput as CoreCompileOutput, OutputFile as CoreOutputFile};

#[cfg(target_family = "wasm")]
wit_bindgen::generate!({
    world: "window-compiler",
    path: "wit",
});

#[cfg(target_family = "wasm")]
struct Component;

/// Source file passed from the plugin to the compiler component.
#[derive(Clone, Debug)]
pub struct SourceFileInput {
    pub path: String,
    pub contents: Vec<u8>,
}

/// Pack and optional Kotlin artifacts produced by one compiler pass.
#[derive(Clone, Debug)]
pub struct PluginCompileOutput {
    /// Resource-pack artifacts and the typed in-memory manifest.
    pub core: CoreCompileOutput,
    /// Kotlin bindings generated from that exact manifest.
    pub kotlin_files: Vec<CoreOutputFile>,
}

/// Compile a TypeScript-authored Window project.
pub fn compile_project(
    namespace: String,
    project_json: String,
    files: Vec<SourceFileInput>,
    kotlin_package: Option<String>,
) -> Result<PluginCompileOutput, String> {
    let files = files.into_iter().map(|file| (file.path, file.contents)).collect::<BTreeMap<_, _>>();
    let input = CompileInput { namespace, files };
    let core = window_core::pipeline::compile_project_json(project_json.as_bytes(), &input)
        .map_err(|error| error.to_string())?;
    let kotlin_files = kotlin_package
        .map(|package_name| window_core::codegen::generate_kotlin(&core.manifest, &package_name))
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    Ok(PluginCompileOutput { core, kotlin_files })
}

#[cfg(target_family = "wasm")]
impl Guest for Component {
    fn compile(
        namespace: String,
        project_json: String,
        files: Vec<SourceFile>,
        kotlin_package: Option<String>,
    ) -> Result<CompileOutput, String> {
        let files =
            files.into_iter().map(|file| SourceFileInput { path: file.path, contents: file.contents }).collect();
        compile_project(namespace, project_json, files, kotlin_package).map(to_wit_compile_output)
    }
}

#[cfg(target_family = "wasm")]
fn to_wit_compile_output(output: PluginCompileOutput) -> CompileOutput {
    CompileOutput {
        files: output.core.files.into_iter().map(to_wit_output_file).collect(),
        kotlin_files: output.kotlin_files.into_iter().map(to_wit_output_file).collect(),
        warnings: output.core.warnings,
    }
}

#[cfg(target_family = "wasm")]
fn to_wit_output_file(file: CoreOutputFile) -> OutputFile {
    OutputFile { path: file.path, contents: file.contents }
}

#[cfg(target_family = "wasm")]
export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_kotlin_from_project() {
        let project = r##"{
          "windows": [
            {
              "name": "shop",
              "container": "generic_9x3",
              "children": [
                {
                  "type": "slot",
                  "name": "balance",
                  "x": 8,
                  "y": 24,
                  "width": 160,
                  "align": "center",
                  "color": "#ffffff"
                }
              ]
            }
          ]
        }"##;

        let output = compile_project(
            "window".into(),
            project.into(),
            Vec::new(),
            Some("dev.oglass.window.example.generated".into()),
        )
        .unwrap();
        assert!(output.kotlin_files.iter().any(|file| file.path == "WindowPack.kt"));
        assert!(output.kotlin_files.iter().any(|file| file.path == "WindowFonts.kt"));
        assert!(output.kotlin_files.iter().any(|file| file.path == "ShopView.kt"));
    }

    #[test]
    fn omits_kotlin_without_a_package() {
        let output = compile_project("window".into(), r#"{"windows":[]}"#.into(), Vec::new(), None).unwrap();
        assert!(output.kotlin_files.is_empty());
    }
}
