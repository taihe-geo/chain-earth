use tg_render_engine::{run};
mod demo;
use demo::MyDemo;
// mod geometry;
fn main() {
    async_std::task::block_on(run::<MyDemo>());
}