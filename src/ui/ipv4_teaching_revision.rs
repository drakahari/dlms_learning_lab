//! Focused CCNA IPv4 teaching; assignment state and grading stay in existing models.
use super::*;
use dlms_learning_lab::magic_subnet::Problem;
impl LabApp {
    pub(super) fn ipv4_readable_introduction(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.address_teaching.as_ref().unwrap();
        let engine::State::Subnet { ip, prefix, .. } = lesson.fixture else {
            return;
        };
        let p = Problem { ip, prefix };
        let c = p.calculate().unwrap();
        ui.heading(&s.title);
        ccna_presentation::lead(
            ui,
            "WHAT YOU’LL LEARN",
            "Use subnet boundaries to configure two workstations.",
        );
        ccna_presentation::prose(
            ui,
            "A subnet is a group of addresses. Its first address identifies the network; its last is reserved for broadcast in this exercise. Workstations use the addresses between them.",
        );
        if self.scenarios.iter().any(|s| s.id == "ccna-magic-blocks")
            && ui
                .link("New to subnetting? Open IPv4 foundations: the magic-number method.")
                .clicked()
        {
            self.open_magic_introduction();
        }
        ui.separator();
        let steps = |ui: &mut egui::Ui| {
            for (title,text) in [
                ("1. Read the prefix",format!("IPv4 has 32 bits. /{prefix} fixes {prefix} network bits and leaves {} host bits. The mask is {}.",32-prefix,c.mask)),
                ("2. Calculate the block size",format!("The prefix ends in octet {}. Use its mask value {}: 256 − {} = {}. These are steps in that octet.",c.octet+1,c.mask_octet,c.mask_octet,c.block)),
                ("3. Locate the computer",format!("The boundaries are 0, 64, 128 and 192. The last number in {ip} is {}; it falls in {}–{}.",ip.octets()[3],c.start,c.end)),
                ("4. Reserve the two endpoints","Network is the first address in the block. Broadcast is one before the next block. Usable hosts start one after network and end one before broadcast.".into())
            ] {ui.strong(title);ccna_presentation::prose(ui,&text);ui.add_space(6.);}
        };
        let mut worked = |ui: &mut egui::Ui| {
            ui.strong("Worked example · different from the assignment");
            ui.monospace(format!("{ip}/{prefix}"));
            let diagram = ui.vertical(|ui| magic_subnet::range_diagram(ui, p));
            self.metrics.address_diagram = Some(diagram.response.rect);
            magic_subnet::worked_table(ui, p);
        };
        if ui.available_width() >= 920. {
            ui.columns(2, |cols| {
                steps(&mut cols[0]);
                worked(&mut cols[1]);
            });
        } else {
            steps(ui);
            ui.separator();
            worked(ui);
        }
        ui.add_space(8.);
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(primary(ui, "Try this example"));
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                self.set_address_view(i, teaching::View::Example);
            }
            let r = ui.add(button("Go straight to assignment"));
            self.metrics.assignment_action = Some(r.rect);
            if r.clicked() {
                self.set_address_view(i, teaching::View::Assignment);
            }
        });
    }
    pub(super) fn address_practice_actions(&self, ui: &mut egui::Ui, i: usize, heading: bool) {
        let s = &self.scenarios[i];
        if s.id != "ccna-ipv4" {
            ccna_presentation::example(ui, s, heading);
            return;
        }
        let engine::State::Subnet { ip, prefix, .. } = s.address_teaching.as_ref().unwrap().fixture
        else {
            return;
        };
        let c = Problem { ip, prefix }.calculate().unwrap();
        ccna_presentation::lead(
            ui,
            "TRY EXAMPLE",
            "Change one workstation and test the result.",
        );
        ui.strong("What to do");
        for text in [
            "1. Select Fill worked example.",
            "2. Select Check configuration.",
            "3. Change Workstation 2 to another usable address. Select Check configuration again.",
            &format!(
                "4. Enter the broadcast address, {}, in Workstation 2. Check again and read its feedback.",
                c.broadcast
            ),
        ] {
            ccna_presentation::prose(ui, text);
            ui.add_space(4.);
        }
        ui.separator();
        ui.strong("What to observe");
        for text in [
            "Fill populates the fields; it does not check them.",
            "The worked configuration should pass.",
            &format!(
                "Another address from {} to {} should pass, except {} (Workstation 1).",
                c.first, c.last, ip
            ),
            "Broadcast must fail: it is reserved for the group, not a workstation.",
        ] {
            ccna_presentation::prose(ui, text);
            ui.add_space(4.);
        }
        if heading {
            ui.separator();
            ui.strong("Practice workspace");
        }
    }
}
