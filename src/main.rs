use simrng::{rng::{LinearCongruentialGenerator, Random}, dist::normal_box_muller};

fn main() {
    let mut rng = LinearCongruentialGenerator::new(5);
    println!("{}", rng.next());
    println!("{:?}", normal_box_muller(&mut rng, 1.65, 10.0));
}
