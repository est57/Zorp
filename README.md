<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="96" height="96" alt="Ikon Zorp">
</p>

<h1 align="center">Zorp</h1>

<p align="center">Klien API desktop yang ringan untuk menyusun, mengirim, dan menguji request HTTP.</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/screenshot-dark.png">
    <img src="docs/screenshot-light.png" alt="Tampilan Zorp dengan koleksi di sidebar, editor request, dan response JSON">
  </picture>
</p>

## Fitur

- **Request HTTP**: semua method umum, query params yang tersinkron dengan URL, header, body JSON, raw, form-urlencoded, form-data, dan binary.
- **GraphQL**: editor query dan variables, dengan autocomplete dan validasi dari skema hasil introspeksi.
- **Koleksi**: folder bertingkat, drag-and-drop, multi-tab, dan riwayat request.
- **Environment**: variabel `{{nama}}` dengan penanda variabel yang tidak ditemukan. Nilai rahasia disimpan di keychain sistem.
- **Auth**: Bearer, Basic, API key, dan OAuth 2.0 (Authorization Code dengan PKCE, Client Credentials, refresh otomatis). Auth bisa diwariskan dari folder atau koleksi.
- **Script**: pre-request dan test script dengan API mirip Postman (`pm.environment`, `pm.test`, `pm.expect`, dan lainnya).
- **Collection runner**: menjalankan seluruh koleksi atau folder secara berurutan beserta ringkasan tes.
- **Impor dan ekspor**: koleksi Postman v2.1, perintah cURL, serta code snippet untuk cURL, fetch, axios, Python, dan Go.
- **Jaringan**: cookie jar per workspace, rincian waktu (DNS, koneksi, tunggu server, unduh), proxy HTTP/HTTPS/SOCKS5, sertifikat CA kustom, dan sertifikat klien (mTLS).
- **Command palette** (`⌘K` / `Ctrl K`) untuk mencari request dan menjalankan aksi.
- **Pembaruan otomatis**: Zorp memeriksa versi baru saat dibuka dan dapat memasangnya langsung dari aplikasi.

## Instalasi

