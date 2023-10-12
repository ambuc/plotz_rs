use argh::FromArgs;
use plotz_color::*;
use plotz_core::{canvas::Canvas, frame::make_frame, svg::Size};
use plotz_geometry::{
    crop::Croppable,
    obj::Obj,
    shapes::{
        pg::{multiline::Multiline, Pg},
        pt::Pt,
    },
    style::Style,
};
use rand::{thread_rng, Rng};
use std::{collections::HashMap, ops::Range};
use typed_builder::TypedBuilder;
use uuid::Uuid;

const CHARGE_RANGE: Range<f64> = -1.0..1.0;
const CLUSTER_DISTANCE: f64 = 100.0;
const CLUSTER_RANGE: Range<f64> = (-1.0 * CLUSTER_DISTANCE)..CLUSTER_DISTANCE;
const GRID_GRANULARITY: usize = 50;
const MOMENTUM: f64 = 1.00;
const NUM_CLUSTERS: usize = 20;
const NUM_PARTICLES_PER_CLUSTER: usize = 20;
const NUM_STEPS: usize = 200;

#[derive(FromArgs)]
#[argh(description = "...")]
struct Args {
    #[argh(option, description = "output path")]
    output_path_prefix: String,
}

enum Mobility {
    Fixed,
    Mobile,
}

enum Visibility {
    Visible,
    Invisible,
}

#[derive(TypedBuilder)]
struct Particle<T> {
    #[builder(setter(into))]
    position: Pt,

    mobility: Mobility,

    #[builder(default=Visibility::Visible)]
    visibility: Visibility,

    #[builder(default = 1.0)]
    charge: f64,

    #[builder(default=None, setter(strip_option))]
    metadata: Option<T>,

    #[builder(default=vec![])]
    history: Vec<Pt>,
}

impl<T> Particle<T> {
    fn is_fixed(&self) -> bool {
        matches!(self.mobility, Mobility::Fixed)
    }

    fn is_visible(&self) -> bool {
        matches!(self.visibility, Visibility::Visible)
    }
}

struct Framework<T> {
    particles: HashMap<Uuid, Particle<T>>,
}

impl<T> Default for Framework<T> {
    fn default() -> Self {
        Self {
            particles: Default::default(),
        }
    }
}

impl<T> Framework<T> {
    fn add_particle(&mut self, p: Particle<T>) {
        self.particles.insert(uuid::Uuid::new_v4(), p);
    }

    fn into_particles_visible(self) -> impl Iterator<Item = (Uuid, Particle<T>)> {
        self.particles.into_iter().filter(|(_u, p)| p.is_visible())
    }

    fn particles_mobile(&self) -> impl Iterator<Item = (&Uuid, &Particle<T>)> {
        self.particles.iter().filter(|(_u, p)| !p.is_fixed())
    }

    fn particles_which_are_not(&self, uuid: Uuid) -> impl Iterator<Item = (&Uuid, &Particle<T>)> {
        self.particles
            .iter()
            .filter_map(move |(k, v)| if *k == uuid { None } else { Some((k, v)) })
    }

    fn advance(&mut self) {
        // make array of next positions
        let mut deltas: Vec<(uuid::Uuid, Pt)> = vec![];

        for (uuid, particle) in self.particles_mobile() {
            let delta: Pt = self
                .particles_which_are_not(*uuid)
                .map(|(_uuid, extant_particle)| -> Pt {
                    let m1 = particle.charge;
                    let m2 = extant_particle.charge;
                    let r = particle.position.dist(&extant_particle.position);
                    let d = extant_particle.position - particle.position;
                    d * m1 * m2 / r.powf(2.0) / MOMENTUM
                })
                .fold(Pt(0, 0), |acc, x| acc + x);

            deltas.push((*uuid, delta));
        }

        // update positions in-place
        for (uuid, delta) in deltas.into_iter() {
            let p: &mut Particle<_> = &mut self.particles.get_mut(&uuid).unwrap();
            p.history.push(p.position);
            p.position += delta;
        }
    }
}

struct Metadata {
    color: &'static ColorRGB,
}

fn main() {
    let args: Args = argh::from_env();
    let mut os: Vec<(Obj, Style)> = vec![];
    let margin = 25.0;

    let frame = make_frame(
        (800.0 - 2.0 * margin, 1000.0 - 2.0 * margin),
        /*offset=*/ (margin, margin),
    );

    let mut framework = Framework::default();

    for i in (0..=(900 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
        for j in (0..=(700 / GRID_GRANULARITY)).map(|n| n * GRID_GRANULARITY) {
            // Insert a fixed, invisible high charge particle.
            framework.add_particle(
                Particle::builder()
                    .position((i as f64, j as f64))
                    .mobility(Mobility::Fixed)
                    .charge(thread_rng().gen_range(CHARGE_RANGE.clone()))
                    .visibility(Visibility::Invisible)
                    .build(),
            )
        }
    }

    for _ in 0..=NUM_CLUSTERS {
        let cluster_color = random_color();
        let cluster_center = Pt(
            thread_rng().gen_range(0..=900),
            thread_rng().gen_range(0..=700),
        );
        for _ in 0..=NUM_PARTICLES_PER_CLUSTER {
            // Insert a mobile, visible one-charge particle.
            framework.add_particle(
                Particle::builder()
                    .position(
                        cluster_center
                            + (
                                thread_rng().gen_range(CLUSTER_RANGE.clone()),
                                thread_rng().gen_range(CLUSTER_RANGE.clone()),
                            ),
                    )
                    .mobility(Mobility::Mobile)
                    .visibility(Visibility::Visible)
                    .metadata(Metadata {
                        color: cluster_color,
                    })
                    .build(),
            );
        }
    }

    for _ in 0..=NUM_STEPS {
        framework.advance();
    }

    for (_uuid, p) in framework.into_particles_visible() {
        os.push((
            Multiline(p.history).unwrap().into(),
            Style {
                color: p.metadata.unwrap().color,
                thickness: 1.0,
                ..Default::default()
            },
        ));
    }

    let frame_pg: Pg = frame.0.clone().try_into().unwrap();
    Canvas::from_objs(
        os.into_iter().flat_map(|(obj, style)| {
            obj.crop_to(&frame_pg)
                .into_iter()
                .map(|o| (o, style))
                .collect::<Vec<_>>()
        }),
        /*autobucket=*/
        true,
    )
    .with_frame(frame)
    .write_to_svg_or_die(
        // yeah, i know
        Size {
            width: 1000,
            height: 800,
        },
        &args.output_path_prefix,
    );
}
