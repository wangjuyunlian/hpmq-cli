use crate::cli::parse;
use anyhow::{bail, Context, Error};
use cargo_generate::{TemplatePath, Vcs};
use oci_util::Reference;
use serde_json::Value;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Cli {
    #[structopt(name = "init")]
    Init(Args),
    #[structopt(name = "build")]
    Build(BuildArgs),
    #[structopt(name = "push")]
    Push(PushArgs),
    #[structopt(name = "pull")]
    Pull(PullArgs),
    #[structopt(name = "ct-init")]
    Container(ContainInitArgs),
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(
        short,
        long,
        default_value = "https://github.com/wangjuyunlian/hpmq-template.git"
    )]
    pub git: String,

    // 待开发
    /// Local path to copy the template from. Can not be specified together with --git.
    #[structopt(short, long, conflicts_with = "git")]
    pub path: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct PushArgs {
    #[structopt(short, long)]
    pub user_name: Option<String>,

    #[structopt(short, long)]
    pub password: Option<String>,

    #[structopt(short, long, help = "请输入完整的镜像仓库")]
    pub image: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct BuildArgs {
    #[structopt(short, long)]
    pub image: Option<String>,

    #[structopt(short, long, default_value = "./hpmq.yaml")]
    pub config: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(short, long)]
    pub user_name: Option<String>,

    #[structopt(short, long)]
    pub password: Option<String>,

    #[structopt(short, long, help = "请输入完整的镜像仓库")]
    pub image: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct ContainInitArgs {
    #[structopt(short, long)]
    pub user_name: Option<String>,

    #[structopt(short, long)]
    pub password: Option<String>,
    #[structopt(short, long, help = "请输入完整的镜像仓库")]
    pub image: String,
    #[structopt(short, long, help = "是否强制初始化容器", default_value = "false")]
    pub force: String,
}

impl Args {
    pub fn to_generate_args(self) -> cargo_generate::GenerateArgs {
        let define: Vec<String> = Vec::new();
        // define.push(format!("image-project={}", image_project));

        let template = if self.path.is_some() {
            TemplatePath {
                auto_path: None,
                subfolder: None,
                git: None,
                branch: None,
                path: self.path,
                favorite: None,
            }
        } else {
            TemplatePath {
                auto_path: None,
                subfolder: None,
                git: Some(self.git),
                branch: None,
                path: self.path,
                favorite: None,
            }
        };
        cargo_generate::GenerateArgs {
            template_path: template,
            list_favorites: false,
            // favorite: None,
            // subfolder: None,
            // git: self.git,
            // path: self.path,
            // branch: None,
            name: None,
            force: false,
            verbose: false,
            template_values_file: None,
            silent: false,
            config: None,
            vcs: Vcs::None,
            lib: false,
            bin: false,
            ssh_identity: None,
            define,
            init: false,
            destination: None,
            force_git_init: false,
            allow_commands: false,
        }
    }
}

// pub trait UpdateAuth {
//     fn update_auth(&mut self);
// }
// impl UpdateAuth for BuildArgs {
//     fn update_auth(&mut self) {
//         init_auth(&mut self.user_name, &mut self.password);
//     }
// }
//
// fn init_auth(user_name: &mut Option<String>, password: &mut Option<String>) {
//     if user_name.is_none() {
//         let _ = user_name.insert(interactive::user_name().unwrap());
//     }
//     if password.is_none() {
//         let _ = password.insert(interactive::password().unwrap());
//     }
// }

impl TryFrom<crate::cli::args::BuildArgs> for oci_util::args::BuildArgs {
    type Error = Error;

    fn try_from(value: BuildArgs) -> Result<Self, Self::Error> {
        let BuildArgs { image, config } = value;

        let image = init_image(&image)?;
        let docker_file = dockerfile_parser::Dockerfile::parse(
            std::fs::read_to_string(config.as_str())?.as_str(),
        )?;
        let config = parse(docker_file)?;
        Ok(oci_util::args::BuildArgs { config, image })
    }
}

pub fn init_image(image: &Option<String>) -> anyhow::Result<Reference> {
    Ok(if let Some(ref image) = image {
        image
            .as_str()
            .parse()
            .context("Not a valid image reference")?
    } else {
        let (name, version) = package_metadata()?;
        Reference::with_tag(
            "repo.netfuse.cn".to_string(),
            format!("{}/{}", "hpmq_dev", name),
            version,
        )
    })
}

pub fn package_metadata() -> anyhow::Result<(String, String)> {
    let status = Command::new("cargo")
        .args(["metadata", "--no-deps"])
        .output()
        .context("cargo metadata失败")?;
    let info: Value =
        serde_json::from_slice(status.stdout.as_slice()).context("cargo metadata转json失败")?;
    if let Some(metadata) = info
        .get("packages")
        .and_then(|x| x.as_array())
        .and_then(|x| x.get(0))
        .and_then(|x| {
            if let Some(name) = x.get("name").and_then(|val| val.as_str()) {
                if let Some(version) = x.get("version").and_then(|val| val.as_str()) {
                    Some((name.to_string(), version.to_string()))
                } else {
                    None
                }
            } else {
                None
            }
        })
    {
        Ok(metadata)
    } else {
        bail!("cargo metadata获取项目信息失败")
    }
}

#[cfg(test)]
mod test {
    use crate::cli::args::package_metadata;
    use oci_util::Reference;

    #[test]
    fn test() {
        let image: Reference = "abc/hpmq_dev/my-demos:0.3".parse().unwrap();
        println!("{:?}", image);
        let image: Reference = "hpmq_dev/my-demos:0.3".parse().unwrap();
        println!("{:?}", image);
        let image: Reference = "my-demos:0.3".parse().unwrap();
        println!("{:?}", image);
    }
    #[test]
    fn test_metadate() {
        let image = package_metadata().unwrap();
        println!("{:?}", image);
    }
}
