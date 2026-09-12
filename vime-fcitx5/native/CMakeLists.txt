cmake_minimum_required(VERSION 3.16)
project(vime-fcitx5-native LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(Fcitx5Core REQUIRED)
find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(NOT DEFINED FCITX_INSTALL_ADDONDIR)
    set(FCITX_INSTALL_ADDONDIR "lib/fcitx5")
endif()
if(NOT DEFINED FCITX_INSTALL_PKGDATADIR)
    set(FCITX_INSTALL_PKGDATADIR "share/fcitx5")
endif()

set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
set(RUST_BACKEND "${RUST_TARGET_DIR}/release/libvime_fcitx5.so")
add_custom_command(
    OUTPUT "${RUST_BACKEND}"
    COMMAND "${CARGO_EXECUTABLE}" build --release
            --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/../Cargo.toml"
            --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/.."
    COMMENT "Building the Rust Vietnamese backend"
)
add_custom_target(vime-fcitx5-backend DEPENDS "${RUST_BACKEND}")

add_library(vime MODULE fcitx5.cpp)
get_target_property(FCITX5_INCLUDE_DIRS Fcitx5::Core INTERFACE_INCLUDE_DIRECTORIES)
if(NOT FCITX5_INCLUDE_DIRS)
    set(FCITX5_INCLUDE_DIRS "/usr/include/Fcitx5/Core")
endif()
target_include_directories(vime PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/../include"
    "${FCITX5_INCLUDE_DIRS}")
add_dependencies(vime vime-fcitx5-backend)
target_link_directories(vime PRIVATE "${RUST_TARGET_DIR}/release")
target_link_libraries(vime PRIVATE Fcitx5::Core vime_fcitx5)
set_target_properties(vime PROPERTIES PREFIX "")
set_target_properties(vime PROPERTIES INSTALL_RPATH "$ORIGIN")

install(TARGETS vime DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES "${RUST_BACKEND}" DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES ../addon.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
        RENAME vime.conf)
install(FILES ../vime.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")
