fn main() {
    // https://docs.rs/sqlx/latest/sqlx/macro.migrate.html
    println!("cargo:rerun-if-changed=migrations");
}
