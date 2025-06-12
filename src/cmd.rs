use kovi::bot::AccessControlMode;

#[derive(Debug, Clone)]
pub(crate) struct KoviArgs {
    pub(crate) command: KoviCmd,
}

#[derive(Debug, Clone)]
pub(crate) enum KoviCmd {
    Help(HelpItem),
    Plugin(PluginCmd),
    Acc {
        name: String,
        acc_cmd: AccControlCmd,
    },
    Status,
}

#[derive(Debug, Clone)]
pub(crate) enum HelpItem {
    None,
    Plugin,
    Acc,
}

#[derive(Debug, Clone)]
pub(crate) enum PluginCmd {
    Status,
    Start { name: String },
    Stop { name: String },
    ReStart { name: String },
}

#[derive(Debug, Clone)]
pub(crate) enum AccControlCmd {
    Status,
    Enable(bool),
    SetMode(AccessControlMode),
    Change(CmdSetAccessControlList),
    GroupIsEnable(bool),
}

#[derive(PartialEq, Debug, Clone)]
pub enum CmdSetAccessControlList {
    GroupAdds(Vec<String>),
    GroupRemoves(Vec<String>),

    FriendAdds(Vec<String>),
    FriendRemoves(Vec<String>),
}

impl KoviArgs {
    pub(crate) fn parse(args: Vec<String>) -> Self {
        let mut args = args.iter().skip(1).map(|s| s.trim());

        let command = match args.next() {
            Some(v) => v,
            None => {
                return Self {
                    command: KoviCmd::Help(HelpItem::None),
                };
            }
        };

        match command.to_lowercase().as_str() {
            "status" | "s" => Self {
                command: KoviCmd::Status,
            },
            "help" | "h" => {
                let sub_command = match args.next() {
                    Some(v) => v,
                    None => {
                        return Self {
                            command: KoviCmd::Help(HelpItem::None),
                        };
                    }
                };

                match sub_command {
                    "plugin" | "p" => Self {
                        command: KoviCmd::Help(HelpItem::Plugin),
                    },
                    "acc" | "a" => Self {
                        command: KoviCmd::Help(HelpItem::Acc),
                    },
                    _ => Self {
                        command: KoviCmd::Help(HelpItem::None),
                    },
                }
            }

            "plugin" | "p" => {
                let sub_command = match args.next() {
                    Some(v) => v,
                    None => {
                        return Self {
                            command: KoviCmd::Help(HelpItem::Plugin),
                        };
                    }
                };

                match sub_command {
                    "list" | "status" | "l" => Self {
                        command: KoviCmd::Plugin(PluginCmd::Status),
                    },
                    "start" => {
                        let name = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Plugin),
                                };
                            }
                        };

                        Self {
                            command: KoviCmd::Plugin(PluginCmd::Start {
                                name: name.to_string(),
                            }),
                        }
                    }
                    "stop" => {
                        let name = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Plugin),
                                };
                            }
                        };

                        Self {
                            command: KoviCmd::Plugin(PluginCmd::Stop {
                                name: name.to_string(),
                            }),
                        }
                    }
                    "restart" | "r" => {
                        let name = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Plugin),
                                };
                            }
                        };

                        Self {
                            command: KoviCmd::Plugin(PluginCmd::ReStart {
                                name: name.to_string(),
                            }),
                        }
                    }
                    _ => Self {
                        command: KoviCmd::Help(HelpItem::Plugin),
                    },
                }
            }
            "acc" | "a" => {
                let plugin_name = match args.next() {
                    Some(v) => v,
                    None => {
                        return Self {
                            command: KoviCmd::Help(HelpItem::Acc),
                        };
                    }
                };

                let sub_command = match args.next() {
                    Some(v) => v,
                    None => {
                        return Self {
                            command: KoviCmd::Help(HelpItem::Acc),
                        };
                    }
                };

                match sub_command {
                    "status" | "s" => Self {
                        command: KoviCmd::Acc {
                            name: plugin_name.to_string(),
                            acc_cmd: AccControlCmd::Status,
                        },
                    },
                    "mode" | "m" => {
                        let mode = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Acc),
                                };
                            }
                        };

                        match mode {
                            "w" | "white" => Self {
                                command: KoviCmd::Acc {
                                    name: plugin_name.to_string(),
                                    acc_cmd: AccControlCmd::SetMode(AccessControlMode::WhiteList),
                                },
                            },
                            "b" | "black" => Self {
                                command: KoviCmd::Acc {
                                    name: plugin_name.to_string(),
                                    acc_cmd: AccControlCmd::SetMode(AccessControlMode::BlackList),
                                },
                            },
                            _ => Self {
                                command: KoviCmd::Help(HelpItem::Acc),
                            },
                        }
                    }
                    "add" | "a" => {
                        let type_ = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Acc),
                                };
                            }
                        };

                        match type_ {
                            "friend" | "f" => {
                                let list: Vec<String> = args.map(|v| v.to_string()).collect();

                                if list.is_empty() {
                                    Self {
                                        command: KoviCmd::Help(HelpItem::Acc),
                                    }
                                } else {
                                    Self {
                                        command: KoviCmd::Acc {
                                            name: plugin_name.to_string(),
                                            acc_cmd: AccControlCmd::Change(
                                                CmdSetAccessControlList::FriendAdds(list),
                                            ),
                                        },
                                    }
                                }
                            }
                            "group" | "g" => {
                                let list: Vec<String> = args.map(|v| v.to_string()).collect();

                                if list.is_empty() {
                                    Self {
                                        command: KoviCmd::Help(HelpItem::Acc),
                                    }
                                } else {
                                    Self {
                                        command: KoviCmd::Acc {
                                            name: plugin_name.to_string(),
                                            acc_cmd: AccControlCmd::Change(
                                                CmdSetAccessControlList::GroupAdds(list),
                                            ),
                                        },
                                    }
                                }
                            }
                            _ => Self {
                                command: KoviCmd::Help(HelpItem::Acc),
                            },
                        }
                    }
                    "remove" | "r" => {
                        let type_ = match args.next() {
                            Some(v) => v,
                            None => {
                                return Self {
                                    command: KoviCmd::Help(HelpItem::Acc),
                                };
                            }
                        };

                        match type_ {
                            "friend" | "f" => {
                                let list: Vec<String> = args.map(|v| v.to_string()).collect();

                                if list.is_empty() {
                                    Self {
                                        command: KoviCmd::Help(HelpItem::Acc),
                                    }
                                } else {
                                    Self {
                                        command: KoviCmd::Acc {
                                            name: plugin_name.to_string(),
                                            acc_cmd: AccControlCmd::Change(
                                                CmdSetAccessControlList::FriendRemoves(list),
                                            ),
                                        },
                                    }
                                }
                            }
                            "group" | "g" => {
                                let list: Vec<String> = args.map(|v| v.to_string()).collect();

                                if list.is_empty() {
                                    Self {
                                        command: KoviCmd::Help(HelpItem::Acc),
                                    }
                                } else {
                                    Self {
                                        command: KoviCmd::Acc {
                                            name: plugin_name.to_string(),
                                            acc_cmd: AccControlCmd::Change(
                                                CmdSetAccessControlList::GroupRemoves(list),
                                            ),
                                        },
                                    }
                                }
                            }
                            _ => Self {
                                command: KoviCmd::Help(HelpItem::Acc),
                            },
                        }
                    }
                    "on" => Self {
                        command: KoviCmd::Acc {
                            name: plugin_name.to_string(),
                            acc_cmd: AccControlCmd::GroupIsEnable(true),
                        },
                    },
                    "off" => Self {
                        command: KoviCmd::Acc {
                            name: plugin_name.to_string(),
                            acc_cmd: AccControlCmd::GroupIsEnable(false),
                        },
                    },
                    "enable" | "e" => Self {
                        command: KoviCmd::Acc {
                            name: plugin_name.to_string(),
                            acc_cmd: AccControlCmd::Enable(true),
                        },
                    },
                    "disable" | "d" => Self {
                        command: KoviCmd::Acc {
                            name: plugin_name.to_string(),
                            acc_cmd: AccControlCmd::Enable(false),
                        },
                    },
                    _ => Self {
                        command: KoviCmd::Help(HelpItem::Acc),
                    },
                }
            }
            _ => Self {
                command: KoviCmd::Help(HelpItem::None),
            },
        }
    }
}

impl PartialEq for AccControlCmd {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AccControlCmd::Status, AccControlCmd::Status) => true,
            (AccControlCmd::Enable(a), AccControlCmd::Enable(b)) => a == b,
            (AccControlCmd::SetMode(a), AccControlCmd::SetMode(b)) => match a {
                AccessControlMode::WhiteList => matches!(b, AccessControlMode::WhiteList),
                AccessControlMode::BlackList => matches!(b, AccessControlMode::BlackList),
            },
            (AccControlCmd::Change(a), AccControlCmd::Change(b)) => a == b,
            (AccControlCmd::GroupIsEnable(a), AccControlCmd::GroupIsEnable(b)) => a == b,
            _ => false,
        }
    }
}
