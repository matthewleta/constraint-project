use core::f32;
use std::collections::BTreeMap;
use std::iter::Enumerate;
use thiserror::Error;

use crate::drawing_manager::{DrawingManager, Edge};

use egui::{Pos2, Vec2};

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

    pub fn get_constraint(&self, ch: ConstraintHandle) -> Result<&Constraint, ConstraintError> {
        self.constraint_map
            .get(&ch)
            .ok_or(ConstraintError::ConstraintNotFound(ch))
    }

    pub fn solve_for_edge(
        &self,
        eh: EdgeHandle,
        fixed_pos: &Pos2,
        try_pos: &Pos2,
        v1_fixed_pos: &Pos2,
        v1_try_pos: &Pos2,
        v2_fixed_pos: &Pos2,
        v2_try_pos: &Pos2,
    ) -> EdgeSolverResponse {
        let dm_shared = if let Some(v) = &self.drawing_manager {
            v
        } else {
            return EdgeSolverResponse::default();
        };
        
        let vh_1 = dm_shared.borrow().get_edge(eh).unwrap().start_point_vh;
        let vh_2 = dm_shared.borrow().get_edge(eh).unwrap().end_point_vh;

        // solve endpoint vertices to get valid paths

        let vert_response_1 = self.solve_for_vertex(vh_1, v1_fixed_pos, v1_try_pos);

        let vert_response_2 = self.solve_for_vertex(vh_2, v2_fixed_pos, v2_try_pos);

        // exit early if either are locked
        if let SolverState::Locked = vert_response_1.state { return EdgeSolverResponse::locked();}
        if let SolverState::Locked = vert_response_2.state { return EdgeSolverResponse::locked();}

        // exit early if both are free
        let mut free_1 = false;
        let mut free_2 = false;
        if let SolverState::Free = vert_response_1.state { free_1 = true}
        if let SolverState::Free = vert_response_2.state { free_2 = true}

        if free_1 && free_2{
            return EdgeSolverResponse::default();
        }

        // create Line from edge
        let line_pt_1 = dm_shared.borrow().get_vertex(vh_1).unwrap().position;
        let line_pt_2 = dm_shared.borrow().get_vertex(vh_2).unwrap().position;

        let edge_line = Line{origin : try_pos.clone(), direction : (line_pt_2 - line_pt_1).normalized()};

        // we have valid paths, now do an intersection of the line created by the edge

        let intersect_func = |constraint_path : &ConstraintPath| -> Option<Pos2> {
            match constraint_path {
                ConstraintPath::Line(l2) => {
                    let adjusted_origin_1 = edge_line.origin + - edge_line.direction * 500.0;
                    let adjusted_origin_2 = l2.origin + -l2.direction * 500.0;
    
                    let inter_result = ray_ray_intersection(
                        &Ray {
                            origin: adjusted_origin_1,
                            direction: edge_line.direction,
                        },
                        &Ray {
                            origin: adjusted_origin_2,
                            direction: l2.direction,
                        },
                    );
    
                    if let Some(inter) = inter_result {
                        Some(inter.origin)
                    } else {
                        None
                    }
                }
                ConstraintPath::Ray(r) => {
                    // Handle Line-Ray or Ray-Line case
                    let adjusted_origin = edge_line.origin + -edge_line.direction * 500.0;
                    let inter_result = ray_ray_intersection(
                        &Ray {
                            origin: adjusted_origin,
                            direction: edge_line.direction,
                        },
                        r,
                    );
    
                    if let Some(inter) = inter_result {
                        Some(inter.origin)
                    } else {
                        None
                    }
                }
                ConstraintPath::Circle(c) => {
                    let adjusted_origin = edge_line.origin + -edge_line.direction * 500.0;
                    let valid_points = ray_circle_intersection(
                        &Ray {
                            origin: adjusted_origin,
                            direction: edge_line.direction,
                        },
                        c,
                    );
                    if valid_points.is_empty(){
                        None
                    }else{
                        Some(valid_points[0].origin)
                    }

                }
                _ => None
            }
    
        };

        let mut new_pt_1 = Pos2::default();
        let mut new_pt_2 = Pos2::default();

        if free_1 {
            new_pt_1 = v1_try_pos.clone();
            // if vert_response_2.valid_path.is_none(){
            //     // has no valid path, so the vert is locked, making the edge locked
            //     return EdgeSolverResponse::locked();
            // }
            let v_path = vert_response_2.valid_path.unwrap();
            let inter_2 = intersect_func(&v_path);

            if inter_2.is_some(){
                new_pt_2 = inter_2.unwrap();
            }
        }
        else if free_2 {
            new_pt_2 = v2_try_pos.clone();
            // if vert_response_1.valid_path.is_none(){
            //     // has no valid path, so the vert is locked, making the edge locked
            //     return EdgeSolverResponse::locked();
            // }
            let v_path = vert_response_1.valid_path.unwrap();
            let inter_1 = intersect_func(&v_path);

            if inter_1.is_some(){
                new_pt_1 = inter_1.unwrap();
            }
        }
        else{
            let v_path_1 = vert_response_1.valid_path.unwrap();
            let inter_1 = intersect_func(&v_path_1);

            if inter_1.is_none(){
                return EdgeSolverResponse::locked();
            }

            let v_path_2 = vert_response_2.valid_path.unwrap();
            let inter_2 = intersect_func(&v_path_2);

            if inter_2.is_none(){
                return EdgeSolverResponse::locked();
            }
            new_pt_1 = inter_1.unwrap();
            new_pt_2 = inter_2.unwrap();
        }
        println!("pt1: {} pt2: {}",new_pt_1,new_pt_2);
        EdgeSolverResponse{state : SolverState::Partial, new_pos : Some([new_pt_1, new_pt_2])}
        
    }

    pub fn solve_for_vertex(
        &self,
        vh: VertexHandle,
        fixed_pos: &Pos2,
        try_pos: &Pos2,
    ) -> SolverResponse {
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

        if length_end_constraints.is_empty()
            && angle_center_constraints.is_empty()
            && angle_end_constraints.is_empty()
            && parallel_end_constraints.is_empty()
        {
            return SolverResponse {
                state: SolverState::Free,
                valid_path: None,
                new_pos: Some(try_pos.clone()),
            };
        }

        // now that all constraints are found that are associated,

        // 1 - individual archetype lock cases (full lock regardless of other constraints)

        //If angle-center archetype, return Locked if two arms are not 0 or 180 degrees
        if !angle_center_constraints.is_empty() {
            for acc in &angle_center_constraints {
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
                        valid_path: None,
                        new_pos: None,
                    };
                }
            }
        }

        // 2 - analytical intersections
        let mut constraint_paths: Vec<ConstraintPath> = vec![];

        // 2a - length path (circle)

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
                origin: other_vertex.position,
                radius: path_radius,
            };

            constraint_paths.push(ConstraintPath::Circle(path));
        }

        let line_data_generator = |center_vh: VertexHandle, end_vh: VertexHandle| {
            let end_vertex = dm_borrow.get_vertex(end_vh).unwrap();
            let center_vertex = dm_borrow.get_vertex(center_vh).unwrap();

            let line_dir = end_vertex.position - center_vertex.position;

            if line_dir.length() < 0.01 {
                panic!()
            }

            let line_dir = line_dir.normalized();
            (center_vertex.position + line_dir * 10.0, line_dir)
        };

        // 2b - angle path (ray)
        for ac in angle_end_constraints {
            // we assume here that one of the two is guaranteed
            // since we only pushed to angle_end_constraints if it was one of them
            let (origin, direction) = line_data_generator(ac.pivot_vert_handle, vh);
            constraint_paths.push(ConstraintPath::Ray(Ray { origin, direction }));
        }

        // 2c - angle path, but only when it's 180 degrees (line)

        for ac in angle_center_constraints {
            // we assume here that the a endpoint_1 and the pivot point
            // are on the same line as endpoint_2 and the pivot point since we
            // force any other condition to return locked earlier
            let (origin, direction) = line_data_generator(ac.pivot_vert_handle, vh);

            constraint_paths.push(ConstraintPath::Line(Line { origin, direction }));
        }

        // 2d - Parallel path (Line)

        for pc in parallel_end_constraints {
            
            let edge_1 = dm_borrow.get_edge(pc.edge_1_handle).unwrap();
            let edge_2 = dm_borrow.get_edge(pc.edge_2_handle).unwrap();

            if edge_1.start_point_vh == vh || edge_1.end_point_vh == vh{
                let (origin, direction) = line_data_generator(edge_1.start_point_vh, edge_1.end_point_vh);
                constraint_paths.push(ConstraintPath::Line(Line { origin, direction }));
            }else{
                let (origin, direction) = line_data_generator(edge_2.start_point_vh, edge_2.end_point_vh);
                constraint_paths.push(ConstraintPath::Line(Line { origin, direction }));
            }
        }

        // 3 calculate path intersections
        let mut valid_path: Option<ConstraintPath> = None;
        let mut valid_points: Vec<Point> = vec![];

        if !constraint_paths.is_empty() {
            valid_path = Some(constraint_paths[0].clone());
        }

        // once valid analytical paths are not produced, exit

        for i in 0..constraint_paths.len().saturating_sub(1) {
            if valid_path.is_none() {
                break;
            }

            let current_raw = valid_path.clone().unwrap();
            let current = &current_raw;
            let next = &constraint_paths[i + 1];

            match (current, next) {
                (ConstraintPath::Circle(c1), ConstraintPath::Circle(c2)) => {
                    // Handle Circle-Circle case
                }
                (ConstraintPath::Line(l1), ConstraintPath::Line(l2)) => {
                    let adjusted_origin_1 = l1.origin + -l1.direction * 500.0;
                    let adjusted_origin_2 = l2.origin + -l2.direction * 500.0;

                    let inter_result = ray_ray_intersection(
                        &Ray {
                            origin: adjusted_origin_1,
                            direction: l1.direction,
                        },
                        &Ray {
                            origin: adjusted_origin_2,
                            direction: l2.direction,
                        },
                    );

                    if let Some(inter) = inter_result {
                        valid_points = vec![inter];
                        valid_path = None;
                    } else {
                        // check if they are coincident

                        let cp = l2.closest_point(&l1.origin);
                        if cp.distance(l1.origin) < 0.001 {
                            // on line, overlapping
                            //niave approact -- full one would take the smallest ray, OR line segment (which isn't supported)
                            valid_path = Some(ConstraintPath::Line(l1.clone()));
                        } else {
                            valid_path = None;
                        }
                    }
                }
                (ConstraintPath::Ray(r1), ConstraintPath::Ray(r2)) => {
                    let inter_result = ray_ray_intersection(r1, r2);

                    if let Some(inter) = inter_result {
                        valid_points = vec![inter];
                        valid_path = None;
                    } else {
                        // check if they are coincident

                        let temp_line = Line {
                            origin: r2.origin,
                            direction: r2.direction,
                        };

                        let cp = temp_line.closest_point(&r1.origin);
                        if cp.distance(r1.origin) < 0.001 {
                            // on line, overlapping
                            //niave approact -- full one would take the smallest ray, OR line segment (which isn't supported)
                            valid_path = Some(ConstraintPath::Ray(Ray {
                                origin: r1.origin,
                                direction: r1.direction,
                            }));
                        } else {
                            valid_path = None;
                        }
                    }

                    // Handle Ray-Ray case
                }
                (ConstraintPath::Circle(c), ConstraintPath::Line(l))
                | (ConstraintPath::Line(l), ConstraintPath::Circle(c)) => {
                    // This case will never return paths
                    valid_path = None;

                    let adjusted_origin = l.origin + -l.direction * 500.0;
                    valid_points = ray_circle_intersection(
                        &Ray {
                            origin: adjusted_origin,
                            direction: l.direction,
                        },
                        c,
                    );
                }
                (ConstraintPath::Circle(c), ConstraintPath::Ray(r))
                | (ConstraintPath::Ray(r), ConstraintPath::Circle(c)) => {
                    // This case will never return paths
                    valid_path = None;

                    valid_points = ray_circle_intersection(
                        &Ray {
                            origin: r.origin,
                            direction: r.direction,
                        },
                        c,
                    );
                }
                (ConstraintPath::Line(l), ConstraintPath::Ray(r))
                | (ConstraintPath::Ray(r), ConstraintPath::Line(l)) => {
                    // Handle Line-Ray or Ray-Line case
                    let adjusted_origin = l.origin + -l.direction * 500.0;
                    let inter_result = ray_ray_intersection(
                        &Ray {
                            origin: adjusted_origin,
                            direction: l.direction,
                        },
                        r,
                    );

                    if let Some(inter) = inter_result {
                        valid_points = vec![inter];
                        valid_path = None;
                    } else {
                        // check if they are coincident

                        let cp = l.closest_point(&r.origin);
                        if cp.distance(r.origin) < 0.001 {
                            // on line, overlapping
                            //niave approact -- full one would take the smallest ray, OR line segment (which isn't supported)
                            valid_path = Some(ConstraintPath::Ray(r.clone()));
                        } else {
                            valid_path = None;
                        }
                    }
                }
                _ => {}
            }
        }

        match valid_path {
            Some(vp) => {
                let adjusted_pt = vp.closest_point(&try_pos);
                return SolverResponse {
                    state: SolverState::Partial,
                    valid_path: Some(vp),
                    new_pos: Some(adjusted_pt),
                };
            }
            None => {
                return SolverResponse {
                    state: SolverState::Locked,
                    valid_path : None,
                    new_pos: None,
                };
            }
        }

        // for (i, path) in constraint_paths.iter().enumerate() {
        //     if i == 0 {
        //         valid_paths.push(path);
        //     }
        // }

        // if !constraint_paths.is_empty() {
        //     let adjusted_pt = constraint_paths[0].closest_point(try_pos);

        //     return SolverResponse {
        //         state: SolverState::Partial,
        //         new_pos: Some(adjusted_pt),
        //     };
        // }

        // if none passed before, return free solverResponse
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

