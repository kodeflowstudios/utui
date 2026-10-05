use std::io;
use serde_json::Value;
use crate::error::AppError;
use crate::project::Project;
use crate::template::Template;
use crate::command::Command;
use std::process::{Stdio};
use std::process::Command as Cmd;

pub struct UnityCLI {
    binary: String,
}

pub fn run_shell_command(cmd: &str, args: &[&str]) -> io::Result<(bool, String)> {
    let output = Cmd::new(cmd.trim())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if !stdout.is_empty() {
        return Ok((true, stdout));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((false, stderr))
}

impl UnityCLI {
    pub fn discover() -> Result<Self, AppError> {
        match run_shell_command("which", &["unity"]) {
            Ok((true, path)) => Ok(Self {
                binary: path.trim().to_string(),
            }),
            Ok((false, err)) => {
                Err(AppError::Unity(err))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn is_loggedin(&self) -> Result<(bool, String), AppError> {
        let value = self.invoke(&["auth", "status", "--json"])?;

        let data = value
            .get("data")
            .ok_or_else(|| AppError::Parse("Failed to fetch auth status!".into()))?;

        let logged_in = data
            .get("loggedIn")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if !logged_in {
            return Ok((false, String::new()));
        }

        let name = data
            .get("user")
            .and_then(|u| u.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        Ok((true, name))
    }

    pub fn login(&self) -> Result<(bool, String), AppError> {
        let value = self.invoke_ndjson(&["auth", "login", "--json"])?;

        let success = value
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if !success {
            return Ok((false, String::new()));
        }

        let name = value
            .get("data")
            .and_then(|d| d.get("user"))
            .and_then(|u| u.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        Ok((true, name))
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let json = self.invoke(&["p", "list", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().filter_map(Project::from_json).collect())
    }

    pub fn install_pipeline(&self) -> Result<(bool, Option<String>), AppError> {
        let json = self.invoke(&["pipeline", "install", "--json"])?;

        let success = json.get("success").and_then(Value::as_bool).unwrap_or(false);
        if success {
            return Ok((true, None));
        }

        let error = json
            .pointer("/errors/0")
            .map(|e| {
                let code = e.get("code").and_then(Value::as_str).unwrap_or("UNKNOWN");
                let message = e
                    .get("message")
                    .and_then(Value::as_str)
                    .and_then(|m| m.lines().next())
                    .unwrap_or("pipeline install failed");
                format!("{code}: {message}")
            })
        .unwrap_or_else(|| "UNKNOWN: pipeline install failed".to_string());

        Ok((false, Some(error)))
    }

    pub fn run_command(&self, command: String) -> Result<(bool, Option<String>), AppError> {
        let json = self.invoke(&["cmd", &command, "--json"])?;

        let outer = json.get("success").and_then(Value::as_bool).unwrap_or(false);
        let inner = json
            .pointer("/data/success")
            .and_then(Value::as_bool)
            .unwrap_or(true);

        if outer && inner {
            return Ok((true, None));
        }

        let error = json
            .pointer("/errors/0")
            .map(|e| {
                let code = e.get("code").and_then(Value::as_str).unwrap_or("UNKNOWN");
                let message = e
                    .get("message")
                    .and_then(Value::as_str)
                    .and_then(|m| m.lines().next())
                    .unwrap_or("failed to run command");
                format!("{code}: {message}")
            })
        .unwrap_or_else(|| "UNKNOWN: failed to run command".to_string());

        Ok((false, Some(error)))
    }

    pub fn list_commands(&self) -> Result<Vec<Command>, AppError> {
        let json = self.invoke(&["cmd", "--json"])?;
        Ok(json
            .pointer("/data/commands")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default())
    }

    pub fn install_editor(&self, editor: &str) -> Result<(), AppError> {
        let output = self.raw(&["install", editor, "-y", "--resume", "--json"])?;

        let json: Value = serde_json::Deserializer::from_str(output.trim())
            .into_iter::<Value>()
            .filter_map(Result::ok)
            .last()
            .ok_or_else(|| AppError::Parse(String::from("An unexpected error occured")))?;

        let reason = json
            .get("data")
            .and_then(|d| d.get("reason"))
            .and_then(Value::as_str);

        if reason == Some("no_paused_downloads") {
            self.raw(&["install", editor, "-y", "--json"])?;
        }

        Ok(())
    }

    pub fn uninstall_editor(&self, editor: &str) -> Result<(), AppError> {
        self.raw(&["uninstall", editor, "-y"])?;
        Ok(())
    }

    pub fn list_editors(&self) -> Result<Vec<(bool, String)>, AppError> {
        let json = self.invoke(&["editors", "list", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data
            .iter()
            .filter_map(|entry| {
                let version = entry.get("version").and_then(Value::as_str)?;
                let installed = entry.get("location").is_some();
                Some((installed, version.to_owned()))
            })
            .collect())
    }

    pub fn list_installed_editors(&self) -> Result<Vec<String>, AppError> {
        let json = self.invoke(&["editors", "list", "--installed", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data
            .iter()
            .filter_map(|entry| {
                entry
                    .get("version")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect())
    }

    pub fn list_templates(&self, editor_version: &str) -> Result<Vec<Template>, AppError> {
        let editor_arg = format!("--editor={editor_version}");
        let json = self.invoke(&["templates", "list", &editor_arg, "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().map(Template::from_json).collect())
    }

    pub fn list_offline_templates(&self, editor_version: &str) -> Result<Vec<Template>, AppError> {
        let editor_arg = format!("--editor={editor_version}");
        let json = self.invoke(&["templates", "list", &editor_arg, "--installed", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().map(Template::from_json).collect())
    }

    pub fn open_project(&self, project: &Project) -> Result<(), AppError> {
        let output_result = self.raw(&["p", "open", &project.path]);

        let message_text = match output_result {
            Ok(stdout_string) => stdout_string,
            Err(app_error) => {
                let err_string = app_error.to_string();
                if err_string.is_empty() {
                    "".to_string()
                } else {
                    err_string
                }
            }
        };

        parse_cli_response(&message_text)?;

        Ok(())
    }

    pub fn delete_project(&self, project: &Project, remove_files: bool) -> Result<(), AppError> {
        let output_result = self.raw(&["p", "remove", "-f", &project.path, "--json"]);

        let message_text = match output_result {
            Ok(stdout_string) => stdout_string,
            Err(app_error) => {
                let err_string = app_error.to_string();
                if err_string.is_empty() {
                    "".to_string()
                } else {
                    err_string
                }
            }
        };

        parse_cli_response(&message_text)?;

        if remove_files {
            if let Err(err) = run_shell_command("rm", &["-r", "-f", &project.path]) {
                return Err(AppError::Command(err));
            }
        }

        Ok(())
    }

    pub fn create_project(&self, project: &Project) -> Result<(), AppError> {
        let editor_arg = format!("--editor-version={}", project.editor_version);
        let template_arg = format!("--template={}", project.template);
        let path_arg = format!("--path={}", project.path);

        let output = self.raw(&[
            "p",
            "create",
            &project.name,
            &editor_arg,
            &template_arg,
            &path_arg,
            "--json",
        ])?;
        
        parse_cli_response(&output)?;
        Ok(())
    }

    fn invoke_ndjson(&self, args: &[&str]) -> Result<Value, AppError> {
        let stdout = self.raw(args)?;

        let stream = serde_json::Deserializer::from_str(&stdout).into_iter::<Value>();
        let mut last_value = None;

        for result in stream {
            match result {
                Ok(v) => last_value = Some(v),
                Err(e) => return Err(AppError::from(e)),
            }
        }

        last_value.ok_or_else(|| AppError::Parse("Empty command output".into()))
    }

    fn invoke(&self, args: &[&str]) -> Result<Value, AppError> {
        let stdout = self.raw(args)?;
        serde_json::from_str(&stdout).map_err(AppError::from)
    }

    fn raw(&self, args: &[&str]) -> Result<String, AppError> {
        match run_shell_command(&self.binary, args) {
            Ok((true, stdout)) => Ok(stdout),
            Ok((false, err)) => Err(AppError::Unity(err)),
            Err(err) => Err(err.into()),
        }
    }
}

fn json_data_array(value: &Value) -> Result<&Vec<Value>, AppError> {
    match value.get("data").and_then(Value::as_array) {
        Some(data) => Ok(data),
        None => Err(AppError::Parse("Failed to fetch json array".into())),
    }
}

fn parse_cli_response(message: &str) -> Result<(), AppError> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let json: Value = serde_json::Deserializer::from_str(trimmed)
        .into_iter::<Value>()
        .filter_map(Result::ok)
        .last()
        .ok_or_else(|| AppError::Parse(format!("No valid JSON in response: {trimmed}")))?;

    match json.get("success").and_then(Value::as_bool) {
        Some(false) => {
            let messages: Vec<String> = json
                .get("errors")
                .and_then(Value::as_array)
                .map(|errs| {
                    errs.iter()
                        .filter_map(|e| e.get("message").and_then(Value::as_str))
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();

            if messages.is_empty() {
                Err(AppError::Parse("Command failed with no error message".into()))
            } else {
                Err(AppError::Parse(messages.join("; ")))
            }
        }
        Some(true) | None => Ok(()),
    }
}
