use egui::{CollapsingHeader, Id, Response, ScrollArea, Ui};
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use grafiek_engine::StringKind;

const INLINE_MAX_HEIGHT: f32 = 200.0;
const INLINE_ROWS: usize = 8;
const INLINE_FONT_SIZE: f32 = 12.0;
const POPUP_ROWS: usize = 20;
const POPUP_FONT_SIZE: f32 = 13.0;
const POPUP_SIZE: [f32; 2] = [600.0, 400.0];

pub fn code_editor_field(ui: &mut Ui, id: Id, code: &mut String, kind: &StringKind) -> Response {
    let popup_id = Id::new(("code_popup", id));
    let popup_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
    let syntax = syntax_for_kind(kind);
    let line_count = code.lines().count();

    let response = ui
        .vertical(|ui| {
            CollapsingHeader::new(format!("Code ({line_count} lines)"))
                .id_salt(id)
                .default_open(false)
                .show(ui, |ui| {
                    // Toolbar
                    ui.horizontal(|ui| {
                        if ui.small_button("Detach").clicked() {
                            ui.data_mut(|d| d.insert_temp(popup_id, true));
                        }
                        if ui.small_button("Open External").clicked() {
                            log::info!("External editor not yet implemented");
                        }
                    });
                    ui.add_space(4.0);

                    // Inline editor
                    ScrollArea::vertical()
                        .max_height(INLINE_MAX_HEIGHT)
                        .show(ui, |ui| {
                            make_editor(
                                &format!("{id:?}_inline"),
                                INLINE_ROWS,
                                INLINE_FONT_SIZE,
                                &syntax,
                            )
                            .show(ui, code);
                        });
                });
        })
        .response;

    // Popup window (rendered at context level)
    if popup_open {
        let mut open = true;
        egui::Window::new("Code Editor")
            .id(Id::new(("code_window", id)))
            .open(&mut open)
            .default_size(POPUP_SIZE)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ScrollArea::both().show(ui, |ui| {
                    make_editor(
                        &format!("{id:?}_popup"),
                        POPUP_ROWS,
                        POPUP_FONT_SIZE,
                        &syntax,
                    )
                    .show(ui, code);
                });
            });
        if !open {
            ui.data_mut(|d| d.insert_temp(popup_id, false));
        }
    }

    response
}

fn make_editor(id: &str, rows: usize, font_size: f32, syntax: &Option<Syntax>) -> CodeEditor {
    let mut editor = CodeEditor::default()
        .id_source(id)
        .with_rows(rows)
        .with_fontsize(font_size)
        .with_theme(ColorTheme::GRUVBOX_DARK)
        .with_numlines(true);
    if let Some(s) = syntax {
        editor = editor.with_syntax(s.clone());
    }
    editor
}

fn syntax_for_kind(_kind: &StringKind) -> Option<Syntax> {
    None
}
