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
- **WebSocket dan SSE**: pilih **WS** atau **SSE** di pilihan method untuk membuka koneksi real-time, mengirim pesan WebSocket, dan melihat log pesan masuk dan keluar beserta waktunya. Keduanya mengikuti pengaturan proxy, sertifikat CA, dan sertifikat klien (mTLS). Untuk WebSocket, proxy yang didukung adalah `http://` dan `socks5://`/`socks5h://`.
- **Koleksi**: folder bertingkat, drag-and-drop, multi-tab yang tetap terbuka setelah aplikasi ditutup, dan riwayat request.
- **Contoh respons dan dokumentasi**: respons bisa disimpan sebagai contoh di bawah request, dan setiap request punya tab Docs berbasis Markdown. Keduanya ikut tersimpan di folder workspace.
- **Workspace**: beberapa workspace terpisah, masing-masing dengan koleksi, environment, riwayat, cookie, dan pengaturan jaringan sendiri. Workspace bisa disimpan sebagai folder file YAML agar dapat dibagikan lewat Git.
- **Environment**: variabel `{{nama}}` dengan penanda variabel yang tidak ditemukan. Nilai rahasia disimpan di keychain sistem.
- **Auth**: Bearer, Basic, API key, Digest, JWT Bearer (HS/RS/ES), AWS Signature v4, OAuth 1.0, dan OAuth 2.0 (Authorization Code dengan PKCE, Client Credentials, refresh otomatis). Auth bisa diwariskan dari folder atau koleksi. Kolom kredensial disembunyikan secara default dan ditandai jika nilainya ditulis langsung.
- **Script**: pre-request dan test script dengan API mirip Postman (`pm.environment`, `pm.test`, `pm.expect`, dan lainnya), di level request, folder, dan koleksi.
- **Collection runner**: menjalankan seluruh koleksi atau folder secara berurutan beserta ringkasan tes, sekali atau berulang dengan data dari file CSV atau JSON, lalu mengekspor laporannya ke HTML atau JSON.
- **Peringatan kredensial**: request yang berisi token, password, atau API key yang ditulis langsung ditandai di sidebar dan status bar, agar dipindahkan ke variabel rahasia sebelum dibagikan.
- **Impor dan ekspor**: koleksi dan environment Postman v2.1 (dua arah), spesifikasi OpenAPI 3.x dan Swagger 2.0 (file atau URL, JSON atau YAML), perintah cURL, serta code snippet untuk cURL, fetch, axios, Python, dan Go. Nilai variabel rahasia tidak pernah ikut diekspor.
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

Di macOS, setelah pembaruan, sistem bisa kembali meminta izin Keychain saat Zorp membaca variabel rahasia. Klik **Always Allow** dan masukkan password login Mac. Ini terjadi karena aplikasi belum ditandatangani dengan Apple Developer ID.

## Panduan singkat

### Impor OpenAPI / Swagger

Klik ikon unduh di header **Koleksi**, lalu pilih **OpenAPI / Swagger**. Pilih file JSON atau YAML, atau tempel URL spesifikasi, lalu periksa ringkasannya sebelum mengimpor.

- Request dikelompokkan ke folder berdasarkan tag atau path.
- Base URL disimpan sebagai variabel koleksi `{{baseUrl}}`, dan parameter path seperti `{id}` menjadi `{{id}}`.
- Body diisi dari contoh atau skema. Auth koleksi dibuat dari `securitySchemes` dan selalu memakai variabel, misalnya `{{token}}`.
- URL diunduh oleh backend dengan pengaturan proxy dan sertifikat yang berlaku. Ukuran dokumen maksimal 20 MB.

### Contoh respons dan dokumentasi

