// build.rs
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use clap_mangen::Man;
use std::fs::{self, File};
use std::path::Path;

include!("src/cli.rs");

fn main() {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let cmd = Args::command();

    generate_man_page(&cmd);
    generate_completions(&cmd);
}

fn generate_man_page(cmd: &clap::Command) {
    let out_dir = "assets/man";
    fs::create_dir_all(out_dir).unwrap();

    let man = Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer).unwrap();

    fs::write(Path::new(out_dir).join("rucat.1"), buffer).unwrap();
}

fn generate_completions(cmd: &clap::Command) {
    let completions_dir = "assets/completions";
    fs::create_dir_all(completions_dir).unwrap();

    let mut cmd = cmd.clone();
    let bin_name = cmd.get_name().to_string();

    // Fish
    let fish_dir = Path::new(completions_dir).join("fish");
    fs::create_dir_all(&fish_dir).unwrap();
    let mut fish_file = File::create(fish_dir.join(format!("{bin_name}.fish"))).unwrap();
    generate(Shell::Fish, &mut cmd, &bin_name, &mut fish_file);

    // Bash
    let bash_dir = Path::new(completions_dir).join("bash");
    fs::create_dir_all(&bash_dir).unwrap();
    let mut bash_file = File::create(bash_dir.join(&bin_name)).unwrap();
    generate(Shell::Bash, &mut cmd, &bin_name, &mut bash_file);

    // Zsh
    let zsh_dir = Path::new(completions_dir).join("zsh");
    fs::create_dir_all(&zsh_dir).unwrap();
    let mut zsh_file = File::create(zsh_dir.join(format!("_{bin_name}"))).unwrap();
    generate(Shell::Zsh, &mut cmd, &bin_name, &mut zsh_file);
}
