#!/bin/sh
# Removes owned application files; local progress and user-added files are retained.
set -eu
prefix=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
[ -f "$prefix/.dlms-installation" ] && [ ! -L "$prefix/INSTALL_FILES.txt" ] && [ -f "$prefix/INSTALL_FILES.txt" ] || { printf 'Not an installed DLMS application folder\n' >&2; exit 2; }
[ -f "$prefix/.install-paths" ] && [ ! -L "$prefix/.install-paths" ] || exit 2
# Refuse changed links anywhere before touching launchers or resources. Checking
# only final filenames would follow a replaced intermediate directory on removal.
links=$(find "$prefix" -type l)
[ -z "$links" ] || { printf 'Changed symbolic-link resources retained; nothing removed\n' >&2; exit 2; }
manifest=$(cat "$prefix/INSTALL_FILES.txt")
printf '%s\n' "$manifest" | while IFS= read -r relative; do
    case "$relative" in ''|/*|.|./*|*/./*|*/.|..|../*|*/../*|*/..|*//*|*/) printf 'Invalid installation manifest; nothing removed\n' >&2; exit 2;; esac
    if printf '%s' "$relative" | LC_ALL=C grep -q '[[:cntrl:]]'; then printf 'Invalid manifest characters; nothing removed\n' >&2; exit 2; fi
done
bin_dir=$(sed -n '1p' "$prefix/.install-paths")
desktop_dir=$(sed -n '2p' "$prefix/.install-paths")
launcher=$bin_dir/dlms-learning-lab
if [ -L "$launcher" ] && [ "$(readlink "$launcher")" = "$prefix/dlms-learning-lab" ]; then rm -- "$launcher"; fi
entry=$desktop_dir/org.dlms.LearningLab.desktop
if [ -f "$entry" ] && [ ! -L "$entry" ] && grep -q '^X-DLMS-Learning-Lab=true$' "$entry" && grep -F -q -- "Exec=\"$prefix/dlms-learning-lab\"" "$entry"; then rm -- "$entry"; fi
# The complete manifest was validated before any removal.
printf '%s\n' "$manifest" | while IFS= read -r relative; do
    case "$relative" in ''|/*|..|../*|*/../*|*/..) printf 'Invalid installation manifest; retained\n' >&2; exit 2;; esac
    [ ! -L "$prefix/$relative" ] || { printf 'Changed symbolic-link file retained\n' >&2; exit 2; }
    rm -f -- "$prefix/$relative"
done
rm -f -- "$prefix/.dlms-installation" "$prefix/.install-paths"
find "$prefix" -depth -type d -empty -delete
printf 'Application removed. Progress and user-added files were retained.\n'
