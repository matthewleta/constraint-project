use egui::{Frame, Pos2, Sense, Vec2};

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
    pub fn setup_test_values_1(&mut self) {
        #[allow(unused_assignments)]
        let mut edge_handle_1 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_2 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_3 = 0;

        let mut display_manager_mut = self.display_manager.borrow_mut();
        {
            let mut drawing_manager_mut = self.drawing_manager.borrow_mut();

            let vh_1 = drawing_manager_mut.add_vertex(Pos2::new(50., 50.));
            display_manager_mut.add_vertex(vh_1);

            let vh_2 = drawing_manager_mut.add_vertex(Pos2::new(50., 200.));
            display_manager_mut.add_vertex(vh_2);
            let vh_3 = drawing_manager_mut.add_vertex(Pos2::new(200., 200.));
            display_manager_mut.add_vertex(vh_3);
            let vh_4 = drawing_manager_mut.add_vertex(Pos2::new(200., 50.));
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

        let mut constraint_manager_mut = self.constraint_manager.borrow_mut();

        let ch_3 = constraint_manager_mut
            .add_parallel_constraint(edge_handle_1, edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_3);

        // let ch_1 = constraint_manager_mut.add_length_constraint(edge_handle_3).unwrap();
        // display_manager_mut.add_constraint(ch_1);
        let ch_2 = constraint_manager_mut
            .add_angle_constraint(edge_handle_2, edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_2);
    }
    pub fn setup_test_values_2(&mut self) {
        #[allow(unused_assignments)]
        let mut edge_handle_1 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_2 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_3 = 0;

        let mut display_manager_mut = self.display_manager.borrow_mut();
        {
            let mut drawing_manager_mut = self.drawing_manager.borrow_mut();

            let vh_1 = drawing_manager_mut.add_vertex(Pos2::new(50., 350.));
            display_manager_mut.add_vertex(vh_1);

            let vh_2 = drawing_manager_mut.add_vertex(Pos2::new(50., 500.));
            display_manager_mut.add_vertex(vh_2);
            let vh_3 = drawing_manager_mut.add_vertex(Pos2::new(200., 500.));
            display_manager_mut.add_vertex(vh_3);
            let vh_4 = drawing_manager_mut.add_vertex(Pos2::new(200., 350.));
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

        let ch_3 = constraint_manager_mut
            .add_parallel_constraint(edge_handle_1, edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_3);

        let ch_1 = constraint_manager_mut
            .add_length_constraint(edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_1);
    }
    pub fn setup_test_values_3(&mut self) {
        #[allow(unused_assignments)]
        let mut edge_handle_1 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_2 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_3 = 0; 
        #[allow(unused_assignments)]
        let mut edge_handle_4 = 0;

        let mut display_manager_mut = self.display_manager.borrow_mut();
        {
            let mut drawing_manager_mut = self.drawing_manager.borrow_mut();

            let vh_1 = drawing_manager_mut.add_vertex(Pos2::new(450., 78.));
            display_manager_mut.add_vertex(vh_1);
            let vh_2 = drawing_manager_mut.add_vertex(Pos2::new(450., 186.));
            display_manager_mut.add_vertex(vh_2);
            let vh_3 = drawing_manager_mut.add_vertex(Pos2::new(717., 186.));
            display_manager_mut.add_vertex(vh_3);
            let vh_4 = drawing_manager_mut.add_vertex(Pos2::new(628., 78.));
            display_manager_mut.add_vertex(vh_4);

            edge_handle_1 = drawing_manager_mut.add_edge(vh_1, vh_2).unwrap();
            display_manager_mut.add_edge(edge_handle_1);
            edge_handle_2 = drawing_manager_mut.add_edge(vh_2, vh_3).unwrap();
            display_manager_mut.add_edge(edge_handle_2);
            edge_handle_3 = drawing_manager_mut.add_edge(vh_3, vh_4).unwrap();
            display_manager_mut.add_edge(edge_handle_3);
            edge_handle_4 = drawing_manager_mut.add_edge(vh_4, vh_1).unwrap();
            display_manager_mut.add_edge(edge_handle_4);
        }

        // separate scope required since constraint_manager will mutably borrow
        // the drawing manager internally, and rust won't allow it to happen twice
        let mut constraint_manager_mut = self.constraint_manager.borrow_mut();

        let ch_1 = constraint_manager_mut
            .add_angle_constraint(edge_handle_1, edge_handle_2)
            .unwrap();
        display_manager_mut.add_constraint(ch_1);
        let ch_2 = constraint_manager_mut
            .add_angle_constraint(edge_handle_2, edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_2);
    }

    pub fn setup_test_values_4(&mut self) {
        #[allow(unused_assignments)]
        let mut edge_handle_1 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_2 = 0;
        #[allow(unused_assignments)]
        let mut edge_handle_3 = 0; 
        #[allow(unused_assignments)]
        let mut edge_handle_4 = 0;

        let mut display_manager_mut = self.display_manager.borrow_mut();
        {
            let mut drawing_manager_mut = self.drawing_manager.borrow_mut();

            let vh_1 = drawing_manager_mut.add_vertex(Pos2::new(450., 378.));
            display_manager_mut.add_vertex(vh_1);
            let vh_2 = drawing_manager_mut.add_vertex(Pos2::new(450., 486.));
            display_manager_mut.add_vertex(vh_2);
            let vh_3 = drawing_manager_mut.add_vertex(Pos2::new(717., 486.));
            display_manager_mut.add_vertex(vh_3);
            let vh_4 = drawing_manager_mut.add_vertex(Pos2::new(628., 378.));
            display_manager_mut.add_vertex(vh_4);

            edge_handle_1 = drawing_manager_mut.add_edge(vh_1, vh_2).unwrap();
            display_manager_mut.add_edge(edge_handle_1);
            edge_handle_2 = drawing_manager_mut.add_edge(vh_2, vh_3).unwrap();
            display_manager_mut.add_edge(edge_handle_2);
            edge_handle_3 = drawing_manager_mut.add_edge(vh_3, vh_4).unwrap();
            display_manager_mut.add_edge(edge_handle_3);
            edge_handle_4 = drawing_manager_mut.add_edge(vh_4, vh_1).unwrap();
            display_manager_mut.add_edge(edge_handle_4);
        }

        // separate scope required since constraint_manager will mutably borrow
        // the drawing manager internally, and rust won't allow it to happen twice
        let mut constraint_manager_mut = self.constraint_manager.borrow_mut();

        let ch_1 = constraint_manager_mut
            .add_length_constraint(edge_handle_1)
            .unwrap();
        display_manager_mut.add_constraint(ch_1);
        let ch_2 = constraint_manager_mut
            .add_angle_constraint(edge_handle_2, edge_handle_3)
            .unwrap();
        display_manager_mut.add_constraint(ch_2);
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        let display_manager = Rc::clone(&self.display_manager);
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), 600.0), Sense::hover());

            display_manager
                .borrow_mut()
                .update_interaction(ui, &response);

            display_manager.borrow().draw(&response, &painter);
        });
    }

    pub fn print_values(&self) {
        let edges_ref = self.drawing_manager.borrow();
        let edges = edges_ref.get_all_edges();

        for edge in edges {
            println!("{}", edge.start_point_vh);
        }
    }
}
