# JSON5

A Rust [JSON5] serializer and deserializer which speaks [Serde].

## API

Deserialize a JSON5 string with `from_str`. Go the other way with `to_string`.
The serializer is very basic at the moment, it just produces plain old JSON.
See the [Serde documentation] for details on implementing `Serialize` and
`Deserialize`. (Usually it's just a case of sprinkling in some derives.)

The [Serde data model] is mostly supported, with the exception of bytes and
borrowed strings.

## Example

Read some config in to a struct.

```rust
extern crate json5;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Debug, PartialEq)]
struct Config {
    message: String,
    n: i32,
}

fn main() {
    let config = "
        {
          // A traditional message.
          message: 'hello world',

          // A number for some reason.
          n: 42,
        }
    ";

    assert_eq!(
        json5::from_str(config),
        Ok(Config {
            message: "hello world".to_string(),
            n: 42,
        }),
    );
}
```

[JSON5]: https://json5.org/
[Serde]: https://serde.rs/
[Serde documentation]: https://serde.rs/custom-serialization.html
[Serde data model]: https://serde.rs/data-model.html
