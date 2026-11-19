use super::f;
use dlms_learning_lab::rpm_packages as rpm;
pub const PATH: &str = "/etc/site.conf";
pub fn fixture(kind: rpm::FileKind, local: Option<&str>, incoming: &str) -> rpm::State {
    let state = f::state();
    let mut c = state.catalog().clone();
    let mut r = state.runtime().clone();
    let old = c.archives["site-v1"].files[PATH].file.clone();
    for id in ["site-v1", "site-v2"] {
        let p = c.archives.get_mut(id).unwrap().files.get_mut(PATH).unwrap();
        p.kind = kind;
        p.file = old.clone();
    }
    let new = c
        .archives
        .get_mut("site-v2")
        .unwrap()
        .files
        .get_mut(PATH)
        .unwrap();
    new.file.contents = incoming.into();
    new.file.mtime = 200;
    match local {
        Some(text) => {
            let mut file = old;
            file.contents = text.into();
            file.mtime = 99;
            r.files.insert(PATH.into(), file);
        }
        None => {
            r.files.remove(PATH);
        }
    }
    c.allowed_paths
        .extend([format!("{PATH}.rpmnew"), format!("{PATH}.rpmsave")]);
    rpm::State::new(c, r).unwrap()
}
