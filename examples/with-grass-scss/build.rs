fn main() {
    let contents = grass::from_path("./styles.scss", &Default::default()).unwrap();
    std::fs::write(prest_build::out_path("styles.css"), contents).unwrap();
}