use crate::drawing_manager::DrawingManager;
use std::collections::HashMap;

use egui::{Vec2, Ui, Sense, Painter};

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;
pub struct DisplayManager<'a> {
    drawing_manager: &'a mut DrawingManager,
    edges: HashMap<VertexHandle, VertexDisplay>,
    vertices: HashMap<EdgeHandle, EdgeDisplay>,
    constraints: HashMap<ConstraintHandle, ConstraintDisplay>,
}

impl<'a> DisplayManager<'a> {
    pub fn new(drawing_manager: &'a mut DrawingManager) -> Self {
        Self {
            drawing_manager,
            edges: HashMap::new(),
            vertices: HashMap::new(),
            constraints: HashMap::new(),
        }
    }

    pub fn draw(&self, ui : &mut Ui){
        let (response, painter) =
        ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());


    }
}

pub struct VertexDisplay {
    vertex_handle: VertexHandle,
}

impl VertexDisplay {
    pub fn draw(&self, ui: &egui::Ui, painter : Painter) {
        let control_point_radius = 5.;

        let size = Vec2::splat(2.0 * control_point_radius);

        let mut pt_data = vertex.position;
        let point_in_screen = to_screen.transform_pos(pt_data);
        let stroke = ui.style().interact(&point_response).fg_stroke;
        let stroke2 = Stroke::new(10., Color32::LIGHT_RED);
        Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
    }
}

pub struct EdgeDisplay {
    edge_handle: EdgeHandle,
}

pub struct ConstraintDisplay {
    constraint_handle: ConstraintHandle,
}
