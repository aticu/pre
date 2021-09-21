# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
However, since version 1.0.0 has not been reached, breaking changes between minor releases are
possible, though if possible they are avoided.

## [0.2.1] - 2021-09-21

### Changed

- Version `0.2.1` is compatible with `nightly-2021-09-19`.
- Minor improvements were made to the generated documentation.

## [0.2.0] - 2020-07-26

### Added

- Preconditions for the `unsafe` functions and methods in the following modules are now supported:
    - `core::slice`/`std::slice`
    - `alloc::vec`/`std::vec`
    - `alloc::string`/`std::string`
    - `core::str`/`alloc::str`/`std::str`
- Preconditions for the methods on the following primitive types are now supported:
    - `*const T` (with `#[forward(impl pre::std::const_pointer)]`)
    - `*mut T` (with `#[forward(impl pre::std::mut_pointer)]`)
- The `proper_align` precondition type was added. It allows specifying that a pointer has a proper
  alignment for the type its pointing to.
- pre-related attributes behind a `cfg_attr` attribute are now supported.
  With [some limitations](https://github.com/aticu/pre#known-limitations).

### Changed

- Functions that previously stated "`ptr` is proper aligned" now use the `proper_align`
  precondition type. **This is a breaking change.**
- The main pre crate now depends on the exact same version of the proc-macro crate. A version
  mismatch between them could have caused things to misbehave. Version 0.1.0 was yanked because of
  that.

## [0.1.0] - 2020-07-14 [YANKED]

Initial release
