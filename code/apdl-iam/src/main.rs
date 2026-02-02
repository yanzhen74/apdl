//! APDL 交互访问模块 (IAM)
//!
//! 提供用户交互和访问接口

use apdl_iam::GuiApp;
use eframe::NativeOptions;

fn main() -> Result<(), eframe::Error> {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "APDL Interaction Access Module",
        native_options,
        Box::new(|_cc| Ok(Box::new(GuiApp::new()))),
    )
}
