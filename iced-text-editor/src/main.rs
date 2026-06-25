#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    Border, Color, Element, Font, Length, Padding, Size, Subscription, Task,
    event,
    keyboard::{self, Key},
    widget::{
        button, column, container, horizontal_rule, row, text, text_editor,
        Column, Space,
    },
    window,
};
use rfd::AsyncFileDialog;
use std::{path::PathBuf, time::Duration};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

const APP_NAME: &str = "iced-text-editor";

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&toggle_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon_rgba: Vec<u8> = (0..32 * 32).flat_map(|_| [0x26u8, 0x8b, 0xd2, 0xff]).collect();
    let tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon_rgba, 32, 32).unwrap())
        .with_menu(Box::new(tray_menu))
        .with_tooltip(APP_NAME)
        .build()
        .unwrap();
    // Keep alive for the program's lifetime.
    Box::leak(Box::new(tray));

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .exit_on_close_request(false)
        .window(window::Settings {
            size: Size::new(900.0, 650.0),
            ..Default::default()
        })
        .run_with(move || {
            let app = App::new(toggle_id, quit_id);
            let task = window::get_latest().map(|id| Message::WindowIdReady(id));
            (app, task)
        })
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileMenuToggle,
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    Quit,
    // Async results
    FileOpened(Option<(PathBuf, String)>),
    FileSaved(Option<PathBuf>), // None = cancelled
    SaveError(String),
    // Dialog
    DiscardConfirmed,
    DiscardCancelled,
    ErrorDismissed,
    // Window / tray
    WindowIdReady(Option<window::Id>),
    CloseRequested,
    TrayPoll,
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug)]
enum PendingAction {
    New,
    Open,
    Quit,
}

#[derive(Debug)]
enum Dialog {
    Discard(PendingAction),
    Error(String),
}

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    content: text_editor::Content,
    current_path: Option<PathBuf>,
    modified: bool,
    dialog: Option<Dialog>,
    file_menu_open: bool,
    window_id: Option<window::Id>,
    is_hidden: bool,
    toggle_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
}

impl App {
    fn new(
        toggle_id: tray_icon::menu::MenuId,
        quit_id: tray_icon::menu::MenuId,
    ) -> Self {
        Self {
            content: text_editor::Content::new(),
            current_path: None,
            modified: false,
            dialog: None,
            file_menu_open: false,
            window_id: None,
            is_hidden: false,
            toggle_id,
            quit_id,
        }
    }

    fn title(&self) -> String {
        let name = self
            .current_path
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string());
        match (self.current_path.is_some(), self.modified) {
            (_, true) => format!("• {} — {APP_NAME}", name),
            (true, false) => format!("{} — {APP_NAME}", name),
            (false, false) => APP_NAME.to_string(),
        }
    }
}

