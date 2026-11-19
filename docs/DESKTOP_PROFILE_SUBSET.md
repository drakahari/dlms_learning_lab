# Supplied graphical workstation profiles

This LPIC-1 unit covers parts of v5.0 objectives 106.1, 106.2 and 106.3.
Each lesson has a different virtual profile for the guided example, with
independent saved drafts and status.

- **Xorg device layout.** Model one native Xorg session with two to six
  supplied keyboard devices and up to eight ordered InputClass sections.
  Accept quoted literal Identifier, MatchProduct, MatchIsKeyboard and
  Option "XkbLayout". Matching classes apply in configuration order.
  MatchProduct searches the supplied product name for a literal substring.
  The lesson renders only four unmodified key positions for the US,
  German and French layouts. Applying validates the virtual draft before
  replacing the previous applied text. Native Wayland uses a different
  configuration path. This workspace excludes real hardware, full XKB
  maps or Xorg functionality, display managers, X authority and networked
  X applications.
- **GNOME desktop profiles.** The supplied modern GNOME configuration
  models existing-session Desktop Sharing in view-only or control mode,
  and independent Remote Login. When both are enabled, the fixture gives
  Remote Login TCP 3389 and sharing TCP 3390. When sharing is on alone it
  uses TCP 3389. Input trials use fixed RDP-ready or VNC-only client
  records and faculty-supplied access grants. They model no actual
  connections, credentials, encryption, traffic, live sessions or VNC,
  XDMCP and SPICE behavior.
- **Visual and keyboard accessibility samples.** Large text and contrast
  visibly affect only the reading sample. The separate Shift probe
  illustrates a simple Sticky Keys latch. The timing probe processes a
  short, ordered set of mapped physical events. It offers one supplied
  delay for Slow Keys and one for Bounce Keys. It does not read real
  key events, create a chord, synthesize host input, modify any desktop
  setting, use assistive hardware, speak the screen, or imply a model
  of all XKB AccessX or GNOME behavior.

## Primary references checked November 17, 2026

- [LPI LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [Xorg InputClass documentation](https://manpages.debian.org/trixie/xserver-xorg-core/xorg.conf.5.en.html#INPUTCLASS_SECTION)
- [X.org guide to XKB](https://www.x.org/guide/hutterer-kbd/)
- [GNOME Desktop Sharing](https://help.gnome.org/gnome-help/sharing-desktop.html)
- [GNOME Remote Login](https://help.gnome.org/gnome-help/remote-login.html)
- [GNOME Accessibility](https://help.gnome.org/gnome-help/a11y)
- [GNOME Bounce Keys](https://help.gnome.org/gnome-help/a11y-bouncekeys.html)
