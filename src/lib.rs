mod fruit;
mod fruit_harvest;
mod fruit_render;
mod item;
mod perfect_fruit;
mod perfect_fruit_policy;
mod quick_use;
mod quick_use_policy;

#[skyline::main(name = "additions-patches")]
pub fn main() {
    if fruit::install() {
        perfect_fruit::install();
    }

    match nookkit::connect("additions-patches") {
        Ok(ui) => quick_use::install(ui),
        Err(e) => skyline::println!("[additions-patches] Inventory patches skipped: {e}"),
    }
}
