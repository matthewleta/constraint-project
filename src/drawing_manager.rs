use std::collections::BTreeMap;
use thiserror::Error;

use egui::{Pos2, Vec2};

use crate::display_manager::DisplayManager;
use std::cell::RefCell;
use std::rc::Rc;

// BTreeMap used because the highest key value is being queried
// to get the next key and this structure maintains order
#[derive(Default)]
pub struct DrawingManager {
    display_manager: Option<Rc<RefCell<DisplayManager>>>,

    edge_map: BTreeMap<EdgeHandle, Edge>,
    vertex_map: BTreeMap<VertexHandle, Vertex>,
}

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;

impl DrawingManager {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn set_display_manager(&mut self, display_manager : Rc<RefCell<DisplayManager>>){
        self.display_manager = Some(display_manager);
    }

    pub fn has_edge(&self, eh: &EdgeHandle) -> bool{
        self.edge_map.contains_key(&eh)
    }

    pub fn get_all_edges(&self) -> Vec<&Edge> {
        self.edge_map.values().collect()
    }

    pub fn get_edge(&self, eh: EdgeHandle) -> Result<&Edge, DrawingManagerError> {
        self.edge_map
            .get(&eh)
            .ok_or(DrawingManagerError::EdgeNotFound(eh))
    }

    pub fn get_edge_mut(&mut self, eh: EdgeHandle) -> Result<&mut Edge, DrawingManagerError> {
        self.edge_map
            .get_mut(&eh)
            .ok_or(DrawingManagerError::EdgeNotFound(eh))
    }
    pub fn add_edge(
        &mut self,
        vh_1: VertexHandle,
        vh_2: VertexHandle,
    ) -> Result<EdgeHandle, DrawingManagerError> {
        if !self.vertex_map.contains_key(&vh_1) {
            return Err(DrawingManagerError::VertexNotFound(vh_1));
        }
        if !self.vertex_map.contains_key(&vh_2) {
            return Err(DrawingManagerError::VertexNotFound(vh_2));
        }

        // get next Id to use
        let next_id = get_next_id(&self.edge_map);

        self.edge_map.insert(next_id, Edge::new(vh_1, vh_2));

        // add edge reference to vertices
        // let panic to keep simple for maru
        let vertex_1 = self.get_vertex_mut(vh_1).unwrap();
        vertex_1.edge_handles.push(next_id);

        let vertex_2 = self.get_vertex_mut(vh_2).unwrap();
        vertex_2.edge_handles.push(next_id);

        Ok(next_id)
    }

    pub fn has_vertex(&self, vh: &VertexHandle) -> bool{
        self.vertex_map.contains_key(&vh)
    }
    pub fn get_all_vertices_mut(&mut self) -> Vec<&mut Vertex> {
        self.vertex_map.values_mut().collect()
    }

    pub fn get_vertex(&self, vh: VertexHandle) -> Result<&Vertex, DrawingManagerError> {
        self.vertex_map
            .get(&vh)
            .ok_or(DrawingManagerError::VertexNotFound(vh))
    }

    pub fn get_vertex_mut(&mut self, vh: VertexHandle) -> Result<&mut Vertex, DrawingManagerError> {
        self.vertex_map
            .get_mut(&vh)
            .ok_or(DrawingManagerError::VertexNotFound(vh))
    }

    pub fn add_vertex(&mut self, position: Pos2) -> VertexHandle {
        // get next Id to use
        let next_id = get_next_id(&self.vertex_map);
        let vert = Vertex::new(position);
        self.vertex_map.insert(next_id, vert);
        next_id
    }
}

pub struct Edge {
    pub start_point_vh: VertexHandle,
    pub end_point_vh: VertexHandle,
    pub constraints: Vec<ConstraintHandle>
}
impl Edge {
    pub fn new(start_point_vh: VertexHandle, end_point_vh: VertexHandle) -> Self {
        Self {
            start_point_vh,
            end_point_vh,
            constraints : vec![]
        }
    }
    pub fn direction_from_edge( drawing_manager : &DrawingManager, edge : &Edge) -> Vec2{

        let pos_1 = drawing_manager.vertex_map.get(&edge.start_point_vh).unwrap().position;
        let pos_2 = drawing_manager.vertex_map.get(&edge.end_point_vh).unwrap().position;

        let dir = pos_2 - pos_1;
        if dir.length() < 0.001{
            panic!()
        }

        dir.normalized()
    }
    pub fn direction_from_handle( drawing_manager : &DrawingManager, edge_handle : EdgeHandle) -> Vec2{
        let edge = drawing_manager.edge_map.get(&edge_handle).unwrap();
        
        Edge::direction_from_edge(drawing_manager, edge)
    }
}
pub struct Vertex {
    pub position: Pos2,
    pub edge_handles: Vec<EdgeHandle>,
}

impl Vertex {
    pub fn new(position: Pos2) -> Self {
        Self {
            position,
            edge_handles: vec![],
        }
    }
}

//Utilities

fn get_next_id<V>(map: &BTreeMap<i32, V>) -> i32 {
    map.last_key_value().map_or(0, |(k, _)| k + 1)
}

#[derive(Debug, Error)]
pub enum DrawingManagerError {
    #[error("Edge could not be added")]
    EdgeNotAdded,
    #[error("Vertex could not be added")]
    VertexNotAdded,
    #[error("Constraint could not be added")]
    ConstraintNotAdded,
    #[error("Edge {0} not found")]
    EdgeNotFound(EdgeHandle),
    #[error("Vertex {0} not found")]
    VertexNotFound(VertexHandle),
    #[error("Constraint {0} not found")]
    ConstraintNotFound(ConstraintHandle),
    #[error("No shared vertex found")]
    NoSharedVertex,
    #[error("Degenerate edge detected")]
    DegenerateEdge,
    #[error("Full overlap detected")]
    FullOverlap,
}