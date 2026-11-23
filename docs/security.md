# Security model

Who this is for: a reviewer or integrator who needs to know what this crate protects and what it does not.

## Trust boundaries

| Boundary | What crosses it | Who is responsible |
| --- | --- | --- |
| Application to model | Strings and numbers supplied by the caller | The application owns validation, authentication and data lifetime. |
| Model to drawing | `ViewModel` fields, widget inputs | The crate escapes again here because public fields can be built directly. |
| Drawing to terminal | A ratatui buffer | The terminal emulator is trusted to render that buffer. |
| Terminal guard to terminal | Raw mode and alternate screen transitions | The crate restores them on drop and on panic. |

There is no network boundary, no filesystem boundary and no user identity in this crate.

## What it protects

- Untrusted text: `printable` replaces control characters, bidi overrides, zero-width marks, variation selectors, tag characters and other invisible format characters with `?`. Escaping is idempotent.
- Public structs: constructors escape input, and drawing code escapes again, so a directly built `ViewModel`, `Column`, `Gauge`, `Series`, `ToggleChip` or panel title cannot carry a control sequence onto the screen.
- Hostile numbers: fractions clamp to 0..1, counts clamp to displayable ranges, chart points drop non-finite and negative values, and table selection and scroll offsets clamp on every render.
- Hostile layouts: zero-sized, tiny, narrow and wide areas draw without panicking. Tests cover resize and wide Unicode.
- Terminal state: `TerminalGuard` restores raw mode and the alternate screen on drop, and a panic hook restores them before the panic message is printed.
- Supply chain: unsafe code is forbidden, dependencies are locked, workflows pin actions by SHA, and CI runs audit, CodeQL and dependency review.

## What it does not protect

- No authentication, authorisation, session handling or user identity.
- No cryptography, secret storage or redaction of caller data. The privacy mask is a deterministic screenshot aid, not a security control.
- No network client, no file access and no sandbox. The crate cannot validate paths or URLs.
- No terminal emulator hardening. Escape output is safe, but a malicious emulator is outside the model.
- No guarantee that data is meaningful. Clamping prevents panics, not misleading values.

Fail-open and fail-closed, in plain words: text handling is fail-closed, because anything that looks like a control or invisible character becomes `?`. Layout is fail-safe, because hostile numbers clamp and unusable areas draw nothing. There is no access-control decision, so nothing in this crate can fail open to grant access.

## Reporting a vulnerability

Report through the channel in [SECURITY.md](../SECURITY.md). Do not open a public issue for an unfixed vulnerability. Supported versions and response times are listed there.

## Scanning and findings

See [security scanning](security-scanning.md) for what each workflow checks and where its findings appear.