// ── Update ────────────────────────────────────────────────────────────────────

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                if matches!(action, text_editor::Action::Edit(_)) {
                    self.modified = true;
                }
                self.content.perform(action);
                self.file_menu_open = false;
                Task::none()
            }

            Message::FileMenuToggle => {
                self.file_menu_open = !self.file_menu_open;
                Task::none()
            }

            Message::WindowIdReady(id) => {
                self.window_id = id;
                Task::none()
            }

            Message::CloseRequested => self.hide_window(),

            Message::TrayPoll => self.poll_tray(),

            Message::NewFile => {
                self.file_menu_open = false;
                if self.modified {
                    self.dialog = Some(Dialog::Discard(PendingAction::New));
                    Task::none()
                } else {
                    self.do_new();
                    Task::none()
                }
            }

            Message::OpenFile => {
                self.file_menu_open = false;
                if self.modified {
                    self.dialog = Some(Dialog::Discard(PendingAction::Open));
                    Task::none()
                } else {
                    self.open_file_task()
                }
            }

            Message::SaveFile => {
                self.file_menu_open = false;
                self.save_file_task(false)
            }

            Message::SaveFileAs => {
                self.file_menu_open = false;
                self.save_file_task(true)
            }

            Message::Quit => {
                self.file_menu_open = false;
                if self.modified {
                    self.dialog = Some(Dialog::Discard(PendingAction::Quit));
                    Task::none()
                } else {
                    iced::exit()
                }
            }

            Message::FileOpened(result) => {
                if let Some((path, text)) = result {
                    self.content = text_editor::Content::with_text(&text);
                    self.current_path = Some(path);
                    self.modified = false;
                }
                Task::none()
            }

            Message::FileSaved(path) => {
                if let Some(p) = path {
                    self.current_path = Some(p);
                    self.modified = false;
                }
                Task::none()
            }

            Message::SaveError(msg) => {
                self.dialog = Some(Dialog::Error(msg));
                Task::none()
            }

            Message::DiscardConfirmed => {
                let action = match self.dialog.take() {
                    Some(Dialog::Discard(a)) => a,
                    _ => return Task::none(),
                };
                match action {
                    PendingAction::New => {
                        self.do_new();
                        Task::none()
                    }
                    PendingAction::Open => self.open_file_task(),
                    PendingAction::Quit => iced::exit(),
                }
            }

            Message::DiscardCancelled | Message::ErrorDismissed => {
                self.dialog = None;
                Task::none()
            }
        }
    }

    fn do_new(&mut self) {
        self.content = text_editor::Content::new();
        self.current_path = None;
        self.modified = false;
    }

    fn open_file_task(&self) -> Task<Message> {
        Task::perform(
            async {
                let handle = AsyncFileDialog::new().pick_file().await?;
                let path = handle.path().to_owned();
                let text = std::fs::read_to_string(&path).ok()?;
                Some((path, text))
            },
            Message::FileOpened,
        )
    }

    fn save_file_task(&self, force_dialog: bool) -> Task<Message> {
        let path = if force_dialog { None } else { self.current_path.clone() };
        let text = self.content.text();
        Task::perform(
            async move {
                let save_path = match path {
                    Some(p) => p,
                    None => match AsyncFileDialog::new().save_file().await {
                        Some(h) => h.path().to_owned(),
                        None => return Ok(None),
                    },
                };
                std::fs::write(&save_path, &text)
                    .map(|_| Some(save_path))
                    .map_err(|e| e.to_string())
            },
            |result| match result {
                Ok(path) => Message::FileSaved(path),
                Err(e) => Message::SaveError(e),
            },
        )
    }

    fn hide_window(&mut self) -> Task<Message> {
        self.is_hidden = true;
        if let Some(id) = self.window_id {
            window::change_mode(id, window::Mode::Hidden)
        } else {
            Task::none()
        }
    }

    fn show_window(&mut self) -> Task<Message> {
        self.is_hidden = false;
        if let Some(id) = self.window_id {
            window::change_mode(id, window::Mode::Windowed)
        } else {
            Task::none()
        }
    }

    fn poll_tray(&mut self) -> Task<Message> {
        let mut do_toggle = false;
        let mut do_quit = false;

        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == self.toggle_id {
                do_toggle = true;
            } else if ev.id == self.quit_id {
                do_quit = true;
            }
        }
        while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                do_toggle = true;
            }
        }

        if do_quit {
            if self.modified {
                self.dialog = Some(Dialog::Discard(PendingAction::Quit));
                return self.show_window();
            } else {
                return iced::exit();
            }
        }
        if do_toggle {
            if self.is_hidden {
                return self.show_window();
            } else {
                return self.hide_window();
            }
        }
        Task::none()
    }
}

// ── View ──────────────────────────────────────────────────────────────────────

impl App {
    fn view(&self) -> Element<'_, Message> {
        let main: Element<Message> =
            column![self.view_menu_bar(), self.view_editor()].into();

        // Stack dropdown and/or dialog overlays on top.
        let mut layers: Vec<Element<Message>> = vec![main];

        if self.file_menu_open {
            layers.push(self.view_dropdown_overlay());
        }

        if let Some(dialog) = &self.dialog {
            layers.push(self.view_dialog(dialog));
        }

