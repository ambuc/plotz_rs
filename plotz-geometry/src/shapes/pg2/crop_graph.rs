//! Crop graph for polygons.

use {
    super::TryPolygon,
    crate::{
        crop::{CropType, PointLoc},
        isxn::{Intersection, IsxnResult, Pair, Which},
        shapes::{
            pg2::Pg2,
            point::{is_colinear_n, Pt},
        },
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
    typed_builder::TypedBuilder,
};

#[derive(Debug, TypedBuilder)]
pub struct CropGraph<'a> {
    #[builder(default)]
    graph: DiGraphMap<Pt, ()>,

    a: &'a Pg2,

    b: &'a Pg2,

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
    pub fn run(a: &Pg2, b: &Pg2, crop_type: CropType) -> (Vec<Pg2>, DiGraphMap<Pt, ()>) {
        let mut crop_graph = CropGraph::builder().a(a).b(b).build();
        crop_graph.build_from_polygons(crop_type);
        crop_graph.remove_nodes_outside_polygon(Which::A);
        match crop_type {
            CropType::Inclusive => {
                crop_graph.remove_nodes_outside_polygon(Which::B);
                crop_graph.remove_edges_outside(Which::A);
            }
            CropType::Exclusive => {
                crop_graph.remove_nodes_inside_polygon(Which::B);
                crop_graph.remove_edges_inside(Which::B);
            }
        }
        crop_graph.remove_stubs();
        crop_graph.remove_dual_edges();
        crop_graph.remove_nodes_with_no_neighbors_of_any_kind();
        let graph = crop_graph.graph.clone();
        (crop_graph.trim_and_create_resultant_polygons(), graph)
    }

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

    fn get(&self, which: Which) -> &Pg2 {
        Pair {
            a: &self.a,
            b: &self.b,
        }
        .get(which)
    }

    fn build_from_polygons(&mut self, crop_type: CropType) {
        // inelegant way to run a against b, then b against a. oops
        for which in [Which::A, Which::B] {
            let this = self.get(which).clone();
            let that = self.get(which.flip()).clone();

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

    fn remove_nodes_inside_polygon(&mut self, which: Which) {
        while let Some(node) = self
            .graph
            .nodes()
            .find(|node| self.get(which).point_is_inside(node))
        {
            self.graph.remove_node(node);
        }
    }

    fn remove_nodes_outside_polygon(&mut self, which: Which) {
        while let Some(node) = self
            .graph
            .nodes()
            .find(|node| self.get(which).point_is_outside(node))
        {
            self.graph.remove_node(node);
        }
    }

    fn remove_nodes_with_no_neighbors_of_kind(&mut self, direction: Direction) {
        while let Some(node_to_remove) = self
            .graph
            .nodes()
            .find(|&node| self.graph.neighbors_directed(node, direction).count() == 0)
        {
            self.graph.remove_node(node_to_remove);
        }
    }

    fn remove_nodes_with_no_neighbors_of_any_kind(&mut self) {
        self.remove_nodes_with_no_neighbors_of_kind(Outgoing);
        self.remove_nodes_with_no_neighbors_of_kind(Incoming);
    }

    fn remove_stubs(&mut self) {
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

    fn remove_dual_edges(&mut self) {
        while let Some((i, j, ())) = self
            .graph
            .all_edges()
            .find(|(a, b, _)| self.graph.contains_edge(*b, *a))
        {
            self.graph.remove_edge(i, j);
            self.graph.remove_edge(j, i);
        }
    }

    fn remove_edges_outside(&mut self, which: Which) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.get(which).contains_pt(&edge.0.avg(&edge.1)),
                PointLoc::Outside,
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }
    fn remove_edges_inside(&mut self, which: Which) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.get(which).contains_pt(&edge.0.avg(&edge.1)),
                PointLoc::Inside
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }

    fn extract_polygon(&mut self) -> Option<Pg2> {
        let mut pts: Vec<Pt> = vec![];

        if self.graph.node_count() == 0 {
            return None;
        }
        let mut curr_node: Pt = {
            //
            if let Some(pt) = self
                .graph
                .nodes()
                .find(|node| self.graph.neighbors_directed(*node, Outgoing).count() > 1)
            {
                pt
            } else if let Some(pt) = self
                .graph
                .nodes()
                .find(|node| self.graph.neighbors_directed(*node, Incoming).count() > 1)
            {
                pt
            } else {
                self.graph.nodes().next().unwrap()
            }
        };

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
                        _ => None,
                    },
                },
                _ => {
                    let a = self
                        .graph
                        .neighbors_directed(curr_node, Outgoing)
                        .collect::<Vec<_>>();
                    panic!("aborting search: from {:?}, found {:?}", curr_node, a);
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
        self.remove_nodes_with_no_neighbors_of_any_kind();

        TryPolygon(pts).ok()
    }

    #[allow(unused)]
    fn print(&self) {
        println!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        );
    }

    // NB: Destructive, walks and destroys graph.
    fn trim_and_create_resultant_polygons(mut self) -> Vec<Pg2> {
        let mut resultant = vec![];

        while let Some(pg) = self.extract_polygon() {
            if !is_colinear_n(&pg.pts) {
                resultant.push(pg);
            }
        }

        resultant
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{interpolate::extrapolate_2d, p2, shapes::pg2::Rect};
    use itertools::iproduct;
    use test_case::test_case;

    fn u_shape() -> Pg2 {
        let a = p2!(60, 60);
        let b = p2!(70, 60);
        let c = p2!(80, 60);
        let d = p2!(90, 60);
        let e = p2!(70, 75);
        let f = p2!(80, 75);
        let g = p2!(60, 90);
        let h = p2!(90, 90);
        Pg2([a, b, e, f, c, d, h, g, a])
    }

    fn h_shape() -> Pg2 {
        let a = p2!(60, 40);
        let b = p2!(70, 40);
        let c = p2!(70, 70);
        let d = p2!(80, 70);
        let e = p2!(80, 40);
        let f = p2!(90, 40);
        let g = p2!(90, 110);
        let h = p2!(80, 110);
        let i = p2!(80, 80);
        let j = p2!(70, 80);
        let k = p2!(70, 110);
        let l = p2!(60, 110);
        Pg2([a, b, c, d, e, f, g, h, i, j, k, l, a])
    }

    #[test_case(u_shape(), CropType::Exclusive; "u-shape, exclusive")]
    #[test_case(u_shape(), CropType::Inclusive; "u-shape, inclusive")]
    #[test_case(h_shape(), CropType::Exclusive; "h-shape, exclusive")]
    #[test_case(h_shape(), CropType::Inclusive; "h-shape, inclusive")]
    fn test_all_crops(shape: Pg2, crop_type: CropType) {
        let boundary = Rect(p2!(50, 50), (50.0, 50.0)).unwrap();
        let margin = 10.0;
        for (_idx, offset) in iproduct!(0..=5, 0..=4).map(|(i, j)| {
            (
                (i, j),
                Pt((i as f64 - 3.0) * margin, (j as f64 - 3.0) * margin),
            )
        })
        // .filter(|(idx, _)| *idx == (1, 2))
        {
            let inner = shape.clone() + offset;

            let (_resultants, graph) = CropGraph::run(&inner, &boundary, crop_type);

            // // Assert some stuff about the resultant polygon graphs.
            // for node in graph.nodes() {
            //     // Each node should have only one outgoing and only one incoming edge.
            //     assert_eq!(graph.neighbors_directed(node, Outgoing).count(), 1);
            //     assert_eq!(graph.neighbors_directed(node, Incoming).count(), 1);
            // }

            // we should make sure that no resultant points are 100%
            // outside of boundary.
            for node in graph.nodes() {
                match crop_type {
                    CropType::Inclusive => assert!(boundary.point_is_inside_or_on_border(&node)),
                    CropType::Exclusive => assert!(!boundary.point_is_inside(&node)),
                }
            }
            // we should also make sure that, along each line, no
            // intermediate points are 100% outside of boundary.
            // we should also make sure that, along each line, no
            // intermediate points are 100% inside of the boundary
            for (a, b, _) in graph.all_edges() {
                for i in 0..=10 {
                    let p = extrapolate_2d(a, b, (i as f64) / 10.0);
                    match crop_type {
                        CropType::Inclusive => assert!(boundary.point_is_inside_or_on_border(&p)),
                        CropType::Exclusive => assert!(!boundary.point_is_inside(&p)),
                    }
                }
            }
        }
    }
}
