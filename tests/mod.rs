use std::{fs::File, io::Write, process::Command};

fn create_file(path: &str) {
    File::create(path).unwrap();
}

fn create_test_dir(path: &str) {
    std::fs::create_dir_all(path).expect("failed to create dir");
}

fn delete_test_dir(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}

fn cleanup_test_dir(path: &str) {
    delete_test_dir(path);
    create_test_dir(path);
}

#[test]
fn warning_message_on_init_output() {
    cleanup_test_dir("tests/.tmp1");
    create_file("tests/.tmp1/cluster.genin.yml");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("init")
    .arg("-q")
    .current_dir("tests/.tmp1")
    .output()
    .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    assert_eq!(
        result,
        b"WARN: the target file cluster.genin.yml already exists so the new file will \
            be saved with name cluster.genin.copy.yml\n"
    );
}

#[test]
fn warning_message_on_build_output() {
    cleanup_test_dir("tests/.tmp2");
    create_file("tests/.tmp2/inventory.yml");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("init")
    .arg("-q")
    .current_dir("tests/.tmp2")
    .output()
    .expect("Failed to execute command");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("--source")
    .arg("cluster.genin.yml")
    .arg("-q")
    .current_dir("tests/.tmp2")
    .output()
    .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    println!("{}", String::from_utf8(result.clone()).unwrap());

    assert_eq!(
        result,
        b"WARN: the target file inventory.yml already exists so the new file will be saved with name inventory.copy.yml\n"
    );
}
