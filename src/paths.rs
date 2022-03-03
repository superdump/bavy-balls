use std::ops::Range;

use bevy::math::Quat;
use rand::{prelude::SmallRng, Rng};

pub struct WormPathIterator {
    pub rng: SmallRng,
    pub yaw_range: Range<f32>,
    pub pitch_range: Range<f32>,
}

impl Iterator for WormPathIterator {
    type Item = Quat;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            Quat::from_rotation_y(self.rng.gen_range(self.yaw_range.clone()))
                * Quat::from_rotation_x(self.rng.gen_range(self.pitch_range.clone())),
        )
    }
}
