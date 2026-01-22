# Security Policy

## Supported Versions

Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it by:

1. **Do NOT** open a public issue
2. Email the maintainer directly or use GitHub's private vulnerability reporting
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fixes (if any)

We will acknowledge receipt within 48 hours and provide a detailed response within 5 business days.

## Security Measures

This repository follows OSSF (Open Source Security Foundation) best practices:

- Branch protection on main branch with PR reviews required
- Automated dependency vulnerability scanning
- Security advisories enabled
- Admin override allowed for emergency fixes while maintaining security

## Dependencies

We regularly update dependencies to patch security vulnerabilities. Dependencies are managed via:
- Cargo.toml for Rust dependencies
- Automated security alerts via GitHub Dependabot

## OSSF Scorecard

This project aims to maintain a high OSSF Scorecard score by implementing recommended security practices.
