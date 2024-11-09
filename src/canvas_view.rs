use egui::{emath, Color32, Frame, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use crate::drawing_manager::DrawingManager;

pub struct CanvasView {
    drawing_manager: DrawingManager,
    //network : DrawingNetwork
}

impl Default for CanvasView {
    fn default() -> Self {
        let mut drawing_manager = DrawingManager::new();

        let vh_1 = drawing_manager.add_vertex(Pos2::new(10., 10.));
        let vh_2 = drawing_manager.add_vertex(Pos2::new(10., 50.));
        let vh_3 = drawing_manager.add_vertex(Pos2::new(50., 50.));
        let vh_4 = drawing_manager.add_vertex(Pos2::new(50., 10.));

        let eh_1 = drawing_manager.add_edge(vh_1, vh_2).unwrap();
        let eh_2 = drawing_manager.add_edge(vh_2, vh_3).unwrap();
        let eh_3 = drawing_manager.add_edge(vh_3, vh_4).unwrap();

        Self { drawing_manager }
    }
}

impl CanvasView {
    pub fn update(&mut self, ui: &mut egui::Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            self.generate_content(ui);
        });
    }

    pub fn generate_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        let control_point_radius = 5.;
        
        let control_point_shapes: Vec<Shape> = self
            .drawing_manager.get_all_vertices_mut()
            .iter_mut()
            .enumerate()
            .map(|(i, vertex)| {
                
                let mut pt_data = vertex.position;

                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(pt_data);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                pt_data += point_response.drag_delta();

                // update point
                vertex.position = to_screen.from().clamp(pt_data);

                let point_in_screen = to_screen.transform_pos(pt_data);
                let stroke = ui.style().interact(&point_response).fg_stroke;
                let stroke2 = Stroke::new(10., Color32::LIGHT_RED);
                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        // Draw Lines from edges
        let edge_shapes: Vec<Shape> = self
        .drawing_manager.get_all_edges()
        //let control_point_shapes: Vec<Shape> = self
            .iter_mut()
            .enumerate()
            .map(|(i, edge)| {
                let mut start_point = self.drawing_manager.get_vertex(edge.start_point_vh).unwrap().position;
                let mut end_point = self.drawing_manager.get_vertex(edge.end_point_vh).unwrap().position;

                start_point = to_screen.transform_pos(start_point);
                end_point = to_screen.transform_pos(end_point);

                let stroke = Stroke::new(8., Color32::DARK_RED);
                Shape::line_segment([start_point, end_point], stroke)
            })
            .collect();

        //painter.add(PathShape::line(points_in_screen, self.aux_stroke));
        painter.extend(edge_shapes);
        painter.extend(control_point_shapes);

        response
    }
}