#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(rustdoc::invalid_codeblock_attributes)]
#![allow(rustdoc::invalid_rust_codeblocks)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(missing_docs)]
// ^^^ To suppress linting, we prefix all bindings with these allow attributes.

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
