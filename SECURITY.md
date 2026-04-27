# Security Policy

## Supported Versions

Only the latest minor release of Zephyr receives security updates. We strongly recommend keeping your installation up to date, the built-in auto-updater will notify you when a new version is available.

| Version | Supported          |
| ------- | ------------------ |
| 1.2.x   | :white_check_mark: |
| < 1.2   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

If you believe you have found a security vulnerability in Zephyr, please report it through one of the following private channels:

- **GitHub Security Advisories** (preferred): use the [Report a vulnerability](https://github.com/Prismo-Studio/Zephyr/security/advisories/new) button on the Security tab of this repository.
- **Email**: send your report to **contact@prismo-studios.dev** with the subject line `[SECURITY] Zephyr — <short description>`.

When reporting, please include as much of the following information as possible:

- Type of issue (e.g. arbitrary file write, code execution via malicious mod, path traversal, signature bypass, dependency vulnerability, etc.)
- Affected version(s) of Zephyr
- Operating system and architecture (Windows / Linux / macOS, x64 / arm64)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code, if available
- Impact of the issue and how an attacker could exploit it

## Response Timeline

We aim to respond to security reports within the following timelines:

- **Initial acknowledgement**: within 72 hours
- **Triage and severity assessment**: within 7 days
- **Fix and coordinated disclosure**: depending on severity, typically within 30 days for critical issues

You will be kept informed throughout the process. Once a fix is released, we will publish a security advisory crediting the reporter (unless anonymity is requested).

## Scope

The following are considered in scope:

- The Zephyr desktop application
- The official installer and auto-updater mechanism
- Code-signing and update-signing infrastructure
- Official build artifacts published on GitHub Releases

The following are **out of scope**:

- Vulnerabilities in third-party mods installed through Zephyr (please report those to the mod authors)
- Vulnerabilities in upstream dependencies that have not been disclosed publicly (please report those upstream first)
- Issues in the original Gale project that have not been carried over into Zephyr

## Safe Harbor

We support responsible disclosure and will not pursue legal action against researchers who:

- Make a good-faith effort to avoid privacy violations, data destruction, or service disruption
- Report vulnerabilities promptly through the channels listed above
- Do not exploit a vulnerability beyond what is necessary to demonstrate it
- Give us reasonable time to address the issue before public disclosure

Thank you for helping keep Zephyr and its users safe.
