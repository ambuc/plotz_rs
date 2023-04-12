//! A scene, i.e. a holder for 3d objects ready to be projected down onto a 2d
//! plane.

use crate::{polygon3d::Polygon3d, face::Face};

use {
    crate::{camera::Projection, object::Object, object_inner::ObjectInner, style::Style},
    plotz_geometry::{draw_obj::DrawObj, polygon::Polygon, segment::Segment},
};

/// A scene of 3d objects ready to be projected down to a 2d plane.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Some objects.
    pub objects: Vec<Object>,
}

impl Scene {
    /// A new scene.
    pub fn new() -> Scene {
        Scene { objects: vec![] }
    }
    /// Make a scene from some objects.
    pub fn from(a: impl IntoIterator<Item = Object>) -> Scene {
        Scene {
            objects: a.into_iter().collect(),
        }
    }
    /// Make a scene from some objects
    pub fn from_objects_with_style(
        a: impl IntoIterator<Item = ObjectInner>,
        style: Style,
    ) -> Scene {
        Scene {
            objects: a
                .into_iter()
                .map(|a| Object::new(a).with_style(style.clone()))
                .collect(),
        }
    }

    /// Projects the scene onto a camera, renders to 2d, and returns a vector of drawobjects.
    pub fn project_with(&self, projection: Projection) -> Vec<DrawObj> {
        match projection {
            Projection::Oblique(oblique_projection) => {
                let mut v: Vec<DrawObj> = vec![];
                //
                for Object { inner, style } in self.objects.iter() {
                    let mut dos = match inner {
                        ObjectInner::Polygon3d(pg3d) => {
                            let d_o = DrawObj::new(
                                Polygon(
                                    pg3d.pts
                                        .iter()
                                        .map(|pt3d| oblique_projection.project(&pt3d))
                                        .collect::<Vec<_>>(),
                                )
                                .expect("polygon construction failed"),
                            );
                            vec![d_o]
                            //
                        }
                        ObjectInner::Segment3d(sg3d) => {
                            let d_o = DrawObj::new(Segment(
                                oblique_projection.project(&sg3d.i),
                                oblique_projection.project(&sg3d.f),
                            ));
                            vec![d_o]
                            //
                        }
                        ObjectInner::GroupOfFaces(group_of_faces) => {
                            let mut dos = vec![];
                            for Face {
                                pg3d: Polygon3d { pts },
                            } in group_of_faces.items.iter()
                            {
                                dos.push(DrawObj::new(
                                    Polygon(
                                        pts.iter()
                                            .map(|pt3d| oblique_projection.project(&pt3d))
                                            .collect::<Vec<_>>(),
                                    )
                                    .expect("polyugon construction failed"),
                                ));
                            }
                            dos
                        }
                    };
                    if let Some(Style { color, thickness }) = style {
                        dos = dos
                            .into_iter()
                            .map(|d_o| d_o.with_color(color).with_thickness(*thickness))
                            .collect();
                    }
                    v.extend(dos);
                }
                //
                v
            }
        }
    }
}
