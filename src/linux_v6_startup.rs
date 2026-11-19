//! Virtual IPv6 filter file and one explicitly supplied oneshot loader.
//! No host filesystem, systemd, kernel, shell, subprocess or network execution.
use crate::{
    linux_v6_admin as admin,
    linux_v6_filter::{Class, Fault, Verdict},
    linux_v6_saved_filter as file, linux_v6_saved_settings as settings,
    linux_v6_state_filter::Table,
};
use serde::{Deserialize, Serialize};
pub const FILE: &str = "/etc/lab-ipv6-filter.rules";
pub const UNIT: &str = "lab-ipv6-filter.service";
pub const UNIT_TEXT: &str = "[Unit]\nDescription=Provided learning-lab IPv6 filter loader\nAfter=network.target\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart=/usr/sbin/ip6tables-restore /etc/lab-ipv6-filter.rules\n[Install]\nWantedBy=multi-user.target\n";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Status {
    Inactive,
    Loaded,
    Failed {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        detail: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u8,
    pub router: admin::State,
    pub filter: Table,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub saved_filter: String,
    pub enabled: bool,
    pub status: Status,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub exit: u8,
    pub changed: bool,
}
fn fault(class: Class, message: impl Into<String>) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn ascii(s: &str) -> bool {
    s.len() <= file::MAX_BYTES
        && s.bytes()
            .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
}
fn known_unit(s: &str) -> bool {
    s == UNIT || s == "lab-ipv6-filter"
}
impl State {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.version != 1 {
            return Err(fault(Class::Context, "Use startup state version1."));
        }
        self.router
            .validate()
            .map_err(|e| fault(Class::Context, e))?;
        self.filter.validate()?;
        if !self.router.learned.is_empty() {
            return Err(fault(
                Class::Unsupported,
                "This startup profile assumes persistent manual IPv6 routes; learned advertisements are a separate activity.",
            ));
        }
        if !ascii(&self.saved_filter)
            || serde_json::to_vec(self)
                .map_err(|e| fault(Class::Context, e.to_string()))?
                .len()
                > 256 * 1024
        {
            return Err(fault(
                Class::Limit,
                "Keep the virtual filter file within32KiB of ASCII text and the startup state within256KiB.",
            ));
        }
        // Invalid saved syntax is a legitimate unfinished repair, never executed as code.
        if let Status::Failed { detail } = &self.status
            && (detail.len() > 4096 || detail.is_empty())
        {
            return Err(fault(
                Class::Context,
                "Keep one bounded actual loader failure detail.",
            ));
        }
        Ok(())
    }
    pub fn save_draft(&mut self, text: &str) -> Result<(), Fault> {
        if !ascii(text) {
            return Err(fault(
                Class::Limit,
                "Keep the virtual filter draft within32KiB of ASCII text; no file changed.",
            ));
        }
        let mut next = self.clone();
        next.saved_filter = text.into();
        next.validate()?;
        *self = next;
        Ok(())
    }
    fn load_unit(&mut self) -> (String, u8) {
        match file::apply_restore(&mut self.filter, &self.saved_filter, false) {
            Ok(()) => {
                self.status = Status::Loaded;
                ("[Simulator note] Provided oneshot loader completed; the saved IPv6 filter table is now running.\n".into(),0)
            }
            Err(e) => {
                let detail = format!(
                    "[{:?}] {} Previous running rules were retained.",
                    e.class, e.message
                );
                self.status = Status::Failed {
                    detail: detail.clone(),
                };
                (
                    format!("[Simulator loader error] Loader failed: {detail}\n"),
                    1,
                )
            }
        }
    }
    pub fn command(&mut self, line: &str) -> Result<Output, Fault> {
        self.validate()?;
        if line.len() > 512 {
            return Err(fault(
                Class::Limit,
                "Keep one literal startup/filter command within512bytes.",
            ));
        }
        let words = crate::linux_filter::words(line)?;
        let args = words.iter().map(String::as_str).collect::<Vec<_>>();
        let mut next = self.clone();
        let (text,exit)=match args.as_slice(){
   ["cat",FILE]=>(self.saved_filter.clone(),0),
   ["ip6tables-save",rest @ ..]=>{
    let(mut table,mut filename)=(false,false);let mut at=0;
    while at<rest.len(){match rest[at..]{["-t"|"--table","filter",..]if !table=>{table=true;at+=2;},["-f"|"--file",FILE,..]if !filename=>{filename=true;at+=2;},_=>return Err(fault(Class::Unsupported,"Supported here: ip6tables-save [-t filter] [-f /etc/lab-ipv6-filter.rules], in either option order. Counters, module loading, other tables/files and shell redirection are not modeled."))}}
    let text=file::save(&self.filter)?;if filename{next.saved_filter=text;("[Simulator note] Saved the current filter table to the virtual file; loader enablement is unchanged.\n".into(),0)}else{(text,0)}
   },
   ["ip6tables-restore",FILE]=>{file::apply_restore(&mut next.filter,&self.saved_filter,false)?;(String::new(),0)},
   ["ip6tables-restore","--test"|"-t",FILE]=>{file::apply_restore(&mut next.filter,&self.saved_filter,true)?;(String::new(),0)},
   ["ip6tables",..]=>{let out=next.filter.command(line)?;(out.text,0)},
   ["systemctl",verb,unit]if known_unit(unit)=>match *verb{
    "enable"=>{next.enabled=true;(if self.enabled{String::new()}else{format!("Created symlink multi-user.target.wants/{UNIT} -> /etc/systemd/system/{UNIT}.\n")},0)},
    "disable"=>{next.enabled=false;(if self.enabled{format!("Removed multi-user.target.wants/{UNIT}.\n")}else{String::new()},0)},
    "is-enabled"=>(format!("{}\n",if self.enabled{"enabled"}else{"disabled"}),if self.enabled{0}else{1}),
    "is-active"=>(format!("{}\n",match self.status{Status::Loaded=>"active",Status::Inactive=>"inactive",Status::Failed{..}=>"failed"}),if matches!(self.status,Status::Loaded){0}else{3}),
    "cat"=>(format!("# Provided virtual /etc/systemd/system/{UNIT}\n{UNIT_TEXT}"),0),
    "status"=>{let active=match &self.status{Status::Loaded=>"active (exited)",Status::Inactive=>"inactive (dead)",Status::Failed{..}=>"failed (Result: exit-code)"};let detail=match &self.status{Status::Failed{detail}=>format!("\n   {detail}"),_=>String::new()};(format!("{} - Provided learning-lab IPv6 filter loader\n   Loaded: loaded (/etc/systemd/system/{UNIT}; {})\n   Active: {active}{detail}\n",UNIT,if self.enabled{"enabled"}else{"disabled"}),if matches!(self.status,Status::Loaded){0}else{3})},
    "start" if matches!(self.status,Status::Loaded)=>("[Simulator note] Provided oneshot unit is already active (exited); use restart to load a changed file.\n".into(),0),
    "start"|"restart"=>next.load_unit(),
    "stop"=>{next.status=Status::Inactive;("[Simulator note] Provided oneshot unit stopped. It has no ExecStop; running filter rules were retained.\n".into(),0)},
    _=>return Err(fault(Class::Unsupported,"Supported supplied-unit actions: cat, status, is-active, is-enabled, enable, disable, start, restart and stop. No host service changed."))
   },
   _=>{let read_only=match args.as_slice(){["ip","link","show"]|["ip","-6","route","show"]=>true,["ip","-6","address"|"addr","show",..]=>true,["sysctl",key]=>!key.starts_with('-'),["cat",_]=>true,_=>false};if !read_only{return Err(fault(Class::Unsupported,"Use the supplied unit, IPv6 filter/file commands or read-only network inspection; other commands and host-setting changes are outside this activity."));}let mut router=self.router.clone();let out=router.command(line).map_err(|e|fault(Class::Unsupported,e))?;if out.changed{return Err(fault(Class::Unsupported,"This activity keeps the supplied network/settings fixed. Use read-only inspection; firewall changes belong in ip6tables or its virtual saved file."));}(out.text,0)}
  };
        next.validate()?;
        let changed = next != *self;
        *self = next;
        Ok(Output {
            text,
            exit,
            changed,
        })
    }
    /// A scoped supplied startup sequence, not a full Linux/systemd boot model.
    /// Addresses/manual routes persist; settings reset and load from source-owned text.
    /// A fresh filter starts ACCEPT; only an enabled provided loader restores its file.
    pub fn startup_trial(&self, supplied_settings: &str) -> Result<Self, Fault> {
        self.validate()?;
        let router = settings::fresh(&self.router, supplied_settings)
            .map_err(|e| fault(Class::Context, format!("Supplied startup settings: {e}")))?;
        let mut next = Self {
            version: 1,
            router,
            filter: Table::new(Verdict::Accept),
            saved_filter: self.saved_filter.clone(),
            enabled: self.enabled,
            status: Status::Inactive,
        };
        if next.enabled {
            next.load_unit();
        }
        next.validate()?;
        Ok(next)
    }
}
