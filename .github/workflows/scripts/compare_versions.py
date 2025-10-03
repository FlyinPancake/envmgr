# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import os

gh_output = os.getenv("GITHUB_OUTPUT")
if not gh_output:
    raise EnvironmentError("GITHUB_OUTPUT environment variable is not set")

gh_step_summary = os.getenv("GITHUB_STEP_SUMMARY")
if not gh_step_summary:
    raise EnvironmentError("GITHUB_STEP_SUMMARY environment variable is not set")


def write_gh_output(name: str, value: str) -> None:
    with open(gh_output, "a") as f:
        f.write(f"{name}={value}\n")


def write_gh_step_summary(content: str) -> None:
    with open(gh_step_summary, "a") as f:
        f.write(content + "\n")


def main(tag: str, cargo_version: str) -> None:
    tag = tag.lstrip("v")
    write_gh_step_summary(f"## Tag: {tag}")
    write_gh_step_summary(f"## Cargo Version: {cargo_version}")

    if tag > cargo_version:
        write_gh_step_summary(
            "Tag is ahead of Cargo version. Please update Cargo.toml."
            " And recreate the tag."
        )
        exit(1)

    if tag == cargo_version:
        write_gh_step_summary("No release needed.")
        write_gh_output("release_needed", "false")
    else:
        write_gh_step_summary("Release needed.")
        write_gh_output("release_needed", "true")

    write_gh_output("tag", tag)
    write_gh_output("cargo_version", cargo_version)


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 3:
        print("Usage: compare_versions.py <tag> <cargo_version>")
        sys.exit(1)

    tag_arg = sys.argv[1]
    cargo_version_arg = sys.argv[2]

    main(tag_arg, cargo_version_arg)
