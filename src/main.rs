use snowplow_rust_tracker::snowplow::Snowplow;

fn main() {
    let mut sp = Snowplow::new();
    let x = sp.create_tracker("namespace".to_string(), "id".to_string());
}
