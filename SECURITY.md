# Security policy

Luca UI is proprietary software. This policy explains how to report a
security issue and what to expect.

## Supported versions

| Version | Supported |
|---|---|
| 1.0.2 Beta (current working tree) | Yes |
| 1.0 Beta (initial publication) | Yes — critical fixes only |
| 0.9 and earlier alphas | No — upgrade to a 1.0 Beta release |

Luca UI depends on [Luca Code](https://github.com/LucaEcosystem/luca-code)
by relative path. Vulnerabilities in Luca Code itself — its lexer, parser,
type system, interpreter, or module system — must be reported against the
Luca Code repository, not here.

## Reporting a vulnerability

**Do not open a public issue or discussion for a security problem.** Public
reports can expose users before a fix exists.

Report privately through one of these channels:

1. **GitHub Private Vulnerability Reporting** on this repository (enabled on
   the *Security* tab). This is the preferred channel: it keeps the report
   confidential and lets maintainers coordinate a fix and a disclosure
   timeline.
2. If private reporting is unavailable, contact the maintainers directly
   through the repository's private channels (for example, the contact links
   under *Settings → Security → Private vulnerability reporting*).

Include as much of the following as you can:

- The affected version (`luca-ui --version`) and platform.
- A description of the issue and its impact (for example: arbitrary code
  execution, path traversal, denial of service, information disclosure).
- The smallest `.lucu`/`.lucc` input or command that demonstrates it.
- Whether the issue depends on paired `.lucc` logic or on user-supplied
  input.
- Any suggested mitigation.

Do not include proprietary application code, secrets, or private data in the
report; a minimal synthetic reproducer is enough.

## What to expect

- **Acknowledgement** within a few business days.
- A private investigation and, when the report is valid, a fix in the next
  appropriate release.
- Coordinated disclosure: the issue stays private until a fix is available.
  We will credit reporters in the release notes when they ask to be named.

## Scope notes

- Luca UI is a declarative description language and toolchain. It executes
  `.lucc` logic through Luca Code's public interpreter; vulnerabilities in
  that execution path belong to Luca Code.
- The SVG backend renders a **static** snapshot and performs no interaction,
  network, or filesystem access. It is not a browser and is not a remote-code
  surface.
- Luca UI does not phone home, collect telemetry, or contact external
  services.

## Security-relevant behavior to know about

- **Errors are source-located and frozen.** Structural failures surface as
  `Error at line:column: message`; render warnings for the visual assumptions
  are non-fatal and never suppress a render.
- **Unpaired logic is not executed.** `luca-ui check` verifies handler
  declarations only; it never runs handlers.
- **`.lucc` logic is general-purpose code.** Anyone who can supply a `.lucc`
  file can run arbitrary Luca Code logic, subject to Luca Code's own sandbox
  and permissions. Treat `.lucc` files as executable code, not data.
