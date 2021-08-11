use std::env;

fn main() {
    // println!("cargo:rustc-env=VCPKGRS_DYNAMIC=1");
    // println!("cargo:rustc-env=VCPKG_DEFAULT_TRIPLET=x64-windows-static");

    if cfg!(windows) {
        let link_paths = env::var("OPENCV_LINK_PATHS").expect("OPENCV_LINK_PATHS environment variable not set");
        let link_libs = env::var("OPENCV_LINK_LIBS").expect("OPENCV_LINK_LIBS environment variable not set");
        println!("cargo:rustc-link-lib=static={}", link_libs);
        println!("cargo:rustc-link-search=native={}", link_paths);
    }
}