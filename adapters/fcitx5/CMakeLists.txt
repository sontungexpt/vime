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
# libvime.so). For local development point FetchContent at the nested checkout:
#   cmake -B build -DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE=/abs/path/vime/engine
FetchContent_Declare(
    vime_engine
    GIT_REPOSITORY https://github.com/sontungexpt/vime-engine.git
    GIT_TAG main
)
FetchContent_MakeAvailable(vime_engine)
set(FFI_SOURCE_DIR "${vime_engine_SOURCE_DIR}/ffi")

add_custom_command(
    OUTPUT "${RUST_BACKEND}"
    COMMAND "${CARGO_EXECUTABLE}" build --release
            --manifest-path "${FFI_SOURCE_DIR}/Cargo.toml"
            --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY "${FFI_SOURCE_DIR}"
    COMMENT "Building the Rust Vietnamese FFI backend"
)
add_custom_target(vime-ffi-backend DEPENDS "${RUST_BACKEND}")

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
