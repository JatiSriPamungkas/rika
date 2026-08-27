#!/bin/bash
# Script untuk memperbarui ikon & nama aplikasi Rika di Tauri & Linux Taskbar / Dock

RAW_ICON="$1"

if [ -z "$RAW_ICON" ]; then
    if [ -f "rika-logo.png" ]; then
        RAW_ICON="rika-logo.png"
    elif [ -f "rika_icon.png" ]; then
        RAW_ICON="rika_icon.png"
    else
        echo "❌ File ikon tidak ditemukan! Harap sediakan gambar PNG di root folder."
        exit 1
    fi
fi

if [ ! -f "$RAW_ICON" ]; then
    echo "❌ File ikon '$RAW_ICON' tidak ditemukan!"
    echo "Cara pakai: ./update_icon.sh path/ke/gambar.png"
    exit 1
fi

echo "📐 1. Memproses $RAW_ICON menjadi format Square (930x930)..."
python3 -c "
import os
from PIL import Image

img = Image.open('$RAW_ICON')
width, height = img.size
max_dim = max(width, height)
padded_dim = int(max_dim * 1.2)
square_img = Image.new('RGBA', (padded_dim, padded_dim), (0, 0, 0, 0))
offset_x = (padded_dim - width) // 2
offset_y = (padded_dim - height) // 2
square_img.paste(img, (offset_x, offset_y), img if img.mode == 'RGBA' else None)

os.makedirs('src-tauri/icons', exist_ok=True)
square_img.save('src-tauri/icons/icon.png')

os.makedirs('public', exist_ok=True)
img.save('public/rika-logo.png')
"

echo "🎨 2. Menghasilkan semua ukuran ikon Tauri..."
npx tauri icon "src-tauri/icons/icon.png"

echo "🧹 3. Cleansing cache kompilasi Rust..."
touch src-tauri/src/lib.rs
rm -rf src-tauri/target/debug/rika src-tauri/target/debug/tauri-app src-tauri/target/debug/deps/*rika* src-tauri/target/debug/deps/*tauri* 2>/dev/null || true

echo "🐧 4. Mendaftarkan ikon & nama Rika ke sistem Linux Dock (~/.local/share/icons & applications)..."
mkdir -p ~/.local/share/icons/hicolor/512x512/apps
mkdir -p ~/.local/share/icons/hicolor/128x128/apps
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
mkdir -p ~/.local/share/pixmaps

NAMES=("rika" "com.rika.player" "tauri-app" "Tauri-app")

for name in "${NAMES[@]}"; do
    cp src-tauri/icons/icon.png ~/.local/share/icons/hicolor/512x512/apps/"${name}.png" 2>/dev/null || true
    cp src-tauri/icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/"${name}.png" 2>/dev/null || true
    cp src-tauri/icons/icon.png ~/.local/share/pixmaps/"${name}.png" 2>/dev/null || true
done

# Mendaftarkan .desktop entry Rika
mkdir -p ~/.local/share/applications

cat <<EOF > ~/.local/share/applications/rika.desktop
[Desktop Entry]
Name=Rika
Exec=rika
Icon=rika
Type=Application
Terminal=false
Categories=Utility;Audio;
StartupWMClass=rika
EOF

cat <<EOF > ~/.local/share/applications/com.rika.player.desktop
[Desktop Entry]
Name=Rika
Exec=rika
Icon=com.rika.player
Type=Application
Terminal=false
Categories=Utility;Audio;
StartupWMClass=com.rika.player
EOF

cat <<EOF > ~/.local/share/applications/tauri-app.desktop
[Desktop Entry]
Name=Rika
Exec=rika
Icon=rika
Type=Application
Terminal=false
Categories=Utility;Audio;
StartupWMClass=tauri-app
EOF

# Refresh icon cache jika ada gtk-update-icon-cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
fi

echo "✨ SELESAI! Ikon & Nama aplikasi Rika berhasil diperbarui di Tauri & Linux Dock."
echo "Silakan jalankan ulang app: GDK_BACKEND=x11 npm run tauri dev"
