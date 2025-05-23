use std::{
    fs::{read_dir, read_to_string, File},
    io::Write,
    process::{Command, Output},
};

const INVENTORY_HEADER_SIZE: usize = 2;
const GENIN_CMD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/genin",);

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

pub fn read_inventory(path: &str) -> String {
    read_to_string(path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect::<Vec<String>>()[INVENTORY_HEADER_SIZE..]
        .join("\n")
}

#[allow(unused)]
pub fn build_result_from_output(output: Output) -> String {
    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    String::from_utf8(strip_ansi_escapes::strip(String::from_utf8(result).unwrap()).unwrap())
        .unwrap()
}

#[test]
fn genin_init() {
    cleanup_test_dir("tests/.genin_init");

    let output = Command::new(GENIN_CMD)
        .arg("init")
        .arg("--output")
        .arg("tests/.genin_init/cluster.genin.yml")
        .output()
        .expect("Failed to execute command");

    let genin_init = build_result_from_output(output);
    let genin_init = format!(
        "{genin_init}\n{}",
        read_to_string("tests/.genin_init/cluster.genin.yml").unwrap()
    );

    insta::assert_display_snapshot!("genin_init", genin_init);
}

#[test]
fn warning_message_on_init_output() {
    cleanup_test_dir("tests/.warning_message_on_init_output");
    create_file("tests/.warning_message_on_init_output/cluster.genin.yml");

    let output = Command::new(GENIN_CMD)
        .arg("init")
        .arg("-q")
        .current_dir("tests/.warning_message_on_init_output")
        .output()
        .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    println!("{}", String::from_utf8(result.clone()).unwrap());

    assert_eq!(
        result,
        b"WARN: the target file cluster.genin.yml already exists so the new file will \
            be saved with name cluster.genin.copy.yml\n"
    );
}

#[test]
fn genin_inspect() {
    cleanup_test_dir("tests/.genin_inspect");

    Command::new(GENIN_CMD)
        .arg("init")
        .arg("--output")
        .arg("tests/.genin_inspect/cluster.genin.yml")
        .output()
        .expect("Failed to execute command");

    let output = Command::new(GENIN_CMD)
        .arg("inspect")
        .arg("--source")
        .arg("tests/.genin_inspect/cluster.genin.yml")
        .output()
        .expect("Failed to execute command");

    let genin_inspect = build_result_from_output(output);

    insta::assert_display_snapshot!("genin_inspect", genin_inspect);
}

#[test]
fn warning_message_on_build_output() {
    cleanup_test_dir("tests/.warning_message_on_build_output");
    create_file("tests/.warning_message_on_build_output/inventory.yml");

    Command::new(GENIN_CMD)
        .arg("init")
        .arg("-q")
        .current_dir("tests/.warning_message_on_build_output")
        .output()
        .expect("Failed to execute command");

    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("--source")
        .arg("cluster.genin.yml")
        .arg("-q")
        .current_dir("tests/.warning_message_on_build_output")
        .output()
        .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    println!("{}", String::from_utf8(result.clone()).unwrap());

    assert_eq!(
        result,
        b"WARN: the target file inventory.yml already exists so the new file will be \
            saved with name inventory.copy.yml\n"
    );
}

#[test]
fn init_with_comments() {
    cleanup_test_dir("tests/.init_with_comments");

    Command::new(GENIN_CMD)
        .arg("init")
        .arg("-q")
        .current_dir("tests/.init_with_comments")
        .output()
        .expect("Failed to execute command");

    let generated = std::fs::read_to_string("tests/.init_with_comments/cluster.genin.yml").unwrap();

    insta::assert_display_snapshot!(generated)
}

#[test]
fn sequential_upgrade_from_state() {
    cleanup_test_dir("tests/.sequential_upgrade_from_state");

    let output = Command::new(GENIN_CMD)
        .arg("upgrade")
        .arg("--old")
        .arg("tests/resources/cluster.genin.yml")
        .arg("--new")
        .arg("tests/resources/cluster-new.genin.yml")
        .arg("--output")
        .arg("tests/.sequential_upgrade_from_state/v1_inventory.yml")
        .arg("--export-state")
        .arg("tests/.sequential_upgrade_from_state/v1_state.gz")
        .arg("--state-dir")
        .arg("tests/.sequential_upgrade_from_state/.geninstate")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let cluster_to_cluster_new = build_result_from_output(output);

    let cluster_to_cluster_new = format!(
        "{cluster_to_cluster_new}\n{}",
        read_inventory("tests/.sequential_upgrade_from_state/v1_inventory.yml")
    );

    insta::assert_display_snapshot!("cluster_to_cluster_new", cluster_to_cluster_new);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from previous saved state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(GENIN_CMD)
        .arg("upgrade")
        .arg("--old")
        .arg("tests/.sequential_upgrade_from_state/v1_state.gz")
        .arg("--new")
        .arg("tests/resources/cluster-new-v2.genin.yml")
        .arg("--output")
        .arg("tests/.sequential_upgrade_from_state/v2_inventory.yml")
        .arg("--state-dir")
        .arg("tests/.sequential_upgrade_from_state/.geninstate")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let cluster_new_to_cluster_v2 = build_result_from_output(output);

    let cluster_new_to_cluster_v2 = format!(
        "{cluster_new_to_cluster_v2}\n{}",
        read_inventory("tests/.sequential_upgrade_from_state/v2_inventory.yml")
    );

    insta::assert_display_snapshot!("cluster_new_to_cluster_v2", cluster_new_to_cluster_v2);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from latest state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(GENIN_CMD)
        .arg("upgrade")
        .arg("--from-latest-state")
        .arg("--new")
        .arg("tests/resources/cluster-new-v3.genin.yml")
        .arg("--output")
        .arg("tests/.sequential_upgrade_from_state/v3_inventory.yml")
        .arg("--state-dir")
        .arg("tests/.sequential_upgrade_from_state/.geninstate")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let cluster_v2_to_cluster_v3 = build_result_from_output(output);
    let cluster_v2_to_cluster_v3 = format!(
        "{cluster_v2_to_cluster_v3}\n{}",
        read_inventory("tests/.sequential_upgrade_from_state/v3_inventory.yml")
    );

    insta::assert_display_snapshot!("cluster_v2_to_cluster_v3", cluster_v2_to_cluster_v3);
}

#[test]
fn sequential_upgrade_with_decreasing() {
    cleanup_test_dir("tests/.sequential_upgrade_with_decreasing");

    let output = Command::new(GENIN_CMD)
        .arg("upgrade")
        .arg("--old")
        .arg("tests/resources/cluster-new-v3.genin.yml")
        .arg("--new")
        .arg("tests/resources/cluster-new-v4.genin.yml")
        .arg("--output")
        .arg("tests/.sequential_upgrade_with_decreasing/v1_inventory.yml")
        .arg("--state-dir")
        .arg("tests/.sequential_upgrade_with_decreasing/.geninstate")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let cluster_v3_to_cluster_v4 = build_result_from_output(output);

    let cluster_v3_to_cluster_v4 = format!(
        "{cluster_v3_to_cluster_v4}\n{}",
        read_inventory("tests/.sequential_upgrade_with_decreasing/v1_inventory.yml")
    );

    insta::assert_display_snapshot!("cluster_v3_to_cluster_v4", cluster_v3_to_cluster_v4);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from latest state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(GENIN_CMD)
        .arg("upgrade")
        .arg("--from-latest-state")
        .arg("--new")
        .arg("tests/resources/cluster-new-v5.genin.yml")
        .arg("--output")
        .arg("tests/.sequential_upgrade_with_decreasing/v2_inventory.yml")
        .arg("--state-dir")
        .arg("tests/.sequential_upgrade_with_decreasing/.geninstate")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let cluster_v4_to_cluster_v5 = build_result_from_output(output);

    let cluster_v4_to_cluster_v5 = format!(
        "{cluster_v4_to_cluster_v5}\n{}",
        read_inventory("tests/.sequential_upgrade_with_decreasing/v2_inventory.yml")
    );

    insta::assert_display_snapshot!("cluster_v4_to_cluster_v5", cluster_v4_to_cluster_v5);
}

#[test]
fn upgrade_consistency_100_times() {
    cleanup_test_dir("tests/.upgrade_consistency_100_times");

    for _ in 1..=100 {
        let output = Command::new(GENIN_CMD)
            .arg("upgrade")
            .arg("--old")
            .arg("tests/resources/cluster.genin.yml")
            .arg("--new")
            .arg("tests/resources/cluster-new.genin.yml")
            .arg("--output")
            .arg("tests/.upgrade_consistency_100_times/inventory.yml")
            .arg("-f")
            .arg("--state-dir")
            .arg("tests/.upgrade_consistency_100_times/.geninstate")
            .arg("-y")
            .output()
            .expect("Failed to execute command");

        let consistency_100_times = build_result_from_output(output);

        let consistency_100_times = format!(
            "{consistency_100_times}\n{}",
            read_inventory("tests/.upgrade_consistency_100_times/inventory.yml")
        );

        insta::assert_display_snapshot!("consistency_100_times", consistency_100_times);
    }
}

#[test]
fn build_invalid_config() {
    cleanup_test_dir("tests/.build_invalid_config");

    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg("tests/resources/cluster-invalid.genin.yml")
        .arg("-o")
        .arg("tests/.build_invalid_config/inventory.yml")
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let build_invalid_config = build_result_from_output(output);

    insta::assert_display_snapshot!(build_invalid_config);
}

#[test]
fn build_with_recreate() {
    let base_dir = "tests/.build_with_recreate";
    let source = "tests/resources/cluster.genin.yml";
    let state_dir = format!("{base_dir}/.geninstate");
    let inventory = format!("{base_dir}/inventory.yml");
    cleanup_test_dir(base_dir);

    // build from config
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&source)
        .arg("-o")
        .arg(&inventory)
        .arg("--state-dir")
        .arg(&state_dir)
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);

    let result = format!("{result}\n{}", read_to_string(&source).unwrap());
    insta::assert_display_snapshot!("cluster_genin", result);
    assert_eq!(read_dir(&state_dir).unwrap().count(), 2);

    // build with recreate state
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&source)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--recreate")
        .arg("-o")
        .arg(format!("{base_dir}/recreate_inventory.yml"))
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);
    let result = format!("{result}\n{}", read_to_string(&source).unwrap());
    insta::assert_display_snapshot!("cluster_genin", result);
    assert_eq!(read_dir(&state_dir).unwrap().count(), 2);
}

