cmake_minimum_required(VERSION 3.16)
project(vime-fcitx5 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(Fcitx5Core REQUIRED)
find_package(Fcitx5Utils REQUIRED)


find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(NOT DEFINED FCITX_INSTALL_ADDONDIR)
    set(FCITX_INSTALL_ADDONDIR "lib/fcitx5")
endif()
if(NOT DEFINED FCITX_INSTALL_PKGDATADIR)
    set(FCITX_INSTALL_PKGDATADIR "share/fcitx5")
endif()

set(FFI_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../engine/ffi")
set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
set(RUST_BACKEND "${RUST_TARGET_DIR}/release/libvime.so")

include(FetchContent)

# vime-engine is consumed as a dependency from GitHub (its `ffi` crate provides
# libvime.so). In a development layout the engine checkout lives in the sibling
# directory ../../engine, so use the in-tree copy automatically; standalone and
# release builds fall back to cloning from GitHub. An explicit
# -DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE=... always takes precedence.
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../../engine/Cargo.toml"
   AND NOT DEFINED FETCHCONTENT_SOURCE_DIR_VIME_ENGINE)
    set(FETCHCONTENT_SOURCE_DIR_VIME_ENGINE
        "${CMAKE_CURRENT_SOURCE_DIR}/../../engine")
    message(STATUS "vime-engine: using in-tree checkout ../../engine")
endif()
FetchContent_Declare(
    vime_engine
    GIT_REPOSITORY https://github.com/sontungexpt/vime-engine.git
    GIT_TAG main
)
FetchContent_MakeAvailable(vime_engine)
set(FFI_SOURCE_DIR "${vime_engine_SOURCE_DIR}/ffi")

# Always run cargo (it is cheap on no-op builds and detects changes itself);
# this avoids cmake's stale-output problem with add_custom_command when Rust
# sources change.
add_custom_target(vime-ffi-backend ALL
    COMMAND "${CARGO_EXECUTABLE}" build --release
            --manifest-path "${FFI_SOURCE_DIR}/Cargo.toml"
            --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY "${FFI_SOURCE_DIR}"
    BYPRODUCTS "${RUST_BACKEND}"
    COMMENT "Building the Rust Vietnamese FFI backend"
)

add_library(vime MODULE
    src/state.cpp
    src/engine.cpp
    src/factory.cpp
)

target_include_directories(vime PRIVATE
    "${FFI_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
)
add_dependencies(vime vime-ffi-backend)
target_link_directories(vime PRIVATE "${RUST_TARGET_DIR}/release")
target_link_libraries(vime PRIVATE Fcitx5::Core Fcitx5::Utils "${RUST_BACKEND}")
set_target_properties(vime PROPERTIES PREFIX "")
set_target_properties(vime PROPERTIES INSTALL_RPATH "$ORIGIN")

install(TARGETS vime DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES "${RUST_BACKEND}" DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES addon.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
        RENAME vime.conf)
install(FILES vime.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")
