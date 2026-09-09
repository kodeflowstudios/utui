use arboard::Clipboard;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use rust_fuzzy_search::fuzzy_search_threshold;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::Arc;

use crate::dialogue::{Action, Dialogue, DialogueSelection, DialogueState};
use crate::error::AppError;
use crate::help::HelpState;
use crate::input::{InputHandler, InputStep};
use crate::project::Project;
use crate::template::Template;
use crate::unity::UnityCLI;
use crate::threading::{AsyncTask, poll_task};

const FETCH_TIMOUT_SEC: u64 = 15;
const FRAME_DELTA: u64 = 16;

#[derive(Default)]
pub struct Tasks {
    pub projects: Option<AsyncTask<Result<(Vec<String>, Vec<Project>), AppError>>>,
    pub all_editors: Option<AsyncTask<Result<Vec<(bool, String)>, AppError>>>,
    pub installed_editors: Option<AsyncTask<Result<Vec<String>, AppError>>>,
    pub editor_install: Option<AsyncTask<Result<(), AppError>>>,
    pub editor_uninstall: Option<AsyncTask<Result<(), AppError>>>,
    pub templates: Option<AsyncTask<Result<Vec<Template>, AppError>>>,
    pub local_templates: Option<AsyncTask<Result<Vec<Template>, AppError>>>,
    pub proj_create: Option<AsyncTask<Result<(), AppError>>>,
    pub proj_open: Option<AsyncTask<Result<(), AppError>>>,
    pub proj_delete: Option<AsyncTask<Result<(), AppError>>>,
    pub check_auth: Option<AsyncTask<Result<(bool, String), AppError>>>,
    pub login: Option<AsyncTask<Result<(bool, String), AppError>>>
}

pub enum Screen {
    ProjectList,
    EditorList,
    CommandList
}