#[derive(Clone, Debug)]
pub struct Circle {
    origin: Pos2,
    radius: f32,
}

impl Circle {
    pub fn closest_point(&self, point: &Pos2) -> Pos2 {
        let mut dir = point.clone() - self.origin;

        if dir.length() < 0.001 {
            // just get the up direction since all points on the circle would be closest
            dir = Vec2::UP;
        }

        let dir = dir.normalized();

        self.origin + dir * self.radius
    }
}

#[derive(Clone, Debug)]
pub struct Line {
    origin: Pos2,
    direction: Vec2,
}

impl Line {
    pub fn closest_point(&self, point: &Pos2) -> Pos2 {
        let origin_to_point = point.clone() - self.origin;
        let projection_length = origin_to_point.dot(self.direction);
        self.origin + self.direction * projection_length
    }
}
#[derive(Clone, Debug)]
pub struct Ray {
    origin: Pos2,
    direction: Vec2,
}
impl Ray {
    pub fn closest_point(&self, point: &Pos2) -> Pos2 {
        // same as line, except we clamp the param to 0 if it's negative
        let origin_to_point = point.clone() - self.origin;
        let projection_length = origin_to_point.dot(self.direction);
        self.origin + self.direction * projection_length.max(0.0)
    }
}

#[derive(Clone, Debug)]
pub struct Point {
    origin: Pos2,
}

