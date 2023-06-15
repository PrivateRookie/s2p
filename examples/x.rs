#[derive(s2p::Cast)]
struct A {
    #[cast(apply = "ax", col = "fake")]
    name: String,
    age: u64,
}

fn ax(name: String) -> u32 {
    name.parse().unwrap()
}

fn main() {
    let df = A::to_polars(vec![A {
        name: "100".into(),
        age: 20,
    }])
    .unwrap();
    dbg!(df);
}
