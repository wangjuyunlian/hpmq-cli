use anyhow::{bail, Result};
use dockerfile_parser::Dockerfile;
use dockerfile_parser::{BreakableStringComponent, Instruction as Ins, ShellOrExecExpr};
use log::{debug, warn};
use oci_util::image::build::config::instructions::{Copy, Dest, Instruction, Kind};
use oci_util::image::build::config::{BuildConfig, BuildConfigBuilder};
use std::path::PathBuf;

pub mod args;
pub mod interactive;

fn parse(value: Dockerfile) -> Result<BuildConfig> {
    let Dockerfile { instructions, .. } = value;
    let mut builder = BuildConfigBuilder::default();
    debug!("指令: {:?}", instructions);
    for ins in instructions.into_iter() {
        match parse_ins(ins) {
            Ok(instruction) => match instruction {
                Instruction::Copy(copy) => {
                    builder.append_copy(copy);
                }
                Instruction::Kind(kind) => builder.mut_kind(kind),
                Instruction::Cmd(cmd) => {
                    builder.mut_cmd(cmd);
                }
            },
            Err(e) => {
                warn!("指令转化失败：{:?}", e);
            }
        }
    }
    if builder.copys.len() == 0 {
        bail!("缺少COPY指令");
    }
    builder.build()
}

fn parse_ins(value: Ins) -> Result<Instruction> {
    let instruction = match value {
        Ins::Copy(ins) => {
            if let Some(src) = ins.sources.get(0) {
                let src: PathBuf = src.content.clone().into();
                if !src.exists() {
                    bail!("源文件【{}】不存在", src.display())
                }
                if src.is_dir() {
                    bail!("COPY暂不支持多文件")
                }
                // let src_name = src.file_name().ok_or(anyhow!(""))?;
                let dest = ins.destination.content.clone();
                Instruction::Copy(Copy(src, Dest::try_from(dest)?))
            } else {
                bail!("COPY缺少参数: {:?}", ins);
            }
        }
        Ins::Cmd(ins) => match ins.expr {
            ShellOrExecExpr::Shell(args) => {
                if let Some(arg) = args.components.get(0) {
                    match arg {
                        BreakableStringComponent::String(file) => {
                            // if file.content.ends_with("/") {
                            //     bail!("CMD参数必须为文件")
                            // }
                            let cmd: Dest = file.content.clone().try_into()?;
                            if cmd.file_name.is_none() {
                                bail!("CMD参数必须为文件")
                            }
                            Instruction::Cmd(cmd)
                        }
                        _ => {
                            bail!("CMD缺少参数")
                        }
                    }
                } else {
                    bail!("CMD缺少参数")
                }
            }
            ShellOrExecExpr::Exec(args) => {
                if let Some(arg) = args.elements.get(0) {
                    let cmd: Dest = arg.content.clone().try_into()?;
                    if cmd.file_name.is_none() {
                        bail!("CMD参数必须为文件")
                    }
                    Instruction::Cmd(cmd)
                } else {
                    bail!("CMD缺少参数")
                }
            }
        },
        Ins::Misc(ins) => {
            if ins.instruction.as_ref().to_uppercase().as_str() == "KIND" {
                if let Some(kind) = ins.arguments.components.get(0) {
                    match kind {
                        BreakableStringComponent::String(kind) => {
                            if kind.content.to_uppercase().as_str() == "APP" {
                                Instruction::Kind(Kind::App)
                            } else {
                                Instruction::Kind(Kind::Wasi)
                            }
                        }
                        _ => Instruction::Kind(Kind::Wasi),
                    }
                } else {
                    Instruction::Kind(Kind::Wasi)
                }
            } else {
                bail!("不支持该指令：{:?}", ins);
            }
        }
        _ins => {
            bail!("不支持该指令：{:?}", _ins);
        }
    };
    Ok(instruction)
}