#[test]
fn build_with_upgrade() {
    let src = "tests/resources/cluster.genin.yml";
    let upg_src = "tests/resources/cluster-new.genin.yml";
    let base_dir = "tests/.build_with_upgrade";
    let state_dir = format!("{base_dir}/.geninstate");
    let inventory = format!("{base_dir}/inventory.yml");
    cleanup_test_dir(base_dir);

    // build from config
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&src)
        .arg("-o")
        .arg(&inventory)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);
    let result = format!("{result}\n{}", read_to_string(&src).unwrap());
    insta::assert_display_snapshot!("cluster_genin", result);
    assert_eq!(read_dir(&state_dir).unwrap().count(), 2);

    // build with upgrade from latest
    for i in 0..3 {
        let output = Command::new(GENIN_CMD)
            .arg("build")
            .arg("-s")
            .arg(&upg_src)
            .arg("--state-dir")
            .arg(&state_dir)
            .arg("-o")
            .arg(format!("{base_dir}/upg_inventory.{i}.yml"))
            .arg("-y")
            .output()
            .expect("Failed to execute command");

        let result = build_result_from_output(output);
        let result = format!("{result}\n{}", read_to_string(&upg_src).unwrap());
        insta::assert_display_snapshot!("cluster_new_genin", result);
        assert_eq!(read_dir(&state_dir).unwrap().count(), 3);
    }
}

