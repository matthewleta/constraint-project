use core::f32;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::drawing_manager::{DrawingManager, Edge};

use egui::{Pos2,Vec2};

use std::cell::RefCell;
use std::rc::{Rc, Weak};

type EdgeHandle = i32;
type VertexHandle = i32;
type ConstraintHandle = i32;

#[derive(Default)]
pub struct ConstraintManager {
    drawing_manager: Option<Rc<RefCell<DrawingManager>>>,
    constraint_map: BTreeMap<ConstraintHandle, Constraint>,
}

impl ConstraintManager {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn set_drawing_manager(&mut self, drawing_manager: Rc<RefCell<DrawingManager>>) {
        self.drawing_manager = Some(drawing_manager);
    }

    pub fn solve_for_vertex(&self, vh: VertexHandle, fixed_pos : &Pos2, try_pos : &Pos2) -> SolverResponse {
        let dm_shared = if let Some(v) = &self.drawing_manager {
            v
        } else {
            return SolverResponse::default();
        };

        let dm_borrow = dm_shared.borrow();

        let mut length_end_constraints: Vec<&LengthConstraint> = vec![];
        let mut angle_center_constraints: Vec<&AngleConstraint> = vec![];
        let mut angle_end_constraints: Vec<&AngleConstraint> = vec![];
        let mut parallel_end_constraints: Vec<&ParallelConstraint> = vec![];

        //find constraints associated with vertex
        for (ch, constraint) in &self.constraint_map {
            match constraint {
                Constraint::LENGTH(length_constraint) => {
                    let edge = dm_borrow.get_edge(length_constraint.edge_handle).unwrap();

                    if edge.end_point_vh == vh || edge.start_point_vh == vh {
                        length_end_constraints.push(length_constraint);
                    }
                }
                Constraint::ANGLE(angle_constraint) => {
                    if angle_constraint.pivot_vert_handle == vh {
                        angle_center_constraints.push(angle_constraint);
                    } else if angle_constraint.edge_1_outer_vert_handle == vh
                        || angle_constraint.edge_2_outer_vert_handle == vh
                    {
                        angle_end_constraints.push(angle_constraint);
                    }
                }
                Constraint::PARALLEL(parallel_constraint) => {
                    let edge_1 = dm_borrow
                        .get_edge(parallel_constraint.edge_1_handle)
                        .unwrap();
                    let edge_2 = dm_borrow
                        .get_edge(parallel_constraint.edge_2_handle)
                        .unwrap();

                    if edge_1.end_point_vh == vh
                        || edge_1.start_point_vh == vh
                        || edge_2.end_point_vh == vh
                        || edge_2.start_point_vh == vh
                    {
                        parallel_end_constraints.push(parallel_constraint);
                    }
                }
            }
        }

        // now that all constraints are found that are associated,

        // 1 - individual archetype lock cases (full lock regardless of other constraints)

        //If angle-center archetype, return Locked if two arms are not 0 or 180 degrees
        if !angle_center_constraints.is_empty() {
            for acc in angle_center_constraints {
                let dir_1 = Edge::direction_from_handle(&dm_borrow, acc.edge_1_handle);
                let dir_2 = Edge::direction_from_handle(&dm_borrow, acc.edge_2_handle);

                let delta = dir_2 - dir_1;
                let angle = delta.y.atan2(delta.x); // [-pi to pi]

                if angle.abs() > 0.001 || //effectively 0 degs
                (f32::consts::PI - angle).abs() > 0.001
                // effectively 180 degs
                {
                    return SolverResponse {
                        state: SolverState::Locked,
                        new_pos: None,
                    };
                }
            }
        }

        // 2 - analytical intersections
        let mut constraint_paths: Vec<ConstraintPath> = vec![];

        // 2a - length path (circle)

        //let current_vertex = dm_borrow.get_vertex(vh).unwrap();

        for lc in length_end_constraints {
            let edge = dm_borrow.get_edge(lc.edge_handle).unwrap();

            let other_vh = if edge.start_point_vh == vh {
                edge.end_point_vh
            } else {
                edge.start_point_vh
            };

            let other_vertex = dm_borrow.get_vertex(other_vh).unwrap();
            let path_radius = fixed_pos.distance(other_vertex.position);

            let path = Circle {
                center: other_vertex.position,
                radius: path_radius,
            };

            constraint_paths.push(ConstraintPath::Circle(path));
        }
        // 3 calculate possible paths

        if !constraint_paths.is_empty(){
            let adjusted_pt = constraint_paths[0].closest_point(try_pos);

            return SolverResponse{state : SolverState::Partial, new_pos : Some(adjusted_pt )};
        }

        // if none passed before, return free solverResponse
        SolverResponse{state : SolverState::Free, new_pos : Some(try_pos.clone())}
    }

