#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

mod canvas_view;
pub use canvas_view::CanvasView;


mod drawing_manager;
pub use drawing_manager::DrawingManager;

mod display_manager;