use crate::constraint_manager::{ConstraintManager, SolverResponse, SolverState};
use crate::drawing_manager::DrawingManager;

use std::collections::HashMap;

use egui::{emath, Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

use std::cell::RefCell;
use std::rc::{Rc, Weak};

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;

#[derive(Default)]
pub struct DisplayManager {
    drawing_manager: Option<Rc<RefCell<DrawingManager>>>,
    constraint_manager: Option<Rc<RefCell<ConstraintManager>>>,

    edges: HashMap<VertexHandle, EdgeDisplay>,
    vertices: HashMap<EdgeHandle, VertexDisplay>,
    constraints: HashMap<ConstraintHandle, ConstraintDisplay>,
}

impl DisplayManager {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn set_drawing_manager(&mut self, drawing_manager: Rc<RefCell<DrawingManager>>) {
        self.drawing_manager = Some(drawing_manager);
    }
    pub fn set_constraint_manager(&mut self, constraint_manager: Rc<RefCell<ConstraintManager>>) {
        self.constraint_manager = Some(constraint_manager);
    }

    pub fn update_interaction(&mut self, ui: &Ui, response: &Response) {
        self.vertices.iter_mut().for_each(|v| {
            v.1.interact(ui, response);
        });
    }

    pub fn draw(&self, ui: &mut Ui, response: &Response, painter: &Painter) {
        let segments: Vec<Shape> = self
            .edges
            .iter()
            .map(|(_, edge)| edge.get_shape(&response))
            .collect();

        painter.extend(segments);

        let vertices: Vec<Shape> = self
            .vertices
            .iter()
            .map(|(_, vertex)| vertex.get_shape(&response))
            .collect();

        painter.extend(vertices);
    }

    pub fn print_edge_length(&self) {
        println!("DisplayManager Edge Length: {}", self.edges.len());
    }

    pub fn add_vertex(&mut self, vertex_handle: VertexHandle) {
        let dm_weak = Rc::downgrade(self.drawing_manager.as_mut().unwrap());
        let constr_weak = Rc::downgrade(self.constraint_manager.as_mut().unwrap());
        self.vertices.insert(
            vertex_handle,
            VertexDisplay::new(dm_weak, constr_weak, vertex_handle),
        );
        println!("vert added");
    }

    pub fn add_edge(&mut self, edge_handle: EdgeHandle) {
        let weak = Rc::downgrade(self.drawing_manager.as_mut().unwrap());
        self.edges
            .insert(edge_handle, EdgeDisplay::new(weak, edge_handle));
        println!("edge added");
    }

    pub fn add_constraint(&mut self, constraint_handle: ConstraintHandle) {
        let weak = Rc::downgrade(self.drawing_manager.as_mut().unwrap());
        self.constraints.insert(
            constraint_handle,
            ConstraintDisplay::new(weak, constraint_handle),
        );
    }
}

pub struct VertexDisplay {
    drawing_manager: Weak<RefCell<DrawingManager>>,
    constraint_manager: Weak<RefCell<ConstraintManager>>,

    vertex_handle: VertexHandle,
    is_selected: bool,
    is_being_dragged: bool,
    is_hovered: bool,

    pre_drag_position: Pos2,
    current_drag_position: Pos2,
}

impl VertexDisplay {
    pub fn new(
        drawing_manager: Weak<RefCell<DrawingManager>>,
        constraint_manager: Weak<RefCell<ConstraintManager>>,
        vertex_handle: VertexHandle,
    ) -> Self {
        Self {
            drawing_manager,
            constraint_manager,
            vertex_handle,
            is_selected: false,
            is_being_dragged: false,
            is_hovered: false,
            pre_drag_position: Pos2::new(0., 0.),
            current_drag_position: Pos2::new(0., 0.)
        }
    }

    pub fn interact(&mut self, ui: &Ui, response: &Response) {
        let buffer_size = Vec2::splat(30.0);

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let point_in_screen = to_screen.transform_pos(self.get_vertex_point());
        let point_rect = Rect::from_center_size(point_in_screen, buffer_size);
        let point_id = response.id.with(self.vertex_handle);
        let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());

        self.is_hovered = false;

        let cursor_opt = point_response.hover_pos();

        if point_response.hovered() && cursor_opt.is_some() {
            let cursor_pt = point_response.hover_pos().unwrap();
            let cursor_pt = to_screen.inverse().transform_pos(cursor_pt);

            let is_on_vertex = self.is_point_on_vertex(cursor_pt, 10.0);

            if is_on_vertex {
                self.is_hovered = true;

                if point_response.clicked() {
                    self.is_selected = !self.is_selected;
                }

                // drag begins -- initiate drag parameters
                if !self.is_being_dragged && point_response.dragged(){
                    // set previous position before starting to drag
                    let dm_shared = self.drawing_manager.upgrade().unwrap();
                    let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                    self.pre_drag_position = dm_borrow.get_vertex_mut(self.vertex_handle).unwrap().position;
                    self.current_drag_position = self.pre_drag_position;

                    self.is_being_dragged = true;
                    println!("drag start");

                }
            }
        }

        // drag ends
        // this is outside of the hovered() call so that it will
        // properly be called without being in the interact region
        if self.is_being_dragged && !point_response.dragged(){
            // let dm_shared = self.drawing_manager.upgrade().unwrap();
            // let mut dm_borrow = dm_shared.as_ref().borrow_mut();
            // let vertex = dm_borrow.get_vertex_mut(self.vertex_handle).unwrap();
            // vertex.position = self.current_drag_position;

            self.is_being_dragged = false;
            println!("drag end");
        }

        if self.is_being_dragged {
            // let mut current_vertex_pos = Pos2::new(0., 0.);
            // {
            //     let dm_shared = self.drawing_manager.upgrade().unwrap();
            //     let mut dm_borrow = dm_shared.as_ref().borrow_mut();
            //     let vertex = dm_borrow.get_vertex_mut(self.vertex_handle).unwrap();
            //     current_vertex_pos = vertex.position;
            // }

            if cursor_opt.is_some(){
                let cursor_pt = point_response.hover_pos().unwrap();
                self.current_drag_position = to_screen.inverse().transform_pos(cursor_pt);
            }
    

            let constr_shared = self.constraint_manager.upgrade().unwrap();
            let mut constr_borrow = constr_shared.as_ref().borrow_mut();

            let try_pt = self.current_drag_position;
            
            let solver_response = constr_borrow.solve_for_vertex(self.vertex_handle, &self.pre_drag_position, &try_pt);

            // get mutable vertex again, so we can modify it
            let dm_shared = self.drawing_manager.upgrade().unwrap();
            let mut dm_borrow = dm_shared.as_ref().borrow_mut();
            let vertex = dm_borrow.get_vertex_mut(self.vertex_handle).unwrap();


            match solver_response.state {
                SolverState::Free => {println!("Free"); vertex.position = try_pt},
                SolverState::Locked => {println!("Locked");()},
                SolverState::Partial => {
                    //println!("Partial {}", solver_response.new_pos.unwrap());
                    //vertex.position = Pos2::new(150., 150.);
                    vertex.position = solver_response.new_pos.unwrap()
                },
            }

            //vertex.position = try_pt;
        }

        // point_response.interact_pointer_pos()

        // ui.input(|i| i.pointer.hover_pos())

        // pt_data += point_response.drag_delta();

        // // update point
        // vertex.position = to_screen.from().clamp(pt_data);

        // let point_in_screen = to_screen.transform_pos(pt_data);
    }

    pub fn get_shape(&self, response: &Response) -> Shape {
        let base_color = if self.is_selected || self.is_being_dragged {
            Color32::WHITE.gamma_multiply(0.9)
        } else {
            Color32::GRAY
        };

        // let base_color = if self.is_selected {
        //     Color32::WHITE.gamma_multiply(0.9)
        // } else if self.is_being_dragged {Color32::GREEN}
        // else {
        //     Color32::GRAY
        // };
        
        let hover_color = base_color.clone().gamma_multiply(1.2);

        let current_color = if self.is_hovered {
            base_color
        } else {
            hover_color
        };

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        //let circ_position = if self.is_being_dragged {self.current_drag_position} else {self.get_vertex_point()};
        let circ_position = self.get_vertex_point();

        let point_in_screen = to_screen.transform_pos(circ_position);

        let control_point_radius = 10.0;

        Shape::circle_filled(point_in_screen, control_point_radius, current_color)
    }
    fn get_vertex_point(&self) -> Pos2 {
        let drawing_manager_rc = self.drawing_manager.upgrade().unwrap();
        let drawing_manager = drawing_manager_rc.borrow_mut();

        drawing_manager
            .get_vertex(self.vertex_handle)
            .unwrap()
            .position
    }
    fn is_point_on_vertex(&self, point: Pos2, radius: f32) -> bool {
        let dist = point.distance(self.get_vertex_point());
        dist <= radius
    }
}

