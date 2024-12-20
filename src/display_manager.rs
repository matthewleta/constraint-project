use crate::constraint_manager::{
    Constraint, ConstraintManager, ConstraintPath,  SolverState,
};
use crate::drawing_manager::DrawingManager;

use core::f32;
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

    pub constraint_paths: Vec<ConstraintPath>,
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
        self.constraint_paths.clear();

        self.edges.iter_mut().for_each(|e| {
            e.1.interact(&mut self.constraint_paths, ui, response);
        });
        self.vertices.iter_mut().for_each(|v| {
            v.1.interact(&mut self.constraint_paths, ui, response);
        });
    }

    pub fn draw(&self, response: &Response, painter: &Painter) {
        let const_shapes = self.generate_constraint_shapes(response);

        painter.extend(const_shapes);

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

        let mut constr_shapes: Vec<Shape> = vec![];

        for constraint in &self.constraints {
            constr_shapes.extend(constraint.1.get_shape(&response));
        }

        painter.extend(constr_shapes);
    }

    pub fn generate_constraint_shapes(&self, response: &Response) -> Vec<Shape> {
        let constraint_color = Color32::LIGHT_RED;
        let mut shapes: Vec<Shape> = vec![];

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );
        for path in &self.constraint_paths {
            match path {
                ConstraintPath::Circle(c) => {
                    let point_in_screen = to_screen.transform_pos(c.origin);

                    shapes.push(Shape::circle_stroke(
                        point_in_screen,
                        c.radius,
                        Stroke::new(2.0, constraint_color),
                    ));
                }
                ConstraintPath::Line(l) => {
                    let point_in_screen = to_screen.transform_pos(l.origin);
                    shapes.push(Shape::line_segment(
                        [
                            point_in_screen + l.direction * -5000.,
                            point_in_screen + l.direction * 5000.,
                        ],
                        Stroke::new(2.0, constraint_color),
                    ));
                }
                ConstraintPath::Ray(r) => {
                    let point_in_screen = to_screen.transform_pos(r.origin);

                    shapes.push(Shape::line_segment(
                        [point_in_screen, point_in_screen + r.direction * 5000.],
                        Stroke::new(2.0, constraint_color),
                    ));
                }
                // ConstraintPath::Point(p) => p.closest_point(point),
                _ => (),
            }
        }

        shapes
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
        let constr_weak = Rc::downgrade(self.constraint_manager.as_mut().unwrap());
        self.edges.insert(
            edge_handle,
            EdgeDisplay::new(weak, constr_weak, edge_handle),
        );
        println!("edge added");
    }

    pub fn add_constraint(&mut self, constraint_handle: ConstraintHandle) {
        let weak_dm = Rc::downgrade(self.drawing_manager.as_mut().unwrap());
        let weak_cm = Rc::downgrade(self.constraint_manager.as_mut().unwrap());
        self.constraints.insert(
            constraint_handle,
            ConstraintDisplay::new(weak_dm, weak_cm, constraint_handle),
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
            current_drag_position: Pos2::new(0., 0.),
        }
    }

    pub fn interact(
        &mut self,
        constraint_paths: &mut Vec<ConstraintPath>,
        ui: &Ui,
        response: &Response,
    ) {
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
                if !self.is_being_dragged && point_response.dragged() {
                    // set previous position before starting to drag
                    let dm_shared = self.drawing_manager.upgrade().unwrap();
                    let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                    self.pre_drag_position = dm_borrow
                        .get_vertex_mut(self.vertex_handle)
                        .unwrap()
                        .position;
                    self.current_drag_position = self.pre_drag_position;

                    self.is_being_dragged = true;
                    println!("drag start");
                }
            }
        }

        // drag ends
        // this is outside of the hovered() call so that it will
        // properly be called without being in the interact region
        if self.is_being_dragged && !point_response.dragged() {
            self.is_being_dragged = false;
            println!("drag end");
        }

        if self.is_being_dragged {
            if cursor_opt.is_some() {
                let cursor_pt = point_response.hover_pos().unwrap();
                self.current_drag_position = to_screen.inverse().transform_pos(cursor_pt);
            }

            let constr_shared = self.constraint_manager.upgrade().unwrap();
            let constr_borrow = constr_shared.as_ref().borrow_mut();

            let try_pt = self.current_drag_position;

            let solver_response = constr_borrow.solve_for_vertex(
                self.vertex_handle,
                &self.pre_drag_position,
                &try_pt,
                vec![],
            );

            if let Some(p) = solver_response.valid_path {
                constraint_paths.push(p.clone())
            }

            // get mutable vertex again, so we can modify it
            let dm_shared = self.drawing_manager.upgrade().unwrap();
            let mut dm_borrow = dm_shared.as_ref().borrow_mut();
            let vertex = dm_borrow.get_vertex_mut(self.vertex_handle).unwrap();

            match solver_response.state {
                SolverState::Free => {
                    //println!("Free");
                    vertex.position = try_pt
                }
                SolverState::Locked => {
                    //println!("Locked");
                    ()
                }
                SolverState::Partial => vertex.position = solver_response.new_pos.unwrap(),
            }
        }
    }

    pub fn get_shape(&self, response: &Response) -> Shape {
        let base_color = if self.is_selected || self.is_being_dragged {
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
    constraint_manager: Weak<RefCell<ConstraintManager>>,

    edge_handle: EdgeHandle,
    is_selected: bool,
    is_being_dragged: bool,
    is_hovered: bool,

    pre_drag_position: Pos2,
    current_drag_position: Pos2,

    pre_drag_start_point: Pos2,
    pre_drag_end_point: Pos2,
}

impl EdgeDisplay {
    pub fn new(
        drawing_manager: Weak<RefCell<DrawingManager>>,
        constraint_manager: Weak<RefCell<ConstraintManager>>,
        edge_handle: EdgeHandle,
    ) -> Self {
        Self {
            drawing_manager,
            constraint_manager,
            edge_handle,
            is_selected: false,
            is_being_dragged: false,
            is_hovered: false,
            pre_drag_position: Pos2::new(0., 0.),
            current_drag_position: Pos2::new(0., 0.),
            pre_drag_start_point: Pos2::new(0., 0.),
            pre_drag_end_point: Pos2::new(0., 0.),
        }
    }

    pub fn interact(
        &mut self,
        constraint_paths: &mut Vec<ConstraintPath>,
        ui: &Ui,
        response: &Response,
    ) {
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let end_points: Vec<Pos2> = self
            .get_end_points()
            .iter()
            .map(|p| to_screen.transform_pos(p.clone()))
            .collect();

        //let point_in_screen = to_screen.transform_pos(self.get_vertex_point());
        let point_rect = Rect::from_two_pos(end_points[0], end_points[1]).expand(15.);
        let point_id = response.id.with(self.edge_handle + 10000);
        let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());

        self.is_hovered = false;

        let cursor_opt = point_response.hover_pos();

        if point_response.hovered() && cursor_opt.is_some() {
            let cursor_pt = point_response.hover_pos().unwrap();
            let cursor_pt = to_screen.inverse().transform_pos(cursor_pt);
            //self.is_hovered = true;
            //let is_on_edge = true;
            let is_on_edge = self.is_point_on_edge(cursor_pt, 10.0);
            if is_on_edge {
                self.is_hovered = true;

                if point_response.clicked() {
                    self.is_selected = !self.is_selected;
                }

                // drag begins -- initiate drag parameters
                if !self.is_being_dragged && point_response.dragged() {
                    // set previous position before starting to drag
                    //let dm_shared = self.drawing_manager.upgrade().unwrap();
                    //let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                    self.pre_drag_position = cursor_pt;
                    self.current_drag_position = self.pre_drag_position;

                    let [edge_pt_1, edge_pt_2] = self.get_end_points();

                    self.pre_drag_start_point = edge_pt_1;
                    self.pre_drag_end_point = edge_pt_2;

                    self.is_being_dragged = true;
                    println!("drag start");
                }
            }
        }

        // drag ends
        // this is outside of the hovered() call so that it will
        // properly be called without being in the interact region
        if self.is_being_dragged && !point_response.dragged() {
            self.is_being_dragged = false;
            println!("drag end");
        }

        if self.is_being_dragged {
            if cursor_opt.is_some() {
                let cursor_pt = point_response.hover_pos().unwrap();
                self.current_drag_position = to_screen.inverse().transform_pos(cursor_pt);
            }
            let try_pt = self.current_drag_position;
            let delta = self.current_drag_position - self.pre_drag_position;

            let constr_shared = self.constraint_manager.upgrade().unwrap();
            let constr_borrow = constr_shared.as_ref().borrow_mut();

            let solver_response = constr_borrow.solve_for_edge(
                self.edge_handle,
                &self.pre_drag_position,
                &try_pt,
                &self.pre_drag_start_point,
                &(self.pre_drag_start_point + delta),
                &self.pre_drag_end_point,
                &(self.pre_drag_end_point + delta),
            );

            if let Some(p) = solver_response.valid_paths {
                constraint_paths.extend(p.clone());
            }

            // get mutable vertex again, so we can modify it
            let dm_shared = self.drawing_manager.upgrade().unwrap();

            #[allow(unused_assignments)]
            let mut eh_1 = 0;
            #[allow(unused_assignments)]
            let mut eh_2 = 0;
            {
                let dm_borrow = dm_shared.as_ref().borrow_mut();
                let edge = dm_borrow.get_edge(self.edge_handle).unwrap();
                eh_1 = edge.start_point_vh;
                eh_2 = edge.end_point_vh;
            }

            match solver_response.state {
                SolverState::Free => {
                    //println!("Free");
                    {
                        let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                        let vert_1 = dm_borrow.get_vertex_mut(eh_1).unwrap();
                        vert_1.position = self.pre_drag_start_point + delta;
                    }
                    {
                        let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                        let vert_2 = dm_borrow.get_vertex_mut(eh_2).unwrap();
                        vert_2.position = self.pre_drag_end_point + delta;
                    }
                }
                SolverState::Locked => {
                    //println!("Locked");
                    ()
                }
                //SolverState::Partial => vertex.position = solver_response.new_pos.unwrap(),
                SolverState::Partial => {
                    {
                        let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                        let vert_1 = dm_borrow.get_vertex_mut(eh_1).unwrap();
                        vert_1.position = solver_response.new_pos.unwrap()[0];
                    }
                    {
                        let mut dm_borrow = dm_shared.as_ref().borrow_mut();
                        let vert_2 = dm_borrow.get_vertex_mut(eh_2).unwrap();
                        vert_2.position = solver_response.new_pos.unwrap()[1];
                    }
                }
            }
        }
    }

    pub fn get_shape(&self, response: &Response) -> Shape {
        let base_color = if self.is_selected {
            Color32::LIGHT_BLUE.gamma_multiply(0.9)
        } else {
            Color32::LIGHT_BLUE.gamma_multiply(0.5)
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

    fn is_point_on_edge(&self, point: Pos2, thickness: f32) -> bool {
        let end_points = self.get_end_points();
        let line_vector = end_points[1] - end_points[0];
        let line_length_sq = line_vector.length_sq();

        if line_length_sq < 0.001 {
            let dist = point.distance(end_points[0]);
            return dist <= thickness / 2.0;
        }
        let t = ((point - end_points[0]).dot(line_vector) / line_length_sq).clamp(0.0, 1.0);
        let closest_point = end_points[0] + t * line_vector;

        (point - closest_point).length() <= thickness / 2.0
    }
}

pub struct ConstraintDisplay {
    drawing_manager: Weak<RefCell<DrawingManager>>,
    constraint_manager: Weak<RefCell<ConstraintManager>>,
    constraint_handle: ConstraintHandle,
}

impl ConstraintDisplay {
    pub fn new(
        drawing_manager: Weak<RefCell<DrawingManager>>,
        constraint_manager: Weak<RefCell<ConstraintManager>>,
        constraint_handle: ConstraintHandle,
    ) -> Self {
        Self {
            drawing_manager,
            constraint_manager,
            constraint_handle,
        }
    }

    pub fn get_shape(&self, response: &Response) -> Vec<Shape> {
        let constraint_manager_rc = self.constraint_manager.upgrade().unwrap();
        let constraint_manager = constraint_manager_rc.borrow_mut();

        let drawing_manager_rc = self.drawing_manager.upgrade().unwrap();
        let drawing_manager = drawing_manager_rc.borrow_mut();

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let constraint = constraint_manager
            .get_constraint(self.constraint_handle)
            .unwrap();

        match constraint {
            Constraint::ANGLE(a) => {
                let pos = drawing_manager
                    .get_vertex(a.pivot_vert_handle)
                    .unwrap()
                    .position;
                let pos = to_screen.transform_pos(pos);

                return vec![Shape::circle_stroke(
                    pos,
                    15.0,
                    Stroke::new(3.0, Color32::LIGHT_GREEN),
                )];
            }
            Constraint::LENGTH(l) => {
                let edge = drawing_manager.get_edge(l.edge_handle).unwrap();

                let start_pt = drawing_manager
                    .get_vertex(edge.start_point_vh)
                    .unwrap()
                    .position;

                let start_pt = to_screen.transform_pos(start_pt);

                let end_pt = drawing_manager
                    .get_vertex(edge.end_point_vh)
                    .unwrap()
                    .position;
                let end_pt = to_screen.transform_pos(end_pt);

                let main_dir = end_pt - start_pt;

                let perp_dir = main_dir.normalized();
                let perp_dir = rotate_vec2(perp_dir, f32::consts::FRAC_PI_2);

                let stroke = Stroke::new(3.0, Color32::LIGHT_GREEN);

                let peg_1 = Shape::line_segment([start_pt, start_pt + perp_dir * 32.0], stroke);
                let peg_2 = Shape::line_segment([end_pt, end_pt + perp_dir * 32.0], stroke);
                let line = Shape::line_segment(
                    [start_pt + perp_dir * 16.0, end_pt + perp_dir * 16.0],
                    stroke,
                );

                vec![peg_1, peg_2, line]
            }
            Constraint::PARALLEL(p) => {
                let edge_1 = drawing_manager.get_edge(p.edge_1_handle).unwrap();
                let edge_2 = drawing_manager.get_edge(p.edge_2_handle).unwrap();

                let get_pos_func = |vh: VertexHandle| -> Pos2 {
                    let pos = drawing_manager.get_vertex(vh).unwrap().position;
                    to_screen.transform_pos(pos)
                };

                let e1_v_1 = get_pos_func(edge_1.start_point_vh);
                let e1_v_2 = get_pos_func(edge_1.end_point_vh);

                let e2_v_1 = get_pos_func(edge_2.start_point_vh);
                let e2_v_2 = get_pos_func(edge_2.end_point_vh);

                let stroke = Stroke::new(3.0, Color32::LIGHT_GREEN);
                vec![
                    Shape::rect_stroke(
                        Rect::from_center_size(e1_v_1.lerp(e1_v_2, 0.5), Vec2::splat(10.0)),
                        3.0,
                        stroke,
                    ),
                    Shape::rect_stroke(
                        Rect::from_center_size(e2_v_1.lerp(e2_v_2, 0.5), Vec2::splat(10.0)),
                        3.0,
                        stroke,
                    ),
                ]
            }
        }
    }
}

fn rotate_vec2(vec: Vec2, angle: f32) -> Vec2 {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    Vec2 {
        x: vec.x * cos_angle - vec.y * sin_angle,
        y: vec.x * sin_angle + vec.y * cos_angle,
    }
}