- Setelah request yang tersimpan dikirim, klik **Simpan sebagai contoh** di header Response. Contoh muncul di bawah request di sidebar dan dibuka sebagai tab hanya-baca. Hanya respons teks hingga 1 MB yang bisa disimpan, dan header `Set-Cookie`, `Cookie`, serta `Authorization` dibuang.
- Tab **Docs** pada request menerima Markdown, lengkap dengan pratinjau. HTML mentah tidak dirender, dan link dibuka di browser sistem.

### Pengaturan koleksi dan folder

Klik kanan koleksi atau folder, lalu pilih **Pengaturan**. Dialog ini berisi tab Auth, Script, Variabel (khusus koleksi), dan Docs.

- Script bisa mengirim request sendiri dengan `pm.sendRequest(request, callback)`, misalnya untuk login otomatis di script koleksi lalu menyimpan token ke variabel koleksi. Respons juga dikembalikan langsung, dan variabel `{{nama}}` di URL, header, dan body diisi otomatis.
- Script koleksi dan folder berjalan untuk setiap request di dalamnya. Urutannya: koleksi, lalu folder dari luar ke dalam, lalu request. Test script mengikuti urutan yang sama. Tab **Script** di request menampilkan urutan ini.
- Koleksi atau folder yang punya script ditandai **JS** di sidebar.
- Variabel koleksi ikut tersimpan di folder workspace, jadi jangan isi dengan nilai rahasia.

### Runner dengan data

Pada dialog **Jalankan**, pilih **File CSV / JSON** di bagian Data iterasi. File CSV memakai baris pertama sebagai nama kolom; file JSON berupa array objek. Setiap baris menjadi satu iterasi, dan setiap kolom menjadi variabel yang menimpa environment selama iterasi itu, misalnya `{{email}}`. Hasil dikelompokkan per iterasi dan bisa diekspor lewat **Ekspor laporan**. File data hanya dibaca saat runner berjalan dan tidak disimpan ke workspace.

### Jenis auth

| Jenis | Catatan |
|---|---|
| Bearer, Basic, API key | Dikirim lewat header, atau query untuk API key. |
| Digest | Request dikirim sekali tanpa kredensial; jika server membalas tantangan Digest, Zorp menjawabnya sekali (MD5 atau SHA-256). Belum mendukung body multipart atau file. |
| JWT Bearer | Token ditandatangani ulang setiap pengiriman (HS256 sampai ES384). Klaim `iat` ditambahkan jika belum ada. |
| AWS Signature v4 | Mengisi access key, secret key, session token (opsional), region, dan service. |
| OAuth 1.0 | HMAC-SHA1, HMAC-SHA256, atau PLAINTEXT. |
| OAuth 2.0 | Authorization Code dengan PKCE dan Client Credentials; token disimpan di keychain dan diperbarui otomatis. |

Untuk WebSocket, auth yang diterapkan adalah Bearer, Basic, API key, JWT, dan OAuth 2.0.

## Workspace di folder (Git)

Pilih nama workspace di title bar, lalu:

- **Simpan ke folder** menyimpan workspace aktif ke folder kosong. Sejak itu setiap perubahan di aplikasi langsung ditulis ke folder.
- **Buka folder** membuka folder workspace yang sudah ada, misalnya hasil `git clone`. Perubahan file dari luar aplikasi (misalnya setelah `git pull`) dibaca otomatis. Folder milik workspace yang sinkronnya pernah dihentikan akan disambungkan kembali ke workspace tersebut, dengan isi folder sebagai acuan.

Struktur folder:

```text
zorp.yaml
collections/<koleksi>/collection.yaml
collections/<koleksi>/<folder>/folder.yaml
collections/<koleksi>/<nama-request>.request.yaml
environments/<nama>.yaml
.gitignore
```

Dokumentasi dan contoh respons disimpan di dalam file `.request.yaml` milik request tersebut, sehingga ikut di-review bersama perubahan request.

