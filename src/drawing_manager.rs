use std::collections::BTreeMap;
use thiserror::Error;

use egui::Pos2;

// BTreeMap used because the highest key value is being queried
// to get the next key and this structure maintains order
#[derive(Default)]
pub struct DrawingManager {
    edge_map: BTreeMap<EdgeHandle, Edge>,
    vertex_map: BTreeMap<VertexHandle, Vertex>,
    constraint_map: BTreeMap<ConstraintHandle, Constraint>,
}

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;

impl DrawingManager {
    pub fn new() -> Self {
        Self {
            edge_map: BTreeMap::new(),
            vertex_map: BTreeMap::new(),
            constraint_map: BTreeMap::new(),
        }
    }

    pub fn get_all_edges(&self) -> Vec<&Edge> {
        self.edge_map.values().collect()
    }

    pub fn get_edge(&self, eh: EdgeHandle) -> Result<&Edge, DrawingManagerError> {
        self.edge_map
            .get(&eh)
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

    // TODO add solver check for collision on existing constraints
    pub fn add_length_constraint(
        &mut self,
        eh: EdgeHandle,
    ) -> Result<ConstraintHandle, DrawingManagerError> {
        if !self.edge_map.contains_key(&eh) {
            return Err(DrawingManagerError::EdgeNotFound(eh));
        }

        let length_constraint = LengthConstraint { edge_handle: eh };

        let next_id = get_next_id(&self.constraint_map);

        self.constraint_map
            .insert(next_id, Constraint::LENGTH(length_constraint));

        Ok(next_id)
    }

    // TODO add solver check for collision on existing constraints
    pub fn add_angle_constraint(
        &mut self,
        eh_1: EdgeHandle,
        eh_2: EdgeHandle,
    ) -> Result<ConstraintHandle, DrawingManagerError> {
        if !self.edge_map.contains_key(&eh_1) {
            return Err(DrawingManagerError::EdgeNotFound(eh_1));
        }
        if !self.edge_map.contains_key(&eh_2) {
            return Err(DrawingManagerError::EdgeNotFound(eh_2));
        }

        let edge_1 = self.get_edge(eh_1).unwrap();
        let edge_2 = self.get_edge(eh_2).unwrap();

        let verts = find_shared_and_unmatched_vertices(
            edge_1.start_point_vh,
            edge_1.end_point_vh,
            edge_2.start_point_vh,
            edge_2.end_point_vh,
        )?; // forward error

        let angle_constraint = AngleConstraint {
            pivot_vert_handle: verts.0,
            edge_1_handle: eh_1,
            edge_1_outer_vert_handle: verts.1 .0,
            edge_2_handle: eh_2,
            edge_2_outer_vert_handle: verts.1 .1,
        };

        let next_id = get_next_id(&self.constraint_map);
        self.constraint_map
            .insert(next_id, Constraint::ANGLE(angle_constraint));

        Ok(next_id)
    }

    // TODO add solver check for collision on existing constraints
    pub fn add_parallel_constraint(
        &mut self,
        edge_1_handle: EdgeHandle,
        edge_2_handle: EdgeHandle,
    ) -> Result<ConstraintHandle, DrawingManagerError> {
        if !self.edge_map.contains_key(&edge_1_handle) {
            return Err(DrawingManagerError::EdgeNotFound(edge_1_handle));
        }
        if !self.edge_map.contains_key(&edge_2_handle) {
            return Err(DrawingManagerError::EdgeNotFound(edge_2_handle));
        }

        // get next Id to use
        let parallel_constraint = ParallelConstraint {
            edge_1_handle,
            edge_2_handle,
        };

        let next_id = get_next_id(&self.constraint_map);

        self.constraint_map
            .insert(next_id, Constraint::PARALLEL(parallel_constraint));
        Ok(next_id)
    }
}

pub struct Edge {
    pub start_point_vh: VertexHandle,
    pub end_point_vh: VertexHandle,
}
impl Edge {
    pub fn new(start_point_vh: VertexHandle, end_point_vh: VertexHandle) -> Self {
        Self {
            start_point_vh,
            end_point_vh,
        }
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

enum Constraint {
    LENGTH(LengthConstraint),
    ANGLE(AngleConstraint),
    PARALLEL(ParallelConstraint),
}

impl Constraint {
    fn try_get_length(&self) -> Option<&LengthConstraint> {
        match self {
            Constraint::LENGTH(constraint) => Some(constraint),
            _ => None,
        }
    }

    fn try_get_angle(&self) -> Option<&AngleConstraint> {
        match self {
            Constraint::ANGLE(constraint) => Some(constraint),
            _ => None,
        }
    }

    fn try_get_parallel(&self) -> Option<&ParallelConstraint> {
        match self {
            Constraint::PARALLEL(constraint) => Some(constraint),
            _ => None,
        }
    }
}

// Length Constraint is primarily around an edge only
pub struct LengthConstraint {
    edge_handle: EdgeHandle,
}
// Angle is relative to edge_1_handle counterclockwise
// pivot_vert_handle must refer to a vertex that both edges share
pub struct AngleConstraint {
    pivot_vert_handle: VertexHandle,
    edge_1_handle: EdgeHandle,
    edge_1_outer_vert_handle: VertexHandle,
    edge_2_handle: EdgeHandle,
    edge_2_outer_vert_handle: VertexHandle,
}

// Parallel constraint between two edges
// order does not matter here
pub struct ParallelConstraint {
    edge_1_handle: EdgeHandle,
    edge_2_handle: EdgeHandle,
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

// generated with chatGPT
fn find_shared_and_unmatched_vertices(
    e_1_vh_1: VertexHandle,
    e_1_vh_2: VertexHandle,
    e_2_vh_1: VertexHandle,
    e_2_vh_2: VertexHandle,
) -> Result<(VertexHandle, (VertexHandle, VertexHandle)), DrawingManagerError> {
    // Check for degenerate edges
    if e_1_vh_1 == e_1_vh_2 || e_2_vh_1 == e_2_vh_2 {
        return Err(DrawingManagerError::DegenerateEdge);
    }

    // Check for shared vertices and unmatched vertices
    match (
        (e_1_vh_1 == e_2_vh_1, e_1_vh_1 == e_2_vh_2),
        (e_1_vh_2 == e_2_vh_1, e_1_vh_2 == e_2_vh_2),
    ) {
        // Both vertices of each edge match, indicating full overlap
        ((true, true), _) | (_, (true, true)) => Err(DrawingManagerError::FullOverlap),

        // Single shared vertex with unmatched vertices
        ((true, false), (false, false)) => Ok((e_1_vh_1, (e_1_vh_2, e_2_vh_2))),
        ((false, true), (false, false)) => Ok((e_1_vh_1, (e_1_vh_2, e_2_vh_1))),
        ((false, false), (true, false)) => Ok((e_1_vh_2, (e_1_vh_1, e_2_vh_2))),
        ((false, false), (false, true)) => Ok((e_1_vh_2, (e_1_vh_1, e_2_vh_1))),

        // No shared vertices
        ((false, false), (false, false)) => Err(DrawingManagerError::NoSharedVertex),

        // Handle cases that don’t match any above cases as safe errors
        _ => Err(DrawingManagerError::NoSharedVertex),
    }
}
