set -euxo pipefail

main() {
    cargo check -p alloc-many --target $T
    cargo check -p alloc-many-collections --target $T

    if [ $T = x86_64-unknown-linux-gnu ] && [ $TRAVIS_RUST_VERSION = nightly ]; then
        cargo test -p alloc-many-collections
        cargo test -p alloc-many-collections --release

        cd bump

        cargo test
        cargo test --release

        export RUSTFLAGS="-Z sanitizer=thread"
        export RUST_TEST_THREADS=1
        export TSAN_OPTIONS="suppressions=$(pwd)/suppressions.txt"

        cargo test --test tsan --target $T
        cargo test --test tsan --target $T --release
    fi
}

# fake Travis variables to be able to run this on a local machine
if [ -z ${TRAVIS_BRANCH-} ]; then
    TRAVIS_BRANCH=auto
fi

if [ -z ${TRAVIS_PULL_REQUEST-} ]; then
    TRAVIS_PULL_REQUEST=false
fi

if [ -z ${TRAVIS_RUST_VERSION-} ]; then
    case $(rustc -V) in
        *nightly*)
            TRAVIS_RUST_VERSION=nightly
            ;;
        *beta*)
            TRAVIS_RUST_VERSION=beta
            ;;
        *)
            TRAVIS_RUST_VERSION=stable
            ;;
    esac
fi

if [ -z ${T-} ]; then
    T=$(rustc -Vv | grep host | cut -d ' ' -f2)
fi

if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
    main
fi