Yang ikut tersimpan di folder: koleksi, folder, request, script, dokumentasi, contoh respons (tanpa header Set-Cookie dan Authorization), variabel koleksi, dan environment. Yang tetap lokal: riwayat, cookie, token OAuth, pengaturan proxy dan sertifikat, environment aktif, serta nilai variabel rahasia (tetap di keychain; file hanya mencatat nama variabelnya). Nilai auth yang ditulis langsung di request ikut tersimpan, jadi gunakan variabel rahasia seperti `{{token}}` untuk kredensial.

Folder baru otomatis diberi `.gitignore` untuk file sistem seperti `.DS_Store`. Request yang berisi kredensial tertulis langsung (token, password, API key) ditandai di sidebar dan status bar agar dipindahkan ke variabel rahasia sebelum masuk Git.

Jika ada file yang tidak bisa dibaca, misalnya karena konflik merge yang belum diselesaikan (status bar menyebut file mana), sinkron dijeda dan status bar menampilkan **Sinkron dijeda**. Selama dijeda, file di folder tidak diubah. Setelah file diperbaiki, sinkron berlanjut otomatis.

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

Build pengembangan memakai database terpisah (`zorp-dev.db`) di folder data aplikasi, sehingga tidak mengubah database milik Zorp yang terpasang.

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
src/                    Frontend Vue 3 + TypeScript
  api/                  Wrapper perintah Tauri
  components/           Komponen UI per area (layout, request, response, sidebar, runner, settings)
  composables/          Logika UI bersama: perintah, impor, workspace, contoh respons
  stores/               State Pinia, termasuk sesi tab per workspace
  utils/                Parser cURL, generator snippet, Markdown, deteksi kredensial
  dev/                  Backend tiruan untuk mode ?mock (hanya saat pengembangan)
src-tauri/              Backend Rust
  migrations/           Migrasi skema SQLite
  src/commands/         Perintah IPC
  src/pipeline/         Alur kirim: pre-request script, auth, kirim, test script
  src/services/         Mesin HTTP, resolver variabel, OAuth, signing (JWT, AWS, OAuth 1.0, Digest),
                        script, cookie, jaringan, WebSocket dan SSE, impor dan ekspor
  src/services/workspace_sync/  Format YAML folder workspace: tulis, baca, validasi
  src/sync/             Sinkron otomatis antara aplikasi dan folder workspace
  src/db/               Penyimpanan SQLite
```

## Keamanan

- Semua request jaringan dijalankan oleh backend Rust, bukan oleh webview.
- Nilai rahasia (variabel secret, token OAuth, password proxy) disimpan di keychain sistem dan tidak pernah ditulis ke database.
- Script berjalan di runtime QuickJS yang terisolasi, tanpa akses file atau sistem, dengan batas waktu 5 detik dan memori 64 MB. Satu-satunya akses jaringan adalah `pm.sendRequest`, yang dikirim oleh backend Rust (maksimal 20 request per script, 30 detik per request, respons hingga 5 MB) dan selalu tercatat di Konsol. Periksa script koleksi dari sumber yang tidak Anda kenal sebelum menjalankannya, karena script dapat mengirim nilai variabel ke server lain.
- Webview memakai Content Security Policy yang ketat dan permission Tauri yang minimal.
- Folder workspace tidak pernah berisi nilai variabel rahasia. File yang dibaca dari folder dibatasi ukurannya, symbolic link dilewati, dan file yang tidak valid menjeda sinkron tanpa menimpa apa pun.
- Dokumentasi Markdown dirender tanpa HTML mentah, dan hanya link `http`/`https` yang bisa dibuka.
- Penandatanganan JWT, AWS Signature v4, OAuth 1.0, dan Digest dilakukan di backend Rust.

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
   git tag -a v0.6.0 -m "Zorp v0.6.0" && git push origin v0.6.0
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
git push origin :refs/tags/v0.6.0 && git tag -fa v0.6.0 -m "Zorp v0.6.0" && git push origin v0.6.0
```

## Lisensi

Zorp dirilis di bawah [lisensi MIT](LICENSE).
