use egui::epaint::{PathShape, PathStroke};
use egui::{Frame, Pos2, Vec2, Rect, emath, Sense, Color32, Stroke, Shape};
use std::borrow::Borrow;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Default)]
pub struct CanvasView{
    network : DrawingNetwork
}

impl CanvasView{
    pub fn update(&mut self, ui:&mut egui::Ui){
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
            let control_point_shapes : Vec<Shape> = self.network.vertices
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {

                let mut pt_data = point.borrow_mut().data;

                let size = Vec2::splat(2.0 * control_point_radius);
                
                let point_in_screen = to_screen.transform_pos(pt_data);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                pt_data += point_response.drag_delta();

                point.borrow_mut().data = to_screen.from().clamp(pt_data);

                let point_in_screen = to_screen.transform_pos(pt_data);
                let stroke = ui.style().interact(&point_response).fg_stroke;
                let stroke2 = Stroke::new(10., Color32::LIGHT_RED);
                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

            // Draw Lines from edges
            let edge_shapes : Vec<Shape> = self.network.edges
            //let control_point_shapes: Vec<Shape> = self
            .iter_mut()
            .enumerate()
            .map(|(i, edge)| {

                let mut start_point = edge.borrow_mut().start_point();
                let mut end_point = edge.borrow_mut().end_point();
                
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

//#[derive(Default)]
pub struct DrawingNetwork{
    vertices : Vec<Rc<RefCell<Vertex>>>,
    edges : Vec<Rc<RefCell<Edge>>>
    
}

// temporary default for testing
impl Default for DrawingNetwork{
    fn default() -> Self {
            let vertices = vec![
                Rc::new(RefCell::new(Vertex{ data: Pos2::new(10., 10.)})),
                Rc::new(RefCell::new(Vertex{ data: Pos2::new(10., 40.)})),
                Rc::new(RefCell::new(Vertex{ data: Pos2::new(40., 40.)})),
                Rc::new(RefCell::new(Vertex{ data: Pos2::new(40., 10.)}))
            ];

            let edges = vec![
                Edge::from_verts(Rc::clone(&vertices[0]), Rc::clone(&vertices[1])),
                Edge::from_verts(Rc::clone(&vertices[1]), Rc::clone(&vertices[2])),
                Edge::from_verts(Rc::clone(&vertices[2]), Rc::clone(&vertices[3])),
                ];

        Self{vertices, edges}
    }
}

impl DrawingNetwork{
    pub fn new() -> Self{
        Self{
        edges : vec![],
        vertices : vec![]
    }
    }
}

// Edge will have direct ownership over its endpoints
pub struct Edge{
    end_point_1: Rc<RefCell<EndPoint>>,
    end_point_2: Rc<RefCell<EndPoint>>,

    is_selected: bool
}

impl Edge{
    pub fn from_verts(vert_1 : Rc<RefCell<Vertex>>, vert_2 : Rc<RefCell<Vertex>>) -> Rc<RefCell<Self>>{

        // create end_point to populate later with weak ref to Edge
        let ep1 = Rc::new(RefCell::new(EndPoint{
            edge : Weak::new() , vertex : vert_1
        }));
        let ep2 = Rc::new(RefCell::new(EndPoint{
            edge : Weak::new() , vertex : vert_2
        }));
        
        let edge = Rc::new(RefCell::new(Edge{ end_point_1 : ep1, end_point_2 : ep2, is_selected : false}));

        (*edge).borrow_mut().end_point_1.borrow_mut().edge = Rc::downgrade(&edge);
        (*edge).borrow_mut().end_point_1.borrow_mut().edge = Rc::downgrade(&edge);

        edge
    }

    pub fn start_point(&self) -> Pos2{
        self.end_point_1.borrow_mut().vertex.borrow_mut().data.clone()
    }
    pub fn end_point(&self) -> Pos2{
        self.end_point_2.borrow_mut().vertex.borrow_mut().data.clone()
    }
}

pub struct EndPoint{
edge : Weak<RefCell<Edge>>,
vertex : Rc<RefCell<Vertex>>
}

impl EndPoint{
    pub fn new(edge : Weak<RefCell<Edge>>, vertex:Rc<RefCell<Vertex>>) -> Self{
        EndPoint{edge,vertex}
    }
}

#[derive(Default)]
pub struct Vertex{
    data: Pos2,
    //end_points: Vec<Weak<EndPoint>>
}