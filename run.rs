fn main() {
    let host_target_executable_path = if cfg!(debug_assertions) {
        "./target/host/debug/host"
    } else {
        "./target/host/release/host"
    };
    std::process::Command::new(host_target_executable_path)
        .status()
        .unwrap();
}
