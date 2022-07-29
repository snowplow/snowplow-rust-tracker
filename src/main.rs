use snowplow_rust_tracker::snowplow::Snowplow;
// use serde_json::json;

fn main() {
    let mut sp = Snowplow::new();
    let x = sp.create_tracker("namespace".to_string(), "id".to_string());

    // let y = SelfDescribingJson {
    //     schema: "namespace".to_string(),
    //     data: json!({
    //     "name": "John Doe",
    //     "age": 43,
    //     "phones": [
    //     "+44 1234567",
    //     "+44 2345678"
    //     ]
    //     }),
    // };
}
