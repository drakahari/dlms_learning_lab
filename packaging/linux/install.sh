#!/bin/sh
# Installs application resources only. Learner commands never run this script.
set -eu
source_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix=${HOME}/.local/opt/dlms-learning-lab
bin_dir=${HOME}/.local/bin
desktop_dir=${XDG_DATA_HOME:-${HOME}/.local/share}/applications
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix|--bin-dir|--desktop-dir)
            [ "$#" -ge 2 ] || { printf 'Missing option value\n' >&2; exit 2; }
            case "$1" in --prefix) prefix=$2;; --bin-dir) bin_dir=$2;; --desktop-dir) desktop_dir=$2;; esac
            shift 2;;
        *) printf 'Usage: sh install.sh [--prefix ABSOLUTE_DIR] [--bin-dir ABSOLUTE_DIR] [--desktop-dir ABSOLUTE_DIR]\n' >&2; exit 2;;
    esac
done
for path in "$prefix" "$bin_dir" "$desktop_dir"; do
    case "$path" in /*) :;; *) printf 'Installation paths must be absolute\n' >&2; exit 2;; esac
    case "$path" in *'"'*|*'`'*|*'$'*|*'\'*|*'%'*) printf 'Choose paths without quotes, backticks, dollar signs, percent signs or backslashes\n' >&2; exit 2;; esac
    if printf '%s' "$path" | LC_ALL=C grep -q '[[:cntrl:]]'; then printf 'Control characters are not allowed in paths\n' >&2; exit 2; fi
    [ ! -L "$path" ] || { printf 'Refusing a symbolic-link installation directory\n' >&2; exit 2; }
done
command -v realpath >/dev/null 2>&1 || { printf 'This Linux installer needs coreutils realpath\n' >&2; exit 2; }
prefix=$(realpath -m -- "$prefix")
bin_dir=$(realpath -m -- "$bin_dir")
desktop_dir=$(realpath -m -- "$desktop_dir")
for first in "$prefix" "$bin_dir" "$desktop_dir"; do
    for second in "$prefix" "$bin_dir" "$desktop_dir"; do
        [ "$first" = "$second" ] && continue
        case "$first" in "$second"/*) printf 'Application, launcher and desktop folders must be separate, non-nested locations\n' >&2; exit 2;; esac
    done
done
[ "$prefix" != "$bin_dir" ] && [ "$prefix" != "$desktop_dir" ] && [ "$bin_dir" != "$desktop_dir" ] || { printf 'Application, launcher and desktop folders must differ\n' >&2; exit 2; }
[ "$prefix" != / ] && [ "$prefix" != "$HOME" ] || { printf 'Choose a dedicated application folder\n' >&2; exit 2; }
if [ -e "$prefix" ]; then
    [ -f "$prefix/.dlms-installation" ] && [ ! -L "$prefix/.dlms-installation" ] || { printf 'Existing folder is not a DLMS installation; it was retained\n' >&2; exit 2; }
    [ -f "$prefix/INSTALL_FILES.txt" ] && [ ! -L "$prefix/INSTALL_FILES.txt" ] || exit 2
    # Do not discard user-added files during an upgrade.
    links=$(find "$prefix" -type l)
    [ -z "$links" ] || { printf 'Changed symbolic-link resources retained; restore them before upgrading\n' >&2; exit 2; }
    files=$(find "$prefix" -type f)
    printf '%s\n' "$files" | while IFS= read -r file; do
        relative=${file#"$prefix"/}
        case "$relative" in .dlms-installation|.install-paths) continue;; esac
        grep -F -x -q -- "$relative" "$prefix/INSTALL_FILES.txt" || { printf 'Unrecognized file in application folder; retain/move it before upgrading: %s\n' "$relative" >&2; exit 2; }
    done
fi
launcher=$bin_dir/dlms-learning-lab
if [ -e "$launcher" ] || [ -L "$launcher" ]; then
    [ -L "$launcher" ] && [ "$(readlink "$launcher")" = "$prefix/dlms-learning-lab" ] || { printf 'Existing launcher is unrelated; retained\n' >&2; exit 2; }
fi
entry=$desktop_dir/org.dlms.LearningLab.desktop
if [ -e "$entry" ] || [ -L "$entry" ]; then
    [ ! -L "$entry" ] && grep -q '^X-DLMS-Learning-Lab=true$' "$entry" && grep -F -q -- "Exec=\"$prefix/dlms-learning-lab\"" "$entry" || { printf 'Existing desktop entry is unrelated; retained\n' >&2; exit 2; }
fi
stage=$prefix.new.$$
[ ! -e "$stage" ] && [ ! -L "$stage" ] || exit 2
mkdir -p -- "$(dirname -- "$prefix")" "$bin_dir" "$desktop_dir"
mkdir -- "$stage"
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
cp -R -- "$source_dir/." "$stage/"
printf 'DLMS Learning Lab application resources\n' > "$stage/.dlms-installation"
printf '%s\n%s\n' "$bin_dir" "$desktop_dir" > "$stage/.install-paths"
old=$prefix.previous.$$
[ ! -e "$old" ] && [ ! -L "$old" ] || exit 2
if [ -e "$prefix" ]; then mv -- "$prefix" "$old"; fi
if ! mv -- "$stage" "$prefix"; then
    if [ -e "$old" ]; then mv -- "$old" "$prefix"; fi
    exit 1
fi
ln -sf -- "$prefix/dlms-learning-lab" "$launcher"
cat > "$entry" <<EOF
[Desktop Entry]
Type=Application
Name=DLMS Learning Lab
Comment=Offline networking, Linux and shell practice
Exec="$prefix/dlms-learning-lab"
Icon=$prefix/icon.svg
Terminal=false
Categories=Education;
X-DLMS-Learning-Lab=true
EOF
if [ -e "$old" ]; then rm -rf -- "$old"; fi
printf 'Installed. Open DLMS Learning Lab from the application menu, or run:\n%s\nProgress remains in the separate application-data folder.\n' "$launcher"
