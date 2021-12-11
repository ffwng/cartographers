mod mask;
mod board;
mod scoring;

use mask::Mask;

fn main() {
    let m = Mask::full() & !Mask::row(8) & !Mask::column(4) & !Mask::cell(1, 2);
    println!("{:?}", m);
}
