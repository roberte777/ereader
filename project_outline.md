# Custom E-Reader Ecosystem — High-Level Project Scope

Below is a deliberately **high-level, decomposed outline** you can take and “fill in the blanks” later. It assumes: **everything you write is Rust**, you want a **self-owned ebook library + sync**, and a **custom e-reader device + custom OS (SteamOS-ish: appliance-like, immutable-ish, updateable, controllable)**.

---

## 0) North Star and constraints (write these down first)

**Goals**
- You own your files, metadata, and sync.
- One “library” UX across devices.
- Simple ingest (drop in a file, it shows up nicely).
- Fast library browsing (even with thousands of books).
- Reliable offline reading + progress sync.
- The device feels like an appliance: boots fast, stable, updateable, hard to brick.

**Non-goals (for v1)**
- Any Kindle/Amazon DRM interoperability.
- Perfect typography for every format.
- A full app ecosystem.

**Assumptions**
- Books are *your* files (EPUB/PDF/CBZ/etc.) and/or DRM-free.
- You’ll run your backend on something you control (home server, VPS, NAS).

---

## 1) System architecture at a glance

You’ll likely end up with **three major software surfaces**:

1) **Library Server (backend)**
- Stores files + metadata + covers + reading states
- Exposes API for clients
- Handles ingest and indexing

2) **Clients**
- The e-reader device (your main target)
- Optional: desktop/web/mobile clients later (for upload, management, reading)

3) **Device OS + UI stack**
- OS image, update mechanism, device services, rendering pipeline, power mgmt
- Reader app + library app + settings

Keep it modular so the device can be “just another client.”

---

## 2) Backend: ebook storage + API (server)

### 2.1 Core capabilities
**Ingest**
- Add book by uploading file
- Add book by pointing at a directory / watched folder
- Add book by URL fetch (optional later)
- Generate cover thumbnails + basic metadata extraction
- Deduplicate (hashing), versioning, and re-ingest rules

**Library queries**
- List books with pagination/filter/sort
- Search (title/author/series/tags/full-text optional later)
- Provide “collections” (tags, series, shelves)
- Provide “recently added,” “recently opened,” “in progress”

**Download/stream**
- Download original file
- Optionally deliver a “prepared” version for the device (preprocessed, cached)
- Partial download support (resume), content hashing, integrity checks

**Sync**
- Reading position, annotations, bookmarks
- “Last opened” and per-device state
- Conflict handling (two devices read offline)

**Admin**
- Users (even if only you), auth tokens
- Backup/export/import
- Audit/logging

---

### 2.2 Data model (rough entities)
- **Book**: id, title, authors, language, identifiers, description, tags
- **FileAsset**: format, size, hash, storage path, encryption flag, original filename
- **Series**: name, index, relationship to Book
- **Cover**: image variants (small/medium/large), generation timestamp
- **ReadingState**: user_id, book_id, device_id, location, timestamp, progress percent
- **Annotation**: book_id, location range, note, highlight style, created_at
- **Collection/Shelf**: named groups and membership

Keep the model flexible: you’ll want to revise metadata extraction later.

---

### 2.3 API surface (just categories)
**Auth**
- login/token issuance
- device registration
- token refresh/revoke

**Library**
- list/search/filter
- get details
- update metadata (manual edits)
- covers endpoints

**Assets**
- upload
- download
- verify/integrity endpoint

**Sync**
- get/set reading state
- get/set annotations
- sync batch endpoint for offline

**Admin**
- backup/export jobs
- reindex/rescan
- health/status metrics

---

### 2.4 Backend components (Rust crates/binaries you’ll write)
- **api_server** (HTTP API, auth, request validation, pagination)
- **indexer** (metadata extraction, cover generation, optional OCR/full-text pipeline later)
- **storage_layer** (filesystem abstraction, object-store abstraction optional)
- **db_layer** (schema migrations, query API)
- **sync_engine** (merge/conflict rules)
- **worker_daemon** (background tasks: thumbnails, hashing, reindex)
- **cli_tool** (admin commands: import/export, rescan, user mgmt)

---

### 2.5 Storage approach choices (keep undecided for now)
- **Metadata DB**: relational vs embedded vs document
- **File storage**: flat filesystem vs content-addressed store vs object store
- **Search**: DB FTS vs separate search service vs embedded index
- **Async tasks**: in-process queue vs external queue vs cron-like worker

You can postpone decisions by designing interfaces first.

---

## 3) Device hardware: “parts you’ll need” (no specific models)

Think in **subsystems**:

### 3.1 Display + touch
- **E-ink display module**
- **Display controller / interface** (often separate from the panel)
- **Touch layer** (optional: capacitive or other) + touch controller
- **Frontlight system** (optional): LEDs + diffuser + driver

### 3.2 Compute
- **Main SoC / CPU module**
- **RAM**
- **Non-volatile storage** (for OS + cache + downloads)
- **Optional secure element / TPM-like chip** (device keys)

### 3.3 Power
- **Battery**
- **Battery management / charging IC**
- **Fuel gauge / battery monitoring**
- **Power rails / regulators**
- **Power button + wake circuitry**

### 3.4 Connectivity
- **Wi-Fi module**
- Optional: **Bluetooth**
- Optional: **USB data interface** (for sideloading + debugging)

### 3.5 Input/UX
- **Physical buttons** (page turn, home, power) (optional)
- **Haptics** (optional)
- **Status LED** (optional)

### 3.6 Audio (optional)
- Speaker / buzzer
- Audio codec / amp
- Mic (if you ever want dictation)

### 3.7 Sensors (optional)
- Ambient light sensor (for frontlight auto)
- Accelerometer (orientation)

