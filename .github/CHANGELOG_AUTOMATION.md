# Changelog Automation

This repository uses [git-cliff](https://git-cliff.org/) for automated changelog generation based on conventional commits.

## How It Works

The changelog is automatically generated from git commit messages that follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Supported Types

- `feat`: New features → **Added** section
- `fix`: Bug fixes → **Fixed** section
- `docs`: Documentation → **Documentation** section
- `perf`: Performance improvements → **Performance** section
- `refactor`: Code refactoring → **Refactored** section
- `style`: Code style changes → **Styling** section
- `test`: Test additions/changes → **Testing** section
- `chore`: Maintenance tasks → **Miscellaneous** section

### Examples

```bash
# Feature addition
git commit -m "feat: add batch subscription creation"

# Bug fix with scope
git commit -m "fix(subscribe): prevent duplicate topic entries"

# Breaking change
git commit -m "feat!: change subscription ID to UUID"

# With issue reference
git commit -m "feat: add transfer functionality (#2)"
```

## Automatic Generation

The changelog is automatically updated in these scenarios:

1. **On Release**: When a new GitHub release is created, the changelog is updated with all changes since the last release.

2. **Manual Trigger**: You can manually generate the changelog via GitHub Actions:
   - Go to Actions → "Generate Changelog" → "Run workflow"
   - Optionally specify a tag, or leave empty for unreleased changes

3. **Pre-release Check**: Run locally before creating a release:
   ```bash
   git cliff --unreleased
   ```

## Local Usage

### Installation

```bash
# Using cargo
cargo install git-cliff

# Using homebrew (macOS)
brew install git-cliff

# Using winget (Windows)
winget install git-cliff
```

### Commands

```bash
# View unreleased changes
git cliff --unreleased

# Generate full changelog
git cliff -o CHANGELOG.md

# Generate changelog for specific version
git cliff --tag v0.2.0 -o CHANGELOG.md

# Preview without writing to file
git cliff --unreleased --strip all

# Generate changelog for a range
git cliff v0.1.0..HEAD
```

## Configuration

The changelog behavior is configured in `cliff.toml`:

- **Header**: Changelog header with project description
- **Body**: Template for version sections
- **Commit Parsers**: Rules for grouping commits by type
- **Filters**: Skip non-conventional commits
- **Tag Pattern**: Match version tags like `v0.1.0`

## Release Workflow

1. **Develop features** using conventional commits:
   ```bash
   git commit -m "feat: add batch subscribe endpoint"
   ```

2. **Preview unreleased changes**:
   ```bash
   git cliff --unreleased
   ```

3. **Create a release**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **GitHub Release**: Create a release on GitHub, and the changelog will be automatically updated.

5. **Review the PR**: The workflow will create a PR with the updated CHANGELOG.md.

## Best Practices

1. **Use conventional commits**: This ensures your changes appear in the changelog.

2. **Write clear descriptions**: The commit message becomes the changelog entry.

3. **Reference issues**: Use `#123` in commit messages to link to issues.

4. **Breaking changes**: Use `!` after the type for breaking changes:
   ```bash
   git commit -m "feat!: change API response format"
   ```

5. **Scopes for clarity**: Add scopes to group related changes:
   ```bash
   git commit -m "feat(admin): add bulk configuration update"
   ```

## Integration with CI

The changelog workflow is defined in `.github/workflows/changelog.yml` and:

- Runs on release creation
- Can be triggered manually
- Commits changes back to the repository
- Creates PRs for review when needed

## Troubleshooting

### Commits not appearing in changelog

- Ensure commits follow conventional commit format
- Check if the commit type is configured in `cliff.toml`
- Verify the tag pattern matches your version tags

### Changelog not updating on release

- Check the workflow run in GitHub Actions
- Ensure the `GITHUB_TOKEN` has write permissions
- Verify git-cliff installation succeeded

### Manual regeneration needed

```bash
# Regenerate entire changelog
git cliff -o CHANGELOG.md

# Commit and push
git add CHANGELOG.md
git commit -m "chore: regenerate changelog"
git push
```

## References

- [git-cliff documentation](https://git-cliff.org/docs/)
- [Conventional Commits specification](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
