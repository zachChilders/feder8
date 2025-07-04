# GitHub Actions Workflows

This directory contains GitHub Actions workflows for continuous integration (CI) of this Rust project.

## Available Workflows

### 1. `ci.yml` - Comprehensive CI Pipeline

A full-featured CI pipeline that includes:

**Jobs that run in parallel:**
- **Format Check** - Verifies code formatting with `cargo fmt`
- **Lint** - Runs Clippy linter with strict settings
- **Build** - Builds debug and release versions on stable and beta Rust
- **Test** - Runs all tests including integration tests and examples on stable and beta Rust
- **Security Audit** - Checks dependencies for known vulnerabilities
- **Minimal Versions** - Ensures compatibility with minimal dependency versions

**Features:**
- Matrix strategy testing on stable and beta Rust versions
- Dependency caching for faster builds
- Strict linting (treats warnings as errors)
- Security vulnerability scanning
- Final status check that requires all jobs to pass

### 2. `ci-simple.yml` - Streamlined CI Pipeline

A simpler, faster CI pipeline for basic projects:

**Jobs that run in parallel:**
- **Lint & Format** - Combined formatting and linting checks
- **Build** - Builds debug and release versions
- **Test** - Runs all tests including integration tests

**Features:**
- Single Rust version (stable)
- Dependency caching
- Essential checks only
- Faster execution time

## Choosing a Workflow

- **Use `ci.yml`** for production projects that need comprehensive testing and security checks
- **Use `ci-simple.yml`** for smaller projects or faster feedback cycles
- You can rename your preferred workflow to remove the suffix and delete the other

## Triggered Events

Both workflows trigger on:
- Pushes to `main` and `develop` branches
- Pull requests targeting `main` and `develop` branches

## Customization

You can customize the workflows by:
- Modifying the `branches` list in the `on` section
- Adding or removing Rust versions in the matrix strategy
- Adding additional steps like documentation generation or deployment
- Adjusting Clippy rules by adding a `clippy.toml` file to your project root