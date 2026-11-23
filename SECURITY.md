# Security Policy

## Reporting a vulnerability

Report vulnerabilities through [GitHub private vulnerability reporting](https://github.com/ddproxy/fw-tui/security/advisories/new). Do not open a public issue for an unfixed vulnerability.

## Supported versions

Only the latest release receives security fixes. The crate is at preview version 0.1.0 and is not published on crates.io. Upgrade to the newest [release](https://github.com/ddproxy/fw-tui/releases/latest) before reporting.

## Threat model

See [security model](docs/security.md) for trust boundaries, what the crate protects and what it does not. Text handling fails closed and layout fails safe; there is no access-control path in this library.

## Response times

- Acknowledgement: within 3 business days.
- Initial assessment: within 7 business days.
- Fix or mitigation plan for confirmed issues: within 30 days.

## Scanning

See [Security scanning](docs/security-scanning.md) for what each workflow checks and where its findings appear.
