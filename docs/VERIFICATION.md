# IPv4 teaching pilot: verification

Checked 2026-10-21 on Fedora 44 / COSMIC Wayland, x86_64, Rust 1.99.1, egui/eframe 0.33.3 with native Glow/OpenGL. No new dependency or system package. Nine authored labs remain; numerical practice is not counted as new curriculum. No release or installer publication.

## Results

- `cargo build --locked --offline`: passed; runnable `target/debug/dlms-learning-lab`.
- `cargo test --locked --offline`: **44 passed**, zero failures (14 egui UI tests, eight focused IPv4 model tests, 22 engine/content/progress/catalog regression tests).
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `--validate-pack scenarios/starter-v1.json`: passed; all nine reference solutions still reach their goals.

The tests verify known network/broadcast/host results at /0, /8, /20, /23, /25, /27 and /30, including carries between octets and upper IPv4 boundaries. Every supported prefix /0…/30 exercises every network/host bit position: network bits cannot toggle, host-bit toggles change the expected binary weight, and all-zero/all-one host patterns match both boundaries. /31, /32 and invalid larger prefixes are explicitly rejected by the lesson math.

Five hundred deterministic generated cases produce valid private host addresses across multiple prefixes, have unique tested address/prefix pairs, and agree with the simulation grader when solved. Numerical variants do not enter the catalog. Tests distinguish blank, malformed, host-as-network, wrong block boundary, reserved host endpoint, outside-subnet and wrong usable-end cases. Feedback is checked not to dump the expected address.

Prediction/reveal gates are tested separately: typing without checking reveals nothing; a wrong prediction can unlock requested help; blank predictions cannot; broadcast cannot inherit network review permission; usable-range review requires both endpoint predictions. Clearing an answer preserves requested review history but removes current success. Original and practice answers, walkthrough positions and grading are independent. Resume preserves malformed practice drafts, original completion and other labs. A legacy fixture still loads all nine version-1 attempts without a read-time rewrite.

UI tests cover native Check/Reveal state, Challenge hiding prior Learn review, a separate initially collapsed Challenge sandbox, bit focus/click/Space behavior, fixed-bit protection, unchanged answer fields during sandbox use, completion/invalidation, reset isolation, and old save compatibility. Octet and answer-field bounds are checked at 480–1400 logical/physical test widths with 100%/150% scaling and prefixes /8, /16, /20, /23, /27 and /30. Existing library alignment/filter/scale, VLSM, service editor, insertion and large-catalog checks remain passing.

## What ran on the Fedora desktop

`--ipv4-smoke` opens the real native window and renderer, uses fresh isolated data under `/tmp/dlms-ipv4-smoke`, and injects egui input events. It verifies a typed host-address prediction, native Check and Reveal button clicks, no unrequested worked result, a sandbox host-bit click with the resulting decimal update, independent Challenge practice, and native bit bounds at 150% text size. The window moves between a tiled 1268×632 framebuffer and 2560×1440 full-screen. Some broadcast/range review states are seeded through the tested lesson model to make captures deterministic; this is not a claim of physical mouse/keyboard or complete human end-to-end acceptance.

Screenshots are application-only, local and ignored by Git:

| View | Capture |
| --- | --- |
| Four original octets, separate mask, persistent explanation | `screenshots/ipv4/01-original-and-mask.png` |
| Specific host-versus-network misconception | `screenshots/ipv4/02-host-versus-network-feedback.png` |
| Explicitly requested network result | `screenshots/ipv4/03-requested-network-result.png` |
| Explicitly requested broadcast result | `screenshots/ipv4/04-requested-broadcast-result.png` |
| Requested usable endpoints | `screenshots/ipv4/05-requested-usable-range.png` |
| Host-bit sandbox and updated decimal value | `screenshots/ipv4/06-host-bit-sandbox.png` |
| Fresh generated Challenge case | `screenshots/ipv4/07-fresh-challenge-variant.png` |
| Stacked view at 150% | `screenshots/ipv4/08-stacked-150-percent.png` |
| Alex-specific permissions library description | `screenshots/ipv4/09-permissions-description.png` |

## Limits and compatibility

- Original scenario IDs, versions, goals and answer fields are unchanged. Optional lesson metadata adds predictions/reviews, host sandbox state and the active numerical variant; legacy step indexes map conservatively to the four views. Original/practice positions are stored independently. Reset explicitly clears the selected lab’s generated practice too. The original completion flag is still derived from original state, never from review or generated success.
- New optional metadata is written only on later saves. Earlier binaries do not support these added fields; downgrade compatibility is not claimed. Normal progress was not used by native test harnesses. Corrupt/incompatible originals remain preserved with a read-only explanation; safe atomic replacement is unchanged. Concurrent writers remain unsupported.
- Prefixes /31 and /32 remain excluded from this conventional broadcast-subnet exercise. The generator uses a selection of prefixes between /8 and /30; the underlying lesson math also handles /0…/7. No IPv6, routing, packets, real shell or host networking is performed.
- Tooltips, meaningful accessible bit labels, textual N/H/H* markers and persistent focus explanations are implemented. Keyboard behavior and geometry were tested, but screen-reader, IME, physical-device, high-contrast-theme and multi-monitor DPI acceptance were not performed. Small windows need vertical scrolling.
- Windows/macOS were not built or run in this pass. No broader curriculum conversion or full exam coverage is claimed. License selection remains the owner’s pending decision.
