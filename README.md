# 🌼 RIKA PLAYER — Spotify Lyric HUD Overlay

**RIKA PLAYER** adalah aplikasi desktop overlay lirik lagu Spotify untuk Linux (dibuat menggunakan Tauri & React) yang melayang di atas aplikasi lain (*always-on-top*) dengan transisi visual gulir sinematik yang sangat halus (*smooth-scrolling*). 

Aplikasi ini mendeteksi lagu yang sedang diputar di Spotify secara real-time melalui protokol MPRIS D-Bus, mencari lirik terperinci via API LrcLib, dan menampilkannya dengan tema gelap premium lengkap dengan dukungan terjemahan dwi-bahasa (dual-language lyrics).

---

## ✨ Fitur Utama

- **🎬 Smooth Cinematic Scrolling**: Lirik bergeser naik/turun secara dinamis dan mulus menggunakan transisi kurva *cubic-bezier* khusus. Baris aktif akan membesar dan mendapatkan fokus visual utama, sementara baris lainnya meredup secara halus.
- **🌐 Dual-Language Lyrics Support**: Otomatis mem-parsing dan menampilkan baris lirik asli beserta terjemahannya (jika tersedia di database) dalam satu baris aktif.
- **🔒 Interaction Lock (Click-Through Overlay)**: 
  - Tekan tombol gembok di UI, gunakan shortcut global **`Ctrl + Alt + K`**, atau jalankan **`./toggle.sh`** untuk mengunci/membuka HUD.
  - Saat dikunci, jendela aplikasi (header, background, border) akan menjadi transparan penuh dan mengabaikan klik mouse (*click-through*), menyisakan teks lirik mengambang indah di atas game, browser, atau code editor Anda.
- **📥 System Tray Support (100% Wayland Fallback)**: Menambahkan ikon di panel sistem tray (kanan bawah/atas). **Klik kiri** pada ikon tray untuk langsung mengunci/membuka HUD tanpa terpengaruh kebijakan pemblokiran shortcut global oleh Wayland. Anda juga bisa mengintegrasikan perintah `./toggle.sh` ke dalam Pintasan Keyboard bawaan OS Anda.
- **📌 Always on Top (Linux/Wayland)**: HUD dipaksa untuk selalu berada di lapisan teratas layar.
- **🎨 Premium Dark Aesthetics**: Menggunakan palet warna gelap pekat `#1e1e1e` dipadu dengan aksen gradasi *cyan/teal* (`#0da2b6`) serta header abu-abu terang yang menawan.
- **🖥️ Custom Title Bar & Controls**: Memiliki area drag terdedikasi di bagian atas jendela (header) serta tombol minimize, maximize, dan close kustom yang dirancang minimalis.

---

## 🛠️ Cara Kerja Sistem

1. **Rust Backend (`src-tauri/src/lib.rs`)**:
   - Memantau D-Bus Linux menggunakan *crate* `mpris` untuk melacak status Spotify.
   - Mengambil lirik ter-sinkronisasi (format LRC) secara otomatis dari API LrcLib berdasarkan judul, penyanyi, dan durasi lagu yang sedang diputar.
   - Mengirim status pemutar & teks lirik ke frontend melalui event emisi Tauri.
   - Mengontrol transparansi input jendela (click-through) secara dinamis.
2. **React Frontend (`src/App.tsx` & `src/App.css`)**:
   - Melakukan interpolasi posisi waktu lagu secara real-time menggunakan `requestAnimationFrame` untuk transisi pencarian indeks lirik yang presisi.
   - Menghitung tinggi dan posisi offsets lirik untuk memosisikan baris aktif selalu tepat di pusat jendela.

---

## 🚀 Cara Menjalankan Aplikasi

### 1. Prasyarat (Prerequisites)
Pastikan dependensi sistem Linux Anda sudah terinstal (terutama dbus, libsoup, dan pustaka Tauri standar):
```bash
# Untuk Debian/Ubuntu
sudo apt install libdbus-1-dev libsoup-3.0-dev webkit2gtk-4.1
```

### 2. Jalankan Mode Pengembangan (Dev Mode)
Gunakan port cleanup untuk menghindari bentrokan port dan jalankan dengan variabel lingkungan X11 agar fitur *Always-on-Top* bekerja sempurna di lingkungan desktop Linux (GNOME/KDE/Hyprland):
```bash
# Bersihkan port 1420 jika tersangkut dari sesi sebelumnya
fuser -k 1420/tcp

# Jalankan aplikasi dengan backend GDK X11
GDK_BACKEND=x11 npm run tauri dev
```

### 3. Build Production
Untuk mengemas aplikasi menjadi berkas eksekusi portabel `.deb`, `.rpm`, atau AppImage:
```bash
npm run tauri build
```

---

## ⌨️ Pintasan & Navigasi (Shortcuts & Interaction)

| Metode | Aksi | Deskripsi |
| :--- | :--- | :--- |
| **`Ctrl + Alt + K`** | Toggle Lock HUD | Mengaktifkan/menonaktifkan mode *Click-Through* (tembus klik) secara global (Khusus sesi X11). |
| **Klik Kiri Ikon Tray** | Toggle Lock HUD | Alternatif mengunci/membuka HUD secara instan (Bekerja 100% di Wayland & X11). |
| **Klik Kanan Ikon Tray** | Buka Menu Tray | Pilihan untuk Toggle Lock atau keluar dari aplikasi (*Quit*). |
| **`./toggle.sh`** | Toggle Lock HUD | Script pembantu lokal untuk mengunci/membuka kunci via terminal/hotkey OS (Bekerja 100% di semua lingkungan). |

---

## 📝 Catatan Tambahan (Troubleshooting)

- Jika jendela tertutup oleh aplikasi lain, pastikan Anda menjalankannya menggunakan variabel lingkungan `GDK_BACKEND=x11` sebelum memanggil Tauri. Ini memaksa *Window Manager* di Linux untuk memprioritaskan status *Always-on-Top* jendela HUD.
