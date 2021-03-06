extern crate cargo_toml_builder;
extern crate dirs;

use crate::builders::{Application, ProjectBuilder};
use crate::{k8s_deploy, set_deployment_vars, Emojis};
use cargo_toml_builder::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::{env, fs};

pub struct RustBuilder;

impl RustBuilder {
    fn get_cargo_toml(name: &str, version: &str) -> String {
        let log_dep = Dependency::version("log", "0.4.8");
        let log_rs_dep = Dependency::version("log4rs", "0.8.3");
        let cloud_state_dep = Dependency::tag(
            "cloudstate",
            "https://github.com/sleipnir/cloudstate-rust",
            "0.1.4",
        );
        //let cloud_state_dep = Dependency::version("cloudstate", "0.0.1");

        let dependencies = vec![log_dep, log_rs_dep, cloud_state_dep];

        let cargo_toml = CargoToml::builder()
            .name(name)
            .version(version)
            .author(whoami::username().as_ref())
            .dependencies(&dependencies)
            .build();

        cargo_toml.unwrap().to_string()
    }
}

impl ProjectBuilder for RustBuilder {
    fn is_dependencies_ok(&self) -> bool {
        Command::new("which")
            .arg("cargo")
            .stdout(Stdio::null())
            .status()
            .is_ok()
    }

    fn pre_compile(&self, app: &Application) {
        env::set_current_dir(&app.work_dir);

        // Find and replace occurrences of {docker-image} and {tag} in deployment.yml
        set_deployment_vars(&app);

        // Set Cargo content
        let cargo_contents = RustBuilder::get_cargo_toml(&app.name, &app.tag);
        let mut cargo_file = File::create("Cargo.toml").unwrap();
        cargo_file.write_all(cargo_contents.as_ref());

        // set dockerfile
        let docker_path = Path::new(&app.work_dir).join("Dockerfile");
        let docker_template_content = fs::read_to_string(docker_path.clone()).unwrap();

        let dockerfile = docker_template_content.replace("{application-name}", app.name.as_ref());

        let mut docker_file = File::create(docker_path).unwrap();
        docker_file.write_all(dockerfile.as_ref());
    }

    fn compile(&self, app: &Application) {
        env::set_current_dir(&app.work_dir);

        println!(
            "{} Downloading and install dependencies...",
            Emojis::default().floppy_disk()
        );
        println!("{} Compiling project...", Emojis::default().coffee());
        let status = Command::new("cargo").arg("build").arg("--release").status();

        if status.is_ok() {
            println!("{} Project successfully compiled", Emojis::default().ok())
        }
    }

    fn build(self, app: Application) {
        println!("{} Building Project...", Emojis::default().worker());
        env::set_current_dir(&app.work_dir);
        self.compile(&app);

        let status = Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(format!("{}:{}", app.registry, app.tag))
            .arg(&app.work_dir)
            .status();

        if status.is_ok() {
            println!(
                "{} Image created successfully!",
                Emojis::default().frame_picture()
            )
        }
    }

    fn push(self, app: Application) {
        env::set_current_dir(&app.work_dir);

        println!("{} Push container image...", Emojis::default().rocket());
        let status = Command::new("docker")
            .arg("push")
            .arg(format!("{}:{}", app.registry, app.tag))
            .status();

        if status.is_ok() {
            println!("{} Pushed!", Emojis::default().moon());
        }
    }

    fn deploy(self, app: Application) {
        k8s_deploy(&app)
    }
}