pub struct EdgeDisplay {
    drawing_manager: Weak<RefCell<DrawingManager>>,
    edge_handle: EdgeHandle,
    is_selected: bool,
    is_hovered: bool,
}

impl EdgeDisplay {
    pub fn new(drawing_manager: Weak<RefCell<DrawingManager>>, edge_handle: EdgeHandle) -> Self {
        Self {
            drawing_manager,
            edge_handle,
            is_selected: false,
            is_hovered: false,
        }
    }

    pub fn get_shape(&self, response: &Response) -> Shape {
        let base_color = if self.is_selected {
            Color32::WHITE.gamma_multiply(0.9)
        } else {
            Color32::GRAY
        };
        let hover_color = base_color.clone().gamma_multiply(1.2);

        let current_color = if self.is_hovered {
            base_color
        } else {
            hover_color
        };

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let stroke = Stroke::new(5., current_color);
        let [p_1, p_2] = self.get_end_points();

        Shape::line_segment(
            [to_screen.transform_pos(p_1), to_screen.transform_pos(p_2)],
            stroke,
        )
    }

    fn get_end_points(&self) -> [Pos2; 2] {
        let drawing_manager_rc = self.drawing_manager.upgrade().unwrap();
        let drawing_manager = drawing_manager_rc.borrow_mut();

        // Now, get the vertex and access its position
        let edge = drawing_manager.get_edge(self.edge_handle).unwrap();

        //let edge = self.get_edge();
        let vert_1 = drawing_manager
            .get_vertex(edge.start_point_vh)
            .unwrap()
            .position;
        let vert_2 = drawing_manager
            .get_vertex(edge.end_point_vh)
            .unwrap()
            .position;
        [vert_1, vert_2]
    }
}

pub struct ConstraintDisplay {
    drawing_manager: Weak<RefCell<DrawingManager>>,
    constraint_handle: ConstraintHandle,
}

impl ConstraintDisplay {
    pub fn new(
        drawing_manager: Weak<RefCell<DrawingManager>>,
        constraint_handle: ConstraintHandle,
    ) -> Self {
        Self {
            drawing_manager,
            constraint_handle,
        }
    }
}
