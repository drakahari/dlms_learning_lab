use dlms_learning_lab::{
    boot_entry as boot, kernel_build as core,
    kernel_stage::Store,
    kernel_workspace::{Action, Kind, Source, Task},
};

pub fn config(modular: bool, suffix: &str) -> String {
    format!(
        "CONFIG_MODULES={}\nCONFIG_BLK_DEV_INITRD=y\nCONFIG_BLK_DEV_NVME={}\nCONFIG_EXT4_FS={}\nCONFIG_LOCALVERSION=\"{suffix}\"\n",
        if modular { "y" } else { "n" },
        if modular { "m" } else { "y" },
        if modular { "m" } else { "y" }
    )
}
pub fn entry(release: &str) -> String {
    format!(
        "title Candidate\nlinux /vmlinuz-{release}\ninitrd /initramfs-{release}.img\noptions root=UUID=11111111-2222-3333-4444-555555555555 ro\n"
    )
}
pub fn root() -> boot::Root {
    boot::Root {
        uuid: "11111111-2222-3333-4444-555555555555".into(),
        label: "Supplied ext4 root".into(),
        required_drivers: ["nvme_core", "nvme", "jbd2", "mbcache", "ext4"]
            .map(str::to_owned)
            .to_vec(),
        init_present: true,
    }
}
pub fn image() -> Task {
    Task{id:"model-image-task".into(),kind:Kind::Image,instructions:"Build the supplied target image with external initramfs support and working NVMe/ext4 root drivers.".into(),sources:vec![Source{id:"new-source".into(),release:"6.6.50".into(),title:"Supplied target source".into()}],initial_source:0,wanted_source:0,initial_config:config(true,"-review").replace("CONFIG_BLK_DEV_NVME=m","CONFIG_BLK_DEV_NVME=n"),stage_root:"/review".into(),initial_entry_filename:"candidate.conf".into(),initial_entry:String::new(),initial_store:Store::default(),root:None,require_initrd_support:true,reference:vec![Action::Config{text:config(true,"-review")},Action::SaveConfig,Action::Run{input:"make all".into()},Action::WorkingInitramfs,Action::Check]}
}
pub fn boot() -> Task {
    let mut t = image();
    t.id = "model-staged-task".into();
    t.kind = Kind::Boot;
    t.root = Some(root());
    t.sources.insert(
        0,
        Source {
            id: "old-source".into(),
            release: "6.6.49".into(),
            title: "Working fallback source".into(),
        },
    );
    t.wanted_source = 1;
    t.initial_source = 0;
    let mut a = core::Artifacts::default();
    a.build(
        "old-source",
        "6.6.49",
        &core::Config::parse(&config(true, "-working")).unwrap(),
        core::Target::All,
    )
    .unwrap();
    let path = t
        .initial_store
        .image("/review", a.image.as_ref().unwrap())
        .unwrap();
    t.initial_store
        .modules("/review", a.modules.as_ref().unwrap())
        .unwrap();
    t.initial_store
        .assemble_initramfs("/review", &path)
        .unwrap();
    t.initial_store
        .entry("/review", "fallback.conf", &entry("6.6.49-working"))
        .unwrap();
    t.initial_entry = "title Still editing\nlinux /missing\n".into();
    t.reference = vec![
        Action::Source { index: 1 },
        Action::Config {
            text: config(true, "-review"),
        },
        Action::SaveConfig,
        Action::Run {
            input: "make all".into(),
        },
        Action::StageImage,
        Action::Run {
            input: "make INSTALL_MOD_PATH=/review modules_install".into(),
        },
        Action::Initramfs {
            image_path: "/review/boot/vmlinuz-6.6.50-review".into(),
        },
        Action::EntryDraft {
            filename: "candidate.conf".into(),
            text: entry("6.6.50-review"),
        },
        Action::SaveEntry,
        Action::Check,
    ];
    t
}
