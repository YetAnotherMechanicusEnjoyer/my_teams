fn main() {
    println!("cargo:rustc-link-search=native=./libs/myteams/");
    println!("cargo:rustc-link-lib=dylib=myteams");
    println!("cargo:rerun-if-changed=libs/myteams/libmyteams.so");
}
