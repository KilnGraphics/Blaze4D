extern crate cmake;

#[cfg(feature = "docs-rs")]
fn main() {
}

#[cfg(not(feature = "docs-rs"))]
fn main() {
    let dst = cmake::Config::new("libvma").profile("Release").build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=VulkanMemoryAllocator");
}