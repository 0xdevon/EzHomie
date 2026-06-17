use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    OpenApp {
        name: String,
        delay_ms: u64,
    },
    OpenUrl {
        url: String,
        delay_ms: u64,
    },
    RunCommand {
        name: Option<String>,
        cmd: String,
        delay_ms: u64,
    },
    QuitApp {
        name: String,
        timeout_secs: u64,
    },
}

impl Action {
    pub fn describe(&self) -> String {
        match self {
            Self::OpenApp { name, delay_ms } => {
                format!("open app: {name}{}", describe_delay(*delay_ms))
            }
            Self::OpenUrl { url, delay_ms } => {
                format!("open url: {url}{}", describe_delay(*delay_ms))
            }
            Self::RunCommand {
                name,
                cmd,
                delay_ms,
            } => {
                let label = name.as_deref().unwrap_or(cmd);
                format!("run command: {label}{}", describe_delay(*delay_ms))
            }
            Self::QuitApp { name, timeout_secs } => {
                format!("quit app: {name} (timeout {timeout_secs}s)")
            }
        }
    }

    pub fn delay_ms(&self) -> u64 {
        match self {
            Self::OpenApp { delay_ms, .. } => *delay_ms,
            Self::OpenUrl { delay_ms, .. } => *delay_ms,
            Self::RunCommand { delay_ms, .. } => *delay_ms,
            Self::QuitApp { .. } => 0,
        }
    }
}

pub fn build_on_plan(config: &Config) -> Vec<Action> {
    let mut actions =
        Vec::with_capacity(config.apps.len() + config.urls.len() + config.commands.len());

    actions.extend(config.apps.iter().map(|app| Action::OpenApp {
        name: app.name.clone(),
        delay_ms: app.delay_ms,
    }));
    actions.extend(config.urls.iter().map(|url| Action::OpenUrl {
        url: url.url.clone(),
        delay_ms: url.delay_ms,
    }));
    actions.extend(config.commands.iter().map(|command| Action::RunCommand {
        name: command.name.clone(),
        cmd: command.cmd.clone(),
        delay_ms: command.delay_ms,
    }));

    actions
}

pub fn build_off_plan(config: &Config) -> Vec<Action> {
    config
        .apps
        .iter()
        .rev()
        .map(|app| Action::QuitApp {
            name: app.name.clone(),
            timeout_secs: app.timeout_secs.unwrap_or(config.off.quit_timeout_secs),
        })
        .collect()
}

fn describe_delay(delay_ms: u64) -> String {
    if delay_ms == 0 {
        String::new()
    } else {
        format!(" (delay {delay_ms}ms)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppTarget, CommandTarget, Config, OffConfig, UrlTarget};

    #[test]
    fn builds_on_plan_in_config_order() {
        let config = Config {
            off: OffConfig::default(),
            apps: vec![AppTarget {
                name: "Cursor".to_string(),
                delay_ms: 100,
                timeout_secs: None,
            }],
            urls: vec![UrlTarget {
                url: "https://example.com".to_string(),
                delay_ms: 200,
            }],
            commands: vec![CommandTarget {
                name: Some("dev".to_string()),
                cmd: "npm run dev".to_string(),
                delay_ms: 300,
            }],
        };

        assert_eq!(
            build_on_plan(&config),
            vec![
                Action::OpenApp {
                    name: "Cursor".to_string(),
                    delay_ms: 100
                },
                Action::OpenUrl {
                    url: "https://example.com".to_string(),
                    delay_ms: 200
                },
                Action::RunCommand {
                    name: Some("dev".to_string()),
                    cmd: "npm run dev".to_string(),
                    delay_ms: 300
                }
            ]
        );
    }

    #[test]
    fn builds_off_plan_in_reverse_app_order() {
        let mut config = Config::default();
        config.apps = vec![
            AppTarget {
                name: "A".to_string(),
                delay_ms: 0,
                timeout_secs: None,
            },
            AppTarget {
                name: "B".to_string(),
                delay_ms: 0,
                timeout_secs: Some(3),
            },
        ];

        assert_eq!(
            build_off_plan(&config),
            vec![
                Action::QuitApp {
                    name: "B".to_string(),
                    timeout_secs: 3
                },
                Action::QuitApp {
                    name: "A".to_string(),
                    timeout_secs: config.off.quit_timeout_secs
                }
            ]
        );
    }
}
