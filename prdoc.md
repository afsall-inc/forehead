## PRDoc

Forehead uses PRDoc for structured pull request documentation. Each PR should include a `prdoc/pr_<number>.prdoc` file with YAML frontmatter describing the changes.

### Creating a PRDoc

```bash
cp templates/prdoc/.template.prdoc prdoc/pr_<pr-number>.prdoc
```

Fill in the fields:
- `title`: Short description of the change
- `doc.audience`: Who needs to know about this change (`Framework Dev` or `App Dev`)
- `doc.description`: Detailed description
- `crates`: List of affected crates with semver bump level

### CI Validation

On pull requests, the CI pipeline checks that `prdoc/pr_<PR_NUMBER>.prdoc` exists and is valid. This ensures every PR has proper documentation for release notes.