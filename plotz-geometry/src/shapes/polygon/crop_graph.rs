//! Crop graph for polygons.

use crate::{
    crop::{CropType, PointLocation},
    intersection::{Intersection, IntersectionResult},
    overlaps::{opinion::PolygonOp, polygon_overlaps_point},
    shapes::{
        point::{is_colinear_n, Point},
        polygon::Polygon,
    },
    utils::{Pair, Which},
};
use anyhow::{anyhow, Context, Result};
use approx::*;
use float_ord::FloatOrd;
use itertools::Itertools;
use petgraph::{
    dot::{Config, Dot},
    prelude::DiGraphMap,
    Direction,
    Direction::{Incoming, Outgoing},
};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct CropGraph<'a> {
    #[builder(default)]
    graph: DiGraphMap<Point, ()>,

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
    known_pts: Vec<Point>,
}

impl<'a> CropGraph<'a> {
    pub fn run(
        a: &Polygon,
        b: &Polygon,
        crop_type: CropType,
    ) -> Result<(Vec<Polygon>, DiGraphMap<Point, ()>)> {
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
        Ok((
            crop_graph
                .trim_and_create_resultant_polygons()
                .context("trim and create resultant polygons")?,
            graph,
        ))
    }

    fn normalize_pt(&mut self, pt: &Point) -> Point {
        // if something in self.known_pts matches, return that instead.
        // otherwise insert pt into known_pts and return it.

        if let Some(extant) = self.known_pts.iter().find(|extant| {
            let e = f64::EPSILON * 1_000_000_000.0;
            relative_eq!(extant.x, pt.x, epsilon = e) && relative_eq!(extant.y, pt.y, epsilon = e)
        }) {
            *extant
        } else {
            self.known_pts.push(*pt);
            *pt
        }
    }

    fn get(&self, which: Which) -> &Polygon {
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
                        IntersectionResult::Ok(isxn) => Some(isxn),
                        _ => None,
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

                        let to_pt = isxns[0].pt;
                        let to_pt_normalized = self.normalize_pt(&to_pt);
                        let to = self.graph.add_node(to_pt_normalized);
                        assert_eq!(to, to_pt_normalized);

                        if from != to {
                            self.graph.add_edge(from, to, ());
                        }
                    }

                    for (i, j) in isxns.iter().tuple_windows() {
                        let from_pt = i.pt;
                        let from_pt_normalized = self.normalize_pt(&from_pt);
                        let from = self.graph.add_node(from_pt_normalized);
                        assert_eq!(from, from_pt_normalized);

                        let to_pt = j.pt;
                        let to_pt_normalized = self.normalize_pt(&to_pt);
                        let to = self.graph.add_node(to_pt_normalized);
                        assert_eq!(to, to_pt_normalized);

                        self.graph.add_edge(from, to, ());
                    }

                    {
                        let from_pt = isxns.last().unwrap().pt;
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
        while let Some(node) = self.graph.nodes().find(|node| {
            matches!(
                polygon_overlaps_point(self.get(which), node).unwrap(),
                Some((PolygonOp::WithinArea, _))
            )
        }) {
            self.graph.remove_node(node);
        }
    }

    fn remove_nodes_outside_polygon(&mut self, which: Which) {
        while let Some(node) = self
            .graph
            .nodes()
            .find(|node| matches!(polygon_overlaps_point(self.get(which), node).unwrap(), None))
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
                self.get(which).contains_pt_deprecated(&edge.0.avg(&edge.1)),
                Ok(PointLocation::Outside),
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }
    fn remove_edges_inside(&mut self, which: Which) {
        while let Some((i, j, ())) = self.graph.all_edges().find(|edge| {
            matches!(
                self.get(which).contains_pt_deprecated(&edge.0.avg(&edge.1)),
                Ok(PointLocation::Inside)
            )
        }) {
            self.graph.remove_edge(i, j);
        }
    }

