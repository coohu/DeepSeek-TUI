//! Utility command area: attachments, background tasks, jobs, MCP, and
//! network inspection.

mod attachment;
mod jobs;
mod mcp;
mod network;
mod task;

use crate::commands::traits::{Command, CommandGroup, CommandInfo, FunctionCommand, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use crate::commands::CommandResult;

pub struct UtilityCommands;

impl CommandGroup for UtilityCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(
                attachment::AttachCmd::info(),
                attachment::AttachCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                task::TaskCmd::info(),
                task::TaskCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                jobs::JobsCmd::info(),
                jobs::JobsCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                mcp::McpCmd::info(),
                mcp::McpCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                network::NetworkCmd::info(),
                network::NetworkCmd::execute,
            )),
            // `/plugins` is registered here but remains discretionary for
            // FEAT-007 extraction. The CommandInfo and dispatch function
            // stay inline until scope is explicitly widened.
            Box::new(FunctionCommand::new(&PLUGINS_INFO, run_plugins)),
        ]
    }
}

/// `/plugins` command metadata — discretionary, kept inline.
static PLUGINS_INFO: CommandInfo = CommandInfo {
    name: "plugins",
    aliases: &["plugin"],
    usage: "/plugins [name]",
    description_id: MessageId::CmdPluginDescription,
};

fn run_plugins(app: &mut App, arg: Option<&str>) -> CommandResult {
    crate::commands::plugins::plugins(app, arg)
}
