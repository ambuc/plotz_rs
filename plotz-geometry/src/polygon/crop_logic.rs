//! Crop logic for polygons.

use approx::ulps_eq;
use petgraph::dot::{Config, Dot};

use crate::{
    crop::{CropType, PointLoc},
    isxn::{Intersection, IsxnResult, Pair, Which},
    point::Pt,
    polygon::Polygon,
};
use float_ord::FloatOrd;
use itertools::Itertools;
use petgraph::{
    prelude::DiGraphMap,
    Direction::{Incoming, Outgoing},
};
use std::fmt::Debug;

/// An IsxnResult which knows the polygon segments of its two lines.
#[derive(PartialEq, Copy, Clone)]
pub struct AnnotatedIsxnResult {
    pub isxn_result: IsxnResult,
    pub a_segment_idx: usize,
    pub b_segment_idx: usize,
}

impl Debug for AnnotatedIsxnResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let AnnotatedIsxnResult {
            isxn_result,
            a_segment_idx,
            b_segment_idx,
        } = self;
        write!(
            f,
            "{:?} on [segment #{:?} of a, segment #{:?} of b]",
            isxn_result, a_segment_idx, b_segment_idx
        )
    }
}

pub struct CropGraph<'a> {
    graph: DiGraphMap<Pt, ()>,
    a: &'a Polygon,
    b: &'a Polygon,
    // why do we have a known_pts vector?
    //
    // it's simple -- points are allowed to implement a fuzzy equality (using
    // approx_eq!(f64, i, j) or somesuch) but unfortunately hashmaps really do
    // not like it when a==b =/> hash(a)==hash(b).
    //
    // instead, we check if a point is known (i.e. if it's approximately equal
    // to one in this vec) before treating it as a hashable value. if it is
    // known,
    known_pts: Vec<Pt>,
}

