# Release Process

This project is configured for PyPI releases through GitHub Actions. Publishing
is triggered by creating a GitHub Release. The release workflow builds binary
wheels and an sdist with maturin, uploads the distributions as an artifact, and
publishes them to PyPI through trusted publishing.

Before the first release, configure a PyPI trusted publisher for this GitHub
repository with:

- owner: `OhadRubin`
- repository: `dua-py`
- workflow: `python-publish.yml`
- environment: `pypi`

Local release artifact check:

```bash
maturin build --release --locked --compatibility pypi --out dist
maturin sdist --out dist
python -m twine check dist/*
```