impl Point {
    pub fn closest_point(&self, point: &Pos2) -> Pos2 {
        self.origin
    }
}

#[derive(Clone, Debug)]
pub enum ConstraintPath {
    Circle(Circle),
    Line(Line),
    Ray(Ray),
    Point(Point),
}

impl ConstraintPath {
    pub fn closest_point(&self, point: &Pos2) -> Pos2 {
        match self {
            ConstraintPath::Circle(c) => c.closest_point(point),
            ConstraintPath::Line(l) => l.closest_point(point),
            ConstraintPath::Ray(r) => r.closest_point(point),
            ConstraintPath::Point(p) => p.closest_point(point),
            _ => Pos2::new(1., 1.),
        }
    }
}

// fn compare_paths(paths: &[ConstraintPath]) {
//     for i in 0..paths.len().saturating_sub(1) {
//         let current = &paths[i];
//         let next = &paths[i + 1];

//         match (current, next) {
//             (ConstraintPath::Circle(c1), ConstraintPath::Circle(c2)) => {
//                 // Handle Circle-Circle case
//             }
//             (ConstraintPath::Line(l1), ConstraintPath::Line(l2)) => {
//                 // Handle Line-Line case
//             }
//             (ConstraintPath::Ray(r1), ConstraintPath::Ray(r2)) => {
//                 // Handle Ray-Ray case
//             }
//             (ConstraintPath::Circle(c), ConstraintPath::Line(l))
//             | (ConstraintPath::Line(l), ConstraintPath::Circle(c)) => {
//                 // Handle Circle-Line or Line-Circle case
//             }
//             (ConstraintPath::Circle(c), ConstraintPath::Ray(r))
//             | (ConstraintPath::Ray(r), ConstraintPath::Circle(c)) => {
//                 // Handle Circle-Ray or Ray-Circle case
//             }
//             (ConstraintPath::Line(l), ConstraintPath::Ray(r))
//             | (ConstraintPath::Ray(r), ConstraintPath::Line(l)) => {
//                 // Handle Line-Ray or Ray-Line case
//             }
//             _ =>{}
//         }
//     }
// }

