//! Fixed current model probe. No caller script, key or file selector.
use dlms_learning_lab::modal_editing::{Editor, MODEL_INPUTS};
#[path = "support/automation_digest.rs"]
mod digest;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(
        args.is_empty()
            || matches!(args.as_slice(),[name] if name=="navigation" || name=="insertion"),
        "Closed fixed profile only"
    );
    let profile = args.first().map(String::as_str).unwrap_or("foundation");
    let cases = [
        ("empty-write", "", "", "wq"),
        ("empty-line-write", "\n", "", "wq"),
        ("delete-last-line", "abc\n", "dd", "wq"),
        ("delete-last-character", "x\n", "x", "wq"),
        ("insert-text", "abc\n", "iX\x1b", "wq"),
        ("insert-without-typing", "abc\n", "li\x1b", "wq"),
        ("append-without-typing", "abc\n", "la\x1b", "wq"),
        ("grouped-undo", "abc\n", "iHello\x1bu", "wq"),
        ("open-empty-buffer", "", "o\x1b", "wq"),
        ("discard-edit", "abc\n", "iX\x1b", "q!"),
    ];
    let navigation_cases = [
        ("vertical-column", "abcdef\nx\nuvwxyz\n", "lllljj", "wq"),
        ("vertical-end", "abc\nx\nuvwxyz\n", "$jj", "wq"),
        ("vertical-reset", "abcdef\nx\nuvwxyz\n", "llllj0j", "wq"),
        (
            "vertical-left-reset",
            "abcdef\nx\nuvwxyz\n",
            "lllljhj",
            "wq",
        ),
        ("jump-first", "  abc\nx\n  last\n", "Ggg", "wq"),
        ("jump-last", "abc\nx\n  last\n", "G", "wq"),
        ("copy-after", "abc\ndef\n", "yyp", "wq"),
        ("copy-before", "abc\ndef\n", "jyyP", "wq"),
        ("move-line", "abc\ndef\n", "ddp", "wq"),
        ("restore-only-line", "abc\n", "ddp", "wq"),
        ("copy-empty-line", "\nabc\n", "yyp", "wq"),
        ("put-character-after", "abc\n", "xp", "wq"),
        ("put-character-before", "abc\n", "xP", "wq"),
        ("undo-put", "abc\ndef\n", "yypu", "wq"),
        ("put-survives-undo", "abc\ndef\n", "ddup", "wq"),
        ("search-forward", "red blue\nred green\n", "/red\r", "wq"),
        ("search-repeat", "red blue\nred green\n", "/red\rn", "wq"),
        ("search-backward", "red blue\nred green\n", "G?red\r", "wq"),
        ("search-wrap", "red blue\nred green\n", "?red\r", "wq"),
        ("search-overlap", "aaa\n", "/aa\rn", "wq"),
        (
            "search-repeat-empty",
            "red blue\nred green\n",
            "/red\r/\r",
            "wq",
        ),
    ];
    let insertion_cases = [
        ("typed-backspace", "abc\n", "iXY\x08\x1b", "wq"),
        ("backspace-insert-start", "abc\n", "li\x08\x1b", "wq"),
        ("backspace-newline", "abc\n", "aX\rY\x08\x08\x1b", "wq"),
        ("command-backspace", "abc\n", ":wx\x08\r", "wq"),
        ("put-into-blank", "x\n", "xpp", "wq"),
        ("line-before-empty", "abc\n", "ddP", "wq"),
        ("open-backspace", "abc\n", "oX\x08\x1b", "wq"),
        ("undo-erased-insert", "abc\n", "iX\x08\x1bu", "wq"),
    ];
    let cases = match profile {
        "navigation" => &navigation_cases[..],
        "insertion" => &insertion_cases[..],
        _ => &cases[..],
    };
    let mut rows = vec![];
    for &(id, before, keys, finish) in cases {
        let mut e = Editor::new(before).unwrap();
        e.replay(keys).unwrap();
        let (row, col) = e.cursor().unwrap();
        e.replay(&format!(":{finish}\r")).unwrap();
        assert!(e.closed());
        rows.push(serde_json::json!({"id":id,"before":before,"keys_hex":keys.as_bytes().iter().map(|v|format!("{v:02x}")).collect::<String>(),"finish":finish,"written":e.file_bytes(),"cursor_one_based":[row+1,col+1]}));
    }
    let inputs:Vec<_>=MODEL_INPUTS.iter().map(|(name,raw)|serde_json::json!({"path":name,"bytes":raw.len(),"sha256":digest::hash(raw)})).collect();
    println!(
        "{}",
        serde_json::json!({"purpose":"fixed-modal-model-probe-v1","selected_linked_inputs":inputs,"probe_sha256":digest::hash(include_bytes!("modal_editing_reference.rs")),"observations":rows,"scope":"Selected linked model input identity and closed fixed current profile only; not full compiler closure, complete vi or lesson acceptance"})
    );
}
