//! Opt-in MBR presentation over the existing native disk workspace.
use super::*;
pub(super) fn table(ui: &mut egui::Ui, m: &mut Metrics, path: &str, d: &core::Disk) {
    if d.table.is_some() {
        super::table(ui, m, path, d);
        return;
    }
    let r=ui.vertical(|ui| {
        line(ui,m,format!("Applied disk {path} · {} · {} sectors of {} B · alignment {} sectors",capacity(d.sectors*d.sector_bytes),d.sectors,d.sector_bytes,d.optimal_alignment));
        let mut protection=Vec::new();
        if !d.writable {protection.push("Read-only");}
        if d.in_use {protection.push("In use");}
        if d.occupied_without_table {protection.push("Existing data to preserve");}
        if !protection.is_empty(){line(ui,m,format!("Protection · {}. Destructive replacement is unavailable.",protection.join(" · ")));}
        let Some(t)=&d.mbr else {line(ui,m,"No partition table. Create MBR only on a supplied blank, writable, unused disk.");return;};
        line(ui,m,"MBR: four primary table slots; an extended container holds logical payloads numbered from 5. Type hint and actual content are separate.");
        if t.entries.is_empty(){line(ui,m,"The MBR is empty; no payload or filesystem exists yet.");}
        let headers=["Partition / role","Start–end sectors","Capacity","Type hint","Actual content"];
        let rows=t.entries.iter().map(|e|{
            let p=&e.partition;
            [format!("{path}{} / {}",p.number,work::mbr::role_name(e.kind)),format!("{}–{}",p.start,p.end),capacity(p.bytes().unwrap()),
                if e.kind==core::mbr::Kind::Extended {"Not applicable".into()}else{work::type_hint_name(p.type_hint).into()},
                if e.kind==core::mbr::Kind::Extended {"Container only".into()}else{signature(p)}]
        }).collect::<Vec<_>>();
        let font=egui::TextStyle::Body.resolve(ui.style());let color=palette(ui).text;
        let widths=(0..5).map(|i|std::iter::once(headers[i]).chain(rows.iter().map(|r|r[i].as_str()))
            .map(|s|ui.fonts_mut(|f|f.layout_no_wrap(s.into(),font.clone(),color).size().x)).fold(0.,f32::max)+12.).collect::<Vec<_>>();
        if widths.iter().sum::<f32>()+4.*ui.spacing().item_spacing.x<=ui.available_width(){
            egui::Grid::new(("mbr-applied",path)).num_columns(5).striped(true).show(ui,|ui|{
                for (i,h)in headers.iter().enumerate(){controls::cell(ui,widths[i],|ui|{ui.strong(*h);});}ui.end_row();
                for row in &rows{for(i,text)in row.iter().enumerate(){controls::cell(ui,widths[i],|ui|{line(ui,m,text);});}ui.end_row();}
            });
        }else{
            for row in &rows{ui.group(|ui|{line(ui,m,format!("{} · {}",row[0],row[2]));line(ui,m,format!("Sectors {} inclusive · hint {}",row[1],row[3]));line(ui,m,&row[4]);});}
        }
        let reservations = t.entries.iter().filter_map(|e| e.ebr.map(|sector| format!("{} at {sector}", e.partition.number))).collect::<Vec<_>>();
        if !reservations.is_empty() { line(ui,m,format!("Modeled EBR sectors by logical partition: {}. Each precedes its payload; these are exercise facts, not actual parted placement.",reservations.join("; "))); }
    });
    m.table = Some(r.response.rect);
}
pub(super) fn role(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    salt: egui::Id,
    draft: &mut work::Draft,
    actions: bool,
) {
    ui.add_enabled_ui(actions, |ui| {
        let current = draft.mbr_role.unwrap_or(core::mbr::Kind::Primary);
        let pick = controls::dropdown(ui, salt.with("mbr-role"), ui.available_width())
            .selected_text(format!("Partition role: {}", work::mbr::role_name(current)))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for value in [
                    core::mbr::Kind::Primary,
                    core::mbr::Kind::Extended,
                    core::mbr::Kind::Logical,
                ] {
                    let r = ui.selectable_value(
                        &mut draft.mbr_role,
                        Some(value),
                        work::mbr::role_name(value),
                    );
                    m.choices
                        .push((format!("role-{value:?}"), r.rect, ui.clip_rect()));
                }
            });
        m.controls
            .push(("mbr-role".into(), pick.response.rect, ui.clip_rect()));
    });
}
