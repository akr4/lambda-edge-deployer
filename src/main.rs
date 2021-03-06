use crate::git::Tag;
use std::fs::File;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod config;
mod git;
mod lambda;
mod models;
mod npm;
mod packager;

#[derive(Debug, StructOpt)]
#[structopt(name = "webapp-deployer", about = "Deploy webapp to CloudFront")]
struct Opt {
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,

    function: String,

    /// Run npm run build
    #[structopt(short, long)]
    build: bool,

    /// Keep Lambda zip file
    #[structopt(short, long)]
    keep_zip: bool,

    /// Force deploy even if current commit has the tag of the given function
    #[structopt(short, long)]
    force_deploy: bool,
}

trait Colored {
    fn error(self) -> colored::ColoredString;
    fn warn(self) -> colored::ColoredString;
    fn info(self) -> colored::ColoredString;
}

impl<'a> Colored for &'a str {
    fn error(self) -> colored::ColoredString {
        use colored::*;
        self.red()
    }

    fn warn(self) -> colored::ColoredString {
        use colored::*;
        self.yellow()
    }

    fn info(self) -> colored::ColoredString {
        use colored::*;
        self.cyan()
    }
}

fn main() {
    let opt = Opt::from_args();

    let config = config::load_config(&opt.config_file).unwrap();
    let function = config.functions.iter().find(|x| x.name == opt.function);
    if function.is_none() {
        eprintln!("{}", "No function found".error());
        std::process::exit(1);
    }
    let function = function.unwrap();

    let git_repo = git::Repository::open(std::env::current_dir().unwrap()).unwrap();

    if git_repo.has_uncommitted_files().unwrap() {
        eprintln!("{}", "There are uncommitted files".error());
        std::process::exit(1);
    }

    if !opt.force_deploy
        && git_repo
            .get_tags()
            .unwrap()
            .iter()
            .filter_map(|t| t.function_name())
            .any(|name| name == function.name)
    {
        eprintln!("{}", "Function already deployed.".warn());
        std::process::exit(1);
    }

    if opt.build {
        npm::build().unwrap();
    }

    match packager::package(&function.bundle) {
        Ok(zip) => {
            eprintln!(
                "{} {}",
                "Packaged".info(),
                &function.bundle.to_string_lossy()
            );

            let zip_file = if opt.keep_zip {
                let zip_path = std::env::current_dir()
                    .unwrap()
                    .join(Path::new(&format!("{}.zip", &function.name)));
                zip.persist(zip_path.as_path()).unwrap();
                eprintln!("zip file: {}", zip_path.as_path().to_string_lossy());
                File::open(zip_path).unwrap()
            } else {
                zip.reopen().unwrap()
            };

            let update_result = lambda::update_function(&function.name, zip_file).unwrap();
            eprintln!("{} {}", "Updated function".info(), function.name);

            let tag = git_repo.tag(&update_result.tag_name()).unwrap();
            eprintln!("{} {}", "Tag".info(), tag.name());

            let remove_result =
                lambda::remove_old_versions(&function.name, &update_result.version).unwrap();
            for version in remove_result.deleted_versions {
                eprintln!("{} {}", "Removed old version".info(), version);
            }
            for failure in remove_result.failures {
                eprintln!(
                    "{} {} - {}",
                    "Remove failure".warn(),
                    failure.version,
                    failure.reason
                )
            }

            eprintln!("{} {}", "New version".info(), update_result.version);
        }
        Err(e) => panic!(e),
    }
}

impl Tag {
    fn function_name(&self) -> Option<String> {
        self.name().split("@").next().map(|x| x.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_function_name_from_tag() {
        let tag = Tag::new("aaa_bbb@3");
        assert_eq!(tag.function_name(), Some("aaa_bbb".to_owned()));
    }
}
