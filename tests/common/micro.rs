use testcontainers::core::WaitFor;
use testcontainers::{Image, ImageArgs};

// Implementation of a custom image with args was copied from the following example in the testcontainers-rs repo:
// https://github.com/testcontainers/testcontainers-rs/blob/dev/testcontainers/src/images/trufflesuite_ganachecli.rs

// The default Iglu Resolver for micro has no cache, causing slow responses from Micro as it will always attempt to fetch schemas from Iglu Central.
// We can pass an iglu resolver config file to specify a cache size to resolve this, but to do so we need to set up an implementation of `ImageArgs`.

// Once Micro has been updated to have a cache by default, this can be removed, and we can simply run a GenericImage with:

// let running_message = WaitFor::message_on_stderr("REST interface bound to /0.0.0.0:9090");
// GenericImage::new("snowplow/snowplow-micro", "latest-distroless")
//      .with_exposed_port(9090)
//      .with_wait_for(running_message.clone())

#[derive(Debug, Default)]
pub struct Micro;

#[derive(Debug, Clone)]
pub struct MicroArgs {
    pub iglu: String,
}

impl Default for MicroArgs {
    fn default() -> Self {
        Self {
            iglu: "/config/iglu.json".to_string(),
        }
    }
}

// The setup for Micro to accept args requires an implementation of ImageArgs
impl ImageArgs for MicroArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(vec!["--iglu".to_string(), self.iglu].into_iter())
    }
}

// We also need to implement `Image` for Micro, to be able to call RunnableImage::from on it,
// which converts an Image implentation into a RunnableImage which we can run with Docker
impl Image for Micro {
    type Args = MicroArgs;

    fn name(&self) -> String {
        "snowplow/snowplow-micro".to_string()
    }

    fn tag(&self) -> String {
        "latest-distroless".to_string()
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![9090]
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "REST interface bound to /0.0.0.0:9090",
        )]
    }
}