Unduh installer untuk sistem operasi Anda dari halaman [Releases](https://github.com/est57/Zorp/releases).

| Sistem | File |
|---|---|
| macOS (Apple Silicon) | `Zorp_<versi>_aarch64.dmg` |
| macOS (Intel) | `Zorp_<versi>_x64.dmg` |
| Windows | `Zorp_<versi>_x64-setup.exe` atau `.msi` |
| Linux | `.AppImage`, `.deb`, atau `.rpm` |

Installer saat ini belum ditandatangani (unsigned), sehingga sistem operasi akan menampilkan peringatan saat pertama kali dibuka.

- **macOS**: buka Zorp sekali, lalu buka **System Settings > Privacy & Security** dan klik **Open Anyway**. Cara lain lewat Terminal:
  ```bash
  xattr -dr com.apple.quarantine /Applications/Zorp.app
  ```
- **Windows**: pada layar SmartScreen, klik **More info**, lalu **Run anyway**.
- **Linux (AppImage)**: jadikan file dapat dieksekusi dengan `chmod +x Zorp*.AppImage` (atau `zorp*.AppImage`, sesuai nama file yang diunduh).

### Pembaruan

Mulai versi 0.2.0, Zorp memeriksa versi baru beberapa detik setelah dibuka. Jika ada, status bar menampilkan **Versi x.y.z tersedia** beserta tombol **Perbarui**. Setelah unduhan selesai, klik **Mulai ulang** untuk memakai versi baru. Pemeriksaan juga bisa dijalankan kapan saja lewat command palette dengan perintah **Periksa pembaruan**.

Setiap pembaruan ditandatangani dan diverifikasi oleh aplikasi sebelum dipasang. Versi 0.1.0 belum memiliki fitur ini, sehingga perlu diperbarui manual satu kali.

## Shortcut

| Aksi | macOS | Windows / Linux |
|---|---|---|
| Kirim request | `⌘ Enter` | `Ctrl Enter` |
| Simpan request | `⌘ S` | `Ctrl S` |
| Request baru / tutup tab | `⌘ T` / `⌘ W` | `Ctrl T` / `Ctrl W` |
| Command palette | `⌘ K` | `Ctrl K` |
| Pilih environment | `⌘ E` | `Ctrl E` |
| Cari di sidebar | `⌘ ⇧ F` | `Ctrl Shift F` |
| Fokus ke URL | `⌘ L` | `Ctrl L` |
| Tampilkan atau sembunyikan sidebar | `⌘ B` | `Ctrl B` |

Di editor GraphQL, tekan `Ctrl Space` untuk saran field setelah skema dimuat dengan tombol **Ambil skema**.

## Pengembangan

Kebutuhan:

- Node.js 22 dan pnpm
- Rust stable
- Dependensi sistem Tauri 2 sesuai [panduan resmi](https://tauri.app/start/prerequisites/). Di Linux: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, dan `patchelf`.

```bash
pnpm install
pnpm tauri dev
```

UI juga bisa dibuka di browser biasa dengan data contoh, tanpa backend, melalui `http://localhost:1420/?mock` saat `pnpm dev` berjalan. Mode ini hanya tersedia saat pengembangan.

Pemeriksaan yang juga dijalankan oleh CI:

```bash
pnpm lint && pnpm format:check && pnpm typecheck && pnpm test
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Tipe TypeScript di `src/bindings` dibuat otomatis dari struct Rust saat `cargo test` dijalankan. Jangan diedit manual.

### Struktur proyek

```
src/                 Frontend Vue 3 + TypeScript
  api/               Wrapper perintah Tauri
  components/        Komponen UI per area (request, response, sidebar, runner, settings)
  stores/            State Pinia
  utils/             Parser cURL, generator snippet, util URL
src-tauri/           Backend Rust
  src/commands/      Perintah IPC
  src/pipeline/      Alur kirim: pre-request script, kirim, test script
  src/services/      Mesin HTTP, resolver variabel, OAuth, script, cookie, jaringan
  src/db/            Penyimpanan SQLite dan migrasi
```

## Keamanan

- Semua request jaringan dijalankan oleh backend Rust, bukan oleh webview.
- Nilai rahasia (variabel secret, token OAuth, password proxy) disimpan di keychain sistem dan tidak pernah ditulis ke database.
- Script berjalan di runtime QuickJS yang terisolasi, tanpa akses file, jaringan, atau sistem, dengan batas waktu 5 detik dan memori 64 MB.
- Webview memakai Content Security Policy yang ketat dan permission Tauri yang minimal.

## Rilis

### Persiapan satu kali

Pembaruan otomatis membutuhkan kunci penanda tangan updater. Public key-nya sudah tercantum di `src-tauri/tauri.conf.json`, sedangkan private key dan password-nya disimpan sebagai secret repository di GitHub (**Settings > Secrets and variables > Actions > New repository secret**):

| Nama secret | Isi |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Isi file private key, misalnya hasil `pbcopy < ~/.tauri/zorp-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password private key, misalnya hasil `pbcopy < ~/.tauri/zorp-updater.password` |

Simpan cadangan kedua file tersebut di tempat aman. Jika kunci hilang, aplikasi yang sudah terpasang tidak dapat menerima pembaruan otomatis dan perlu dipasang ulang secara manual. Kunci yang sama dibutuhkan untuk `pnpm tauri build` di komputer lokal, melalui environment variable dengan nama yang sama.

### Membuat rilis

1. Samakan versi di `package.json`, `src-tauri/Cargo.toml`, dan `src-tauri/tauri.conf.json`.
2. Buat tag dengan format `v<versi>`, lalu push:
   ```bash
   git tag -a v0.2.0 -m "Zorp v0.2.0" && git push origin v0.2.0
   ```
3. Workflow **Release** lebih dulu menjalankan pemeriksaan singkat (job `preflight`): tag harus sama dengan versi aplikasi, dan kedua secret harus bisa dipakai untuk menandatangani. Jika lolos, workflow membuat installer untuk macOS, Windows, dan Linux, beserta `latest.json` untuk pembaruan otomatis, sebagai draft release.
4. Periksa draft di halaman Releases, lalu klik **Publish release**. Aplikasi yang sudah terpasang baru melihat versi baru setelah release diterbitkan.

### Jika rilis gagal

| Pesan di job `preflight` | Penyebab dan solusi |
|---|---|
| `Tag vX tidak sama dengan versi aplikasi vY` | Versi di file konfigurasi belum dinaikkan. Samakan versinya, lalu buat ulang tag. |
| `Secret ... kosong atau belum dibuat` | Secret belum ditambahkan atau namanya salah ketik. |
| `Kunci updater tidak bisa dipakai: ... Wrong password for that key` | Password tidak cocok dengan private key, atau salah satu secret terisi teks yang keliru. Buat ulang kedua secret dari file kunci yang sama. |

Untuk mengulang rilis dengan versi yang sama setelah memperbaiki secret, buka run yang gagal di tab **Actions**, lalu klik **Re-run all jobs**. Jika perbaikannya membutuhkan perubahan kode, hapus draft release dan tag lamanya, lalu buat tag di commit terbaru:

```bash
git push origin :refs/tags/v0.2.0 && git tag -fa v0.2.0 -m "Zorp v0.2.0" && git push origin v0.2.0
```

## Lisensi

Zorp dirilis di bawah [lisensi MIT](LICENSE).
