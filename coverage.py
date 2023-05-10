#!/usr/bin/python3
"""
To run it needs the following:
cargo install rustfilt
rustup component add --toolchain nightly llvm-tools-preview
"""
import os
import re
import subprocess

# Compute the coverage for all fuzzing targets, or not
FUZZING = False

ROOT = os.path.dirname(os.path.abspath(__file__))
target_folder = os.path.join(ROOT, "target")
test_cov_path = os.path.join(target_folder, "test_cov.profraw")
rustup_info = subprocess.check_output("rustup show", shell=True).decode()
arch = re.findall(r"Default host: (.+)", rustup_info)[0]

# Get where it's installed
sysroot = subprocess.check_output("rustc --print sysroot", shell=True).decode().strip()
llvm_path = os.path.join(sysroot, "lib", "rustlib", arch, "bin")
final_cov_path = os.path.join(target_folder, "total_coverage.profdata")
test_cov_path = os.path.join(target_folder, "test_cov.profdata")
if FUZZING:
    # Get the list of fuzzing targets
    fuzz_targets = (
        subprocess.check_output(
            "cargo fuzz list",
            shell=True,
            cwd=ROOT,
        )
        .decode()
        .split("\n")[:-1]
    )
# Clean up the targets folder
subprocess.check_call(
    "cargo clean",
    shell=True,
    cwd=ROOT,
)
# Create a folder for the test coverage
os.makedirs(target_folder, exist_ok=True)
# Generate coverage from the test
subprocess.check_call(
    "cargo test",
    shell=True,
    cwd=ROOT,
    env={
        **os.environ,
        "RUSTFLAGS": "-C instrument-coverage",
        "LLVM_PROFILE_FILE": test_cov_path,
    },
)

if FUZZING:
    # Generate coverage for all the targets
    for fuzz_target in fuzz_targets:
        subprocess.check_call(
            "cargo fuzz coverage {}".format(fuzz_target),
            shell=True,
            cwd=ROOT,
        )
    fuzz_paths = [
        os.path.join(ROOT, "fuzz", "coverage", fuzz_target, "coverage.profdata")
        for fuzz_target in fuzz_targets
    ]
else:
    fuzz_paths = []


# Merge the coverages into an unique file
subprocess.check_call(
    "{}/llvm-profdata merge -sparse {} -o {}".format(
        llvm_path,
        " ".join(
            [test_cov_path] + fuzz_paths
        ),
        final_cov_path,
    ),
    shell=True,
    cwd=ROOT,
)

if FUZZING:
    fuzz_paths = [  # TODO!: add also doc tests binary
        os.path.join(
            ROOT, "target", arch, "coverage", arch, "release", fuzz_target
        )
        for fuzz_target in fuzz_targets
    ]
else:
    fuzz_paths = []

test_execs = [
    path
    for path in [
        os.path.join(ROOT, "target", "debug", "deps", file)
        for file in os.listdir(os.path.join(ROOT, "target", "debug", "deps"))
    ]
    if os.access(path, os.X_OK) and os.path.basename(path).startswith("test_")
]

# Create the report!
subprocess.check_call(
    (
        "{}/llvm-cov report --Xdemangler=rustfilt --instr-profile={} {} "
        "-ignore-filename-regex='.cargo' "
    ).format(
        llvm_path,
        final_cov_path,
        " ".join(test_execs + fuzz_paths),
    ),
    shell=True,
    cwd=ROOT,
)