impl<'a> CropGraph<'a> {
    pub fn new(a: &'a Polygon, b: &'a Polygon) -> CropGraph<'a> {
        CropGraph {
            graph: DiGraphMap::<Pt, ()>::new(),
            a,
            b,
            known_pts: vec![],
        }
    }

    fn normalize_pt(&mut self, pt: &Pt) -> Pt {
        // if something in self.known_pts matches, return that instead.
        // otherwise insert pt into known_pts and return it.

        if let Some(extant) = self.known_pts.iter().find(|extant| {
            ulps_eq!(extant.x.0, pt.x.0, max_ulps = 10)
                && ulps_eq!(extant.y.0, pt.y.0, max_ulps = 10)
        }) {
            *extant
        } else {
            self.known_pts.push(*pt);
            *pt
        }
    }

    pub fn build_from_polygons(&mut self, crop_type: CropType) {
        // inelegant way to run a against b, then b against a. oops
        let pair = Pair {
            a: self.a,
            b: self.b,
        };
        for which in [Which::A, Which::B] {
            let this = pair.get(which);
            let that = pair.get(which.flip());

            for sg in this.to_segments() {
                let sg = match (which, crop_type) {
                    (Which::B, CropType::Exclusive) => sg.flip(),
                    _ => sg,
                };
                let mut isxns: Vec<Intersection> = that
                    .intersects_segment_detailed(&sg)
                    .into_iter()
                    .filter_map(|isxn| match isxn {
                        IsxnResult::MultipleIntersections(_) => None,
                        IsxnResult::OneIntersection(isxn) => Some(isxn),
                    })
                    .map(|isxn| match which {
                        // ugh... this one is stupid. when we call
                        // intersects_segment_details it assumes (a,b) order.
                        Which::A => isxn.flip_pcts(),
                        Which::B => isxn,
                    })
                    .collect();

                if isxns.is_empty() {
                    let from_pt_normalized = self.normalize_pt(&sg.i);
                    let from = self.graph.add_node(from_pt_normalized);
                    assert_eq!(from, from_pt_normalized);

                    let to_pt_normalized = self.normalize_pt(&sg.f);
                    let to = self.graph.add_node(to_pt_normalized);
                    assert_eq!(to, to_pt_normalized);

                    self.graph.add_edge(from, to, ());
                } else {
                    isxns.sort_by_key(|isxn| FloatOrd(isxn.percent_along(which).0));

                    {
                        let from_pt_normalized = self.normalize_pt(&sg.i);
                        let from = self.graph.add_node(from_pt_normalized);
                        assert_eq!(from, from_pt_normalized);

                        let to_pt = isxns[0].pt();
                        let to_pt_normalized = self.normalize_pt(&to_pt);
                        let to = self.graph.add_node(to_pt_normalized);
                        assert_eq!(to, to_pt_normalized);

                        if from != to {
                            self.graph.add_edge(from, to, ());
                        }
                    }

                    for (i, j) in isxns.iter().tuple_windows() {
                        let from_pt = i.pt();
                        let from_pt_normalized = self.normalize_pt(&from_pt);
                        let from = self.graph.add_node(from_pt_normalized);
                        assert_eq!(from, from_pt_normalized);

                        let to_pt = j.pt();
                        let to_pt_normalized = self.normalize_pt(&to_pt);
                        let to = self.graph.add_node(to_pt_normalized);
                        assert_eq!(to, to_pt_normalized);

                        self.graph.add_edge(from, to, ());
                    }

                    {
                        let from_pt = isxns.last().unwrap().pt();
                        let from_pt_normalized = self.normalize_pt(&from_pt);
                        let from = self.graph.add_node(from_pt_normalized);
                        assert_eq!(from, from_pt_normalized);

                        let to_pt = &sg.f;
                        let to_pt_normalized = self.normalize_pt(&to_pt);
                        let to = self.graph.add_node(to_pt_normalized);
                        assert_eq!(to, to_pt_normalized);

                        if from != to {
                            self.graph.add_edge(from, to, ());
                        }
                    }
                }
            }
        }
    }

    pub fn remove_outside_nodes(&mut self) {
        // remove nodes which are outside.
        for node in self
            .graph
            .nodes()
            .filter(|node| {
                matches!(
                    self.a.contains_pt(node).expect("contains"),
                    PointLoc::Outside
                ) || matches!(
                    self.b.contains_pt(node).expect("contains"),
                    PointLoc::Outside
                )
            })
            .collect::<Vec<_>>()
        {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_nodes_inside_b_or_outside_a(&mut self) {
        // remove nodes which are inside.
        for node in self
            .graph
            .nodes()
            .filter(|node| {
                matches!(
                    self.b.contains_pt(node).expect("contains"),
                    PointLoc::Inside
                ) || matches!(
                    self.a.contains_pt(node).expect("contains"),
                    PointLoc::Outside
                )
            })
            .collect::<Vec<_>>()
        {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_acycle_nodes(&mut self) {
        // also, remove all nodes that aren't part of a cycle (i.e. have at
        // least one incoming and at least one outgoing)
        while let Some(node_to_remove) = self.graph.nodes().find(|&node| {
            self.graph.neighbors_directed(node, Incoming).count() == 0
                || self.graph.neighbors_directed(node, Outgoing).count() == 0
        }) {
            self.graph.remove_node(node_to_remove);
        }
    }

    pub fn remove_stubs(&mut self) {
        // a _stub_ is like this:
        //
        // a -> b <-> s
        // ^    v
        // d <- c
        //
        // where s is the stub -- it's connected (didn't get removed by
        // |remove_acycle_nodes|) but is still degenerate.
        while let Some(node) = self.graph.nodes().find(|node| {
            let inn: Vec<Pt> = self
                .graph
                .neighbors_directed(*node, Incoming)
                .into_iter()
                .collect::<Vec<_>>();
            let out: Vec<Pt> = self
                .graph
                .neighbors_directed(*node, Outgoing)
                .into_iter()
                .collect::<Vec<_>>();

            matches!((&inn[..], &out[..]), ([a], [b]) if a==b)
        }) {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_back_and_forth(&mut self) {
        while let Some((a, b, ())) = self.graph.all_edges().find(|edge| {
            let i = edge.0;
            let f = edge.1;
            self.graph.contains_edge(f, i)
        }) {
            self.graph.remove_edge(a, b);
            self.graph.remove_edge(b, a);
        }
    }

    pub fn nodes_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn remove_segments_which_cross_other(&mut self) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.b.contains_pt(&edge.0.avg(&edge.1)).expect("contains"),
                PointLoc::Inside
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }

    pub fn extract_polygon(&mut self) -> Option<Polygon> {
        let mut pts: Vec<Pt> = vec![];

        if self.nodes_count() == 0 {
            return None;
        }
        let mut curr_node: Pt = self.graph.nodes().next().unwrap();

        while !pts.contains(&curr_node) {
            pts.push(curr_node);

            curr_node = match self
                .graph
                .neighbors_directed(curr_node, Outgoing)
                .collect::<Vec<_>>()[..]
            {
                [n] => n,
                [n, _] if self.a.pts.contains(&n) => n,
                [_, n] if self.a.pts.contains(&n) => n,
                [a, b] => {
                    println!("aborting search: found two {:?},{:?}", a, b);
                    return None;
                }
                _ => {
                    let a = self
                        .graph
                        .neighbors_directed(curr_node, Outgoing)
                        .collect::<Vec<_>>();
                    println!("aborting search: from {:?}, found {:?}", curr_node, a);
                    return None;
                }
            };
        }

        for pt in &pts {
            self.graph.remove_node(*pt);
        }

        if let Ok(pg) = Polygon(pts) {
            Some(pg)
        } else {
            None
        }
    }

    pub fn to_resultant_polygons(mut self) -> Vec<Polygon> {
        // println!(
        //     "{:?}",
        //     Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        // );

        let mut resultant = vec![];

        while let Some(pg) = self.extract_polygon() {
            resultant.push(pg);
        }

        resultant
    }
}
