use super::{track_non_rust_path, out_path, bench};

pub fn bundle_sass(input: &str, filename: &str) {
    let start = std::time::Instant::now();
    track_non_rust_path(input); // track imports?
    let contents = grass::from_path(input, &Default::default()).unwrap();
    std::fs::write(out_path(filename), contents).unwrap();
    bench(&format!("{input} transpiled and bundled as {filename}"), start);
}