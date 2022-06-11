mod index;
mod camera;
mod model;
mod resources;
mod texture;

use crate::index::run;

fn main() {
    async_std::task::block_on(run());
}