fn main() {
    println!("cargo::rerun-if-changed=src/lib/format/templates/");
    // Declaring any `rerun-if-changed` opts out of the default of watching the
    // whole crate, so the template config has to be named too or a change to
    // the escaper would not rebuild the templates that use it.
    println!("cargo::rerun-if-changed=askama.toml");
}
