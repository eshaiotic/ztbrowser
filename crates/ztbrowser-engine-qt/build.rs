use cxx_qt_build::CxxQtBuilder;

fn main() {
    CxxQtBuilder::new()
        .file("src/engine_bridge.rs")
        .include_dir("cpp")
        .cpp_file("cpp/engine_hooks.cpp")
        .qt_module("Quick")
        .qt_module("Gui")
        .qt_module("Network")
        .qt_module("WebEngineQuick")
        .qt_module("WebEngineCore")
        .build();
}
