# Security Policy

## Supported Versions

| Version | Supported |
|:--------|:---------:|
| 0.1.x   | Yes       |

## Reporting Vulnerabilities

**DO NOT open public issues for security vulnerabilities.**

Report vulnerabilities to: **security@agentralabs.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

| Stage | Timeline |
|:------|:---------|
| Acknowledgment | Within 48 hours |
| Initial assessment | Within 5 business days |
| Fix development | Depends on severity |
| Disclosure | Coordinated with reporter |

## Scope

We care about:
- `.acomm` file integrity and data corruption
- Message confidentiality and channel isolation
- MCP server sandbox bypasses
- Communication data leaks between channels or projects
- Memory safety in the binary format parser
- Authentication and authorization bypass in pub/sub
- Message replay or tampering attacks

## Out of Scope

- Denial of service via large message volumes (expected local usage)
- Issues in third-party dependencies (report upstream)
- Social engineering attacks
- Issues requiring physical access to the machine
