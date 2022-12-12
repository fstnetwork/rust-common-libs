use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindings.h");

    #[cfg(feature = "bindgen")]
    generate_bindings();

    if cfg!(feature = "dynamic-linking") {
        println!("cargo:rustc-link-lib=pulsar");
    } else {
        build_pulsar();
    }
}

#[cfg(feature = "bindgen")]
fn generate_bindings() {
    let cargo_manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("cargo must provide `CARGO_MANIFEST_DIR`; qed"),
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo must provide `OUT_DIR`; qed"));

    let bindings = bindgen::builder()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("pulsar.*")
        .allowlist_type("_?pulsar.*|initial_position|token_supplier")
        .allowlist_var("pulsar.*|PULSAR_.*")
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .clang_arg(format!(
            "-I{}",
            cargo_manifest_dir.join("pulsar-client-cpp").join("include").display()
        ))
        .header("bindings.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(out_dir.join("bindings.rs")).expect("Couldn't write bindings!");
}

fn build_pulsar() {
    let mut config = cmake::Config::new("pulsar-client-cpp");

    config
        .define("BUILD_DYNAMIC_LIB", "OFF")
        .define("BUILD_STATIC_LIB", "ON")
        .define("BUILD_TESTS", "OFF")
        .define("BUILD_WIRESHARK", "OFF")
        .define("BUILD_PERF_TOOLS", "OFF")
        .define("USE_LOG4CXX", "OFF");

    eprintln!("Configuring and compiling pulsar-client-cpp");
    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=crypto");
    println!("cargo:rustc-link-lib=dylib=curl");
    println!("cargo:rustc-link-lib=dylib=protobuf");
    println!("cargo:rustc-link-lib=dylib=ssl");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=zstd");

    println!("cargo:rustc-link-lib=static=pulsar");

    link_cxx();
}

fn link_cxx() {
    if let Ok(cxx) = env::var("PULSAR_CLIENT_SYS_CXX") {
        if !cxx.is_empty() {
            println!("cargo:rustc-link-lib={cxx}");
        }
    } else {
        let target = std::env::var("TARGET").unwrap();
        if target.contains("apple") || target.contains("bsd") {
            println!("cargo:rustc-link-lib=dylib=c++");
        } else if target.contains("musl") {
            println!("cargo:rustc-link-lib=static=stdc++");
        } else {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }
}