    // Returns a polygon, if possible. An error state here represents some vailed invariant.
    fn extract_polygon(&mut self) -> Result<Option<Polygon>> {
        let mut pts: Vec<Point> = vec![];

        if self.graph.node_count() == 0 {
            return Ok(None);
        }
        let mut curr_node: Point = {
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

            let next_node: Option<Point> = match self
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
                [i, j, k] => match (pts.contains(&i), pts.contains(&j), pts.contains(&k)) {
                    (true, false, false) => Some(i),
                    (false, true, false) => Some(j),
                    (false, false, true) => Some(k),
                    _ => match (
                        self.a.pts.contains(&i),
                        self.a.pts.contains(&j),
                        self.a.pts.contains(&k),
                    ) {
                        (true, _, _) => Some(i),
                        (_, true, _) => Some(j),
                        (_, _, true) => Some(k),
                        _ => None,
                    },
                },
                _ => {
                    let a = self
                        .graph
                        .neighbors_directed(curr_node, Outgoing)
                        .collect::<Vec<_>>();
                    return Err(anyhow!(
                        "aborting search: from {:?}, found {:?}",
                        curr_node,
                        a
                    ));
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

        // if the polygon is just pts=[x, x]; that's a dot.
        if pts.len() == 2 && pts[0] == pts[1] {
            return Ok(None);
        }

        let dbg = format!("pts: {:?}", pts);
        Ok(Some(Polygon(pts).context(dbg)?))
    }

    #[allow(unused)]
    fn print(&self) {
        println!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        );
    }

    // NB: Destructive, walks and destroys graph.
    fn trim_and_create_resultant_polygons(mut self) -> Result<Vec<Polygon>> {
        let mut resultant = vec![];

        while let Some(pg) = self.extract_polygon().context("extract polygon")? {
            if !is_colinear_n(&pg.pts) {
                resultant.push(pg);
            }
        }

        Ok(resultant)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{crop::Croppable, interpolate::extrapolate_2d, shapes::polygon::Rect};
    use itertools::iproduct;
    use test_case::test_case;

    fn u_shape() -> Polygon {
        let a = Point(60, 60);
        let b = Point(70, 60);
        let c = Point(80, 60);
        let d = Point(90, 60);
        let e = Point(70, 75);
        let f = Point(80, 75);
        let g = Point(60, 90);
        let h = Point(90, 90);
        Polygon([a, b, e, f, c, d, h, g, a]).unwrap()
    }

    fn h_shape() -> Polygon {
        let a = Point(60, 40);
        let b = Point(70, 40);
        let c = Point(70, 70);
        let d = Point(80, 70);
        let e = Point(80, 40);
        let f = Point(90, 40);
        let g = Point(90, 110);
        let h = Point(80, 110);
        let i = Point(80, 80);
        let j = Point(70, 80);
        let k = Point(70, 110);
        let l = Point(60, 110);
        Polygon([a, b, c, d, e, f, g, h, i, j, k, l, a]).unwrap()
    }

    #[test_case(u_shape(), CropType::Exclusive; "u-shape, exclusive")]
    #[test_case(u_shape(), CropType::Inclusive; "u-shape, inclusive")]
    #[test_case(h_shape(), CropType::Exclusive; "h-shape, exclusive")]
    #[test_case(h_shape(), CropType::Inclusive; "h-shape, inclusive")]
    fn test_all_crops(shape: Polygon, crop_type: CropType) -> Result<()> {
        let boundary = Rect((50, 50), (50, 50)).unwrap();
        let margin = 10.0;
        for (_idx, offset) in iproduct!(0..=5, 0..=4).map(|(i, j)| {
            (
                (i, j),
                Point((i as f64 - 3.0) * margin, (j as f64 - 3.0) * margin),
            )
        })
        // .filter(|(idx, _)| *idx == (1, 2))
        {
            let inner = shape.clone() + offset;

            let (_resultants, graph) = CropGraph::run(&inner, &boundary, crop_type)?;

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
                    CropType::Inclusive => {
                        let x = boundary.point_is_inside_or_on_border_deprecated(&node);
                        assert!(x);
                    }
                    CropType::Exclusive => {
                        let x = !boundary.point_is_inside_deprecated(&node);
                        assert!(x);
                    }
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
                        CropType::Inclusive => {
                            let x = boundary.point_is_inside_or_on_border_deprecated(&p);
                            assert!(x);
                        }
                        CropType::Exclusive => {
                            let x = !boundary.point_is_inside_deprecated(&p);
                            assert!(x);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_reproduce_error() -> Result<()> {
        let a = Polygon([
            Point(0.19999999999999995559, -0.11299423149111920139),
            Point(0.19999999999999995559, 0.16984848098349947243),
            Point(0.50710678118654750612, 0.38700576850888046554),
            Point(0.49999999999999988898, 0.38198051533946364433),
            Point(0.00000000000000000000, 0.02842712474619002450),
            Point(0.00000000000000000000, -0.25441558772842887137),
            Point(-0.09289321881345258269, -0.32010101267766694066),
            Point(0.00000000000000000000, -0.38578643762690512098),
            Point(0.29289321881345276033, -0.17867965644035743722),
        ])?;

        let b = Polygon([
            Point(0.80000000000000004441, -0.53725830020304798929),
            Point(0.19999999999999995559, -0.11299423149111920139),
            Point(0.19999999999999995559, 0.16984848098349947243),
            Point(0.36568542494923800268, 0.28700576850888048774),
            Point(0.00000000000000000000, 0.02842712474619002450),
            Point(0.00000000000000000000, -0.25441558772842887137),
            Point(0.80000000000000004441, -0.82010101267766688515),
        ])?;

        let _ = a.crop(&b, CropType::Exclusive)?;
        let _ = a.crop(&b, CropType::Inclusive)?;
        let _ = b.crop(&a, CropType::Exclusive)?;
        let _ = b.crop(&a, CropType::Inclusive)?;

        Ok(())
    }
}
