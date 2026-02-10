#!/usr/bin/env bash
set -euo pipefail

APPIMAGE=${1:-/home/winget/fourchef/target/release/bundle/appimage/4chef_0.1.0_amd64.AppImage}

if [[ ! -f "$APPIMAGE" ]]; then
  echo "AppImage not found: $APPIMAGE" >&2
  exit 1
fi

OFFSET=$("$APPIMAGE" --appimage-offset)
TMPDIR=$(mktemp -d)
SQUASH="$TMPDIR/app.squashfs"
APPDIR="$TMPDIR/AppDir"

# Extract squashfs and AppDir (ignore xattrs)
 tail -c +$((OFFSET+1)) "$APPIMAGE" > "$SQUASH"
 unsquashfs -no-xattrs -d "$APPDIR" "$SQUASH" >/dev/null

LIBDIR="$APPDIR/usr/lib"

# Remove problematic bundled libs to force system libs
rm -f \
  "$LIBDIR"/libsystemd.so.* \
  "$LIBDIR"/libudev.so.* \
  "$LIBDIR"/libcap.so.* \
  "$LIBDIR"/libffi.so.* \
  "$LIBDIR"/libglib-2.0.so.* \
  "$LIBDIR"/libgobject-2.0.so.* \
  "$LIBDIR"/libgio-2.0.so.* \
  "$LIBDIR"/libgmodule-2.0.so.* \
  "$LIBDIR"/libdbus-1.so.* \
  "$LIBDIR"/libXau.so.* \
  "$LIBDIR"/libX11*.so* \
  "$LIBDIR"/libxcb*.so* \
  "$LIBDIR"/libmp3lame.so.* \
  "$LIBDIR"/libmpg123.so.* \
  "$LIBDIR"/libogg.so.* \
  "$LIBDIR"/libvorbis.so.* \
  "$LIBDIR"/libvorbisenc.so.* \
  "$LIBDIR"/libFLAC.so.* \
  "$LIBDIR"/libsndfile.so.* \
  "$LIBDIR"/libopus.so.* \
  "$LIBDIR"/libpulse*.so* \
  "$LIBDIR"/libgsm.so.* \
  "$LIBDIR"/libasyncns.so.* \
  "$LIBDIR"/libspeex*.so* \
  "$LIBDIR"/libvmaf.so.* \
  "$LIBDIR"/libnettle.so.* \
  "$LIBDIR"/libhogweed.so.* \
  "$LIBDIR"/libgnutls.so.* \
  "$LIBDIR"/libp11-kit.so.* \
  "$LIBDIR"/libcrypto.so.* \
  "$LIBDIR"/libssl.so.* \
  "$LIBDIR"/libzstd.so.* || true

# Replace AppRun to bypass bundled LD_LIBRARY_PATH
cat > "$APPDIR/AppRun" <<'APP_RUN'
#!/usr/bin/env bash
set -e
HERE="$(readlink -f "$(dirname "$0")")"
# Prefer system libraries; keep AppDir for resources only
export LD_LIBRARY_PATH="/usr/lib64:/lib64:/usr/lib:/lib"
exec "$HERE/usr/bin/fourchef" "$@"
APP_RUN
chmod +x "$APPDIR/AppRun"

# Repack AppImage using appimagetool embedded in linuxdeploy plugin
APPIMAGETOOL=/tmp/appimage_extracted_7e8589db794e038210094a32fc3949f1/appimagetool-prefix/usr/bin/appimagetool
if [[ ! -x "$APPIMAGETOOL" ]]; then
  echo "appimagetool not found at $APPIMAGETOOL" >&2
  exit 1
fi

ARCH=x86_64 "$APPIMAGETOOL" "$APPDIR" "$APPIMAGE" >/dev/null

echo "Repacked AppImage with libs removed and AppRun overridden: $APPIMAGE"