#[derive(Debug)]
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
    pub valid_path: Option<ConstraintPath>,
    pub new_pos: Option<Pos2>,
}

impl SolverResponse{
    pub fn locked() -> Self {
        Self{state: SolverState::Locked, valid_path: None, new_pos: None}
    }
}

#[derive(Default)]
pub struct EdgeSolverResponse {
    pub state: SolverState,
    pub new_pos: Option<[Pos2;2]>,
}
impl EdgeSolverResponse{
    pub fn locked() -> Self {
        Self{state: SolverState::Locked, new_pos: None}
    }
}

pub enum Constraint {
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
    pub edge_handle: EdgeHandle,
}
// Angle is relative to edge_1_handle counterclockwise
// pivot_vert_handle must refer to a vertex that both edges share
pub struct AngleConstraint {
    pub pivot_vert_handle: VertexHandle,
    pub edge_1_handle: EdgeHandle,
    pub edge_1_outer_vert_handle: VertexHandle,
    pub edge_2_handle: EdgeHandle,
    pub edge_2_outer_vert_handle: VertexHandle,
}

// Parallel constraint between two edges
// order does not matter here
pub struct ParallelConstraint {
    pub edge_1_handle: EdgeHandle,
    pub edge_2_handle: EdgeHandle,
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
fn ray_circle_intersection(ray: &Ray, circle: &Circle) -> Vec<Point> {
    let dx = ray.direction.x;
    let dy = ray.direction.y;
    let cx = circle.origin.x;
    let cy = circle.origin.y;
    let x0 = ray.origin.x;
    let y0 = ray.origin.y;

    // Calculate coefficients of the quadratic equation
    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (x0 - cx) + dy * (y0 - cy));
    let c = (x0 - cx) * (x0 - cx) + (y0 - cy) * (y0 - cy) - circle.radius * circle.radius;