    // TODO add solver check for collision on existing constraints
    pub fn add_length_constraint(
        &mut self,
        eh: EdgeHandle,
    ) -> Result<ConstraintHandle, ConstraintError> {
        let dm_shared = self.drawing_manager.as_ref().unwrap();
        let dm_borrowed = dm_shared.borrow();

        if !dm_borrowed.has_edge(&eh) {
            return Err(ConstraintError::ConstraintNotAdded);
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
    ) -> Result<ConstraintHandle, ConstraintError> {
        let dm_shared = self.drawing_manager.as_ref().unwrap();
        let dm_borrowed = dm_shared.borrow();

        if !dm_borrowed.has_edge(&eh_1) {
            return Err(ConstraintError::ConstraintNotAdded);
        }
        if !dm_borrowed.has_edge(&eh_2) {
            return Err(ConstraintError::ConstraintNotAdded);
        }

        let edge_1 = dm_borrowed.get_edge(eh_1).unwrap();
        let edge_2 = dm_borrowed.get_edge(eh_2).unwrap();

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
    ) -> Result<ConstraintHandle, ConstraintError> {
        let dm_shared = self.drawing_manager.as_ref().unwrap();
        let dm_borrowed = dm_shared.borrow();

        if !dm_borrowed.has_edge(&edge_1_handle) {
            return Err(ConstraintError::ConstraintNotAdded);
        }
        if !dm_borrowed.has_edge(&edge_2_handle) {
            return Err(ConstraintError::ConstraintNotAdded);
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

pub struct Circle {
    center: Pos2,
    radius: f32,
}

impl Circle{
    pub fn closest_point(&self, point : &Pos2) -> Pos2{
        let mut dir = point.clone() - self.center;

        if dir.length() < 0.001{
            // just get the up direction since all points on the circle would be closest
            dir = Vec2::UP;
        }
        
        let dir = dir.normalized();
        println!("dir from center to circle {}", dir);

        self.center + dir  * self.radius
    }
}

pub struct Line;

pub struct Ray;

pub struct Point;

pub enum ConstraintPath {
    Circle(Circle),
    Line(Line),
    Ray(Ray),
    Point(Point),
}

impl ConstraintPath{
    pub fn closest_point(&self, point : &Pos2) -> Pos2{
        match self {
            ConstraintPath::Circle(v) => v.closest_point(point),
            _ => {Pos2::new(1., 1.)}
        }
    }
}

pub enum SolverState {
    Locked,
    Partial,
    Free,
}
impl Default for SolverState {
    fn default() -> Self {
        SolverState::Free
    }
}

#[derive(Default)]
pub struct SolverResponse {
    pub state: SolverState,
    pub new_pos: Option<Pos2>,
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
pub enum ConstraintError {
    #[error("Constraint could not be added")]
    ConstraintNotAdded,
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
) -> Result<(VertexHandle, (VertexHandle, VertexHandle)), ConstraintError> {
    // Check for degenerate edges
    if e_1_vh_1 == e_1_vh_2 || e_2_vh_1 == e_2_vh_2 {
        return Err(ConstraintError::DegenerateEdge);
    }

    // Check for shared vertices and unmatched vertices
    match (
        (e_1_vh_1 == e_2_vh_1, e_1_vh_1 == e_2_vh_2),
        (e_1_vh_2 == e_2_vh_1, e_1_vh_2 == e_2_vh_2),
    ) {
        // Both vertices of each edge match, indicating full overlap
        ((true, true), _) | (_, (true, true)) => Err(ConstraintError::FullOverlap),

        // Single shared vertex with unmatched vertices
        ((true, false), (false, false)) => Ok((e_1_vh_1, (e_1_vh_2, e_2_vh_2))),
        ((false, true), (false, false)) => Ok((e_1_vh_1, (e_1_vh_2, e_2_vh_1))),
        ((false, false), (true, false)) => Ok((e_1_vh_2, (e_1_vh_1, e_2_vh_2))),
        ((false, false), (false, true)) => Ok((e_1_vh_2, (e_1_vh_1, e_2_vh_1))),

        // No shared vertices
        ((false, false), (false, false)) => Err(ConstraintError::NoSharedVertex),

        // Handle cases that don’t match any above cases as safe errors
        _ => Err(ConstraintError::NoSharedVertex),
    }
}
