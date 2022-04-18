# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2022-04-18

### Added
- optimize binary_args clone
- *(WorkerPool)* add warmup()
- make WorkerPool thread friendly
- *(WorkerPool)* add set_binary option
- *(WorkerPool)* simplify run_worker API
- directly expose APIs and setup rustdoc
- handle child errors
- add debug option
- add examples
- simplifies API with AsPayload
- support returning result from worker
- add bridge
- support sending payloads

### Fixed
- properly wait for idle worker

### Other
- update doc
- [**breaking**] add worker_path to WorkerPool::setup
- release 0.7.0
- fix CI
- format code
- improve documentation
- remove unecessary layer of indirection
- format code
- fix clippy error
- fix tsc compilation
- improve CI
- release 0.6.0
- format code
- improve documentation
- improve examples
- improve examples
- *(WorkerPool)* [**breaking**] rename run_task to perform
- improve github action workflows
- fix npm package version
- release 0.5.2
- fix npm package name
- correct typos
- fix incorrect workflow
- release 0.5.1
- improve docs
- fix CI
- fix CI
- fix CI
- prepare for release
- format code
- remove unecessary code
- fix clippy errors
- add tests
- improve timing benchmark
- add benchmark
- fix clippy errors
- remove napi-rs
- setup project with initial impl

## [0.7.0] - 2022-04-15

### Added
- *(WorkerPool)* add warmup()
- make WorkerPool thread friendly
- *(WorkerPool)* add set_binary option
- *(WorkerPool)* simplify run_worker API
- directly expose APIs and setup rustdoc
- handle child errors
- add debug option
- add examples
- simplifies API with AsPayload
- support returning result from worker
- add bridge
- support sending payloads

### Fixed
- properly wait for idle worker

### Other
- fix CI
- format code
- improve documentation
- remove unecessary layer of indirection
- format code
- fix clippy error
- fix tsc compilation
- improve CI
- release 0.6.0
- format code
- improve documentation
- improve examples
- improve examples
- *(WorkerPool)* [**breaking**] rename run_task to perform
- improve github action workflows
- fix npm package version
- release 0.5.2
- fix npm package name
- correct typos
- fix incorrect workflow
- release 0.5.1
- improve docs
- fix CI
- fix CI
- fix CI
- prepare for release
- format code
- remove unecessary code
- fix clippy errors
- add tests
- improve timing benchmark
- add benchmark
- fix clippy errors
- remove napi-rs
- setup project with initial impl

## [0.6.0] - 2022-04-12

### Added
- *(WorkerPool)* add set_binary option
- *(WorkerPool)* simplify run_worker API
- directly expose APIs and setup rustdoc
- handle child errors
- add debug option
- add examples
- simplifies API with AsPayload
- support returning result from worker
- add bridge
- support sending payloads

### Fixed
- properly wait for idle worker

### Other
- format code
- improve documentation
- improve examples
- improve examples
- *(WorkerPool)* [**breaking**] rename run_task to perform
- improve github action workflows
- fix npm package version
- release 0.6.0
- fix npm package name
- correct typos
- fix incorrect workflow
- release 0.5.1
- improve docs
- fix CI
- fix CI
- fix CI
- prepare for release
- format code
- remove unecessary code
- fix clippy errors
- add tests
- improve timing benchmark
- add benchmark
- fix clippy errors
- remove napi-rs
- setup project with initial impl

## [0.5.2] - 2022-04-11

### Added
- directly expose APIs and setup rustdoc
- handle child errors
- add debug option
- add examples
- simplifies API with AsPayload
- support returning result from worker
- add bridge
- support sending payloads

### Fixed
- properly wait for idle worker

### Other
- fix npm package name
- correct typos
- fix incorrect workflow
- release 0.5.1
- improve docs
- fix CI
- fix CI
- fix CI
- prepare for release
- format code
- remove unecessary code
- fix clippy errors
- add tests
- improve timing benchmark
- add benchmark
- fix clippy errors
- remove napi-rs
- setup project with initial impl

## [0.5.1] - 2022-04-11

### Added
- directly expose APIs and setup rustdoc
- handle child errors
- add debug option
- add examples
- simplifies API with AsPayload
- support returning result from worker
- add bridge
- support sending payloads

### Fixed
- properly wait for idle worker

### Other
- improve docs
- fix CI
- fix CI
- fix CI
- prepare for release
- format code
- remove unecessary code
- fix clippy errors
- add tests
- improve timing benchmark
- add benchmark
- fix clippy errors
- remove napi-rs
- setup project with initial impl
