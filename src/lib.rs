use cmd::{AccControlCmd, CmdSetAccessControlList, HelpItem, KoviArgs, KoviCmd, PluginCmd};
use kovi::{
    bot::{runtimebot::kovi_api::SetAccessControlList, AccessControlMode},
    error::BotError,
    event::AdminMsgEvent,
    log, serde_json, PluginBuilder as P, RuntimeBot,
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{Pid, ProcessesToUpdate, System};

mod cmd;

// #[derive(Debug, serde::Deserialize, serde::Serialize)]
// struct CMDInfo {
//     cmd_start_with: String,
// }

#[kovi::plugin]
async fn main() {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let start_time = Arc::new(start_time);

    let bot = P::get_runtime_bot();
    // let data_path = bot.get_data_path();
    // let cmd = CMDInfo {
    //     cmd_start_with: ".kovi".to_string(),
    // };
    // let cmd: CMDInfo = load_json_data(cmd, data_path.join("cmd.json")).unwrap();
    // let cmd = Arc::new(cmd);
    P::on_admin_msg(move |e| {
        let bot = bot.clone();
        // let cmd = cmd.clone();
        let start_time = start_time.clone();
        async move {
            let text = if let Some(v) = e.borrow_text() {
                v
            } else {
                return;
            };
            // if !text.starts_with(cmd.cmd_start_with.as_str()) {
            //     return;
            // }

            if !text.starts_with(".kovi") {
                return;
            }

            let vec_text: Vec<&str> = text.split_whitespace().collect();

            let cmd = KoviArgs::parse(vec_text.iter().map(|v| v.to_string()).collect());

            match cmd.command {
                KoviCmd::Help(item) => {
                    help(&e, item);
                }
                KoviCmd::Plugin(plugin_cmd) => match plugin_cmd {
                    PluginCmd::Status => plugin_status(&e, &bot),
                    PluginCmd::Start { name } => {
                        plugin_start(&e, &bot, &name);
                    }
                    PluginCmd::Stop { name } => {
                        plugin_stop(&e, &bot, &name);
                    }
                    PluginCmd::ReStart { name } => {
                        plugin_restart(&e, &bot, &name).await;
                    }
                },
                KoviCmd::Status => status(&e, &bot, &start_time).await,
                KoviCmd::Acc { name, acc_cmd } => acc(&e, &bot, &name, acc_cmd),
            }
        }
    });
}

static HELP_MSG: &str = r#"┄ 📜 帮助列表 ┄
.kovi plugin <T>: 插件管理
.kovi acc <name> <T>: 访问控制
.kovi status: 状态信息
部分命令可缩写为第一个字母"#;

static HELP_PLUGIN: &str = r#"┄ 📜 插件管理 ┄:
.kovi plugin <T>

<T>:
list: 列出所有插件
start <name>: 启动插件
stop <name>: 停止插件
restart <name>: 重载插件"#;

static ACC_CONTROL_PLUGIN: &str = r#"┄ 📜 访问控制 ┄:
.kovi acc <name> <T>

<T>:
status: 列出插件访问控制信息
enable: 启用插件访问控制
disable: 禁用插件访问控制
mode <white | black>: 插件访问控制模式
on: 添加本群到列表
off: 移除本群到列表
add <friend | group> [id]: 添加多个
remove <friend | group> [id]: 移除多个"#;

fn help(e: &AdminMsgEvent, item: HelpItem) {
    match item {
        HelpItem::Plugin => {
            e.reply(HELP_PLUGIN);
        }
        HelpItem::Acc => {
            e.reply(ACC_CONTROL_PLUGIN);
        }
        HelpItem::None => {
            e.reply(HELP_MSG);
        }
    }
}

async fn status(e: &AdminMsgEvent, bot: &RuntimeBot, start_time: &u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let duration = now - *start_time;

    // 计算运行时间
    let days = duration / (24 * 3600);
    let hours = (duration % (24 * 3600)) / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    // 获取内存使用情况
    let mut sys = System::new();

    let pid = Pid::from_u32(std::process::id());
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    sys.refresh_memory();

    let self_memory_usage = sys
        .process(pid)
        .map(|process| process.memory() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);

    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let memory_usage_percent = (used_memory / total_memory) * 100.0;

    let time_str = if days > 0 {
        format!("{}d{}h{}m{}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    };

    let plugin_info = bot.get_plugin_info().unwrap();

    let plugin_start_len = plugin_info.iter().filter(|v| v.enabled).count();

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct OnebotInfo {
        app_name: Option<String>,
        app_version: Option<String>,
    }

    let onebot_info: Option<OnebotInfo> = match bot.get_version_info().await {
        Ok(v) => match serde_json::from_value::<OnebotInfo>(v.data) {
            Ok(v) => Some(v),
            Err(_) => None,
        },
        Err(_) => None,
    };

    let onebot_info_str = match onebot_info {
        Some(v) => {
            let mut msg = "服务端:\n  ".to_string();

            if let Some(app_name) = v.app_name {
                msg.push_str(&app_name);
            }
            if let Some(app_version) = v.app_version {
                msg.push_str(&format!("（{}）", app_version));
            }

            msg
        }
        None => "服务端: 信息获取失败".to_string(),
    };

    let plugin_info_len = plugin_info.len();

    let reply = format!(
        "┄ 📑 状态 ┄\n\
        🕑 运行时间: {time_str}\n\
        📦 插件数量: {plugin_info_len} 启用 {plugin_start_len} 个\n\
        🔋 内存使用: {self_memory_usage:.2}MB\n\
        💻 系统内存:\n  {:.2}GB/{:.2}GB({:.0}%)\n\
        🔗 {}",
        used_memory, total_memory, memory_usage_percent, onebot_info_str
    );

    e.reply(reply);
}

fn acc(e: &AdminMsgEvent, bot: &RuntimeBot, plugin_name: &str, acc_cmd: AccControlCmd) {
    let plugin_name = is_not_empty_or_more_times_and_reply(e, bot, plugin_name);

    let plugin_name = match plugin_name {
        Some(v) => v,
        None => return,
    };

    if plugin_is_self(&plugin_name) && acc_cmd != AccControlCmd::Status {
        e.reply("⛔ 不允许修改CMD插件");
        return;
    }
    match acc_cmd {
        AccControlCmd::Enable(b) => match bot.set_plugin_access_control(&plugin_name, b) {
            Ok(_) => {
                e.reply("✅ 设置成功");
            }
            Err(err) => match err {
                BotError::PluginNotFound(_) => {
                    e.reply(format!("🔎 插件{}不存在", &plugin_name));
                }
                BotError::RefExpired => {
                    panic!("CMD: Bot RefExpired");
                }
            },
        },
        AccControlCmd::SetMode(v) => match bot.set_plugin_access_control_mode(&plugin_name, v) {
            Ok(_) => {
                e.reply("✅ 设置成功");
            }
            Err(err) => match err {
                BotError::PluginNotFound(_) => {
                    e.reply(format!("🔎 插件{}不存在", &plugin_name));
                }
                BotError::RefExpired => {
                    panic!("CMD: Bot RefExpired");
                }
            },
        },
        AccControlCmd::Change(change) => match change {
            CmdSetAccessControlList::GroupAdds(v) => {
                process_ids(v, true, true, &plugin_name, bot, e);
            }
            CmdSetAccessControlList::GroupRemoves(v) => {
                process_ids(v, true, false, &plugin_name, bot, e);
            }
            CmdSetAccessControlList::FriendAdds(v) => {
                process_ids(v, false, true, &plugin_name, bot, e);
            }
            CmdSetAccessControlList::FriendRemoves(v) => {
                process_ids(v, false, false, &plugin_name, bot, e);
            }
        },
        AccControlCmd::Status => {
            let plugin_infos = match bot.get_plugin_info() {
                Ok(v) => v,
                Err(_) => panic!("CMD: Bot RefExpired"),
            };

            for info in plugin_infos {
                if info.name == plugin_name {
                    let boo = if info.access_control { "✅" } else { "❎" };
                    let mode = match info.list_mode {
                        AccessControlMode::BlackList => "黑名单",
                        AccessControlMode::WhiteList => "白名单",
                    };
                    let list = info.access_list;
                    let group_list = list.groups;
                    let friend_list = list.friends;
                    let group_list_str = if group_list.is_empty() {
                        "无".to_string()
                    } else {
                        group_list
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    };
                    let friend_list = if friend_list.is_empty() {
                        "无".to_string()
                    } else {
                        friend_list
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    };

                    let msg = format!(
                        "📦 插件{}\n访问控制：{}\n模式：{}\n群组：{}\n好友列表：{}",
                        plugin_name, boo, mode, group_list_str, friend_list
                    );
                    e.reply(msg);
                    return;
                }
            }

            e.reply("🔎 插件不存在");
        }
        AccControlCmd::GroupIsEnable(boo) => {
            if e.is_private() {
                e.reply("⛔ 只能在群聊中使用");
                return;
            }

            let set_access = if boo {
                SetAccessControlList::Add(e.group_id.unwrap())
            } else {
                SetAccessControlList::Remove(e.group_id.unwrap())
            };

            match bot.set_plugin_access_control_list(&plugin_name, true, set_access) {
                Ok(_) => {
                    let msg = if boo {
                        format!(
                            "✅ 插件{}访问控制已添加{}",
                            plugin_name,
                            e.group_id.unwrap()
                        )
                    } else {
                        format!(
                            "✅ 插件{}访问控制已移除{}",
                            plugin_name,
                            e.group_id.unwrap()
                        )
                    };
                    e.reply(msg);
                }
                Err(err) => match err {
                    BotError::PluginNotFound(_) => {
                        e.reply(format!("🔎 插件{}不存在", plugin_name));
                    }
                    BotError::RefExpired => {
                        panic!("CMD: Bot RefExpired");
                    }
                },
            }
        }
    }
}

/// 设置插件访问控制列表
fn process_ids(
    v: Vec<String>,
    is_group: bool,
    is_add: bool,
    plugin_name: &str,
    bot: &RuntimeBot,
    e: &AdminMsgEvent,
) {
    let mut vec_i64: Vec<i64> = Vec::new();

    for str in v {
        match str.parse() {
            Ok(v) => {
                vec_i64.push(v);
            }
            Err(_) => {
                e.reply("❎ 设置失败");
                return;
            }
        }
    }

    let vec_i64 = if is_add {
        SetAccessControlList::Adds(vec_i64)
    } else {
        SetAccessControlList::Removes(vec_i64)
    };

    match bot.set_plugin_access_control_list(plugin_name, is_group, vec_i64) {
        Ok(_) => {
            e.reply("✅ 设置成功");
        }
        Err(err) => match err {
            BotError::PluginNotFound(_) => {
                e.reply(format!("🔎 插件{}不存在", plugin_name));
            }
            BotError::RefExpired => {
                panic!("CMD: Bot RefExpired");
            }
        },
    }
}

fn plugin_start(e: &AdminMsgEvent, bot: &RuntimeBot, name: &str) {
    let name = is_not_empty_or_more_times_and_reply(e, bot, name);

    let name = match name {
        Some(v) => v,
        None => return,
    };

    if plugin_is_self(&name) {
        e.reply("🏳️ 这么做...，你想干嘛");
        return;
    }
    match bot.enable_plugin(&name) {
        Ok(_) => {
            e.reply(format!("✅ 插件{}启动成功", name));
        }
        Err(err) => match err {
            BotError::PluginNotFound(_) => {
                e.reply(format!("🔎 插件{}不存在", name));
            }
            BotError::RefExpired => {
                panic!("CMD: Bot RefExpired");
            }
        },
    }
}

fn plugin_stop(e: &AdminMsgEvent, bot: &RuntimeBot, name: &str) {
    let name = is_not_empty_or_more_times_and_reply(e, bot, name);

    let name = match name {
        Some(v) => v,
        None => return,
    };

    if plugin_is_self(&name) {
        e.reply("⛔ 不允许关闭CMD插件");
        return;
    }
    match bot.disable_plugin(&name) {
        Ok(_) => {
            e.reply(format!("✅ 插件{}关闭成功", name));
        }
        Err(err) => match err {
            BotError::PluginNotFound(_) => {
                e.reply(format!("🔎 插件{}不存在", name));
            }
            BotError::RefExpired => {
                panic!("CMD: Bot RefExpired");
            }
        },
    }
}

async fn plugin_restart(e: &AdminMsgEvent, bot: &RuntimeBot, name: &str) {
    let name = is_not_empty_or_more_times_and_reply(e, bot, name);

    let name = match name {
        Some(v) => v,
        None => return,
    };

    if plugin_is_self(&name) {
        e.reply("⛔ 不允许重载CMD插件");
        return;
    }
    match bot.restart_plugin(&name).await {
        Ok(_) => {
            e.reply(format!("✅ 插件{}重载成功", name));
        }
        Err(err) => match err {
            BotError::PluginNotFound(_) => {
                e.reply(format!("🔎 插件{}不存在", name));
            }
            BotError::RefExpired => {
                panic!("CMD: Bot RefExpired");
            }
        },
    }
}

fn plugin_status(e: &AdminMsgEvent, bot: &RuntimeBot) {
    let plugin_info = bot.get_plugin_info().unwrap();
    if plugin_info.is_empty() {
        e.reply("🔎 插件列表为空");
        return;
    }

    let mut msg = "┄ 📑 插件列表 ┄\n".to_string();

    plugin_info.iter().for_each(|info| {
        let boo = if info.enabled { "✅" } else { "❎" };

        let msg_ = format!("{} {}(v{})\n", boo, info.name, info.version);
        msg.push_str(&msg_);
    });

    e.reply(msg.trim());
}

/// 检查插件名是否为空或多个插件名，返回第一个插件名或None，顺带回复
fn is_not_empty_or_more_times_and_reply(
    e: &AdminMsgEvent,
    bot: &RuntimeBot,
    name: &str,
) -> Option<String> {
    let names = match get_plugin_full_name(bot, name) {
        Ok(names) => names,
        Err(err) => {
            log::error!("CMD: {}", err);
            panic!("{err}")
        }
    };

    if names.is_empty() {
        e.reply("🔎 插件列表为空");
        return None;
    } else if names.len() > 1 {
        e.reply(format!("┄ 🔎 寻找到多个插件 ┄\n{}", names.join("\n")));
        return None;
    }

    names.into_iter().next()
}

fn get_plugin_full_name(bot: &RuntimeBot, name: &str) -> Result<Vec<String>, BotError> {
    let plugins = match bot.get_plugin_info() {
        Ok(plugins) => plugins,
        Err(err) => {
            log::error!("CMD: {}", err);
            return Err(err);
        }
    };

    let names = plugins
        .iter()
        .filter_map(|v| {
            if v.name.contains(name) {
                Some(v.name.clone())
            } else {
                None
            }
        })
        .collect();

    Ok(names)
}

fn plugin_is_self(name: &str) -> bool {
    name == env!("CARGO_PKG_NAME")
}

#[test]
fn test_parse() {
    let cmd = KoviArgs::parse(vec![".kovi".to_string()]);

    println!("{:?}", cmd);
}
