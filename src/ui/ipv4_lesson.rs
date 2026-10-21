use super::*;
use dlms_learning_lab::ipv4::{self, Diagnosis, Exercise, Lesson, Section};
use std::net::Ipv4Addr;
const NET: Color32 = Color32::from_rgb(48, 96, 126);
const HOST: Color32 = Color32::from_rgb(100, 69, 151);
#[derive(Clone, Copy)]
enum Action {
    None,
    Fresh,
    Original,
    Variant,
    Library,
    Next,
}
impl LabApp {
    pub(super) fn ipv4_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let id = self.scenarios[i].id.clone();
        let mut a = self.progress.attempts.remove(&id).unwrap();
        let original = Exercise::from_state(&a.state).unwrap();
        let mut lesson = a
            .ipv4
            .take()
            .unwrap_or_else(|| Lesson::from_legacy(original.ip, original.prefix, a.step));
        let old = lesson.clone();
        let was_variant = lesson.use_variant;
        let mut e = if was_variant {
            lesson.variant.as_ref().unwrap().clone()
        } else {
            original.clone()
        };
        let before = e.clone();
        let mut action = Action::None;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(if was_variant {
                    "INDEPENDENT PRACTICE · numerical variant"
                } else {
                    "GUIDED EXAMPLE · one authored lab"
                })
                .small()
                .color(ACCENT),
            );
            if was_variant {
                if ui.add(button("Return to original example")).clicked() {
                    action = Action::Original;
                }
            } else if lesson.variant.is_some()
                && ui.add(button("Continue saved practice")).clicked()
            {
                action = Action::Variant;
            }
        });
        ui.horizontal_wrapped(|ui| {
            for section in Section::ALL {
                if ui
                    .selectable_label(lesson.section == section, section.title())
                    .clicked()
                {
                    lesson.section = section;
                }
            }
        });
        self.metrics.split = ui.available_width() >= 960.;
        if self.metrics.split {
            let left = (ui.available_width() * 0.27).clamp(260., 340.);
            let right = ui.available_width() - left - 16.;
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 16.;
                ui.allocate_ui_with_layout(
                    egui::vec2(left, 0.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        self.ipv4_explain(ui, &e, &lesson, a.challenge);
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(right, 0.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(right);
                        self.ipv4_work(ui, &mut e, &mut lesson, a.challenge);
                    },
                );
            });
        } else {
            self.ipv4_work(ui, &mut e, &mut lesson, a.challenge);
            self.ipv4_explain(ui, &e, &lesson, a.challenge);
        }
        let solved = e.solved();
        if self.last_solved && !solved {
            self.invalidated = true;
            self.checked = true;
        }
        if solved {
            self.invalidated = false;
        }
        self.last_solved = solved;
        if self.invalidated {
            ui.colored_label(RED,"Current success no longer applies: an answer changed. Check the predictions and retry.");
        }
        if solved {
            self.metrics.completion = true;
            card().stroke(egui::Stroke::new(1_f32,GREEN)).show(ui,|ui|{
                ui.label(RichText::new(if was_variant{"Practice complete"}else{"Objective complete"}).strong().color(GREEN).size(22.));
                ui.label("All four current predictions are correct. Walkthrough position does not affect completion.");
                ui.label("Worked results stay behind your explicit requests. Choose Network, Broadcast or Usable range above to review that result.");
                ui.horizontal_wrapped(|ui|{if ui.add(button("Back to library")).clicked(){action=Action::Library;}
 if ui.add(button("Next authored lab")).clicked(){action=Action::Next;}});
            });
        }
        card().show(ui,|ui|{
            ui.label(RichText::new("Apply it independently").strong().size(20.));
            ui.label("After reviewing the three worked results, try a fresh address and prefix. This is a numerical variant of this lab, not a new authored scenario. Original answers and completion stay saved.");
            if ui.add_enabled(lesson.worked_example_reviewed(),primary("Generate fresh practice")).clicked(){action=Action::Fresh;}
            if !lesson.worked_example_reviewed(){ui.label(RichText::new("Predict, then request the worked Network, Broadcast and Usable range explanations first.").small().color(MUTED));}
        });
        ui.add_space(8.);
        egui::CollapsingHeader::new("Technical scope & sources").show(ui,|ui|{ui.label("Conventional broadcast subnets only, prefixes /0 through /30. /31 point-to-point semantics and /32 host routes are deliberately excluded. Variants use private 10/8 space and prefixes /8 through /30. No packets or host networking are used.");for source in &self.scenarios[i].sources{ui.add(egui::Label::new(source).wrap());}});
        if was_variant {
            lesson.variant = Some(e.clone());
        } else if let engine::State::Subnet { answers, .. } = &mut a.state {
            *answers = e.answers.clone();
        }
        let changed_variant = matches!(action, Action::Fresh | Action::Original | Action::Variant);
        if was_variant {
            lesson.variant_section = lesson.section;
        } else {
            lesson.original_section = lesson.section;
        }
        match action {
            Action::Fresh => {
                lesson.fresh();
            }
            Action::Original => {
                lesson.use_variant = false;
                lesson.section = lesson.original_section;
                lesson.sandbox_host =
                    u32::from(original.ip) & !ipv4::mask(original.prefix).unwrap();
            }
            Action::Variant => {
                lesson.use_variant = true;
                lesson.section = lesson.variant_section;
                let v = lesson.variant.as_ref().unwrap();
                lesson.sandbox_host = u32::from(v.ip) & !ipv4::mask(v.prefix).unwrap();
            }
            _ => {}
        }
        if changed_variant {
            self.ipv4_focus = None;
            self.ipv4_challenge_revealed = [false; 3];
            self.checked = false;
            self.invalidated = false;
            self.last_solved = false;
            self.focus_input = true;
        }
        // Keep the original three-section save indexes stable; the four-part lesson is additive metadata.
        if old.original_section != lesson.original_section {
            a.step = lesson.original_section.legacy_step();
        }
        if old != lesson || before != e {
            self.dirty = true;
        }
        a.ipv4 = Some(lesson);
        self.progress.attempts.insert(id, a);
        if matches!(action, Action::Library) {
            self.selected = None;
        }
        if matches!(action, Action::Next) {
            self.launch(i + 1);
        }
    }
    fn ipv4_work(&mut self, ui: &mut egui::Ui, e: &mut Exercise, l: &mut Lesson, challenge: bool) {
        card().show(ui,|ui|{
            ui.label(RichText::new(l.section.title()).strong().size(23.));
            ui.label(RichText::new(if challenge{"Challenge: make your predictions. No worked results are shown unless you request them."}else{match l.section{
                Section::Original=>"Read the interface IP and the mask separately. Follow the N/H boundary before predicting an address.",
                Section::Network=>"Prediction: what address has the same network bits, with every host bit set to zero?",
                Section::Broadcast=>"Prediction: what is the final address when the same network bits stay fixed and every host bit is one?",
                Section::Usable=>"Prediction: which two addresses are the first and last usable hosts, after reserving both boundaries?",
            }}).color(MUTED));
            let (_,revealed)=l.gates();let result_visible=l.section.result_index().is_some_and(|n|revealed[n]&&(!challenge||self.ipv4_challenge_revealed[n]));
            let expected=e.expected().unwrap();let displayed=if result_visible{match l.section{Section::Network=>expected[0],Section::Broadcast=>expected[1],Section::Usable=>expected[2],Section::Original=>e.ip}}else{e.ip};
            ui.label(RichText::new(format!("Original interface IP: {}/{}",e.ip,e.prefix)).strong());
            ui.label(format!("Subnet mask: {} · {} selection bits = 1, {} = 0",Ipv4Addr::from(ipv4::mask(e.prefix).unwrap()),e.prefix,32-e.prefix));
            let caption=if result_visible{format!("Worked {}: {displayed}",if l.section==Section::Usable{"first usable host"}else{l.section.title()})}else{"IP row shows the original address. Worked changes are hidden until you submit a prediction and request them.".into()};
            ui.label(RichText::new(caption).small().color(ACCENT));
            self.octets(ui,e,displayed,0,None);
            if result_visible&&l.section==Section::Usable{ui.label(RichText::new(format!("Last usable host: {}",expected[3])).strong());self.octets(ui,e,expected[3],1,None);}
            ui.label(RichText::new("N = fixed network position · H = host position · H* = changed from the original IP. The divider marks the prefix boundary.").small().color(MUTED));
            ui.separator();ui.label(RichText::new("Your predictions").strong());
            for (index,label) in ["Network address","Broadcast address","First usable host","Last usable host"].iter().enumerate(){
                ui.horizontal_wrapped(|ui|{
                    let active=challenge||match l.section{Section::Original|Section::Network=>index==0,Section::Broadcast=>index==1,Section::Usable=>index>=2};
                    let label=ui.label(RichText::new(*label).color(if active{ACCENT}else{MUTED}));let r=ui.add_sized([210_f32.min(ui.available_width().max(120.)),CONTROL],egui::TextEdit::singleline(&mut e.answers[index]).id_salt(("answer",index)).hint_text("IPv4 address").char_limit(64));let r=r.labelled_by(label.id);
                    self.metrics.inputs.push(r.rect);if index==0{self.metrics.focus=Some(r.id);if self.focus_input{r.request_focus();self.focus_input=false;}}
                    if r.changed(){self.dirty=true;}
                });
                let relevant=challenge||match l.section{Section::Original|Section::Network=>index==0,Section::Broadcast=>index==1,Section::Usable=>index>=2};
                if self.checked&&relevant{let(d,text)=ipv4::feedback(e,index);ui.colored_label(if d==Diagnosis::Correct{GREEN}else{RED},text);}
            }
            let check=ui.add(primary("Check predictions"));self.metrics.check=Some(check.rect);if check.clicked(){l.submit(e);self.checked=true;}
            ui.label(RichText::new("Checks explain mistakes without giving the expected address. Learn feedback follows the current view; edit a prediction and retry.").small().color(MUTED));
            if let Some(n)=l.section.result_index(){
                let r=ui.add_enabled(l.can_reveal(l.section,e),button(if revealed[n]&&(!challenge||self.ipv4_challenge_revealed[n]){"Review worked explanation"}else{"Show worked explanation"}));self.metrics.ipv4_reveal=Some(r.rect);
                if r.clicked(){l.reveal(l.section,e);if challenge{self.ipv4_challenge_revealed[n]=true;}}
                if !l.can_reveal(l.section,e){ui.label(RichText::new("First enter and check a prediction for this section. Incorrect predictions are welcome.").small().color(ACCENT));}
                if l.gates().1[n]&&(!challenge||self.ipv4_challenge_revealed[n]){self.metrics.ipv4_worked=true;ui.separator();ui.label(RichText::new("Why those bits change").strong().color(ACCENT));ui.label(ipv4::worked(e,l.section));}
            }
            ui.horizontal_wrapped(|ui|{
                let index=l.section.index();if ui.add_enabled(index>0,button("Previous view")).clicked(){l.section=Section::ALL[index.saturating_sub(1)];}
                if ui.add_enabled(index<3,button("Next view")).clicked(){l.section=Section::ALL[(index+1).min(3)];}
            });
        });
        let sandbox_title = if challenge {
            "Optional host-bit sandbox (exploration aid)"
        } else {
            "Host-bit sandbox · separate from your predictions"
        };
        egui::CollapsingHeader::new(sandbox_title).id_salt(("host-sandbox",challenge,e.ip,e.prefix)).default_open(!challenge).show(ui,|ui|{
            card().show(ui,|ui|{
                ui.label("Click a host bit, or focus it with Tab and press Space. Its value changes the sandbox decimal address immediately. Network bits can be inspected but stay fixed. No answer field is filled for you.");
                let address=ipv4::sandbox_address(e,l.sandbox_host).unwrap();ui.label(RichText::new(format!("Practice address: {address}")).strong());self.octets(ui,e,address,2,Some(&mut l.sandbox_host));
                let host_mask=!ipv4::mask(e.prefix).unwrap();
                if l.sandbox_host==0{ui.label("All host bits are 0. In this conventional exercise, that pattern names the subnet itself: the network address, reserved rather than assigned to a host.");}
                else if l.sandbox_host==host_mask{ui.label("All host bits are 1. In this conventional exercise, that pattern identifies the subnet’s broadcast destination, reserved rather than assigned to a host.");}
                else{ui.label("This is an ordinary host-bit pattern between the two reserved extremes. Try making every H bit 0, then every H bit 1, and observe the decimal changes.");}
                if ui.add(button("Restore original host bits")).clicked(){l.sandbox_host=u32::from(e.ip)&host_mask;}
            });
        });
    }
    fn ipv4_explain(&mut self, ui: &mut egui::Ui, e: &Exercise, lesson: &Lesson, challenge: bool) {
        let section = lesson.section;
        self.metrics.ipv4_explain = true;
        self.metrics.guide = !challenge;
        card().fill(Color32::from_rgb(33,29,57)).show(ui,|ui|{
            ui.label(RichText::new("Explain this diagram").strong().size(21.));
            ui.label("An IPv4 address has four decimal numbers called octets. Each octet is eight bits: 32 bits in total.");
            ui.label("A bit is 0 or 1. Within each octet, the left-to-right weights are 128, 64, 32, 16, 8, 4, 2, 1. Add the weights where the IP bit is 1 to get that octet’s decimal value.");
            ui.separator();ui.label(RichText::new("IP address ≠ subnet mask").strong());
            ui.label(format!("The interface IP is {}. /{} counts the first {} network positions. The mask is a separate selection pattern: 1 means network, 0 means host. It does not replace the IP.",e.ip,e.prefix,e.prefix));
            ui.label(format!("There are {} host positions. Each can be 0 or 1: 2^{} = {} address patterns. Network positions stay fixed across the subnet; only H positions change. N/H labels identify roles without relying on color.",32-e.prefix,32-e.prefix,e.capacity().unwrap()));
            ui.label("Zero host bits select the first boundary (network); all-one host bits select the last (broadcast). This exercise reserves both. /31 and /32 have different uses and are excluded.");
            ui.separator();ui.label(RichText::new("Inspect a bit").strong());
            if let Some((row,position,_))=self.ipv4_focus{
                let visible=section.result_index().is_some_and(|n|lesson.gates().1[n]&&(!challenge||self.ipv4_challenge_revealed[n]));
                let shown=match row{2=>ipv4::sandbox_address(e,lesson.sandbox_host).unwrap(),3=>Ipv4Addr::from(ipv4::mask(e.prefix).unwrap()),_=>if visible{let w=e.expected().unwrap();match section{Section::Network=>w[0],Section::Broadcast=>w[1],Section::Usable=>if row==1{w[3]}else{w[2]},_=>e.ip}}else{e.ip}};
                let value=(u32::from(shown)>>(31-position))&1;
                ui.label(format!("{} Current bit value: {value}.",bit_explanation(e,row,position)));
            }else{ui.label("Hover a bit for its explanation. Tab to a bit or click it to keep its explanation here; Tab/Shift+Tab move focus and Space activates sandbox host bits.");}
            if !challenge{ui.separator();ui.label(RichText::new(format!("Current view: {}",section.title())).color(ACCENT));ui.label(match section{Section::Original=>"Start with the supplied IP, not a rounded address. Read each octet and locate where N ends and H starts.",Section::Network=>"Keep every N bit from the original IP; only H bits become zero. Predict first, then request the worked change.",Section::Broadcast=>"Keep those same N bits; only H bits become one. Predict first, then request the worked change.",Section::Usable=>"Reserve both extremes. Move one address inward from each boundary; carrying between octets follows normal binary arithmetic."});}
        });
    }
    fn octets(
        &mut self,
        ui: &mut egui::Ui,
        e: &Exercise,
        address: Ipv4Addr,
        row_kind: u8,
        mut sandbox: Option<&mut u32>,
    ) {
        let width = ui.available_width();
        let columns = if width >= 800. {
            4
        } else if width >= 390. {
            2
        } else {
            1
        };
        let octets = address.octets();
        let mask = Ipv4Addr::from(ipv4::mask(e.prefix).unwrap()).octets();
        for row in (0..4).step_by(columns) {
            ui.columns(columns, |uis| {
                for (offset, ui) in uis.iter_mut().enumerate().take(columns) {
                    let octet = row + offset;
                    if octet >= 4 {
                        break;
                    }

                    egui::Frame::new()
                        .fill(Color32::from_rgb(13, 23, 40))
                        .inner_margin(8.)
                        .corner_radius(6.)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Octet {}", octet + 1))
                                    .small()
                                    .color(MUTED),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "IP {} · Mask {}",
                                    octets[octet], mask[octet]
                                ))
                                .strong(),
                            );
                            let cell = ((ui.available_width() - 7. * 2.) / 8.).clamp(17., 30.);
                            for is_mask in [false, true] {
                                if sandbox.is_some() && is_mask {
                                    continue;
                                }
                                ui.label(
                                    RichText::new(if is_mask {
                                        "Mask selection bits"
                                    } else {
                                        "IP address bits"
                                    })
                                    .small()
                                    .color(MUTED),
                                );
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.;
                                    ui.spacing_mut().button_padding = egui::vec2(1., 2.);
                                    for bit in 0..8 {
                                        let position = (octet * 8 + bit) as u8;
                                        let network = position < e.prefix;
                                        let value = if is_mask {
                                            u8::from(network)
                                        } else {
                                            (u32::from(address) >> (31 - position)) as u8 & 1
                                        };
                                        let kind = if is_mask { 3 } else { row_kind };
                                        let changed = !is_mask
                                            && value
                                                != ((u32::from(e.ip) >> (31 - position)) as u8 & 1);
                                        let message = format!(
                                            "{} Current bit value: {value}.",
                                            bit_explanation(e, kind, position)
                                        );
                                        let r = ui
                                            .push_id(
                                                ("ipv4-bit", row_kind, octet, bit, is_mask),
                                                |ui| {
                                                    ui.add_sized(
                                                        [cell, 38.],
                                                        egui::Button::new(
                                                            RichText::new(format!(
                                                                "{value}\n{}",
                                                                if network {
                                                                    "N"
                                                                } else if changed {
                                                                    "H*"
                                                                } else {
                                                                    "H"
                                                                }
                                                            ))
                                                            .monospace()
                                                            .size(12.),
                                                        )
                                                        .fill(if network { NET } else { HOST })
                                                        .corner_radius(3.),
                                                    )
                                                },
                                            )
                                            .inner;
                                        r.widget_info(|| {
                                            egui::WidgetInfo::labeled(
                                                egui::WidgetType::Button,
                                                ui.is_enabled(),
                                                &message,
                                            )
                                        });
                                        if r.has_focus() || r.clicked() {
                                            self.ipv4_focus = Some((kind, position, value));
                                            if r.clicked() {
                                                r.request_focus();
                                            }
                                        }
                                        if r.clicked()
                                            && !network
                                            && !is_mask
                                            && let Some(host) = sandbox.as_deref_mut()
                                        {
                                            *host = ipv4::toggle_host(e, *host, position).unwrap();
                                        }
                                        if !is_mask {
                                            self.metrics.ipv4_bits.push((
                                                r.rect,
                                                position,
                                                sandbox.is_some(),
                                            ));
                                        }
                                        if position + 1 == e.prefix {
                                            ui.painter().line_segment(
                                                [
                                                    r.rect.right_top() + egui::vec2(1., -3.),
                                                    r.rect.right_bottom() + egui::vec2(1., 3.),
                                                ],
                                                egui::Stroke::new(2_f32, Color32::WHITE),
                                            );
                                        }
                                        r.on_hover_text(message);
                                    }
                                });
                            }
                            let terms = (0..8)
                                .filter(|bit| octets[octet] & (128 >> bit) != 0)
                                .map(|bit| (128_u16 >> bit).to_string())
                                .collect::<Vec<_>>();
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{} = {}",
                                        octets[octet],
                                        if terms.is_empty() {
                                            "0".into()
                                        } else {
                                            terms.join(" + ")
                                        }
                                    ))
                                    .small()
                                    .color(MUTED),
                                )
                                .wrap(),
                            );
                            let n = (e.prefix as usize).saturating_sub(octet * 8).min(8);
                            ui.label(
                                RichText::new(format!("{n} N positions · {} H positions", 8 - n))
                                    .small()
                                    .color(MUTED),
                            );
                        });
                }
            });
            ui.add_space(4.);
        }
    }
}
fn bit_explanation(e: &Exercise, row: u8, position: u8) -> String {
    let octet = position / 8 + 1;
    let bit = position % 8;
    let weight = 128_u16 >> bit;
    let network = position < e.prefix;
    if row == 3 {
        format!(
            "Subnet mask, octet {octet}, bit {}: {}. A mask 1 selects a fixed network position; a mask 0 selects a host position. This bit’s octet weight is {weight}; the mask is a selector, not the interface IP.",
            bit + 1,
            u8::from(network)
        )
    } else {
        format!(
            "{} octet {octet}, bit {} has decimal weight {weight}. {} Add {weight} when this IP bit is 1; add zero when it is 0.",
            if row == 2 { "Sandbox IP" } else { "IP address" },
            bit + 1,
            if network {
                "N: keep this bit from the original IP fixed when forming network and broadcast."
            } else {
                "H: this host position may change. Zero all H bits for network; set all H bits to one for broadcast."
            }
        )
    }
}
