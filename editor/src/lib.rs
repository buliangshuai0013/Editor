mod rg_editor;
mod rg_rect;

pub use rg_editor::*;
pub use rg_rect::*;

use rand::Rng;

pub fn generate_nodes(n: usize) -> Vec<RgRect> {
    let mut nodes = Vec::with_capacity(n);
    let mut rng = rand::thread_rng();

    for i in 0..n {
        let x = rng.gen_range(50.0..700.0);
        let y = rng.gen_range(50.0..500.0);
        let width = rng.gen_range(80.0..200.0);
        let height = rng.gen_range(60.0..150.0);

        nodes.push(RgRect::new(i as u64, x, y, width, height));
    }

    nodes
}
