# Issue 2: Bilibili auth — QR login + cookie login

## What to build

Implement Bilibili authentication with two login methods, session persistence, and a Tauri command to verify login status. Depends on `bpi-rs` for Wbi签名 and QR login API interactions.

1. **`bili_auth` module** — Manages `BiliSession` (sessdata, bili_jct, dede_user_id) with `Arc<RwLock<Option<BiliSession>>>` pattern (same as DiscogsKeys). Provides:
   - `qr_login_start()` → returns QR code URL + polling key
   - `qr_login_poll(qrcode_key)` → returns status (pending/scanned/confirmed/expired) + session on success
   - `set_cookie(sessdata)` → direct cookie login
   - `session()` → current session state
   - `logout()` → clear session
   - `is_premium()` → check if account has premium tier

2. **Session persistence** — Persist `BiliSession` in `tauri-plugin-store` under key `"bilibili_session"`. On app startup, hydrate from store into managed state.

3. **Tauri commands** — Register in `lib.rs`:
   - `bili_qr_login_start` → `{ url, qrcode_key }`
   - `bili_qr_login_poll(qrcode_key)` → `{ status, session? }`
   - `bili_cookie_login(sessdata)` → `()`
   - `bili_logout` → `()`
   - `bili_session_status` → `{ logged_in, username?, is_premium? }` (calls Bilibili user info API to verify)

4. **Integration with BiliClient** — `BiliClient` (or `bpi-rs` client) reads session from `BiliAuth` on each API call to attach cookies.

## Acceptance criteria

- [ ] QR login flow works end-to-end: generate QR → user scans → poll → session saved
- [ ] Cookie login works: paste SESSDATA → session saved
- [ ] Session persists across app restarts (loaded from tauri-plugin-store)
- [ ] `bili_session_status` returns correct login state after restart
- [ ] `bili_logout` clears both in-memory and persisted session
- [ ] Auth errors (expired session, invalid cookie) return user-friendly messages
- [ ] `cargo check` passes with all new commands registered

## Blocked by

- Issue 1 (bili_types needed for session/user info types)

## PRD reference

- Module 2: `bili_auth`
- Module 6: `commands/bilibili` (auth commands)
- Authentication Persistence Pattern section
