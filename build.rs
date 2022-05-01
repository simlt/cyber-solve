use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

fn main() {
    // println!("cargo:rustc-env=VCPKGRS_DYNAMIC=1");
    // println!("cargo:rustc-env=VCPKG_DEFAULT_TRIPLET=x64-windows-static");

    if cfg!(windows) {
        // Setup dynamic OpenCV link (unused if using static opencv library)
        // let link_paths =
        //     env::var("OPENCV_LINK_PATHS").expect("OPENCV_LINK_PATHS environment variable not set");
        // let link_libs =
        //     env::var("OPENCV_LINK_LIBS").expect("OPENCV_LINK_LIBS environment variable not set");
        // println!("cargo:rustc-link-lib=static={}", link_libs);
        // println!("cargo:rustc-link-search=native={}", link_paths);

        // Add missing GetSaveFileNameA link used by opencv::highgui module
        println!("cargo:rustc-link-lib=dylib=comdlg32");
    }

    // MacOS
    if cfg!(macos) {
        println!("export DYLD_FALLBACK_LIBRARY_PATH=\"$(xcode-select --print-path)/usr/lib/\"");
    }

    if cfg!(linux) {
        println!("export CARGO_BUILD_TARGET=\"x86_64-pc-windows-gnu\"");
    }

    // Bundle assets
    // let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = get_output_path();
    let src = Path::join(&env::current_dir().unwrap(), "assets");
    let dst = Path::join(Path::new(&target_dir), Path::new("assets"));
    println!(
        "cargo:warning=Copying assets from {} to {}",
        src.to_str().unwrap(),
        dst.to_str().unwrap()
    );
    copy_dir_all(src, dst).expect("Failed to copy assets to build folder");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets");


    // Embed resource file for icon
    embed_resource::compile("assets/manifest.rc");
}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type);
    return PathBuf::from(path);
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