pub struct App {
    pub screen: Screen,
    pub list_state: ListState,
    pub selected_index: Option<usize>,
    pub input: InputHandler,
    pub dialogue: DialogueState,
    pub list_items: Vec<String>,
    pub list_items_buffer: Vec<String>,
    pub projects: Vec<Project>,
    pub all_editors: Option<Vec<(bool, String)>>,
    pub installed_editors: Option<Vec<String>>,
    pub prev_editor_count: usize,
    pub templates: Option<Vec<Template>>,
    pub username: String,
    pub open_after_creation: bool,
    pub instant_timer: Instant,
    pub timer: u64,
    pub tick_counter: u64, 
    pub tasks: Tasks,
    pub declined_login: bool,
    pub show_help: bool,
    pub proj_expanded: bool,
    pub help_state: HelpState,
    unity: Option<Arc<UnityCLI>>
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::ProjectList,
            list_state: ListState::default().with_selected(Some(0)),
            selected_index: None,
            input: InputHandler::new(),
            dialogue: DialogueState::default(),
            list_items: Vec::new(),
            list_items_buffer: Vec::new(),
            projects: Vec::new(),
            all_editors: None,
            installed_editors: None,
            prev_editor_count: 0,
            templates: None,
            tasks: Tasks::default(),
            username: String::new(),
            open_after_creation: false,
            instant_timer: Instant::now(),
            timer: 0,
            tick_counter: 0,
            declined_login: false,
            proj_expanded: false,
            show_help: false,
            help_state: HelpState::new(),
            unity: None,
        }
    }

    pub fn run() -> color_eyre::Result<()> {
        let mut app = Self::new();

        app.unity = match UnityCLI::discover() {
            Ok(unity_instance) => {
                Some(Arc::new(unity_instance))
            }
            Err(_) => {
                app.dialogue.current =
                    Dialogue::Panic("UnityCLI not found, please ensure installation.".to_string());
                None
            }
        };

        if app.unity.is_some() {
            app.refresh();
        }

        let mut terminal = ratatui::init();

        let result = loop {
            terminal.draw(|frame| crate::ui::render(frame, &mut app))?;

            app.tick_counter = app.tick_counter.wrapping_add(1);

            // projects
            poll_task(
                &mut app,
                |app| &mut app.tasks.projects,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading(String::from("Loading projects..."));
                    }
                },
                |app, result| match result {
                    Ok((names, projects)) => {
                        app.list_items = names;
                        app.projects = projects;
                        app.dialogue.current = Dialogue::None;
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            // all editors
            poll_task(
                &mut app,
                |app| &mut app.tasks.all_editors,
                |app| {
                    // ensures that if the user gets to InputStep::Version while still Loading
                    // that it shows the loading prompt instead of saying that there are none.
                    if matches!(app.dialogue.current, Dialogue::Info(_)) || matches!(app.input.step, InputStep::Version) {
                        app.dialogue.current = Dialogue::Info(String::new());
                        app.update_loading(String::from("Loading editor versions..."));
                    }
                },
                |app, result| match result {
                    Ok(all_editors) => {
                        if all_editors.is_empty() {
                            app.dialogue.current = Dialogue::Error("Failed to fetch editors...".to_string());
                        } else {
                            app.list_items.clear();
                            app.all_editors = Some(all_editors.clone());

                            let editors: Vec<String> = all_editors.into_iter().map(
                                |(installed, name)| 
                                format!(
                                    "{} {}",
                                    if installed { "✔" } else { "✘" },
                                    name
                                )).collect();
                            app.list_items = editors;

                            app.dialogue.current = Dialogue::None;
                        }
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            // install editor
            poll_task(
                &mut app,
                |app| &mut app.tasks.editor_install,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading(format!(
                                "Installing editor... this may take a while.\n\nElapsed: {}", 
                                format_duration(Instant::now().saturating_duration_since(app.instant_timer))
                        ));
                    }
                },
                |app, result| match result {
                    Ok(_) => {
                        app.dialogue.current = Dialogue::TimedInfo(
                            String::from("Editor installed successfully!"),
                            Instant::now() + Duration::from_secs(3)
                        );
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            // uninstall editor
            poll_task(
                &mut app,
                |app| &mut app.tasks.editor_uninstall,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading(format!(
                                "Uninstalling editor... this may take a while.\n\nElapsed: {}", 
                                format_duration(Instant::now().saturating_duration_since(app.instant_timer))
                        ));
                    }
                },
                |app, result| match result {
                    _ => {
                        app.dialogue.current = Dialogue::TimedInfo(
                            String::from("Editor uninstalled successfully!"),
                            Instant::now() + Duration::from_secs(3)
                        );
                    }
                },
            );


            // installed editors
            poll_task(
                &mut app,
                |app| &mut app.tasks.installed_editors,
                |app| {
                    // ensures that if the user gets to InputStep::Version while still Loading
                    // that it shows the loading prompt instead of saying that there are none.
                    if matches!(app.dialogue.current, Dialogue::Info(_)) || matches!(app.input.step, InputStep::Version) {
                        app.dialogue.current = Dialogue::Info(String::new());
                        app.update_loading(String::from("Loading editor versions..."));
                    }
                },
                |app, result| match result {
                    Ok(editors) => {
                        if editors.is_empty() {
                            app.dialogue.current = Dialogue::Error("No editors available...".to_string());
                        } else {
                            app.installed_editors = Some(editors.clone());
                            app.list_items = editors;
                            if !matches!(app.dialogue.current, Dialogue::Error(_)) {
                                app.dialogue.current = Dialogue::Input;
                            }
                        }
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            let handle_templates_result = 
                |app: &mut App, result: Result<Vec<Template>, AppError>| match result {
                Ok(templates) => {
                    if templates.is_empty() {
                        app.dialogue.current = Dialogue::Error("No templates found...".to_string());
                    } else {
                        app.templates = Some(templates.clone());
                        app.dialogue.current = Dialogue::Input;
                    }
                }
                Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
            };

            // online templates
            poll_task(
                &mut app,
                |app| &mut app.tasks.templates,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) || matches!(app.input.step, InputStep::Template) {
                        app.dialogue.current = Dialogue::Info(String::new());
                        app.update_loading(String::from("Loading templates..."));
                        if app.tick_counter - app.timer >= FETCH_TIMOUT_SEC * 1000 / FRAME_DELTA
                            && app.timer > 0
                                && app.tasks.local_templates.is_none()
                        {
                            app.timer = 0;
                            app.refresh_local_template_suggestions();
                            app.tasks.templates = None;
                        }
                    }
                },
                handle_templates_result,
            );

            // local templates
            poll_task(
                &mut app,
                |app| &mut app.tasks.local_templates,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) || matches!(app.input.step, InputStep::Template) {
                        app.dialogue.current = Dialogue::Info(String::new());
                        app.update_loading(String::from("Timed out, Loading local templates..."));
                    }
                },
                handle_templates_result,
            );

            // proj_create
            poll_task(
                &mut app,
                |app| &mut app.tasks.proj_create,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) || matches!(app.input.step, InputStep::Complete) {
                        app.dialogue.current = Dialogue::Info(String::new());
                        app.update_loading(format!("Creating project... (O)pen: {}", app.open_after_creation));
                    }
                },
                |app, result| {
                    match result {
                        Ok(_) => {
                            app.dialogue.return_to = Dialogue::None;
                            if app.open_after_creation && app.tasks.proj_open.is_none() {
                                if let Some(mut project) = app.dialogue.selected_project.clone() {
                                    if let Some(unity) = &app.unity {
                                        let uclone = unity.clone();
                                        app.dialogue.current = Dialogue::Info(String::new());
                                        app.tasks.proj_open = Some(AsyncTask::new(move || {
                                            project.path = format!("{}/{}", project.path, project.name);
                                            uclone.open_project(&project)
                                        }));
                                    }
                                }
                            } else {
                                app.dialogue.current = Dialogue::TimedInfo(
                                    String::from("Project created successfully!"),
                                    Instant::now() + Duration::from_secs(3),
                                );
                            }
                        }
                        Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                    }
                    app.input.step = InputStep::Name;
                },
            );

            // proj_open
            poll_task(
                &mut app,
                |app| &mut app.tasks.proj_open,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading("Opening project...".to_string());
                    }
                },
                |app, result| match result {
                    Ok(_) => {
                        app.dialogue.current = Dialogue::TimedInfo(
                            String::from("Project opened successfully!"),
                            Instant::now() + Duration::from_secs(3),
                        );
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            // proj_delete
            poll_task(
                &mut app,
                |app| &mut app.tasks.proj_delete,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading(String::from("Deleting project..."));
                    }
                },
                |app, result| match result {
                    Ok(_) => {
                        app.dialogue.current = Dialogue::TimedInfo(
                            String::from("Project deleted successfully!"),
                            Instant::now() + Duration::from_secs(3),
                        );
                    }
                    Err(_) => app.dialogue.current = Dialogue::Error(String::from("Failed to delete...")),
                },
            );

            // login
            poll_task(
                &mut app,
                |app| &mut app.tasks.login,
                |app| {
                    if matches!(app.dialogue.current, Dialogue::Info(_)) {
                        app.update_loading(String::from("Logging in, please continue in browser..."));
                    }
                },
                |app, result| match result {
                    Ok((true, username)) => {
                        app.dialogue.current = Dialogue::TimedInfo(
                            String::from("Logged in successfully!"),
                            Instant::now() + Duration::from_secs(3),
                        );
                        app.username = username;
                    }
                    Ok((false, _)) => {
                        app.dialogue.current = Dialogue::Confirm(String::from("Failed to log in, please try again later."));
                    }
                    Err(err) => app.dialogue.current = Dialogue::Error(err.to_string()),
                },
            );

            // check_auth: no loading indicator while this runs, matching the original
            poll_task(
                &mut app,
                |app| &mut app.tasks.check_auth,
                |_app| {},
                |app, result| match result {
                    Ok((true, username)) => {
                        app.username = username;
                    }
                    Ok((false, _)) if !app.declined_login => {
                        app.username = String::from("NONE");
                        app.dialogue.selection = DialogueSelection::Ok;
                        app.dialogue.current = Dialogue::ConfirmAction(
                            String::from("You're not logged in, would you like to log in?"),
                            Action::Login,
                        );
                    }
                    Err(err) => {
                        app.username = String::from("NONE");
                        app.dialogue.current = Dialogue::Error(err.to_string());
                    },
                    _ => (),
                },
            );

            if let Dialogue::TimedInfo(_, end_time) = app.dialogue.current {
                if Instant::now() >= end_time {
                    app.dialogue.current = Dialogue::None;
                    app.refresh();
                }
            }

            if event::poll(Duration::from_millis(FRAME_DELTA))? {
                if let Some(key) = event::read()?.as_key_press_event() {
                    if app.handle_key(key) {
                        break Ok(());
                    }
                }
            }
        };

        ratatui::restore();
        result
    }

    pub fn update_loading(&mut self, message: String) {
        let spinner_frames = ["⠦", "⠖", "⠲", "⠴"];

        let index = (self.tick_counter / 8) % spinner_frames.len() as u64;
        let spinner = spinner_frames[index as usize];

        self.dialogue.current = Dialogue::Info(format!("{} {}", spinner, message));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.help_state.move_selection(true),
                KeyCode::Char('k') | KeyCode::Up => self.help_state.move_selection(false),
                KeyCode::Char('?') | KeyCode::Esc => self.show_help = false,
                KeyCode::Char('q') => return true,
                _ => {}
            }
            return false;
        }

        if key.code == KeyCode::Char('?') {
            self.show_help = true;
            return false;
        }
        
        match self.screen {
            Screen::ProjectList => {
                match self.dialogue.current {
                    Dialogue::None => self.handle_main_key(key),
                    Dialogue::Input => self.handle_input_key(key),
                    Dialogue::DeleteConfirm { .. } | Dialogue::Error(_) |
                        Dialogue::Confirm(_) | Dialogue::ConfirmAction(_, _) => self.handle_dialogue_key(key),
                    Dialogue::Info(_) => {
                        if key.code == KeyCode::Char('o') {
                            self.open_after_creation = !self.open_after_creation;
                        }
                        self.handle_dialogue_key(key)
                    },
                    Dialogue::TimedInfo(_, _) => {
                        self.dialogue.current = Dialogue::None;
                        if matches!(self.dialogue.return_to, Dialogue::None) {
                            self.refresh();
                        }
                        false
                    },
                    _ => {
                        if key.code == KeyCode::Char('q') { true }
                        else { false }
                    },
                }
            },
            Screen::EditorList => {
                match self.dialogue.current {
                    Dialogue::TimedInfo(_, _) => {
                        self.dialogue.current = Dialogue::None;
                        if matches!(self.dialogue.return_to, Dialogue::None) {
                            self.refresh();
                        }
                        false
                    },
                    _ => self.handle_editor_list_key(key)
                }
            },
            _ => false
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(true),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(true)
            }
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(false),
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(false)
            }
            KeyCode::Enter => self.toggle_project_details(),
            KeyCode::Char('D') | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.open_delete_dialogue(true)
            }
            KeyCode::Char('d') => self.open_delete_dialogue(false),
            KeyCode::Char('o') => self.open_selected_project(),
            KeyCode::Char('c') => self.open_create_dialogue(),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('e') => {
                self.screen = Screen::EditorList;
                self.refresh();
            },
            KeyCode::Esc => self.collapse_project(),
            KeyCode::Char('q') => return true,
            _ => {}
        }
        false
    }

    fn handle_editor_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(true),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(true)
            }
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(false),
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(false)
            }
            KeyCode::Char('i') => {
                if let Some((installed, version)) = self.list_state.selected()
                    .and_then(|idx| self.all_editors.as_ref()?.get(idx))
                        .map(|(i, v)| (*i, v.clone()))
                {
                    if !installed {
                        self.install_editor(version);
                    }
                }
            }
            // KeyCode::Char('m') => todo!("manage editor version modules"),
            KeyCode::Char('d') => {
                if let Some((installed, version)) = self.list_state.selected()
                    .and_then(|idx| self.all_editors.as_ref()?.get(idx))
                        .map(|(i, v)| (*i, v.clone()))
                {
                    if installed {
                        self.uninstall_editor(version);
                    }
                }
            },
            KeyCode::Esc => {
                self.screen = Screen::ProjectList;
                self.refresh();
            },
            KeyCode::Enter if self.dialogue.selection != DialogueSelection::None => self.execute_selection(),
            KeyCode::Char('q') => return true,
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => self.execute_selection(),
            KeyCode::Tab if self.input.step == InputStep::Path => self.append_path_item(),
            KeyCode::Char(to_insert) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.enter_char(to_insert)
            }
            KeyCode::Backspace => self.input.delete_char(),
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(true)
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(false)
            }
            KeyCode::Char('v')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::META) =>
            {
                self.paste_clipboard();
            }
            KeyCode::Esc => {
                self.dialogue.selection = DialogueSelection::Cancel;
                self.execute_selection();
            }
            _ => {}
        }
        false
    }

    fn handle_dialogue_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('l') | KeyCode::Right => {
                self.dialogue.selection = DialogueSelection::Cancel
            }
            KeyCode::Char('h') | KeyCode::Left => self.dialogue.selection = DialogueSelection::Ok,
            KeyCode::Esc => {
                self.dialogue.selection = DialogueSelection::Cancel;
                self.execute_selection();
            }
            KeyCode::Enter if self.dialogue.selection != DialogueSelection::None => self.execute_selection(),
            KeyCode::Char('q') => return true,
            _ => {}
        }
        false
    }

    pub fn check_auth(&mut self) {
        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            self.dialogue.current = Dialogue::Info(String::new());
            if self.tasks.check_auth.is_none() {
                self.tasks.check_auth = Some(AsyncTask::new(move || {
                    uclone.is_loggedin()
                }));
            }
        }
    }

    pub fn login(&mut self) {
        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            self.dialogue.current = Dialogue::Info(String::new());
            if self.tasks.login.is_none() {
                self.tasks.login = Some(AsyncTask::new(move || {
                    uclone.login()
                }));
            }
        }
    }

    pub fn refresh(&mut self) {
        self.proj_expanded = false;
        match self.screen {
            Screen::ProjectList => self.refresh_projects(),
            Screen::EditorList => self.refresh_editors(),
            _ => ()
        }
    }

    pub fn refresh_projects(&mut self) {
        self.list_items.clear();
        self.projects.clear();

        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            if self.tasks.projects.is_none() {
                self.tasks.projects = Some(AsyncTask::new(move || {
                    uclone.list_projects().map(|projects| {
                        let names = projects.iter().map(|p| p.name.clone()).collect();
                        (names, projects)
                    })
                }));
            }
        }

        self.check_auth();

        self.dialogue.close();
        self.list_state.select_first();
    }

    pub fn refresh_editors(&mut self) {
        self.list_items.clear();
        self.all_editors = None;

        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            if self.tasks.all_editors.is_none() {
                self.tasks.all_editors = Some(AsyncTask::new(move || {
                    uclone.list_editors()
                }));
            }
        }

        self.dialogue.close();
        self.list_state.select_first();
    }

    pub fn prepare_input_lists(&mut self) {
        if self.dialogue.current != Dialogue::Input {
            return;
        }

        match self.input.step {
            InputStep::Path => self.refresh_path_suggestions(),
            _ => {}
        }
    }

    fn refresh_path_suggestions(&mut self) {
        if self.input.value.is_empty() {
            self.input.set_text("/");
        }

        if self.input.value.ends_with('/') {
            self.list_items = dir_contents(&self.input.value);
            self.list_items
                .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            self.list_items_buffer = self.list_items.clone();
        } else {
            let query = self
                .input
                .value
                .rsplit_once('/')
                .map(|(_, text)| text)
                .unwrap_or("");
            self.list_items_buffer = fuzzy_filter_sorted(query, self.list_items.clone());
        }

        if self.list_state.selected().is_none() && self.list_items_buffer.len() == 1 {
            self.list_state.select_first();
        }
    }

    fn refresh_version_suggestions(&mut self) {
        if self.installed_editors.is_none() {
            if self.tasks.installed_editors.is_none() {
                if let Some(unity) = &self.unity {
                    let uclone = unity.clone();

                    self.tasks.installed_editors = Some(AsyncTask::new(move || {
                        uclone.list_installed_editors()
                    }));
                }
            }
        }

        self.list_items = fuzzy_filter_sorted(&self.input.value, self.list_items.clone());
    }

    fn refresh_template_suggestions(&mut self) {
        if self.templates.is_none() {
            if self.tasks.templates.is_none() {
                if let Some(unity) = &self.unity {
                    let uclone = unity.clone();
                    let version = self
                        .dialogue
                        .selected_project
                        .as_ref()
                        .map(|project| project.editor_version.clone())
                        .unwrap_or_default();

                    if self.tasks.templates.is_none() {
                        self.tasks.templates = Some(AsyncTask::new(move || {
                            uclone.list_templates(&version)
                        }));
                        self.timer = self.tick_counter;
                    }
                }
            }
        }
    }

    fn refresh_local_template_suggestions(&mut self) {
        if self.templates.is_none() {
            if self.tasks.local_templates.is_none() {
                if let Some(unity) = &self.unity {
                    let uclone = unity.clone();
                    let version = self
                        .dialogue
                        .selected_project
                        .as_ref()
                        .map(|project| project.editor_version.clone())
                        .unwrap_or_default();

                    if self.tasks.local_templates.is_none() {
                        self.tasks.local_templates = Some(AsyncTask::new(move || {
                            uclone.list_offline_templates(&version)
                        }));
                    }
                }
            }
        }
    }

    pub fn template_labels(&self) -> Vec<String> {
        let labels = self
            .templates
            .as_ref()
            .map(|templates| templates.iter().map(Template::list_label).collect())
            .unwrap_or_default();
        fuzzy_filter_sorted(&self.input.value, labels)
    }

    fn finish_create(&mut self) {
        if let Some(project) = self.dialogue.selected_project.clone() {
            if self.tasks.proj_create.is_none() {
                if let Some(unity) = &self.unity {
                    let uclone = unity.clone();
                    self.tasks.proj_create = Some(AsyncTask::new(move || {
                        uclone.create_project(&project)
                    }));
                }
            }
        }
    }

    fn install_editor(&mut self, version: String) {
        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            self.dialogue.current = Dialogue::Info(String::new());
            if self.tasks.editor_install.is_none() {
                self.instant_timer = Instant::now();
                self.tasks.editor_install = Some(AsyncTask::new(move || {
                    uclone.install_editor(&version)
                }));
            }
        }
    }

    fn uninstall_editor(&mut self, version: String) {
        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            self.dialogue.current = Dialogue::Info(String::new());
            if self.tasks.editor_uninstall.is_none() {
                self.instant_timer = Instant::now();
                self.tasks.editor_uninstall = Some(AsyncTask::new(move || {
                    uclone.uninstall_editor(&version)
                }));
            }
        }
    }

    pub fn execute_selection(&mut self) {
        match self.dialogue.current {
            Dialogue::DeleteConfirm { with_dir } => {
                if self.dialogue.selection == DialogueSelection::Ok {
                    self.delete_selected(with_dir);
                }
            }
            Dialogue::Info(_) => (),
            Dialogue::Error(_) => {
                if self.dialogue.return_to != Dialogue::None { 
                    self.dialogue.current = self.dialogue.return_to.clone();
                    return; 
                }
                self.clear_tasks();
                self.reset_after_dialogue();
                self.input.submit_message();
                self.refresh();
            }
            Dialogue::Input => {
                if self.dialogue.selection == DialogueSelection::Cancel {
                    if self.dialogue.return_to == Dialogue::None { 
                        self.clear_tasks();
                    }
                    self.reset_after_dialogue();
                    self.input.submit_message();
                    self.refresh();
                    return;
                }

                if self.input.value.is_empty() && !matches!(self.input.step, InputStep::Template) {
                    self.dialogue.return_to = self.dialogue.current.clone();
                    self.dialogue.current = Dialogue::Error("No input!".to_string());
                    return;
                }

                if self.input.buff.is_empty() && matches!(self.input.step, InputStep::Template) {
                    self.dialogue.return_to = self.dialogue.current.clone();
                    self.dialogue.current = Dialogue::Error("No selection!".to_string());
                    return;
                }

                if !self.advance_create_step() {
                    return;
                }

                self.input.submit_message();
                self.list_state.select(None);
                if self.input.step == InputStep::Complete {
                    self.finish_create();
                }
            }
            Dialogue::TimedInfo(_, _) => self.reset_after_dialogue(),
            Dialogue::Confirm(_) => {
                self.reset_after_dialogue(); 
                self.refresh();
            },
            Dialogue::ConfirmAction(_, action) => {
                match action {
                    Action::Login => {
                        if matches!(self.dialogue.selection, DialogueSelection::Ok) {
                            self.login()
                        }
                        else if matches!(self.dialogue.selection, DialogueSelection::Cancel) {
                            self.declined_login = true;
                            self.reset_after_dialogue();
                        }
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }

    fn clear_tasks(&mut self) {
        self.tasks.projects = None;
        self.tasks.installed_editors = None;
        self.tasks.templates = None;
        self.tasks.proj_open = None;
        self.tasks.proj_create = None;
        self.tasks.proj_delete = None;
    }

    fn advance_create_step(&mut self) -> bool {
        match self.input.step {
            InputStep::Name => {
                let name = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.name = name;
                }
                self.input.step = InputStep::Path;
                true
            }
            InputStep::Path => {
                let path = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    if let contents = dir_contents(&path) {
                        if contents.contains(&project.name) {
                            self.dialogue.return_to = Dialogue::Input;
                            self.dialogue.current = 
                                Dialogue::Error("Project already exists at path!".to_string());
                            self.input.step = InputStep::Name;
                            self.input.submit_message();
                            return false;
                        }
                    }
                    // BUG: we never actually verify the path!
                    project.path = path;
                }
                self.input.step = InputStep::Version;
                true
            }
            InputStep::Version => {
                if let Some(versions) = self.installed_editors.clone() {
                    if !versions.contains(&self.input.value) {
                        self.dialogue.return_to = self.dialogue.current.clone();
                        self.dialogue.current = Dialogue::Error("Please select a valid version!".to_string());
                        return false;
                    }
                }
                let version = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.editor_version = version;
                }
                self.input.step = InputStep::Template;
                self.refresh_template_suggestions();
                true
            }
            InputStep::Template => {
                if let (Some(templates), Some(index)) =
                    (self.templates.as_ref(), self.list_state.selected())
                {
                    let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
                    if !names.contains(&self.input.buff) {
                        return false;
                    }
                }
                let template = self.input.buff.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.template = template;
                }
                self.input.step = InputStep::Complete;
                true
            }
            _ => false,
        }
    }

    pub fn append_path_item(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select_first();
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(value) = self.list_items_buffer.get(index).cloned() else {
            return;
        };

        let prefix_end = self.input.value.rfind('/').unwrap_or(0);
        let mut path = self.input.value[..=prefix_end].to_string();
        path.push_str(&value);
        path.push('/');
        self.input.set_text(&path);
        self.list_state.select(None);
        self.list_items.clear();
    }

    pub fn move_selection(&mut self, next: bool) {
        if self.list_items.is_empty() {
            return;
        }

        if let Some(index) = self.list_state.selected() &&
            matches!(self.screen, Screen::ProjectList){
            let should_collapse = self.dialogue.current == Dialogue::None
                && if next {
                    index < self.list_items.len() - 1
                } else {
                    index > 0
                };
            if should_collapse {
                self.collapse_project();
            }
        }

        if next {
            self.list_state.select_next();
        } else {
            self.list_state.select_previous();
        }

        match self.input.step {
            InputStep::Version => {
                if let Some(index) = self.list_state.selected()
                {
                    if let Some(value) = self.list_items.get(index) {
                        self.input.set_text(value.trim());
                    }
                }
            },
            InputStep::Template => {
                if let (Some(templates), Some(index)) = (&self.templates, self.list_state.selected()) {
                    let t_str: Vec<String> = templates.iter().map(|t| t.name.clone() as String).collect();
                    if let Some(value) = t_str.get(index).cloned() {
                        self.input.set_buffer(value.trim());
                    }
                }
            },
            _ => ()
        }
    }

    fn open_create_dialogue(&mut self) {
        self.open_after_creation = false;
        self.selected_index = self.list_state.selected();
        self.list_state.select(None);
        self.dialogue.current = Dialogue::Input;
        self.dialogue.selected_project = Some(Project::default());
        self.dialogue.selection = DialogueSelection::Ok;
        self.input.step = InputStep::Name;
        self.installed_editors = None;
        self.templates = None;
        self.refresh_version_suggestions()
    }

    fn open_delete_dialogue(&mut self, with_dir: bool) {
        if self.list_items.is_empty() {
            return;
        }

        if let Some(index) = self.list_state.selected() {
            if let Some(project) = self.projects.get(index).cloned() {
                self.dialogue.current = Dialogue::DeleteConfirm { with_dir };
                self.dialogue.selected_project = Some(project);
                self.dialogue.selection = DialogueSelection::Cancel;
            }
        }

        self.selected_index = self.list_state.selected();
        self.list_state.select(None);
    }

    fn open_selected_project(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(project) = self.projects.get(index).cloned() else {
            return;
        };

        if self.tasks.proj_open.is_none() {
            if let Some(unity) = &self.unity {
                let uclone = unity.clone();
                self.tasks.proj_open = Some(AsyncTask::new(move || {
                    uclone.open_project(&project)
                }));
            }
        }
    }

    fn delete_selected(&mut self, with_dir: bool) {
        let Some(project) = self.dialogue.selected_project.clone() else {
            return;
        };

        if self.tasks.proj_delete.is_none() {
            if let Some(unity) = &self.unity {
                let uclone = unity.clone();
                self.dialogue.current = Dialogue::Info(String::new());
                self.tasks.proj_delete = Some(AsyncTask::new(move || {
                    uclone.delete_project(&project, with_dir)
                }));
            }
        }
    }

    fn toggle_project_details(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };

        if self.list_items[index].contains('\n') {
            self.collapse_project();
            return;
        }

        if let Some(project) = self.projects.get(index) {
            self.list_items[index] = project.details_text();
            self.proj_expanded = true;
        }
    }

    fn collapse_project(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };

        if let Some(project) = self.projects.get(index) {
            self.list_items[index] = project.name.clone();
            self.proj_expanded = false;
        }
    }

    fn reset_after_dialogue(&mut self) {
        self.list_state.select(self.selected_index);
        self.dialogue.close();
        self.input.step = InputStep::Name;
        self.list_state.select_first();
    }

    fn paste_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                self.input.set_text(&text);
            }
        }
    }
}

fn dir_contents(dir_path: &str) -> Vec<String> {
    fs::read_dir(dir_path)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

pub fn fuzzy_filter_sorted(query: &str, list_items: Vec<String>) -> Vec<String> {
    if query.is_empty() {
        return list_items;
    }

    let refs: Vec<&str> = list_items.iter().map(String::as_str).collect();
    let mut results = fuzzy_search_threshold(query, &refs, 0.5);
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
        .into_iter()
        .map(|(word, _)| word.to_string())
        .collect()
}

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;

    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}