        if layers.len() == 1 {
            layers.pop().unwrap()
        } else {
            iced::widget::Stack::with_children(layers).into()
        }
    }

    fn view_menu_bar(&self) -> Element<'_, Message> {
        let file_btn = button(text("File").size(13))
            .on_press(Message::FileMenuToggle)
            .style(button::text);

        container(row![file_btn].align_y(iced::Alignment::Center))
            .height(28)
            .width(Length::Fill)
            .style(|theme: &iced::Theme| container::Style {
                border: Border {
                    color: theme.palette().text.scale_alpha(0.15),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_dropdown_overlay(&self) -> Element<'_, Message> {
        let items: Element<Message> = container(
            column![
                menu_item("New", "Ctrl+N", Message::NewFile),
                menu_item("Open…", "Ctrl+O", Message::OpenFile),
                menu_item("Save", "Ctrl+S", Message::SaveFile),
                menu_item("Save As…", "Ctrl+Shift+S", Message::SaveFileAs),
                horizontal_rule(1),
                menu_item("Quit", "Ctrl+Q", Message::Quit),
            ]
            .spacing(1)
            .padding(4)
            .width(220),
        )
        .style(|theme: &iced::Theme| container::Style {
            background: Some(theme.palette().background.into()),
            border: Border {
                color: theme.palette().text.scale_alpha(0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            ..Default::default()
        })
        .into();

        // Position below the 28px menu bar, inset from left edge.
        column![
            Space::with_height(28),
            row![Space::with_width(4), items],
        ]
        .into()
    }

    fn view_editor(&self) -> Element<'_, Message> {
        let line_count = self.content.line_count();

        let line_nums: Element<Message> = container(
            (1..=line_count)
                .fold(Column::new().spacing(0), |col, i| {
                    col.push(
                        text(format!("{i:>4}"))
                            .font(Font::MONOSPACE)
                            .size(14)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    )
                })
                .padding(Padding { top: 4.0, left: 4.0, right: 4.0, bottom: 4.0 }),
        )
        .width(52)
        .style(|theme: &iced::Theme| container::Style {
            border: Border {
                color: theme.palette().text.scale_alpha(0.1),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into();

        let editor: Element<Message> = text_editor(&self.content)
            .on_action(Message::Edit)
            .font(Font::MONOSPACE)
            .size(14)
            .height(Length::Fill)
            .into();

        row![line_nums, editor].height(Length::Fill).into()
    }

    fn view_dialog(&self, dialog: &Dialog) -> Element<'_, Message> {
        let inner: Element<Message> = match dialog {
            Dialog::Discard(_) => container(
                column![
                    text("You have unsaved changes. Discard them?"),
                    row![
                        button("Cancel").on_press(Message::DiscardCancelled),
                        button("Discard").on_press(Message::DiscardConfirmed),
                    ]
                    .spacing(8),
                ]
                .spacing(16)
                .padding(20)
                .width(320),
            )
            .style(container::rounded_box)
            .into(),

            Dialog::Error(msg) => container(
                column![
                    text(msg.clone()),
                    button("OK").on_press(Message::ErrorDismissed),
                ]
                .spacing(16)
                .padding(20)
                .width(320),
            )
            .style(container::rounded_box)
            .into(),
        };

        container(inner)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.4).into()),
                ..Default::default()
            })
            .into()
    }
}

// ── Subscription ──────────────────────────────────────────────────────────────

impl App {
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(16)).map(|_| Message::TrayPoll),
            event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => keyboard_shortcut(key.as_ref(), modifiers),
                iced::Event::Window(window::Event::CloseRequested) => {
                    Some(Message::CloseRequested)
                }
                _ => None,
            }),
        ])
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn keyboard_shortcut(key: Key<&str>, mods: keyboard::Modifiers) -> Option<Message> {
    match (mods.control(), mods.shift(), key) {
        (true, false, Key::Character("n")) => Some(Message::NewFile),
        (true, false, Key::Character("o")) => Some(Message::OpenFile),
        (true, false, Key::Character("s")) => Some(Message::SaveFile),
        (true, true, Key::Character("s")) => Some(Message::SaveFileAs),
        (true, false, Key::Character("q")) => Some(Message::Quit),
        _ => None,
    }
}

fn menu_item(label: &str, shortcut: &str, msg: Message) -> Element<'static, Message> {
    button(
        row![
            text(label.to_string()).width(Length::Fill),
            text(shortcut.to_string())
                .size(11)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .width(210),
    )
    .on_press(msg)
    .style(button::text)
    .width(Length::Fill)
    .into()
}
