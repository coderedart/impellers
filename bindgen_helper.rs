use anyhow::Context;

use tracing::{debug, error, trace};

const IMPELLER_HEADER_SRC: &str = include_str!("impeller.h");
const IMPELLER_API_JSON_STR: &str = include_str!("impeller_api.json");

pub fn generate_bindings(platform: Option<&str>) -> anyhow::Result<String> {
    let impeller_api: serde_json::Value = serde_json::from_str(&IMPELLER_API_JSON_STR)
        .context("failed to parse impeller_api.json file")?;

    let platform = platform.map(|p| format!("--target={p}"));
    let clang_args = platform.as_ref().map(|s| s.as_str());

    let raw_bindings = run_bindgen_and_return_rust_src(
        &IMPELLER_HEADER_SRC,
        ImpellerApiJson(impeller_api.clone()),
        clang_args.as_slice(),
    )
    .context("failed to run bindgen")?;

    Ok(raw_bindings)
}

fn run_bindgen_and_return_rust_src(
    impeller_header_src: &str,
    impeller_api: impl bindgen::callbacks::ParseCallbacks + 'static,
    clang_args: &[&str],
) -> anyhow::Result<String> {
    let generator = bindgen::builder()
        .derive_default(true)
        // .dynamic_library_name("impeller")
        // .dynamic_link_require_all(true)
        .generate_cstr(true)
        .header_contents("impeller.h", impeller_header_src)
        .merge_extern_blocks(true)
        .prepend_enum_name(false)
        .allowlist_item("k*Impeller.*") // filter out distracting compiler internal constants/types.
        .allowlist_item("IMPELLER.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .parse_callbacks(Box::new(impeller_api))
        .clang_args(clang_args)
        .generate()?;

    Ok(generator.to_string())
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

    fn item_name(&self, item_info: bindgen::callbacks::ItemInfo) -> Option<String> {
        let original_item_name = item_info.name;
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
}
