# Changelog

All notable changes to this project will be documented in this file.

## [0.1.2] - 2022-09-17

### Features

#### Classify commas to prepare for the new wildcard selectors

### Bugfix

### Non-ASCII characters like (¡) breaking SIMD classification

## [0.1.1] - 2022-07-26

### Bug Fixes

#### Supported simd is now autodetected

Instead of relying on the target_feature compiler flag the build script now autodetects whether AVX2 is supported and compiles the correct version.

### Dependencies

#### Update to use `criterion_decimal_throughput`

#### Equalise `aligners` versions (`0.0.9` across the project)

#### Remove unnecessary dependencies

Removed `memchr` and `static_assertions`.

## [0.1.0] - 2022-07-15

### Features

- Engine implementation for child and recursive selectors.

<!-- generated by git-cliff -->