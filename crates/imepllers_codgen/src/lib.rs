use anyhow::Context;

use tracing::{debug, error, trace};

pub fn generate_bindigns() -> anyhow::Result<()> {
    let impeller_header_src =
        load_impeller_header("impeller.h").context("failed to load impeller header")?;
    let impeller_api_json = std::fs::read_to_string("impeller_api.json")
        .context("failed to read impeller_api.json file")?;
    let impeller_api: serde_json::Value = serde_json::from_str(&impeller_api_json)
        .context("failed to parse impeller_api.json file")?;

    let raw_bindings =
        run_bindgen_and_return_rust_src(&impeller_header_src, ImpellerApiJson(impeller_api))
            .context("failed to run bindgen")?;
    let prefix = r"#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(rustdoc::invalid_codeblock_attributes)]
#![allow(rustdoc::invalid_rust_codeblocks)]
#![allow(rustdoc::broken_intra_doc_links)]
    ";
    let bindings = format!("{prefix}{raw_bindings}");
    // NOTE: windows wants i32 enums by default, as opposed to u32 used by most other platforms.
    // So, we have this hack to simply replace u32 repr with i32, with the assumption that we run this on non-windows platforms.
    let win_bindings = bindings.replace("repr(u32)", "repr(i32)");
    #[cfg(target_os = "windows")]
    panic!("this is only supposed to be run on non-windows platforms");
    std::fs::write("src/sys.rs", bindings).context("failed to write")?;
    std::fs::write("src/win_sys.rs", win_bindings).context("failed to write win sys")?;
    Ok(())
}

fn run_bindgen_and_return_rust_src(
    impeller_header_src: &str,
    impeller_api: impl bindgen::callbacks::ParseCallbacks + 'static,
) -> anyhow::Result<String> {
    let generator = bindgen::builder()
        .derive_default(true)
        // .dynamic_library_name("impeller")
        // .dynamic_link_require_all(true)
        .generate_cstr(true)
        .header_contents("impeller.h", impeller_header_src)
        .merge_extern_blocks(true)
        .prepend_enum_name(false)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(impeller_api))
        .generate()?;

    Ok(generator.to_string())
}

fn load_impeller_header(path: &str) -> anyhow::Result<String> {
    tracing::debug!(
        "current directory: {}",
        std::env::current_dir()
            .context("failed to get current directory before reading impeller header")?
            .display()
    );
    tracing::debug!("loading impeller header from {}", path);
    std::fs::read_to_string(path).context("failed to read from impeller.h file")
}
#[derive(Debug)]
struct ImpellerApiJson(serde_json::Value);
impl ImpellerApiJson {
    fn has_enum(&self, name: &str) -> bool {
        self.0
            .as_object()
            .expect("failed to downcast impeller_api to object")
            .get("enums")
            .expect("failed to find enums key in impeller_api")
            .as_object()
            .expect("failed to downcast enums to object")
            .contains_key(name)
    }
}
impl bindgen::callbacks::ParseCallbacks for ImpellerApiJson {
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let Some(name) = enum_name else {
            error!("enum variant {} without enum name", original_variant_name);
            return None;
        };
        // bindgen seems to include "enum" keyword inside the enum name
        // Fix: https://github.com/rust-lang/rust-bindgen/issues/3113
        let Some(name) = name.strip_prefix("enum ") else {
            error!("failed to strip enum keyword from enum name {}", name);
            return None;
        };
        if !self.has_enum(name) {
            error!("enum {name} not found in list of impeller enums");
            return None;
        }
        let Some(variant_name) = original_variant_name
            .strip_prefix("k")
            .and_then(|s| s.strip_prefix(name))
        else {
            error!("enum variant {original_variant_name} of {name} has an invalid name after stripping k and enum name");
            return None;
        };
        debug!(
            "renaming enum variant {} to {}",
            original_variant_name, variant_name
        );
        // hack because the variants are numbers (100, 200, 300, etc.) and can't be identifiers
        if name == "ImpellerFontWeight" {
            match variant_name {
                "100" => return Some("Thin".to_string()),
                "200" => return Some("ExtraLight".to_string()),
                "300" => return Some("Light".to_string()),
                "400" => return Some("Regular".to_string()),
                "500" => return Some("Medium".to_string()),
                "600" => return Some("SemiBold".to_string()),
                "700" => return Some("Bold".to_string()),
                "800" => return Some("ExtraBold".to_string()),
                "900" => return Some("Black".to_string()),
                _ => {
                    error!(
                        "enum variant {} of {} has an invalid name after stripping k and enum name",
                        original_variant_name, name
                    );
                    return None;
                }
            }
        }
        Some(variant_name.to_string())
    }

    fn item_name(&self, original_item_name: &str) -> Option<String> {
        if original_item_name.ends_with("_") {
            trace!(
                "skipping renaming item {} as it ends with underscore",
                original_item_name
            );
            return None;
        }

        if self.has_enum(original_item_name) {
            let Some(new_name) = original_item_name.strip_prefix("Impeller") else {
                error!(
                    "failed to strip Impeller prefix from enum {}",
                    original_item_name
                );
                return None;
            };
            debug!("renaming enum {} to {}", original_item_name, new_name);
            return Some(new_name.to_string());
        }
        None
    }

    fn process_comment(&self, comment: &str) -> Option<String> {
        // comments with "```" cause doctest to fail, so we must replace them with "```ignore"
        let mut new_comment = String::new();
        new_comment.reserve(comment.len());
        // we need to replace the ``` with ```ignore
        // but only for the first of the pair (ignoring the second).
        let mut replace = true;
        for line in comment.lines() {
            // indented lines will be considered code blocks too, so lets remove them.
            let line = line.trim();
            // triple backticks are also considered codeblocks, so lets make them cpp blocks to make rust ignore them.
            if line.trim() == "```" {
                // got the first one of the pair
                if replace {
                    let line = line.replace("```", "```cpp\n");
                    new_comment.push_str(&line);
                    replace = false; // the next one will be the second of the pair
                    continue;
                }
                // found the second of the pair. just push new line like normal.
                replace = true;
            }
            new_comment.push_str(line);
            new_comment.push('\n');
        }
        if new_comment.is_empty() {
            None
        } else {
            Some(new_comment)
        }
    }
}
