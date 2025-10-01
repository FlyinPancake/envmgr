# Security Policy

Thanks for helping keep envmgr secure. Because this project is still experimental software, we rely on the community to report potential issues responsibly.

## Supported Versions

envmgr has not reached a stable v1 release yet. Security fixes are applied to the active development branch on a best-effort basis.

| Version / Channel | Support Status |
| --- | --- |
| `main` branch | Best-effort support; fixes land here first |
| Pre-release tags (`0.x` or `v0.x`) | Best-effort support; may lag behind `main` |
| `1.0.0` and later | Not yet released |

Until we ship a v1 release, we do not guarantee backports or timelines for fixes on older tags. Expect rapid iteration and breaking changes as we stabilize the project.

## Reporting a Vulnerability

Please report vulnerabilities privately so we have a chance to investigate before details become public.

1. Use GitHub Security Advisories: <https://github.com/FlyinPancake/envmgr/security/advisories/new>.
2. Include as much detail as possible—steps to reproduce, affected versions/commits, potential impact, and any suggested mitigations.
3. If you cannot access the advisory form, contact the maintainer via their GitHub profile (https://github.com/FlyinPancake) to arrange an alternative private channel.

## Response Expectations

- **Acknowledgement:** We aim to confirm receipt within 7 calendar days.
- **Assessment:** We'll evaluate severity and feasibility as quickly as we can. Because envmgr is pre-v1, timelines for fixes are best-effort and may vary.
- **Disclosure:** We'll coordinate with you on timing for disclosure once a fix or mitigation is available, or if we determine the issue is out of scope.

If we cannot address an issue promptly, we'll keep you updated on its status. No guarantees are implied until a stable v1 release.

## Preferred Disclosure Process

- Please allow us time to review and remediate before sharing details publicly.
- After a fix is ready, we'll publish release notes and credit you (unless you prefer otherwise).
- If you believe the vulnerability is being actively exploited, let us know—urgent issues will take priority.

## Out of Scope

Reports are very welcome, but a few things are out of scope for security bounty or triage purposes:

- Vulnerabilities in third-party dependencies not controlled by this project.
- Issues in downstream projects or configurations unrelated to envmgr.
- Attacks requiring physical access or privileged shell access to a system already running envmgr.

We appreciate your help in keeping envmgr secure while it evolves toward a stable v1 release.