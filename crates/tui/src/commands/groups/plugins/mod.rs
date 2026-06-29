use crate::commands::CommandResult;
use crate::commands::traits::{
    Command, CommandGroup, CommandInfo, FunctionCommand, RegisterCommand,
};
use crate::localization::MessageId;
use crate::plugins;
use crate::tui::app::App;

pub struct PluginsCommands;

impl CommandGroup for PluginsCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![Box::new(FunctionCommand::new(
            PluginListCmd::info(),
            PluginListCmd::execute,
        ))]
    }
}

pub(in crate::commands) const PLUGIN_LIST_INFO: CommandInfo = CommandInfo {
    name: "plugin",
    aliases: &["plugins"],
    usage: "/plugin [list|enable <name>|disable <name>|info <name>]",
    description_id: MessageId::CmdPluginDescription,
};

pub(in crate::commands) struct PluginListCmd;

impl RegisterCommand for PluginListCmd {
    fn info() -> &'static CommandInfo {
        &PLUGIN_LIST_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        let Some(arg) = arg.map(str::trim).filter(|arg| !arg.is_empty()) else {
            return plugin_list(app);
        };

        let mut parts = arg.splitn(2, char::is_whitespace);
        let action = parts.next().unwrap_or_default();
        let rest = parts.next().unwrap_or_default().trim();
        match action {
            "list" | "ls" => plugin_list(app),
            "enable" => {
                if rest.is_empty() {
                    CommandResult::error("Usage: /plugin enable <name>")
                } else {
                    plugin_enable(app, rest)
                }
            }
            "disable" => {
                if rest.is_empty() {
                    CommandResult::error("Usage: /plugin disable <name>")
                } else {
                    plugin_disable(app, rest)
                }
            }
            "info" => {
                if rest.is_empty() {
                    CommandResult::error("Usage: /plugin info <name>")
                } else {
                    plugin_info(app, rest)
                }
            }
            name => plugin_info(app, name),
        }
    }
}

fn plugin_list(_app: &App) -> CommandResult {
    plugins::try_with_registry(|r| {
        if r.is_empty() {
            return CommandResult::message("No plugins discovered.");
        }

        let mut out = String::new();
        let enabled_count = r.enabled_plugins().len();
        out.push_str(&format!(
            "Plugins ({}, {} enabled)\n",
            r.len(),
            enabled_count
        ));
        out.push_str(&"=".repeat(40));
        out.push('\n');

        for (name, plugin) in r.list() {
            let status = if r.is_enabled(name) {
                "enabled"
            } else {
                "disabled"
            };
            let description = plugin
                .manifest
                .plugin
                .description
                .as_deref()
                .unwrap_or("No description");
            out.push_str(&format!("• {} [{}]\n  {}\n", name, status, description));
        }

        CommandResult::message(out)
    })
    .unwrap_or_else(|| CommandResult::error("Plugin registry not initialized."))
}

fn plugin_enable(_app: &App, name: &str) -> CommandResult {
    let result = plugins::with_registry(|r| r.enable(name));

    match result {
        Some(true) => CommandResult::message(format!("Plugin '{}' enabled.", name)),
        Some(false) => CommandResult::error(format!("Plugin '{}' not found.", name)),
        None => CommandResult::error("Plugin registry not initialized."),
    }
}

fn plugin_disable(_app: &App, name: &str) -> CommandResult {
    let result = plugins::with_registry(|r| r.disable(name));

    match result {
        Some(true) => CommandResult::message(format!("Plugin '{}' disabled.", name)),
        Some(false) => CommandResult::error(format!("Plugin '{}' not found.", name)),
        None => CommandResult::error("Plugin registry not initialized."),
    }
}

fn plugin_info(_app: &App, name: &str) -> CommandResult {
    plugins::try_with_registry(|r| match r.get(name) {
        Some(plugin) => {
            let mut out = String::new();
            out.push_str(&format!("{}\n", plugin.manifest.plugin.name));
            out.push_str(&"=".repeat(40));
            out.push('\n');
            if let Some(desc) = &plugin.manifest.plugin.description {
                out.push_str(&format!("Description: {}\n", desc));
            }
            if let Some(version) = &plugin.manifest.plugin.version {
                out.push_str(&format!("Version: {}\n", version));
            }
            if let Some(author) = &plugin.manifest.plugin.author {
                out.push_str(&format!("Author: {}\n", author));
            }
            out.push_str(&format!(
                "Status: {}\n",
                if plugin.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            out.push_str(&format!("Path: {}\n", plugin.base_path.display()));
            if let Some(skills) = &plugin.manifest.skills {
                if let Some(path) = &skills.path {
                    out.push_str(&format!("Skills: {}\n", path));
                }
            }
            if let Some(mcp_servers) = &plugin.manifest.mcp_servers {
                out.push_str(&format!("MCP servers: {}\n", mcp_servers.len()));
                for (server_name, server) in mcp_servers {
                    let command = server.command.as_deref().unwrap_or("<url>");
                    out.push_str(&format!("  - {}: {}\n", server_name, command));
                    if !server.args.is_empty() {
                        out.push_str(&format!("    args: {}\n", server.args.join(" ")));
                    }
                    if !server.env.is_empty() {
                        out.push_str(&format!("    env vars: {}\n", server.env.len()));
                    }
                    if let Some(cwd) = &server.cwd {
                        out.push_str(&format!("    cwd: {}\n", cwd.display()));
                    }
                }
            }
            CommandResult::message(out)
        }
        None => CommandResult::error(format!("Plugin '{}' not found.", name)),
    })
    .unwrap_or_else(|| CommandResult::error("Plugin registry not initialized."))
}