    // Calculate the discriminant
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        // No intersection
        Vec::new()
    } else {
        // Calculate both potential solutions for `t`
        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b + sqrt_disc) / (2.0 * a);
        let t2 = (-b - sqrt_disc) / (2.0 * a);

        // Filter to only include points where t >= 0 (i.e., in the ray's direction)
        let mut intersections = Vec::new();

        if t1 >= 0.0 {
            intersections.push(Point {
                origin: Pos2 {
                    x: x0 + t1 * dx,
                    y: y0 + t1 * dy,
                },
            });
        }

        if t2 >= 0.0 {
            intersections.push(Point {
                origin: Pos2 {
                    x: x0 + t2 * dx,
                    y: y0 + t2 * dy,
                },
            });
        }

        intersections
    }
}

fn ray_ray_intersection(ray1: &Ray, ray2: &Ray) -> Option<Point> {
    let x1 = ray1.origin.x;
    let y1 = ray1.origin.y;
    let dx1 = ray1.direction.x;
    let dy1 = ray1.direction.y;

    let x2 = ray2.origin.x;
    let y2 = ray2.origin.y;
    let dx2 = ray2.direction.x;
    let dy2 = ray2.direction.y;

    // Calculate the denominator of the parameter equations
    let denominator = dx1 * dy2 - dy1 * dx2;

    if denominator.abs() < 0.001 {
        // Rays are parallel, no intersection
        return None;
    }

    // Calculate the parameters t and u
    let t = ((x2 - x1) * dy2 - (y2 - y1) * dx2) / denominator;
    let u = ((x2 - x1) * dy1 - (y2 - y1) * dx1) / denominator;

    // Check if both parameters are non-negative, indicating intersection in the direction of both rays
    if t >= 0.0 && u >= 0.0 {
        // Calculate the intersection point
        Some(Point {
            origin: Pos2 {
                x: x1 + t * dx1,
                y: y1 + t * dy1,
            },
        })
    } else {
        // Intersection exists but is not in the direction of the rays
        None
    }
}
