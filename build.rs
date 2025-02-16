use std::path::{Path, PathBuf};
/*
#[cfg(not(feature = "prebuilt_libs"))]
fn main() {}

#[cfg(feature = "prebuilt_libs")]*/
fn main() {
    const STATIC_MAJOR: u32 = 0;
    const STATIC_MINOR: u32 = 32;
    const STATIC_PATCH: u32 = 0;
    const ENGINE_SHA: &str = "c7047d33c4d23b8b97b183cce90d319521601f7e";
    // gather variables
    let out_dir = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_owned();
    let profile = cfg!(feature = "debug_static_link")
        .then_some("debug")
        .unwrap_or("release");
    let target_os = match std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "windows",
        "macos" => "darwin",
        "linux" => "linux",
        "android" => "android",
        rest => panic!("unsupported target OS: {rest}"),
    };
    let target_arch = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "x86" => "x86",
        "arm" => "arm",
        rest => panic!("unsupported target architecture: {rest}"),
    };
    let static_link = cfg!(feature = "static_link");
    let required_libs: &[&'static str] = if static_link {
        match target_os {
            "windows" => &["impeller.lib"],
            _ => &["libimpeller.a"],
        }
    } else {
        match target_os {
            "windows" => &["impeller.dll", "impeller.dll.lib"],
            "macos" => &["libimpeller.dylib"],
            _ => &["libimpeller.so"],
        }
    };
    let build_name = if static_link {
        format!("{target_os}_{target_arch}_static_{profile}",)
    } else {
        format!("{target_os}_{target_arch}",)
    };
    let cache_dir = if cfg!(feature = "cache_binaries") {
        get_zip_cache_dir(&out_dir, &build_name)
    } else {
        out_dir.clone()
    };

    // print cargo directives
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    if static_link {
        if target_os == "windows" {
            // do we really need all of these?
            for lib in [
                "advapi32", "Rpcrt4", "Shlwapi", "user32", "Gdi32", "Shell32", "Winmm", "msvcrt",
            ] {
                println!("cargo:rustc-link-lib={lib}");
            }
        }
        println!("cargo:rustc-link-lib=static=impeller");
    } else {
        // on windows, you link with impeller.dll.lib
        println!(
            "cargo:rustc-link-lib=dylib=impeller{}",
            (target_os == "windows")
                .then_some(".dll")
                .unwrap_or_default()
        );
    }

    // start doing the actual work
    println!("we are looking for {required_libs:?}");
    if required_libs.iter().all(|l| out_dir.join(l).exists()) {
        println!("libs already exist in out dir, so we don't need to do anything");
        return;
    }
    println!(
        "libs don't exist in out dir yet, so we check if the archive is already downloaded or not"
    );

    let url = if static_link {
        format!(
            "https://github.com/coderedart/impellers/releases/download/a_{}.{}.{}/{build_name}.zip",
            STATIC_MAJOR, STATIC_MINOR, STATIC_PATCH
        )
    } else {
        format!("https://storage.googleapis.com/flutter_infra_release/flutter/{ENGINE_SHA}/{target_os}-{target_arch}/impeller_sdk.zip")
    };
    download_if_not_exists(&url, &cache_dir);
    extract_if_libs_dir_not_exists(&cache_dir);

    let extracted_libs_dir = cache_dir.join("lib");
    for require_lib in required_libs {
        println!(
            "copying {}/{require_lib} to out_dir",
            extracted_libs_dir.display()
        );
        std::fs::copy(
            extracted_libs_dir.join(require_lib),
            out_dir.join(require_lib),
        )
        .expect("failed to copy impeller library to out dir");
    }
    for entry in std::fs::read_dir(out_dir).expect("failed to read entries of out_dir") {
        let entry = entry.expect("failed to get entry of out_dir");
        let md = entry.metadata().expect("failed to get metadata of entry");
        println!(
            "found {} with size {} MB",
            entry.file_name().to_str().unwrap_or_default(),
            md.len() / (1024 * 1024)
        )
    }
    println!("done");
}
fn extract_if_libs_dir_not_exists(cache_dir: &Path) {
    if cache_dir.join("lib").exists() {
        println!("skipping extraction. found extracted impeller library in {cache_dir:?}");
    } else {
        println!(
            "there's no extracted impeller library in {cache_dir:?}, so we extract it from archive"
        );
        let mut command = if cfg!(unix) {
            std::process::Command::new("unzip")
        } else {
            let mut command = std::process::Command::new("tar");
            command.arg("-xvf");
            command
        };
        let tar_status = command
            .arg(LOCAL_IMPELLER_ARCHIVE_NAME)
            .current_dir(&cache_dir)
            .status();
        assert!(
            tar_status
                .expect("failed to run tar/unzip command")
                .success(),
            "tar failed to extract {LOCAL_IMPELLER_ARCHIVE_NAME} and store it in {cache_dir:?}"
        );
        println!(
                "extracted impeller library from {LOCAL_IMPELLER_ARCHIVE_NAME} and stored it in {cache_dir:?}"
            );
    }
}
fn download_if_not_exists(url: &str, cache_dir: &Path) {
    if cache_dir.join(LOCAL_IMPELLER_ARCHIVE_NAME).exists() {
        println!("skipping download. found cached impeller library in {cache_dir:?}");
    } else {
        let curl_status = std::process::Command::new("curl")
            .current_dir(&cache_dir)
            .args([
                "--progress-bar",
                "--fail",
                "-L",
                &url,
                "-o",
                LOCAL_IMPELLER_ARCHIVE_NAME,
            ])
            .status();
        assert!(
            curl_status.expect("failed to run curl command").success(),
            "curl failed to download {url} and store it in {cache_dir:?}"
        );
        println!("downloaded impeller library from {url} and stored it in {cache_dir:?}");
    }
}
const LOCAL_IMPELLER_ARCHIVE_NAME: &str = "impeller_sdk.zip";