#[test]
fn remove_role_with_undescrore_name() {
    let src = "tests/resources/names/cluster-underscore1.genin.yml";
    let upg_src = "tests/resources/names/cluster-underscore2.genin.yml";
    let base_dir = "tests/.remove_role_with_undescrore_name";
    let state_dir = format!("{base_dir}/.geninstate");
    let inventory = format!("{base_dir}/inventory.yml");
    cleanup_test_dir(base_dir);

    // build from config
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&src)
        .arg("-o")
        .arg(&inventory)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);
    let result = format!("{result}\n{}", read_to_string(&src).unwrap());
    insta::assert_display_snapshot!("undescrore_names", result);
    assert_eq!(read_dir(&state_dir).unwrap().count(), 2);

    // remove subscription_status role
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&upg_src)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("-o")
        .arg(format!("{base_dir}/upg_inventory_remove.yml"))
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);
    let result = format!("{result}\n{}", read_to_string(&upg_src).unwrap());
    insta::assert_display_snapshot!("undescrore_names_remove", result);

    // add subscription_status role
    let output = Command::new(GENIN_CMD)
        .arg("build")
        .arg("-s")
        .arg(&src)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("-o")
        .arg(format!("{base_dir}/upg_inventory_add.yml"))
        .arg("-y")
        .output()
        .expect("Failed to execute command");

    let result = build_result_from_output(output);
    let result = format!("{result}\n{}", read_to_string(&src).unwrap());
    insta::assert_display_snapshot!("undescrore_names", result);
}
