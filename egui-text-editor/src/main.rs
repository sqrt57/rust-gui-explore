#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui::{Color32, Key, Modifiers, RichText, ScrollArea, TextEdit, TextStyle};
use rfd::FileDialog;
use std::{fs, path::PathBuf, time::Duration};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};

const APP_NAME: &str = "egui-text-editor";

fn main() -> eframe::Result<()> {
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&toggle_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon_rgba: Vec<u8> = (0u32..32 * 32)
        .flat_map(|_| [0x26u8, 0x8b, 0xd2, 0xff])
        .collect();
    let _tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon_rgba, 32, 32).unwrap())
        .with_menu(Box::new(tray_menu))
        .with_tooltip(APP_NAME)
        .build()
        .unwrap();

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 650.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(TextStyle::Monospace, egui::FontId::monospace(14.0));
            cc.egui_ctx.set_style(style);
            Ok(Box::new(App::new(toggle_id, quit_id)))
        }),
    )
}

enum PendingAction {
    New,
    Open,
    Quit,
}

enum Dialog {
    Discard(PendingAction),
    Error(String),
}

struct App {
    content: String,
    current_path: Option<PathBuf>,
    modified: bool,
    dialog: Option<Dialog>,
    visible: bool,
    toggle_id: MenuId,
    quit_id: MenuId,
}

impl App {
    fn new(toggle_id: MenuId, quit_id: MenuId) -> Self {
        Self {
            content: String::new(),
            current_path: None,
            modified: false,
            dialog: None,
            visible: true,
            toggle_id,
            quit_id,
        }
    }

    fn window_title(&self) -> String {
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

    fn do_new(&mut self) {
        self.content.clear();
        self.current_path = None;
        self.modified = false;
    }

    fn do_open(&mut self) {
        if let Some(path) = FileDialog::new().pick_file() {
            match fs::read_to_string(&path) {
                Ok(text) => {
                    self.content = text;
                    self.current_path = Some(path);
                    self.modified = false;
                }
                Err(e) => {
                    self.dialog = Some(Dialog::Error(format!("Could not open file:\n{e}")));
                }
            }
        }
    }

    fn do_save(&mut self, force_dialog: bool) {
        let path = if force_dialog { None } else { self.current_path.clone() };
        let path = match path {
            Some(p) => p,
            None => match FileDialog::new().save_file() {
                Some(p) => p,
                None => return,
            },
        };
        match fs::write(&path, &self.content) {
            Ok(_) => {
                self.current_path = Some(path);
                self.modified = false;
            }
            Err(e) => {
                self.dialog = Some(Dialog::Error(format!("Could not save file:\n{e}")));
            }
        }
    }

    fn request_file_action(&mut self, action: PendingAction) {
        if self.modified {
            self.dialog = Some(Dialog::Discard(action));
        } else {
            self.run_file_action(action);
        }
    }

    fn run_file_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::New => self.do_new(),
            PendingAction::Open => self.do_open(),
            PendingAction::Quit => unreachable!(),
        }
    }

    fn toggle_visible(&mut self, ctx: &egui::Context) {
        self.visible = !self.visible;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.visible));
        if self.visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Close button → hide to tray
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.visible = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // Poll tray / menu events
        let mut tray_quit = false;
        if let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == self.toggle_id {
                self.toggle_visible(ctx);
            } else if ev.id == self.quit_id {
                if self.modified {
                    self.dialog = Some(Dialog::Discard(PendingAction::Quit));
                } else {
                    tray_quit = true;
                }
            }
        }
        if let Ok(TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }) = TrayIconEvent::receiver().try_recv()
        {
            self.toggle_visible(ctx);
        }
        if tray_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.window_title()));

        // Keyboard shortcuts — consume regardless so editor never sees them
        let new_key = ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::N));
        let open_key = ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::O));
        let save_as_key =
            ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::S));
        let save_key = ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::S));
        let quit_key = ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Q));

        // Dialogs
        let has_dialog = self.dialog.is_some();
        let mut confirmed = false;
        let mut cancelled = false;
        let mut error_closed = false;

        match &self.dialog {
            Some(Dialog::Discard(_)) => {
                egui::Window::new("Unsaved changes")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_width(300.0);
                        ui.label("You have unsaved changes. Discard them?");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                cancelled = true;
                            }
                            if ui.button("Discard").clicked() {
                                confirmed = true;
                            }
                        });
                    });
            }
            Some(Dialog::Error(_)) => {
                let msg = match &self.dialog {
                    Some(Dialog::Error(m)) => m.clone(),
                    _ => unreachable!(),
                };
                egui::Window::new("Error")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_width(320.0);
                        ui.label(&msg);
                        ui.add_space(8.0);
                        if ui.button("OK").clicked() {
                            error_closed = true;
                        }
                    });
            }
            None => {}
        }

        let mut do_quit = false;
        if confirmed {
            if let Some(Dialog::Discard(action)) = self.dialog.take() {
                if matches!(action, PendingAction::Quit) {
                    do_quit = true;
                } else {
                    self.run_file_action(action);
                }
            }
        } else if cancelled || error_closed {
            self.dialog = None;
        }
        if do_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Menu bar
        let mut menu_new = false;
        let mut menu_open = false;
        let mut menu_save = false;
        let mut menu_save_as = false;
        let mut menu_quit = false;
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                        .clicked()
                    {
                        menu_new = true;
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Open…").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        menu_open = true;
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        menu_save = true;
                        ui.close_menu();
                    }
                    if ui
                        .add(egui::Button::new("Save As…").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        menu_save_as = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .add(egui::Button::new("Quit").shortcut_text("Ctrl+Q"))
                        .clicked()
                    {
                        menu_quit = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Process actions — skip if a dialog is open
        if !has_dialog {
            if menu_new || new_key {
                self.request_file_action(PendingAction::New);
            }
            if menu_open || open_key {
                self.request_file_action(PendingAction::Open);
            }
            if menu_save || save_key {
                self.do_save(false);
            }
            if menu_save_as || save_as_key {
                self.do_save(true);
            }
            if menu_quit || quit_key {
                if self.modified {
                    self.dialog = Some(Dialog::Discard(PendingAction::Quit));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Editor
        egui::CentralPanel::default().show(ctx, |ui| {
            let line_count = self.content.lines().count().max(1);
            let line_h =
                ui.text_style_height(&TextStyle::Monospace) + ui.spacing().item_spacing.y;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        // Line numbers gutter
                        ui.vertical(|ui| {
                            ui.set_min_width(48.0);
                            ui.set_max_width(48.0);
                            for i in 1..=line_count {
                                ui.add_sized(
                                    [48.0, line_h],
                                    egui::Label::new(
                                        RichText::new(format!("{i:>4}")).color(Color32::GRAY),
                                    ),
                                );
                            }
                        });
                        ui.separator();
                        // Text area
                        let avail_w = ui.available_width();
                        let response = ui.add(
                            TextEdit::multiline(&mut self.content)
                                .desired_rows(line_count.max(30))
                                .desired_width(avail_w)
                                .lock_focus(true)
                                .frame(false),
                        );
                        if response.changed() {
                            self.modified = true;
                        }
                    });
                });
        });

        // Keep polling for tray events between input events
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
