use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::mpsc::{Sender, channel};
use tracing::{debug, error, info};

use crate::{CustomCommand, server::hx::Media};

pub struct CustomCommandTrigger {
    pub cmd: String,
    pub media: Media,
}

pub fn setup_custom_commands(commands: Vec<CustomCommand>) -> Sender<CustomCommandTrigger> {
    let (tx, mut rx) = channel::<CustomCommandTrigger>(100);

    tokio::spawn(async move {
        while let Some(trigger) = rx.recv().await {
            if let Some(config) = commands.iter().find(|c| c.name == trigger.cmd) {
                if config.cmd.is_empty() {
                    error!("Command '{}' has no executable defined", config.name);
                    continue;
                }

                let processed_cmd: Vec<String> = config
                    .cmd
                    .iter()
                    .map(|arg| {
                        arg.replace("{path}", &trigger.media.path)
                            .replace("{taken_at}", &trigger.media.taken_at.to_string())
                            .replace("{uuid}", &trigger.media.uuid.to_string())
                            .replace("{url}", &trigger.media.url)
                            .replace("{thumbnail}", &trigger.media.thumbnail)
                    })
                    .collect();

                let program = &processed_cmd[0];
                let args = &processed_cmd[1..];

                info!(
                    "Executing custom command '{}': {} {:?}",
                    config.name, program, args
                );

                match Command::new(program)
                    .args(args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(child) => match child.wait_with_output().await {
                        Ok(out) => {
                            if out.status.success() {
                                info!("Command '{}' finished successfully", config.name);
                                if !out.stdout.is_empty() {
                                    debug!("Stdout: {}", String::from_utf8_lossy(&out.stdout));
                                }
                            } else {
                                error!(
                                    "Command '{}' failed with exit code: {:?}",
                                    config.name,
                                    out.status.code()
                                );
                                if !out.stderr.is_empty() {
                                    error!("Stderr: {}", String::from_utf8_lossy(&out.stderr));
                                }
                            }
                        }
                        Err(e) => error!("Failed to wait for command '{}': {}", config.name, e),
                    },
                    Err(e) => error!("Failed to spawn command '{}': {}", config.name, e),
                }
            } else {
                error!("Custom command definition not found for: {}", trigger.cmd);
            }
        }
    });

    tx
}
