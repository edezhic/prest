fn main() {
    #[cfg(windows)] {
        println!(r"cargo:warning=Expecting postgres client libraries in PATH or at C:\libpq");
        println!(r"cargo:rustc-link-search=C:\libpq");
    }
}