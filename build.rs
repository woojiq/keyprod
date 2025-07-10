use std::io::Write;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_var("KEY_.*")
        .generate()
        .expect("Unable to generate bindings");

    let keyval = gen_bindings_keyval(&bindings.to_string());

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("input_event_codes_bindings.rs");
    let mut out_file =
        std::fs::File::create(out_path).expect("Couldn't create file to write bindings.");

    bindings
        .write(Box::new(&out_file))
        .expect("Couldn't write bindings!");

    write!(out_file, "{keyval}").expect("Failed to write HashMap string to source file.");
}

// TODO: extract to separate crate and add unit tests
fn gen_bindings_keyval(bindings: &str) -> String {
    let vars = split_variables(bindings);
    let max_key = vars.iter().map(|e| e.0).max().unwrap();

    let mut res = String::new();

    res += "\n";

    res += &format!(
        "pub const KEY_TO_STR_REPR: [&str; {}] = gen_keyval();\n",
        max_key + 1
    );

    res += "\n";

    res += &format!(
        "const fn gen_keyval() -> [&'static str; {}] {{
\tlet mut a = [\"\"; {}];\n",
        max_key + 1,
        max_key + 1
    );

    res += "\n";

    for (key, val) in &vars {
        res += &format!("\ta[{key}] = \"{val}\";\n");
    }

    res += "\n";

    res += "\
\ta
}\n";

    res
}

fn split_variables(bindings: &str) -> Vec<(usize, String)> {
    bindings
        .lines()
        .filter(|s| is_key_variable(s))
        .map(keyval_from_binding)
        .collect()
}

fn is_key_variable(line: &str) -> bool {
    line.starts_with("pub const KEY_")
}

fn keyval_from_binding(binding: &str) -> (usize, String) {
    (key_from_binding(binding), val_from_binding(binding))
}

// Example: pub const KEY_CAMERA_UP: u32 = 535;
// Result: 535
fn key_from_binding(binding: &str) -> usize {
    binding
        .split("= ")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

// Example: pub const KEY_CAMERA_UP: u32 = 535;
// Result: CAMERA_UP
fn val_from_binding(line: &str) -> String {
    line.split(':')
        .next()
        .unwrap()
        .split_whitespace()
        .nth(2)
        .unwrap()
        .split_once('_')
        .unwrap()
        .1
        .to_string()
}
