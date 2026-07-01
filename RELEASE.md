# Release Process

This project is configured for PyPI releases through GitHub Actions. Publishing
is triggered by creating a GitHub Release. The release workflow builds binary
wheels and an sdist with maturin, uploads the distributions as an artifact, and
publishes them to PyPI with the `PYPI_API_TOKEN` GitHub secret.

Current binary wheels target macOS universal2 and Linux x86_64/aarch64.
Windows wheels are intentionally omitted because `dua-cli` currently requires
an unstable Rust feature for its Windows library build.

Before releasing, make sure the repository has a `PYPI_API_TOKEN` secret with a
PyPI API token that can publish `dua-py`.

Local release artifact check:

```bash
maturin build --release --locked --compatibility pypi --out dist
maturin sdist --out dist
python -m twine check dist/*
```
