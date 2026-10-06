use super::ValidationReport;
use crate::pipeline::CompileOutput;

/// Checks that the descriptor exists and parses; with `rebuild`, also that it matches the descriptor rebuilt from
/// the manifest and generated files.
pub(super) fn validate_debug_descriptor(output: &CompileOutput, rebuild: bool, report: &mut ValidationReport) {
    let path = crate::debug::output_path(&output.manifest.namespace);
    let Some(emitted) = output.files.iter().find(|file| file.path == path) else {
        report.push("debug.descriptor.missing", path, "compiler output did not emit the active-pack debug descriptor");
        return;
    };

    if let Err(error) = crate::debug::DebugDescriptor::from_json(emitted.contents.as_bytes()) {
        report.push("debug.descriptor.invalid", &emitted.path, error.to_string());
    }
    if !rebuild {
        return;
    }

    let expected = match crate::debug::DebugDescriptor::build(&output.manifest, &output.files)
        .and_then(|descriptor| descriptor.to_json())
    {
        Ok(expected) => expected,
        Err(error) => {
            report.push(
                "debug.descriptor.rebuild_failed",
                &emitted.path,
                format!("could not rebuild compiler expectation: {error}"),
            );
            return;
        }
    };
    if emitted.contents.as_bytes() != expected.as_bytes() {
        report.push(
            "debug.descriptor.mismatch",
            &emitted.path,
            format!(
                "emitted descriptor is not the canonical {}-byte descriptor rebuilt from the manifest and generated assets",
                expected.len()
            ),
        );
    }
}
