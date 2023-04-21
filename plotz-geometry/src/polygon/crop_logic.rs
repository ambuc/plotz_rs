//! Crop logic for polygons.

use {
    super::TryPolygon,
    crate::{
        crop::{CropType, PointLoc},
        isxn::{Intersection, IsxnResult, Pair, Which},
        point::Pt,
        polygon::Polygon,
    },
    approx::*,
    float_ord::FloatOrd,
    itertools::Itertools,
    petgraph::{
        dot::{Config, Dot},
        prelude::DiGraphMap,
        Direction,
        Direction::{Incoming, Outgoing},
    },
    std::fmt::Debug,
    tracing::*,
    typed_builder::TypedBuilder,
};

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

#[derive(Debug, TypedBuilder)]
pub struct CropGraph<'a> {
    #[builder(default)]
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
    #[builder(default)]
    known_pts: Vec<Pt>,
}

impl<'a> CropGraph<'a> {
    fn normalize_pt(&mut self, pt: &Pt) -> Pt {
        // if something in self.known_pts matches, return that instead.
        // otherwise insert pt into known_pts and return it.

        if let Some(extant) = self.known_pts.iter().find(|extant| {
            let e = f64::EPSILON * 1_000_000_000.0;
            relative_eq!(extant.x.0, pt.x.0, epsilon = e)
                && relative_eq!(extant.y.0, pt.y.0, epsilon = e)
        }) {
            *extant
        } else {
            self.known_pts.push(*pt);
            *pt
        }
    }

    fn pair(&self) -> Pair<Polygon> {
        Pair {
            a: self.a,
            b: self.b,
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
                        let to_pt_normalized = self.normalize_pt(to_pt);
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

    pub fn remove_nodes_inside_polygon(&mut self, which: Which) {
        while let Some(node) = self
            .graph
            .nodes()
            .find(|node| matches!(self.pair().get(which).contains_pt(node), PointLoc::Inside))
        {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_nodes_outside_polygon(&mut self, which: Which) {
        while let Some(node) = self
            .graph
            .nodes()
            .find(|node| matches!(self.pair().get(which).contains_pt(node), PointLoc::Outside))
        {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_nodes_with_no_neighbors_of_kind(&mut self, direction: Direction) {
        while let Some(node_to_remove) = self
            .graph
            .nodes()
            .find(|&node| self.graph.neighbors_directed(node, direction).count() == 0)
        {
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
            match (
                &self
                    .graph
                    .neighbors_directed(*node, Incoming)
                    .collect::<Vec<_>>()[..],
                &self
                    .graph
                    .neighbors_directed(*node, Outgoing)
                    .collect::<Vec<_>>()[..],
            ) {
                ([a], [b]) => a == b,
                _ => false,
            }
        }) {
            self.graph.remove_node(node);
        }
    }

    pub fn remove_dual_edges(&mut self) {
        while let Some((i, j, ())) = self
            .graph
            .all_edges()
            .find(|edge| self.graph.contains_edge(edge.1, edge.0))
        {
            self.graph.remove_edge(i, j);
            self.graph.remove_edge(j, i);
        }
    }

    pub fn nodes_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn remove_edges_outside(&mut self, which: Which) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.pair().get(which).contains_pt(&edge.0.avg(&edge.1)),
                PointLoc::Outside,
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }
    pub fn remove_edges_inside(&mut self, which: Which) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.pair().get(which).contains_pt(&edge.0.avg(&edge.1)),
                PointLoc::Inside
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }

    pub fn extract_polygon(&mut self) -> Option<Polygon> {
        let mut pts: Vec<Pt> = vec![];

        // info!("dot before extract:");
        // println!( "{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));

        if self.nodes_count() == 0 {
            return None;
        }
        let mut curr_node: Pt = self.graph.nodes().next().unwrap();

        while !pts.contains(&curr_node) {
            pts.push(curr_node);

            let next_node: Option<Pt> = match self
                .graph
                .neighbors_directed(curr_node, Outgoing)
                .collect::<Vec<_>>()[..]
            {
                [i] => Some(i),
                [i, j] => match (pts.contains(&i), pts.contains(&j)) {
                    (true, false) => Some(i),
                    (false, true) => Some(j),
                    _ => match (self.a.pts.contains(&i), self.a.pts.contains(&j)) {
                        (true, _) => Some(i),
                        (_, true) => Some(j),
                        x => {
                            warn!("hit a weird dead end: {:?}", x);
                            println!("DEAD END");
                            None
                        }
                    },
                },
                _ => {
                    let a = self
                        .graph
                        .neighbors_directed(curr_node, Outgoing)
                        .collect::<Vec<_>>();
                    error!("aborting search: from {:?}, found {:?}", curr_node, a);
                    None
                }
            };

            if let Some(next_node) = next_node {
                // remove the edges now...
                self.graph.remove_edge(curr_node, next_node);
                curr_node = next_node;
            } else {
                // hit a dead end -- close the polygon and break.
                pts.push(pts[0]);
                break;
            }
        }

        // and the nodes later.
        self.remove_nodes_with_no_neighbors_of_kind(Direction::Incoming);
        self.remove_nodes_with_no_neighbors_of_kind(Direction::Outgoing);

        TryPolygon(pts).ok()
    }

    pub fn as_resultant_polygons(mut self) -> Vec<Polygon> {
        let mut resultant = vec![];

        while let Some(pg) = self.extract_polygon() {
            if pg.pts.len() == 3 {
                info!("extracted: {:?}", pg);
            }
            resultant.push(pg);

            // clean up in between extractions.
            self.remove_nodes_with_no_neighbors_of_kind(Direction::Incoming);
            self.remove_nodes_with_no_neighbors_of_kind(Direction::Outgoing);
        }

        resultant
    }
}
