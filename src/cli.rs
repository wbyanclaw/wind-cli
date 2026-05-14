//! CLI argument definitions — clap only, no business logic

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "wind",
    about = "wind CLI — 受控 workspace 文件管理",
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

        /// 允许覆盖已存在的文件（默认拒绝覆盖）
        #[arg(long)]
        overwrite: bool,
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

    /// 打开文件或应用（内部使用 windlocal 协议封装）【已弃用，请使用 wft】
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

    /// WFT (Wind Financial Terminal) 集成命令
    Wft {
        #[command(subcommand)]
        action: WftAction,
    },

    /// 检查更新（不实际替换二进制）
    Upgrade {
        /// 仅检查，不下载
        #[arg(long)]
        check: bool,
    },

    /// Agent Protocol 工具接口
    Tools {
        /// 列出所有可用工具
        #[arg(long)]
        list: bool,

        /// 调用指定工具（JSON 格式参数）
        #[arg(long, value_name = "TOOL_NAME")]
        call: Option<String>,

        /// 工具参数（JSON 格式）
        #[arg(long, value_name = "JSON")]
        args: Option<String>,

        /// 显示工具帮助
        #[arg(long)]
        tool_help: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WftAction {
    /// 打开 workspace 内的文件
    File {
        /// 文件路径
        path: std::path::PathBuf,
    },

    /// 在 workspace 内搜索
    Search {
        /// 搜索关键词
        query: String,
    },

    /// 打开应用视图
    App,

    /// 打开设置视图
    Settings,

    /// 显示工作区信息
    Workspace,

    /// 检查更新
    Upgrade,

    /// 直接传递 windlocal URI
    Url {
        /// windlocal URI
        uri: String,
    },
}

pub fn build() -> Cli {
    Cli::parse()
}
