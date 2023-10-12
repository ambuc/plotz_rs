use crate::particle::*;

use plotz_geometry::shapes::pt::Pt;
use std::collections::HashMap;
use uuid::Uuid;

const POW: f64 = 1.1;

pub struct Framework<T> {
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
    pub fn add_particle(&mut self, p: Particle<T>) {
        self.particles.insert(uuid::Uuid::new_v4(), p);
    }

    pub fn into_particles_visible(self) -> impl Iterator<Item = (Uuid, Particle<T>)> {
        self.particles.into_iter().filter(|(_u, p)| p.is_visible())
    }

    pub fn advance(&mut self) {
        // make array of next positions
        let mut deltas: Vec<(uuid::Uuid, Pt)> = vec![];

        for (uuid, particle) in self.particles_mobile() {
            let delta: Pt = self
                .charged_particles_which_are_not(*uuid)
                .map(|(_uuid, extant_particle)| -> Pt {
                    let m1 = particle.charge.unwrap_or(1.0);
                    let m2 = extant_particle.charge.unwrap_or(1.0);
                    let r = particle.position.dist(&extant_particle.position);
                    let d = extant_particle.position - particle.position;
                    d * m1 * m2 / r.powf(POW)
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

    // private

    fn particles_mobile(&self) -> impl Iterator<Item = (&Uuid, &Particle<T>)> {
        self.particles.iter().filter(|(_u, p)| !p.is_fixed())
    }

    fn charged_particles(&self) -> impl Iterator<Item = (&Uuid, &Particle<T>)> {
        self.particles.iter().filter(|(_u, p)| p.charge.is_some())
    }

    fn charged_particles_which_are_not(
        &self,
        uuid: Uuid,
    ) -> impl Iterator<Item = (&Uuid, &Particle<T>)> {
        self.charged_particles()
            .filter_map(move |(k, v)| if *k == uuid { None } else { Some((k, v)) })
    }
}
