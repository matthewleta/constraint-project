use crate::drawing_manager::DrawingManager;
use std::collections::HashMap;

use egui::{emath, Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Painter};

use std::cell::RefCell;
use std::rc::{Rc, Weak};

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;

#[derive(Default)]
pub struct DisplayManager {
    drawing_manager: Option<Rc<RefCell<DrawingManager>>>,

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

    pub fn update_interaction(&mut self, ui: &Ui, response: &Response) {
        self.vertices.iter_mut().for_each(|v| {
            v.1.interact(ui, response);
        });
    }

    pub fn draw(&self, ui: &mut Ui, response: &Response, painter : &Painter) {
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
        let weak = Rc::downgrade(self.drawing_manager.as_mut().unwrap());
        self.vertices
            .insert(vertex_handle, VertexDisplay::new(weak, vertex_handle));
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
    vertex_handle: VertexHandle,
    is_selected: bool,
    is_hovered: bool,
}

impl VertexDisplay {
    pub fn new(
        drawing_manager: Weak<RefCell<DrawingManager>>,
        vertex_handle: VertexHandle,
    ) -> Self {
        Self {
            drawing_manager,
            vertex_handle,
            is_selected: false,
            is_hovered: false,
        }
    }

    pub fn interact(&mut self, ui: &Ui, response: &Response) {
        let mut pt_data = self.get_point();

        let buffer_size = Vec2::splat(30.0);

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let point_in_screen = to_screen.transform_pos(pt_data);
        let point_rect = Rect::from_center_size(point_in_screen, buffer_size);
        let point_id = response.id.with(self.vertex_handle);
        let point_response = ui.interact(point_rect, point_id, Sense::hover());

        self.is_hovered = false;
        self.is_selected = false;

        if point_response.hovered() {
            let cursor_pt = point_response.hover_pos().unwrap();          
            let cursor_pt = to_screen.inverse().transform_pos(cursor_pt);

            if self.is_point_on_vertex(cursor_pt, 10.0) {
                println!("in the zone");
                self.is_hovered = true;

                if point_response.dragged() {
                    self.is_selected = true;
                }
            } else {
                println!("not in the zone");
            }
        }

        // point_response.interact_pointer_pos()

        // ui.input(|i| i.pointer.hover_pos())

        // pt_data += point_response.drag_delta();

        // // update point
        // vertex.position = to_screen.from().clamp(pt_data);

        // let point_in_screen = to_screen.transform_pos(pt_data);
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

        let point_in_screen = to_screen.transform_pos(self.get_point());
        //let stroke = Stroke::new(2., current_color);

        let control_point_radius = 10.0;

        Shape::circle_filled(point_in_screen, control_point_radius, current_color)
    }
    fn get_point(&self) -> Pos2 {
        let drawing_manager_rc = self.drawing_manager.upgrade().unwrap();
        let drawing_manager = drawing_manager_rc.borrow_mut();

        drawing_manager
            .get_vertex(self.vertex_handle)
            .unwrap()
            .position
    }
    fn is_point_on_vertex(&self, point: Pos2, radius: f32) -> bool {
        let dist = point.distance(self.get_point());
        println!("{}",dist);
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
