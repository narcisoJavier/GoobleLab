# Security reports

This repository is a preview. There is no production security certification, service-level agreement or guaranteed response time.

Report vulnerabilities through [GitHub's private vulnerability reporting form](https://github.com/narcisoJavier/GoobleLab/security/advisories/new). Include the affected commit or release, a minimal reproduction and expected impact. Use synthetic data; never send real access tokens, credentials or private media. Do not publish exploit details in an ordinary issue while a report is being assessed.

Browser Google tokens stay in page memory. The builder requests `drive.file`, while mounting Drive inside Colab is a separate authorization with broader notebook access. Downloaded notebooks, saved project records, generated workflow JSON and outputs can contain private content; removing a browser project or disconnecting Google does not delete files in Drive. See [the data lifecycle](docs/WEB.md#where-data-lives-and-how-to-remove-it).

The browser preview and `v0.2.0` notebook are the current maintenance targets. A runtime fix on `main` is not delivered to notebooks pinned to an older release until the notebook bootstrap is updated in a new release.
