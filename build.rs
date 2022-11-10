fn main() {
    println!("cargo:rerun-if-changed=spis-server/migrations");
}
