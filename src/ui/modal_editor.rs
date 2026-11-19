//! Native modal text surface. Keystrokes change only the source-bound simulator.
//! Tab remains application navigation; Escape changes mode without leaving focus.
use super::*;
use dlms_learning_lab::{
    modal_editing::{Key, Mode},
    modal_workspace::{Attempt, Event, Fixture},
};
#[derive(Default)]
pub(super) struct Output {
    pub surface: Option<(egui::Rect, egui::Rect, egui::Id)>,
    pub cursor: Option<egui::Rect>,
    pub diagnostic: Option<String>,
    pub changed: bool,
}
fn apply(a: &mut Attempt, f: &Fixture, event: Event, out: &mut Output) {
    let before = a.clone();
    match a.apply(f, &event) {
        Ok(()) => out.changed |= *a != before,
        Err(e) => out.diagnostic = Some(e),
    }
}
fn input(a: &mut Attempt, f: &Fixture, events: Vec<egui::Event>, out: &mut Output) {
    for event in events {
        match event {
            egui::Event::Paste(text) => apply(a, f, Event::Insert { text }, out),
            egui::Event::Text(text) => {
                if a.editor().mode() == Mode::Insert {
                    apply(a, f, Event::Insert { text }, out);
                } else if text.len() > 128 {
                    out.diagnostic = Some(
                        "Editor key input is too long. Paste text only after entering Insert mode."
                            .into(),
                    );
                } else {
                    for c in text.chars() {
                        apply(
                            a,
                            f,
                            Event::Key {
                                key: Key::Character(c),
                            },
                            out,
                        );
                    }
                }
            }
            egui::Event::Key {
                key, pressed: true, ..
            } => {
                let key = match key {
                    egui::Key::Escape => Some(Key::Escape),
                    egui::Key::Enter => Some(Key::Enter),
                    egui::Key::Backspace => Some(Key::Backspace),
                    egui::Key::ArrowLeft
                    | egui::Key::ArrowRight
                    | egui::Key::ArrowUp
                    | egui::Key::ArrowDown
                    | egui::Key::Home
                    | egui::Key::End
                    | egui::Key::Delete => {
                        if a.editor().mode() != Mode::Normal {
                            out.diagnostic=Some("In this editor subset, leave Insert mode with Escape before moving; use Backspace to correct typed text.".into());
                            None
                        } else {
                            Some(Key::Character(match key {
                                egui::Key::ArrowLeft => 'h',
                                egui::Key::ArrowRight => 'l',
                                egui::Key::ArrowUp => 'k',
                                egui::Key::ArrowDown => 'j',
                                egui::Key::Home => '0',
                                egui::Key::End => '$',
                                _ => 'x',
                            }))
                        }
                    }
                    _ => None,
                };
                if let Some(key) = key {
                    apply(a, f, Event::Key { key }, out);
                }
            }
            _ => {}
        }
    }
}
fn captured(e: &egui::Event) -> bool {
    matches!(
        e,
        egui::Event::Text(_)
            | egui::Event::Paste(_)
            | egui::Event::Key {
                pressed: true,
                key: egui::Key::Escape
                    | egui::Key::Enter
                    | egui::Key::Backspace
                    | egui::Key::ArrowLeft
                    | egui::Key::ArrowRight
                    | egui::Key::ArrowUp
                    | egui::Key::ArrowDown
                    | egui::Key::Home
                    | egui::Key::End
                    | egui::Key::Delete,
                ..
            }
    )
}
pub(super) fn show(ui: &mut egui::Ui, a: &mut Attempt, f: &Fixture, focus: &mut bool) -> Output {
    let mut out = Output::default();
    if let Err(e) = a.valid_for(f) {
        out.diagnostic = Some(e);
        return out;
    }
    let writable = f.files[&a.selected].writable;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&a.selected).monospace().strong());
        ui.label(if writable {
            "Virtual file"
        } else {
            "Read-only virtual file"
        });
    });
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = ui.visuals().text_color();
    let glyph = ui.fonts_mut(|fonts| fonts.layout_no_wrap("M".into(), font.clone(), color));
    let cw = ui.fonts_mut(|fonts| fonts.glyph_width(&font, 'M'));
    let line_height = ui
        .text_style_height(&egui::TextStyle::Monospace)
        .max(glyph.size().y);
    let padding = f32::from(controls::PAD_X);
    let visible_lines = (a.editor().contents().lines().count() + 2).clamp(6, 12);
    let height = line_height * visible_lines as f32 + 2. * padding;
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(1.);
    let enabled = ui.is_enabled() && !a.editor().closed();
    let background = ui.painter().add(egui::Shape::Noop);
    let mut focused = false;
    let area=egui::ScrollArea::both().id_salt("modal-text-scroll").max_height(height).min_scrolled_height(height).show(ui,|ui|{
        let e=a.editor();
        let lines=e.contents().lines().count().max(1);
        let digits=lines.to_string().len();
        let columns=e.contents().lines().map(str::len).max().unwrap_or(0).max(1);
        let size=egui::vec2(width.max((columns+digits+3)as f32*cw+2.*padding),height.max(lines as f32*line_height+2.*padding));
        let (rect,response)=ui.allocate_exact_size(size,if enabled{egui::Sense::click()}else{egui::Sense::hover()});
        response.widget_info(||egui::WidgetInfo::labeled(egui::WidgetType::TextEdit,enabled,"Virtual modal editor. Escape for Normal mode; Tab moves to the next application control."));
        if enabled&&(response.clicked()||*focus){response.request_focus();*focus=false;}
        if enabled&&response.has_focus(){
            ui.memory_mut(|m|m.set_focus_lock_filter(response.id,egui::EventFilter{tab:false,horizontal_arrows:true,vertical_arrows:true,escape:true}));
            // Move owned events instead of cloning arbitrarily large clipboard
            // strings before the bounded simulator can reject their contents.
            let events=ui.input_mut(|i|{
                let mut events=vec![];
                let mut remaining=vec![];
                for event in std::mem::take(&mut i.events){
                    if captured(&event){events.push(event);}else{remaining.push(event);}
                }
                i.events=remaining;
                events
            });
            input(a,f,events,&mut out);
        }
        let e=a.editor();
        let (row,col)=e.cursor().unwrap_or((0,0));
        let painter=ui.painter();
        focused=enabled&&response.has_focus();
        let origin=rect.min+egui::vec2(padding,padding);
        let prefix=(digits+2)as f32*cw;
        let text=e.contents();
        let display=if text.is_empty(){vec![""]}else{text.strip_suffix('\n').unwrap_or(text).split('\n').collect::<Vec<_>>()};
        for (n,line) in display.iter().enumerate(){
            let pos=origin+egui::vec2(0.,n as f32*line_height);
            if pos.y+line_height<ui.clip_rect().top()||pos.y>ui.clip_rect().bottom(){continue;}
            painter.text(pos,egui::Align2::LEFT_TOP,format!("{:>digits$}  {line}",n+1),font.clone(),color);
        }
        let cursor=egui::Rect::from_min_size(origin+egui::vec2(prefix+col as f32*cw,row as f32*line_height),egui::vec2(cw,line_height));
        if enabled&&response.has_focus(){
            if e.mode()==Mode::Insert{painter.line_segment([cursor.left_top(),cursor.left_bottom()],ui.visuals().text_cursor.stroke);}
            else {
                painter.rect_stroke(cursor,0.,ui.visuals().text_cursor.stroke,egui::StrokeKind::Inside);
            }
        }
        out.surface=Some((rect,ui.clip_rect(),response.id));out.cursor=Some(cursor);
        if focused && (out.changed || !ui.clip_rect().contains_rect(cursor)) {ui.scroll_to_rect_animation(cursor,None,egui::style::ScrollAnimation::none());}
    });
    // The frame belongs to the viewport, not the horizontally scrolling text.
    // Keep the focus outline and click/capture bounds stable for long lines.
    let viewport = area.inner_rect;
    let corners = ui.visuals().widgets.inactive.corner_radius;
    ui.painter().set(
        background,
        egui::Shape::rect_filled(viewport, corners, ui.visuals().extreme_bg_color),
    );
    let stroke = if focused {
        ui.visuals().text_cursor.stroke
    } else {
        ui.visuals().widgets.inactive.bg_stroke
    };
    ui.painter()
        .rect_stroke(viewport, corners, stroke, egui::StrokeKind::Inside);
    if let Some((_, clip, id)) = out.surface {
        out.surface = Some((viewport, clip, id));
    }

    let e = a.editor();
    let (row, col) = e.cursor().unwrap_or((0, 0));
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(match e.mode() {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
                Mode::SearchForward | Mode::SearchBackward => "SEARCH",
            })
            .strong(),
        );
        ui.label(format!("Line {} · column {}", row + 1, col + 1));
        ui.label(if e.closed() {
            "Closed"
        } else if e.modified() {
            "Unsaved edits"
        } else {
            "Saved"
        });
        if !e.pending_label().is_empty() {
            ui.monospace(format!("{} …", e.pending_label()));
        }
    });
    if matches!(
        e.mode(),
        Mode::Command | Mode::SearchForward | Mode::SearchBackward
    ) {
        let prefix = match e.mode() {
            Mode::Command => ':',
            Mode::SearchForward => '/',
            _ => '?',
        };
        ui.monospace(format!("{prefix}{}", e.command()));
    }
    if !e.notice().is_empty() {
        lesson_presentation::prose(ui, e.notice());
    }
    out
}
