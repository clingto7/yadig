# Issue 8: Settings UI for Bilibili login

## What to build

Add a Bilibili login section to the settings page, following the existing Discogs API key management pattern.

1. **Bilibili section in settings** — Add a new section below the existing "API Keys" section:
   - **Login status**: Shows "Logged in as {username}" or "Not logged in"
   - **QR login**: Button "Login with Bilibili App" → shows QR code image → polls for scan result → shows success/failure
   - **Cookie login**: Expandable "Advanced: Login with Cookie" → text input for SESSDATA → submit button
   - **Logout**: Button to clear session (only shown when logged in)
   - **Account tier badge**: Shows "Premium" or "Standard" next to username

2. **QR code rendering** — Use a QR code library (e.g., `qrcode` npm package or inline SVG generation) to render the QR code from the URL returned by `bili_qr_login_start`. Poll `bili_qr_login_poll` every 2 seconds until confirmed/expired.

3. **Persistence** — Follow the existing pattern: save session to `tauri-plugin-store` on login, load on app startup via `App.tsx` useEffect, hydrate Rust state via command.

4. **Error states** — Show clear messages for: expired QR code (offer refresh), invalid cookie, network error, session expired.

## Acceptance criteria

- [ ] QR login flow works: scan → confirm → settings shows "Logged in as {username}"
- [x] Cookie login flow works: paste SESSDATA → settings shows logged in
- [x] Login persists across app restart
- [x] Logout clears session and UI updates
- [x] QR code expiration is handled with a "Refresh QR Code" button
- [x] Account tier (premium/standard) is displayed
- [ ] Error messages are user-friendly (not raw API errors)
- [x] `pnpm build` passes

## Notes

- Session persistence is implemented through `src/lib/bili-session-store.ts` and restored in `src/App.tsx` through `bili_restore_session`.
- Settings logout calls `bili_logout`, clears the persisted Bilibili session, and refreshes status.
- Cookie login calls `bili_cookie_login`, saves the returned session to Tauri Store, hides the Cookie form, and refreshes login status.
- QR login expiration now uses `src/lib/bili-login-ui.ts` state mapping and shows an explicit `Refresh QR Code` button.
- Account tier is displayed from `bili_session_status` as Premium or standard max-quality copy.
- Verified with `pnpm build`.

## Blocked by

- Issue 2 (bili_auth backend must exist)

## PRD reference

- User stories #2, #3, #4, #15, #16
- Module 7: Frontend Changes (Settings page section)
