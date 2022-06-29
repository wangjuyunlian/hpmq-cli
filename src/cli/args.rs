use crate::cli::parse;
use anyhow::{Context, Error};
use cargo_generate::{TemplatePath, Vcs};
use oci_util::Reference;
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
        // default_value = "http://git.netfuse.cn/iiot-pub/hpmq-wasi-template.git"
    )]
    pub git: Option<String>,

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

    #[structopt(
        short,
        long,
        help = "请输入完整的镜像仓库，如：repo.netfuse.cn/moss/hello-wasm:0.1"
    )]
    pub image: String,
}

#[derive(Debug, StructOpt)]
pub struct BuildArgs {
    // #[structopt(short, long)]
    // pub user_name: Option<String>,
    // #[structopt(short, long)]
    // pub password: Option<String>,
    #[structopt(
        short,
        long,
        help = "请输入完整的镜像仓库，如：repo.netfuse.cn/moss/hello-wasm:0.1"
    )]
    pub image: String,

    // #[structopt(
    //     short,
    //     long,
    //     conflicts_with = "image",
    //     help = "请输入镜像仓库，如：moss/hello-wasm、hello-wasm"
    // )]
    // pub repository: Option<String>,
    #[structopt(short, long, default_value = "./hpmq.yaml")]
    pub config: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct PullArgs {
    #[structopt(short, long)]
    pub user_name: Option<String>,

    #[structopt(short, long)]
    pub password: Option<String>,

    #[structopt(
        short,
        long,
        help = "请输入完整的镜像仓库，如：repo.netfuse.cn/moss/hello-wasm:0.1"
    )]
    pub image: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct ContainInitArgs {
    #[structopt(short, long)]
    pub user_name: Option<String>,

    #[structopt(short, long)]
    pub password: Option<String>,
    #[structopt(
        short,
        long,
        help = "请输入完整的镜像仓库，如：repo.netfuse.cn/moss/hello-wasm:0.1"
    )]
    pub image: String,
    #[structopt(short, long, help = "是否强制初始化容器", default_value = "false")]
    pub force: String,
}

impl Args {
    pub fn to_generate_args(self) -> cargo_generate::GenerateArgs {
        let define: Vec<String> = Vec::new();
        // define.push(format!("image-project={}", image_project));

        let template = TemplatePath {
            auto_path: None,
            subfolder: None,
            git: self.git,
            branch: None,
            path: self.path,
            favorite: None,
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

    fn try_from(value: crate::cli::args::BuildArgs) -> Result<Self, Self::Error> {
        let crate::cli::args::BuildArgs { image, config } = value;
        let image: Reference = image.parse().context("Not a valid image reference")?;

        let docker_file = dockerfile_parser::Dockerfile::parse(
            std::fs::read_to_string(config.as_str())?.as_str(),
        )?;
        let config = parse(docker_file)?;
        Ok(oci_util::args::BuildArgs { config, image })
    }
}
