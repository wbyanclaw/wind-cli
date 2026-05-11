//! CLI argument definitions — clap only, no business logic

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "wind",
    about = "wind CLI — 受控 workspace 文件管理 + windlocal 安全解析",
    version
)]
pub struct Cli {
    /// Enable JSON output
    #[arg(long, short)]
    pub json: bool,

    /// Show verbose/debug output
    #[arg(long, short)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 输出版本信息
    Version,

    /// 初始化 workspace
    Init {
        /// workspace 路径，默认当前目录
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },

    /// 列文件
    Ls {
        /// 要列出的路径，默认 workspace 根目录
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },

    /// 读取文件内容
    Cat {
        /// 文件路径
        path: std::path::PathBuf,
    },

    /// 写文件（支持 stdin 或 --file）
    Put {
        /// 目标路径
        path: std::path::PathBuf,

        /// 从 stdin 读取内容
        #[arg(long, short = 's')]
        stdin: bool,

        /// 从指定本地文件读取内容
        #[arg(long, short = 'f')]
        file: Option<std::path::PathBuf>,
    },

    /// 创建目录
    Mkdir {
        /// 目录路径
        path: std::path::PathBuf,
    },

    /// 删除文件或目录
    Rm {
        /// 目标路径
        path: std::path::PathBuf,

        /// 递归删除目录
        #[arg(long, short = 'r')]
        recursive: bool,

        /// 确认删除（必须指定才执行）
        #[arg(long, short = 'y')]
        yes: bool,

        /// 预览将要删除的对象，不实际删除
        #[arg(long)]
        dry_run: bool,
    },

    /// 执行 windlocal 协议
    Open {
        /// windlocal URI
        uri: String,
    },

    /// 检查更新（不实际替换二进制）
    Upgrade {
        /// 仅检查，不下载
        #[arg(long)]
        check: bool,
    },
}

pub fn build() -> Cli {
    Cli::parse()
}
