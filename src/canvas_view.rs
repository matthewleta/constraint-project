use egui::{emath, Color32, Frame, Pos2, Rect, Sense, Shape, Stroke, Vec2};

use crate::constraint_manager::ConstraintManager;
use crate::display_manager::DisplayManager;
use crate::drawing_manager::DrawingManager;

use std::cell::RefCell;
use std::rc::Rc;

pub struct CanvasView {
    drawing_manager: Rc<RefCell<DrawingManager>>,
    display_manager: Rc<RefCell<DisplayManager>>,
    constraint_manager: Rc<RefCell<ConstraintManager>>,
    //network : DrawingNetwork
}

impl Default for CanvasView {
    fn default() -> Self {
        let drawing_manager = Rc::new(RefCell::new(DrawingManager::new()));
        let display_manager = Rc::new(RefCell::new(DisplayManager::new()));
        let constraint_manager = Rc::new(RefCell::new(ConstraintManager::new()));

        drawing_manager
            .borrow_mut()
            .set_display_manager(Rc::clone(&display_manager));
        display_manager
            .borrow_mut()
            .set_drawing_manager(Rc::clone(&drawing_manager));
        display_manager
            .borrow_mut()
            .set_constraint_manager(Rc::clone(&constraint_manager));
        constraint_manager
            .borrow_mut()
            .set_drawing_manager(Rc::clone(&drawing_manager));

        Self {
            display_manager,
            drawing_manager,
            constraint_manager,
        }
    }
}

impl CanvasView {
    pub fn setup_test_values(&mut self) {
        let mut edge_handle_1 = 0;
        let mut edge_handle_2 = 0;
        let mut edge_handle_3 = 0;

        {
            let mut drawing_manager_mut = self.drawing_manager.borrow_mut();
            let mut display_manager_mut = self.display_manager.borrow_mut();

            let vh_1 = drawing_manager_mut.add_vertex(Pos2::new(10., 10.));
            display_manager_mut.add_vertex(vh_1);

            let vh_2 = drawing_manager_mut.add_vertex(Pos2::new(10., 50.));
            display_manager_mut.add_vertex(vh_2);
            let vh_3 = drawing_manager_mut.add_vertex(Pos2::new(50., 50.));
            display_manager_mut.add_vertex(vh_3);
            let vh_4 = drawing_manager_mut.add_vertex(Pos2::new(50., 10.));
            display_manager_mut.add_vertex(vh_4);

            edge_handle_1 = drawing_manager_mut.add_edge(vh_1, vh_2).unwrap();
            display_manager_mut.add_edge(edge_handle_1);
            edge_handle_2 = drawing_manager_mut.add_edge(vh_2, vh_3).unwrap();
            display_manager_mut.add_edge(edge_handle_2);
            edge_handle_3 = drawing_manager_mut.add_edge(vh_3, vh_4).unwrap();
            display_manager_mut.add_edge(edge_handle_3);
        }

        // separate scope required since constraint_manager will mutably borrow
        // the drawing manager internally, and rust won't allow it to happen twice

        //let drawing_manager = self.drawing_manager.borrow();
        let mut constraint_manager_mut = self.constraint_manager.borrow_mut();

        let _ = constraint_manager_mut.add_length_constraint(edge_handle_3);
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        let display_manager = Rc::clone(&self.display_manager);
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

            display_manager
                .borrow_mut()
                .update_interaction(ui, &response);

            display_manager.borrow().draw(ui, &response, &painter);

            //self.generate_content(ui);
        });
    }

    pub fn print_values(&self) {
        let edges_ref = self.drawing_manager.borrow();
        let edges = edges_ref.get_all_edges();

        for edge in edges {
            println!("{}", edge.start_point_vh);
        }
    }
    // pub fn generate_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
    //     // let (response, painter) =
    //     //     ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

    //     // let to_screen = emath::RectTransform::from_to(
    //     //     Rect::from_min_size(Pos2::ZERO, response.rect.size()),
    //     //     response.rect,
    //     // );
    //     // let control_point_radius = 5.;

    //     // let control_point_shapes: Vec<Shape> = self
    //     //     .drawing_manager.get_all_vertices_mut()
    //     //     .iter_mut()
    //     //     .enumerate()
    //     //     .map(|(i, vertex)| {

    //     //         let mut pt_data = vertex.position;

    //     //         let size = Vec2::splat(2.0 * control_point_radius);

    //     //         let point_in_screen = to_screen.transform_pos(pt_data);
    //     //         let point_rect = Rect::from_center_size(point_in_screen, size);
    //     //         let point_id = response.id.with(i);
    //     //         let point_response = ui.interact(point_rect, point_id, Sense::drag());

    //     //         pt_data += point_response.drag_delta();

    //     //         // update point
    //     //         vertex.position = to_screen.from().clamp(pt_data);

    //     //         let point_in_screen = to_screen.transform_pos(pt_data);
    //     //         let stroke = ui.style().interact(&point_response).fg_stroke;
    //     //         let stroke2 = Stroke::new(10., Color32::LIGHT_RED);
    //     //         Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
    //     //     })
    //     //     .collect();

    //     // // Draw Lines from edges
    //     // let edge_shapes: Vec<Shape> = self
    //     // .drawing_manager.get_all_edges()
    //     // //let control_point_shapes: Vec<Shape> = self
    //     //     .iter_mut()
    //     //     .enumerate()
    //     //     .map(|(i, edge)| {
    //     //         let mut start_point = self.drawing_manager.get_vertex(edge.start_point_vh).unwrap().position;
    //     //         let mut end_point = self.drawing_manager.get_vertex(edge.end_point_vh).unwrap().position;

    //     //         start_point = to_screen.transform_pos(start_point);
    //     //         end_point = to_screen.transform_pos(end_point);

    //     //         let stroke = Stroke::new(8., Color32::DARK_RED);
    //     //         Shape::line_segment([start_point, end_point], stroke)
    //     //     })
    //     //     .collect();

    //     // //painter.add(PathShape::line(points_in_screen, self.aux_stroke));
    //     // painter.extend(edge_shapes);
    //     // painter.extend(control_point_shapes);

    //     response
    // }
}
