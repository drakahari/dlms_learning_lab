# Native palette inventory and provenance

Checked 2026-10-21. **28 choices:** the accepted Learning Lab Navy default, **all five** stock DLMS palettes and **all 22** official bundled Omarchy palettes at the pinned source revision. No palettes omitted. DLMS Ethereal and upstream Omarchy Ethereal are distinct entries, deliberately not deduplicated.

## Sources and permissions

- DLMS: read-only inspection of the existing `dlms_desktop/dlms/themes.py` `LEGACY_PALETTES` implementation (identical stock definitions also observed in `DLMS_v3_dev`). Source SHA-256 is recorded in `assets/themes/dlms-stock.json`; the inspected copy had no accessible Git metadata, so no commit is invented. The five palettes and their original names/values are retained separately from native derivation. `assets/themes/LICENSE.dlms` preserves the supplied MIT notice, copyright 2026 Mike Buchanan.
- Omarchy: [official repository](https://github.com/omacom/omarchy), release [v4.0.4](https://github.com/omacom/omarchy/releases/tag/v4.0.4), commit **c668141e9c42b13c80c9ca4ea108e11708c5e8a5**. Downloaded the official archive at that immutable commit into temporary development storage. Its complete `themes/*/colors.toml` inventory matched all 22 entries in the DLMS manifest; every file's SHA-256 and parsed fields matched. Raw palette files, immutable URLs, hashes and original values are in `assets/themes/`. `LICENSE.omarchy` retains the official MIT notice, copyright David Heinemeier Hansson.
- Only color data is imported. No wallpapers, fonts, GTK/Hyprland configs, web CSS, theme scripts or host installation steps are shipped/applied. These permissive notices allow inclusion without selecting a license for this repository. The repository owner's root-license decision remains pending.
- GitHub's browser tree view was unavailable; the official archive was successfully verified. There were no unavailable/incompatible palette sources at the selected revision. This is a reproducible pinned inventory, **not** a claim to include later additions from upstream HEAD or community theme repositories.

## Inventory

| Group | Included picker names |
| --- | --- |
| Learning Lab | Learning Lab Navy (existing default) |
| DLMS | Dark; Light; Purple & Gold; Maroon & Gold; Ethereal |
| Omarchy light | catppuccin-latte; flexoki-light; lupine; rose-pine; white |
| Omarchy dark | catppuccin; ethereal; everforest; gruvbox; hackerman; kanagawa; last-horizon; lumon; matte-black; miasma; nord; osaka-jade; retro-82; ristretto; solitude; tokyo-night; vantablack |

Saved IDs are namespaced (`lab-navy`, `dlms-*`, `omarchy-*`). The picker previews immediately; **Settings → Appearance → Theme samples** shows explicitly labeled example native terminal/editor/action/selection/disabled/error/success samples and full attribution. Selection and text size use independent version-1 atomic local preferences. Existing progress format and attempts are not rewritten to choose a theme. Invalid or future preference files are retained; the UI reports an appearance-only problem and uses the default. Unknown IDs fall back on read without rewriting the file.

## Native role adaptation and verification targets

`src/theme.rs` derives opaque background, panel, input, control and hover surfaces from source colors. Body text targets 7:1; secondary/accent/error/success text targets 4.5:1 on every defined surface; control/focus boundaries target 3:1; text on accent/selection targets 4.5:1. Original source values remain intact. Colors are adjusted toward black/white only as necessary; source terminal ANSI names are not used as educational success/error semantics. Semantic feedback always includes words such as Pass, Needs correction, Running or Stopped.

Selected text uses egui's selection foreground and fill; selected items have contrast-aware text. Hover/keyboard focus have a contrasting outline. Disabled samples explicitly say unavailable/idle and remain readable instead of fading text below the contrast target. Terminals/editors follow the selected native palette, including light palettes; no hard-coded navy text surfaces remain. These token checks plus rendered Fedora review are bounded evidence, not full WCAG/assistive-technology acceptance. See milestone verification for what ran.
