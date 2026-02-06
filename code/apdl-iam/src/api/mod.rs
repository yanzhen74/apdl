//! API模块
//!
//! 提供REST API接口

#[derive(Default)]
pub struct RestApiServer;

impl RestApiServer {
    pub fn new() -> Self {
        Self
    }

    pub fn start(&self) {
        println!("Starting REST API server...");
    }
}
