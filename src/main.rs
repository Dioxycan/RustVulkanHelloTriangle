
mod core;
use crate::core::Core;

fn main() {

    let core = Core::build("hello");
    core.burn();

}
