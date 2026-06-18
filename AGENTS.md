## Learned User Preferences

## Learned Workspace Facts

- biTurbo is a Tauri macOS app; the installed bundle lives at `/Applications/biTurbo.app`.
- The MCP server binary is bundled inside the app at `/Applications/biTurbo.app/Contents/MacOS/biturbo-mcp` (Tauri `externalBin`).
- Cursor MCP config belongs in `~/.cursor/mcp.json` or project `.cursor/mcp.json`; `command` must be an absolute path to `biturbo-mcp`.
- GUI and MCP share the same data directory: `~/Library/Application Support/com.biturbo.app`.
- Bundle identifier is `com.biturbo.app`.
- Dev MCP binary paths: `src-tauri/target/debug/biturbo-mcp` or `src-tauri/target/release/biturbo-mcp`.
- Local signed build (no notarization): `pnpm tauri:build`.
- Unsigned build: `pnpm tauri build -- --no-sign`.
- Notarized release build: `pnpm tauri:build:notarized` with `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), and `APPLE_TEAM_ID` exported.
