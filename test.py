# For every folder in umbrel-apps, run
# cargo run -- umbrel-to-citadel umbrel-apps/<folder>
# The test succeeeded if an app.yml is created in umbrel-apps/<folder>
# And the exit code is 0
# If it fails, print the error and continue
import os
import subprocess
import sys

ignoredApps = [
    # Custom implementation on Citadel
    "electrs",
    # Built-in on Citadel
    "bitcoin",
    "lightning",
    "core-lightning",
    # This app is very hacky on Umbrel, and it's available natively on Citadel anyway
    "tailscale",
]

passed = []
passedUnique = []
failed = []
failedUnique = []

subprocess.run(
    [
        "cargo",
        "build",
        "--all-features"
    ],
)

for folder in os.listdir("umbrel-apps"):
    # If it's not a directory or a .git folder, skip it
    if not os.path.isdir(os.path.join("umbrel-apps", folder)) or folder == ".git":
        continue
    if folder in ignoredApps:
        print(f"\033[90m[SKIPPED]\033[0m {folder}")
        continue
    # Delete app.yml if it exists
    if os.path.exists(f"umbrel-apps/{folder}/app.yml"):
        os.remove(f"umbrel-apps/{folder}/app.yml")

    try:
        subprocess.run(
            [
                "cargo",
                "run",
                "--all-features",
                "--",
                "umbrel-to-citadel",
                f"umbrel-apps/{folder}",
            ],
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"\033[31m[FAILED]\033[0m {folder}")
        failed.push(folder)
        if not os.path.exists(f"citadel-apps/v5/{folder}"):
            failedUnique.append(folder)
        print(e.stderr)
        continue
    if not os.path.exists(f"umbrel-apps/{folder}/app.yml"):
        print(f"\033[31m[FAILED]\033[0m {folder}")
        failed.append(folder)
        if not os.path.exists(f"citadel-apps/v5/{folder}"):
            failedUnique.append(folder)
        continue
    print(f"\033[32m[PASSED]\033[0m {folder}")
    passed.append(folder)
    if not os.path.exists(f"citadel-apps/v5/{folder}"):
        passedUnique.append(folder)


total = len(passed) + len(failed) + len(ignoredApps)
print(f"Passed: {len(passed)}/{total} ({round(len(passed)/total*100, 2)}%) ({len(passedUnique)} not available on Citadel)")
print(f"Failed: {len(failed)}/{total} ({round(len(failed)/total*100, 2)}%) ({len(failedUnique)} not available on Citadel)")
print(f"Skipped: {len(ignoredApps)}/{total} ({round(len(ignoredApps)/total*100, 2)}%)")
