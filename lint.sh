#!/bin/bash

run_cargo_fmt() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        cargo fmt --all -- --check
    else
        cargo fmt --all
    fi
    return $?
}

run_cargo_clippy() {
    MODE=$1
    CMD="cargo clippy --all --all-targets --all-features"
    if [ "$MODE" == "fix" ]; then
        $CMD --fix --allow-staged --allow-dirty
    fi
    CMD="$CMD -- -D warnings"
    $CMD
    return $?
}

run_dprint() {
    MODE=$1
    if [ "$MODE" == "check" ]; then
        dprint check
    else
        dprint fmt
    fi
    return $?
}

run_autogen_schema() {
    MODE=$1
    cargo run -p gqlforge-typedefs $MODE
    return $?
}

run_check_non_ascii() {
    local result
    result=$(LC_ALL=C grep -rn \
        --include="*.rs" \
        --include="*.toml" \
        --include="*.yml" \
        --include="*.yaml" \
        --include="*.graphql" \
        --include="*.json" \
        --exclude-dir=target \
        --exclude-dir=.git \
        '[^[:print:][:space:]]' . 2>/dev/null)
    if [ -n "$result" ]; then
        echo "Error: Non-ASCII characters found in source files:"
        echo "$result"
        return 1
    fi
    return 0
}

# Extract the mode from the argument
if [[ $1 == "--mode="* ]]; then
    MODE=${1#--mode=}
else
    echo "Please specify a mode with --mode=check or --mode=fix"
    exit 1
fi

# Run commands based on mode
case $MODE in
    check|fix)
        run_check_non_ascii
        NON_ASCII_EXIT_CODE=$?

        run_autogen_schema $MODE
        AUTOGEN_SCHEMA_EXIT_CODE=$?

        run_dprint $MODE
        DPRINT_EXIT_CODE=$?

        # Commands that uses nightly toolchains are run from `.nightly` directory
        # to read the nightly version from `rust-toolchain.toml` file
        pushd .nightly
        run_cargo_fmt $MODE
        FMT_EXIT_CODE=$?
        run_cargo_clippy $MODE
        CLIPPY_EXIT_CODE=$?
        popd
        ;;
    *)
        echo "Invalid mode. Please use --mode=check or --mode=fix"
        exit 1
        ;;
esac

# If any command failed, exit with a non-zero status code
if [ $FMT_EXIT_CODE -ne 0 ] || [ $CLIPPY_EXIT_CODE -ne 0 ] || \
   [ $DPRINT_EXIT_CODE -ne 0 ] || [ $AUTOGEN_SCHEMA_EXIT_CODE -ne 0 ] || \
   [ $NON_ASCII_EXIT_CODE -ne 0 ]; then
    exit 1
fi