### 3.8 PCB + mechanical
- Main PCB + connectors (display FPC, battery, USB, buttons)
- Enclosure, mounting, thermal considerations (usually minimal)
- ESD protection, shielding, antenna placement

### 3.9 Debug/bring-up
- Debug header (UART/JTAG/SWD depending on platform)
- Recovery mode mechanism (boot selector, button combo, etc.)

---

## 4) Device OS: “SteamOS-ish” e-reader appliance

A practical way to interpret “SteamOS-ish” here:
- **Read-only or A/B root filesystem**
- **Atomic updates + rollback**
- **Minimal background services**
- **A single primary UI shell**
- **Strong separation: system vs apps vs content**

### 4.1 OS building blocks
- **Boot chain**
  - bootloader configuration
  - verified boot (optional)
  - recovery partition

- **Kernel + drivers**
  - e-ink display driver integration
  - touch input driver
  - power management (suspend/resume)
  - Wi-Fi stack
  - USB gadget modes (optional)

- **Root filesystem strategy**
  - immutable base image
  - writable data partition for books/cache/settings
  - A/B update slots or snapshot-based updates

- **Init + service manager**
  - networking
  - time sync
  - your UI shell service
  - update service
  - logging/telemetry service (local-only initially)

### 4.2 What you’ll implement in Rust on-device
- **device_shell**: the main UI process (home/library/reader/settings)
- **renderer**: eink-friendly rendering abstraction (partial refresh, dithering, ghosting mgmt)
- **inputd**: input aggregation (buttons + touch gestures)
- **netd**: connectivity control, captive portal handling (optional), retry logic
- **syncd**: library sync, downloads, upload annotations, background scheduling
- **updated**: update fetch/apply/rollback, signature verification
- **storaged**: local content store, cache, integrity checks
- **powerd**: suspend/resume orchestration, battery monitoring, frontlight policy
- **crashd/logd**: crash capture + log rotation (even if local)

Keep these as separate daemons *or* one process with modules—either way, define clear boundaries.

---

## 5) UI/UX: what screens and behaviors exist (v1)

### 5.1 Library experience
- Home:
  - “Continue reading”
  - “Recently added”
  - “Downloaded”
- Library list/grid:
  - sort/filter/search
  - tap opens details
- Book details:
  - metadata + cover
  - download/remove local
  - open/read
- Collections:
  - tags/series/shelves

### 5.2 Reader experience
- Open book fast (from local cache if possible)
- Page turn gestures + buttons
- Font/spacing/margins/themes (minimal set)
- Progress scrubber
- Bookmarks + simple highlight/note (optional for v1)
- Robust resume at last position

### 5.3 Settings
- Wi-Fi + device pairing
- Sync on/off
- Storage usage
- Update channel + apply update
- Display/frontlight controls (if present)

E-ink note: design with **low animation**, **clear focus states**, **predictable refreshes**.

---

## 6) E-book format pipeline (keep flexible)

You’ll want a “format pipeline” that can evolve:

### 6.1 Supported formats (phased)
- v1: one or two primary formats (e.g., EPUB + PDF)
- later: comics/CBZ, MOBI (non-DRM), etc.

### 6.2 Processing stages
- Parse container
- Extract metadata (title/author/series if present)
- Extract cover
- Compute reading locations (EPUB CFI-like / page map / logical offsets)
- Optional: pre-render/cache pages for speed (depends on hardware)
- Optional: full-text index for search

Define a trait-like interface: `FormatHandler` with methods like `extract_metadata`, `open_reader`, `locate`, `render_page`.

---

## 7) Pairing, auth, and trust model

Even if you’re the only user, you’ll want a clean model:

- **Device provisioning**
  - device generates keypair on first boot
  - “pairing code” shown on device, entered on admin client
  - server issues device token scoped to that device

- **Transport security**
  - TLS everywhere
  - pin server cert optional (nice for home setups)

- **At-rest security**
  - optional encryption for stored ebooks (server-side or device-side)
  - device local storage encryption optional (depends on hardware)

---

## 8) Build, deploy, and update workflows

### 8.1 Server deployment
- Single binary + config file
- Migrations on startup
- Backups: scheduled export of DB + content hashes + manifests
- Observability: health endpoint + minimal metrics

### 8.2 Device image build
- Reproducible image build pipeline
- Emulator/sim target for UI (even rough) to speed dev
- Hardware bring-up profile vs production profile

### 8.3 OTA updates (SteamOS-ish vibe)
- Signed update artifacts
- Download to inactive slot
- Flip active slot on reboot
- Rollback if boot fails / watchdog triggers

---

## 9) Milestones (a sane sequencing)

### Milestone A — “End-to-end happy path”
- Server can ingest an EPUB/PDF
- Library endpoint lists books with covers
- Device can pair, show library list, download, open, and resume

### Milestone B — “Feels like a product”
- Search + filters
- Reliable suspend/resume
- Background sync + conflict rules
- Basic settings + networking robustness

### Milestone C — “Ownership & durability”
- Backup/export/import
- OTA update + rollback
- Local cache management
- Annotation support + sync

### Milestone D — “Nice-to-haves”
- Full-text search
- Collections/shelves UX polish
- Multi-user
- Additional formats

---

## 10) “Guidelines” to keep you from painting yourself into a corner

- **Define interfaces before implementations** (storage, format handlers, rendering, sync).
- **Make everything content-addressable** *somewhere* (hashes solve many problems).
- **Treat the device as unreliable/offline-first**.
- **Keep server stateless where possible** (state in DB/storage).
- **E-ink constraints drive UI**: minimize redraws, design for latency.
- **Plan for migration**: metadata schema will change.
- **Don’t fight DRM**: focus on your owned files ecosystem.
