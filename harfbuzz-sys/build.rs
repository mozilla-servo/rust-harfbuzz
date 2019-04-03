#[cfg(feature = "build-native-harfbuzz")]
extern crate cmake;
#[cfg(feature = "build-native-harfbuzz")]
extern crate pkg_config;

#[cfg(feature = "build-native-harfbuzz")]
fn main() {
    use std::env;
    use std::io::*;
    use std::process::Command;
    use std::path::PathBuf;

    let output = Command::new("ls").args(&["-l", "--full-time", "harfbuzz/src"]).output().expect("failed");
    stderr().write_all(&output.stdout).unwrap();

    println!("cargo:rerun-if-env-changed=HARFBUZZ_SYS_NO_PKG_CONFIG");

    let try_pkg_config = match &env::var_os("HARFBUZZ_SYS_NO_PKG_CONFIG") {
        Some(s) if s == "1" => false,
        Some(s) if s == "0" => true,
        None => true,
        Some(s) => panic!(
            "Expected $HARFBUZZ_SYS_NO_PKG_CONFIG to be 1 or 0. Got: {}",
            s.to_string_lossy()
        ),
    };

    if try_pkg_config {
        if let Ok(lib) = pkg_config::probe_library("harfbuzz") {
            // Avoid printing an empty value
            if !lib.include_paths.is_empty() {
                // DEP_HARFBUZZ_INCLUDE has the paths of harfbuzz and dependencies.
                println!(
                    "cargo:include={}",
                    env::join_paths(lib.include_paths)
                        .unwrap()
                        .to_str()
                        .unwrap()
                );
            }
            return;
        }
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // On Windows, HarfBuzz configures atomics directly; otherwise,
    // it needs assistance from configure to do so.  Just use the makefile
    // build for now elsewhere.
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        let dst = cmake::Config::new("harfbuzz").build();
        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=static=harfbuzz");
        if target.contains("gnu") {
            println!("cargo:rustc-link-lib=stdc++");
        }
    } else {
        assert!(
            Command::new("make")
                .env("MAKEFLAGS", env::var("CARGO_MAKEFLAGS").unwrap_or_default())
                .args(&["-R", "-f", "makefile.cargo"])
                .status()
                .unwrap()
                .success()
        );

        println!(
            "cargo:rustc-link-search=native={}",
            out_dir.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=harfbuzz");
    }

    // DEP_HARFBUZZ_INCLUDE has the path of the vendored harfbuzz.
    println!(
        "cargo:include={}",
        out_dir.join("include").join("harfbuzz").display()
    );

    let output = Command::new("ls").args(&["-l", "--full-time", &format!("{}", out_dir.join("include/harfbuzz").display())]).output().expect("failed");
    stderr().write_all(&output.stdout).unwrap();
}

#[cfg(not(feature = "build-native-harfbuzz"))]
fn main() {}
