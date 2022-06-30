#![allow(unused_imports)]
use anyhow::Result;
use cargo_generate::generate;
use hpmq_cli::cli::{args::*, interactive};
use log::LevelFilter::{Debug, Info};
use log::{debug, error, info};
use oci_util::distribution::{pull::pull, push::push};
use oci_util::Reference;
use structopt::StructOpt;

use oci_util::container::init;
use oci_util::RegistryAuth;

#[tokio::main]
async fn main() -> Result<()> {
    custom_utils::logger::LoggerBuilder::default(Debug)
        .module("hyper", Info)
        .build_default()
        .log_to_stdout()
        .start();

    let args = Cli::from_args();
    match args {
        Cli::Init(args) => {
            // if args.git.is_none() && args.path.is_none() {
            //     args.git =
            //         Some("http://git.netfuse.cn/iiot-pub/hpmq-wasi-template.git".to_string());
            // }
            // let image_project = interactive::image_project().unwrap();
            let args: cargo_generate::GenerateArgs = args.to_generate_args();
            debug!("init.args: {:?}", args);
            generate(args)?;
        }
        Cli::Build(args) => {
            let build_args = args.try_into()?;
            match oci_util::image::build::build(&build_args).await {
                Err(e) => {
                    error!("build fail: {:?}", e);
                }
                Ok(digest) => {
                    info!("{}:{} build successfully", build_args.image, digest);
                }
            }
        }
        Cli::Push(args) => {
            let image = init_image(&args.image)?;
            let auth = init_auth(args.user_name, args.password);
            match push(&image, &auth).await {
                Err(e) => {
                    error!("push fail: {:?}", e);
                }
                Ok(_) => {
                    info!("{} pushed successfully", image);
                }
            }
        }
        Cli::Pull(args) => {
            let image: Reference = args.image.parse()?;
            let auth = init_auth(args.user_name, args.password);
            if let Err(e) = pull(&image, &auth).await {
                error!("pull fail: {:?}", e);
            } else {
                info!("pulled successfully ");
            }
        }
        Cli::Container(args) => {
            let image: Reference = args.image.parse()?;
            let auth = init_auth(args.user_name, args.password);
            let force = if args.force.to_uppercase().as_str() == "TRUE" {
                true
            } else {
                false
            };
            if let Err(e) = init(&image, &auth, force).await {
                error!("container init fail: {:?}", e);
            } else {
                info!("container init successfully ");
            }
        }
    }
    Ok(())
}

fn init_auth(user_name: Option<String>, password: Option<String>) -> RegistryAuth {
    let user_name = if let Some(val) = user_name {
        val
    } else {
        interactive::user_name().unwrap()
    };
    let password = if let Some(val) = password {
        val
    } else {
        interactive::password().unwrap()
    };
    RegistryAuth::Basic(user_name, password)
}
