# GitHub Actions CI/CD

This project uses GitHub Actions for continuous integration and deployment.

## Workflows

### CI Pipeline (`ci.yml`)

Runs on every push and pull request to `main` branch.

**Jobs:**

1. **Format Check**
   - Verifies code formatting with `cargo fmt`
   - Fails if code is not properly formatted

2. **Clippy Lint**
   - Runs Rust linter with `cargo clippy`
   - Treats all warnings as errors (`-D warnings`)
   - Checks all targets and features

3. **Run Tests**
   - Executes all unit and integration tests
   - Runs with verbose output
   - Tests with all features enabled

4. **Build Release**
   - Builds optimized release binary
   - Uploads binary as artifact (7 day retention)
   - Only runs if all previous jobs pass

5. **Docker Build and Push**
   - Only runs on push to `main` (not on PRs)
   - Builds multi-platform Docker image (amd64, arm64)
   - Pushes to Docker Hub
   - Requires Docker Hub secrets (see setup below)
   - Uses GitHub Actions cache for faster builds

6. **Security Audit**
   - Scans dependencies for known vulnerabilities
   - Uses `cargo-audit`
   - Runs on every push/PR

## Setup Instructions

### Required Secrets

To enable Docker Hub push, add these secrets to your GitHub repository:

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add the following secrets:

| Secret Name | Description | Value |
|-------------|-------------|-------|
| `DOCKER_USERNAME` | Docker Hub username | `felipemeriga1` |
| `DOCKER_PASSWORD` | Docker Hub password or access token | Your Docker Hub password or PAT |

**Recommended:** Use a Docker Hub Personal Access Token (PAT) instead of your password:
1. Go to Docker Hub → Account Settings → Security
2. Click "New Access Token"
3. Name it "GitHub Actions"
4. Copy the token and use it as `DOCKER_PASSWORD`

## Caching Strategy

The CI pipeline uses GitHub Actions cache to speed up builds:

- **Cargo registry cache**: Caches downloaded crates
- **Cargo git cache**: Caches git dependencies
- **Cargo build cache**: Caches compiled dependencies
- **Docker layer cache**: Caches Docker build layers

This significantly reduces build times on subsequent runs.

## Manual Triggers

You can manually trigger workflows from the GitHub Actions tab:

1. Go to **Actions** tab in your repository
2. Select the workflow you want to run
3. Click **Run workflow**
4. Select the branch and click **Run workflow**

## Status Badges

Add this badge to your README.md:

```markdown
![CI](https://github.com/felipemeriga/calendar-scraper/workflows/CI/badge.svg)
```

Or with a specific branch:

```markdown
![CI](https://github.com/felipemeriga/calendar-scraper/workflows/CI/badge.svg?branch=main)
```

## Local Testing

Before pushing, you can run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Clippy (with warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --verbose
cargo test --all-features --verbose

# Build release
cargo build --release

# Security audit
cargo install cargo-audit
cargo audit

# Build Docker image
docker build -t calendar-scraper:test .
```

## Troubleshooting

### Docker Push Fails

**Error**: `denied: requested access to the resource is denied`

**Solutions:**
1. Verify `DOCKER_USERNAME` secret is correct
2. Verify `DOCKER_PASSWORD` secret is correct
3. Make sure the Docker Hub repository exists: `felipemeriga1/calendar-scraper`
4. Try regenerating your Docker Hub access token

### Build Cache Issues

If builds are failing due to cache corruption:

1. Go to **Actions** → **Caches**
2. Delete the problematic cache
3. Re-run the workflow

Or, in your workflow run:
- Click **Re-run jobs**
- Select **Re-run all jobs**

### Clippy Warnings Fail the Build

This is intentional! Fix the warnings:

```bash
# See all warnings
cargo clippy --all-targets --all-features

# Fix automatically when possible
cargo clippy --all-targets --all-features --fix
```

### Tests Fail in CI but Pass Locally

Common causes:
- **Environment variables**: CI doesn't have your `.env` file
- **File paths**: Use relative paths, not absolute
- **Dependencies**: Ensure `Cargo.lock` is committed
- **Platform differences**: Test on Linux if possible

## Best Practices

1. **Always run checks locally before pushing**:
   ```bash
   cargo fmt && cargo clippy -- -D warnings && cargo test
   ```

2. **Keep dependencies updated**:
   ```bash
   cargo update
   cargo test
   git commit -am "chore: update dependencies"
   ```

3. **Review security audit results**: Don't ignore security warnings

4. **Use meaningful commit messages**: Follows conventional commits format

5. **Keep CI fast**: Use caching, run jobs in parallel

## Performance

Typical CI run times:
- Format check: ~30 seconds
- Clippy: ~2-3 minutes (with cache)
- Tests: ~2-3 minutes (with cache)
- Build: ~3-5 minutes (with cache)
- Docker: ~5-10 minutes (with cache)
- Security audit: ~1-2 minutes

**Total**: ~10-15 minutes (first run), ~5-8 minutes (cached)

## Future Improvements

Potential additions:
- [ ] Code coverage reporting (tarpaulin)
- [ ] Benchmark regression testing
- [ ] Deploy to staging environment
- [ ] Automated changelog generation
- [ ] Release automation (on tags)
- [ ] Dependabot integration
- [ ] Performance testing

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
