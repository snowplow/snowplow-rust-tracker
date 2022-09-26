# Rust Analytics for Snowplow

[![early-release]][tracker-classificiation]
[![Build Status][gh-actions-image]][gh-actions]
[![License][license-image]][license]

Snowplow is a scalable open-source platform for rich, high quality, low-latency data collection. It is designed to collect high quality, complete behavioral data for enterprise business.

**To find out more, please check out the [Snowplow website][website] and our [documentation][docs].**

## Snowplow Rust Tracker Overview

The Snowplow Rust Tracker allows you to add analytics to your Rust apps when using a [Snowplow][snowplow] pipeline.

With this tracker you can collect granular event-level data as your users interact with your Rust applications.

**Technical documentation can be found for each tracker in our [Documentation][rust-docs].**

## Quick Start

### Installation

Add the `snowplow_tracker` as a dependency in `Cargo.toml` inside your Rust application:

```yml
[dependencies]
snowplow_tracker = "0.1"
```

Use the package APIs in your code:

```rust
use snowplow_tracker::Snowplow;
```

### Using the Tracker

Instantiate a tracker using the `Snowplow::create_tracker` function.
The function takes three required arguments: `namespace` and `app_id`, and `collector_url`.
Tracker namespace identifies the tracker instance; you may create multiple trackers with different namespaces.
The `app_id` identifies your app.
The `collector_url` is the URI of the Snowplow collector to send the events to.

```rust
let tracker = Snowplow::create_tracker("ns", "app_id", "https://...");
```

To track events, simply instantiate their respective types and pass them to the `tracker.track` method with optional context entities.
Please refer to the documentation for specification of event properties.

```rust
// Tracking a screen view event
tracker.track(
    ScreenViewEvent::builder()
        .id(Uuid::new_v4())
        .name("a screen view")
        .build()
        .unwrap(),
    None
).await;

// Tracking a self-describing event with a context entity
tracker.track(
    SelfDescribingEvent {
        schema: "iglu:com.snowplowanalytics.snowplow/link_click/jsonschema/1-0-1".to_string(),
        data: json!({"targetUrl": "http://a-target-url.com"})
    },
    Some(vec![
        SelfDescribingJson::new("iglu:org.schema/WebPage/jsonschema/1-0-0", json!({"keywords": ["tester"]}))
    ])
).await;

// Tracking a structured event
tracker.track(
    StructuredEvent::builder()
        .category("shop")
        .action("add-to-basket")
        .label("Add To Basket")
        .property("pcs")
        .value(2.0)
        .build()
        .unwrap(),
    None
).await
```

## Find Out More

| Technical Docs                    | Setup Guide                 |
|-----------------------------------|-----------------------------|
| [![i1][techdocs-image]][techdocs] | [![i2][setup-image]][setup] |
| [Technical Docs][techdocs]        | [Setup Guide][setup]        |

## Maintainers

| Contributing                                 |
|----------------------------------------------|
| [![i4][contributing-image]](CONTRIBUTING.md) |
| [Contributing](CONTRIBUTING.md)              |

## Testing

## Copyright and License

The Snowplow Rust Tracker is copyright 2022 Snowplow Analytics Ltd.

Licensed under the **[Apache License, Version 2.0][license]** (the "License");
you may not use this software except in compliance with the License.

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[website]: https://snowplowanalytics.com
[snowplow]: https://github.com/snowplow/snowplow
[docs]: https://docs.snowplowanalytics.com/
[rust-docs]: https://docs.snowplowanalytics.com/docs/collecting-data/collecting-from-own-applications/rust-tracker/

[gh-actions]: https://github.com/snowplow-incubator/snowplow-rust-tracker/actions/workflows/build.yml
[gh-actions-image]: https://github.com/snowplow-incubator/snowplow-rust-tracker/actions/workflows/build.yml/badge.svg

[license]: https://www.apache.org/licenses/LICENSE-2.0
[license-image]: https://img.shields.io/badge/license-Apache--2-blue.svg?style=flat

[releases]: https://crates.io/crates/snowplow_tracker

[techdocs]: https://docs.snowplowanalytics.com/docs/collecting-data/collecting-from-own-applications/rust-tracker/
[techdocs-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/techdocs.png
[setup]: https://docs.snowplowanalytics.com/docs/collecting-data/collecting-from-own-applications/rust-tracker/quick-start-guide
[setup-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/setup.png

[api-docs]: https://snowplow.github.io/snowplow-rust-tracker/

[contributing-image]: https://d3i6fms1cm1j0i.cloudfront.net/github/images/contributing.png

[tracker-classificiation]: https://github.com/snowplow/snowplow/wiki/Tracker-Maintenance-Classification
[early-release]: https://img.shields.io/static/v1?style=flat&label=Snowplow&message=Early%20Release&color=014477&labelColor=9ba0aa&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAMAAAAoLQ9TAAAAeFBMVEVMaXGXANeYANeXANZbAJmXANeUANSQAM+XANeMAMpaAJhZAJeZANiXANaXANaOAM2WANVnAKWXANZ9ALtmAKVaAJmXANZaAJlXAJZdAJxaAJlZAJdbAJlbAJmQAM+UANKZANhhAJ+EAL+BAL9oAKZnAKVjAKF1ALNBd8J1AAAAKHRSTlMAa1hWXyteBTQJIEwRgUh2JjJon21wcBgNfmc+JlOBQjwezWF2l5dXzkW3/wAAAHpJREFUeNokhQOCA1EAxTL85hi7dXv/E5YPCYBq5DeN4pcqV1XbtW/xTVMIMAZE0cBHEaZhBmIQwCFofeprPUHqjmD/+7peztd62dWQRkvrQayXkn01f/gWp2CrxfjY7rcZ5V7DEMDQgmEozFpZqLUYDsNwOqbnMLwPAJEwCopZxKttAAAAAElFTkSuQmCC
