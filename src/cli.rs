//! CLI argument definitions — clap only, no business logic

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "windcli",
    about = "windcli — 受控 workspace 文件管理 CLI",
    version
)]
pub struct Cli {
    /// Enable JSON output
    #[arg(long, short)]
    pub json: bool,

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
    Read {
        /// 文件路径
        path: std::path::PathBuf,
    },

    /// 写文件（支持 stdin 或 --content）
    Write {
        /// 目标路径
        path: std::path::PathBuf,

        /// 从 stdin 读取内容
        #[arg(long, short = 's')]
        stdin: bool,

        /// 直接写入文本内容
        #[arg(long, short = 'c')]
        content: Option<String>,
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

        /// 强制删除：等价于 --recursive --yes（AI Agent 推荐用法）
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// 打开 workspace 内容（windlocal 协议封装入口）
    Open {
        /// 打开 workspace 内的文件
        #[arg(long, short = 'f')]
        file: Option<std::path::PathBuf>,

        /// 在 workspace 内搜索
        #[arg(long, short = 's')]
        search: Option<String>,

        /// 打开应用视图
        #[arg(long)]
        app: bool,

        /// 打开设置视图
        #[arg(long)]
        settings: bool,
    },

    /// 检查更新；当前不会自动下载安装
    Upgrade {
        /// 检查 GitHub release，不下载或安装
        #[arg(long)]
        check: bool,
    },

    /// Agent Protocol: list, describe, call tools
    Tools {
        #[command(subcommand)]
        subcommand: ToolsCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ToolsCommand {
    /// 列出所有可用工具（AI Agent 用）
    List,

    /// 查看单工具详情（AI Agent 调用前用）
    Describe {
        /// 工具名称
        name: String,
    },

    /// 调用工具（AI Agent 用）
    Call {
        /// 工具名称
        name: String,

        /// JSON 格式参数
        #[arg(long)]
        params: Option<String>,

        /// 高危操作显式授权
        #[arg(long, short = 'f')]
        force: bool,
    },
}

pub fn build() -> Cli {
    let args: Vec<String> = std::env::args().collect();
    let args = normalize_args(&args);
    Cli::parse_from(args)
}

/// Normalize AI-friendly command aliases: cat→read, put→write, delete→rm
fn normalize_args(args: &[String]) -> Vec<String> {
    let mut result = vec![args[0].clone()];
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "cat" => result.push("read".to_string()),
            "put" => result.push("write".to_string()),
            "delete" => result.push("rm".to_string()),
            arg => result.push(arg.to_string()),
        }
        i += 1;
    }
    result
}