fn get_zip_cache_dir(out_dir: &PathBuf, build_name: &str) -> PathBuf {
    let impeller_cache_dir = get_cache_directory(out_dir);
    let zip_cache_dir = impeller_cache_dir.join(build_name);
    std::fs::create_dir_all(&zip_cache_dir).expect("failed to create zip cache dir");
    zip_cache_dir
}
/// This will get/create cache directory. If the current version doesn't match the version stored in
/// directory, it will remove the directory and create a new one to discard all the old artefacts.
fn get_cache_directory(out_dir: &PathBuf) -> PathBuf {
    // assuming cargo's target directory is located in the current directory.
    let impeller_cache_dir = out_dir // ./target/release/build/impeller_a2142341/out
        .parent()
        .expect("failed to get out_dir parent") // ./target/release/build/impeller_a2142341
        .parent()
        .expect("failed to get out_dir's grandparent") // ./target/release/build
        .parent()
        .expect("failed to get profile directory") // ./target/release
        .parent()
        .expect("failed to get target directory") // ./target
        .parent()
        .expect("failed to get target directory's parent") // ./
        .join(".impeller_cache"); // ./.impeller_cache
    let impeller_cache_version_path = impeller_cache_dir.join("version.txt");
    let impeller_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let version_in_file = std::fs::read_to_string(&impeller_cache_version_path).unwrap_or_default();
    if version_in_file != impeller_version {
        println!("cargo:warning=impeller cache directory is out of date {impeller_version} vs {version_in_file}");
        std::fs::remove_dir_all(&impeller_cache_dir).expect("failed to remove impeller cache dir");
        std::fs::create_dir_all(&impeller_cache_dir).expect("failed to create impeller cache dir");
        std::fs::write(&impeller_cache_version_path, impeller_version).unwrap();
    }
    return impeller_cache_dir;
}
