nextbus
=======

Retrieve real-time location data from the Nextbus API and output it as JSON.

install
=======

If you are a Rust programmer, you can install nextbus with cargo:

```
cargo install nextbus
```

usage
=====

Stream updates for a single route at the maximum request rate:
```
$ nextbus sf-muni 12 | jq '.'
{
    "route": "12",
    "direction": ,..,
    "lat": ...,
    "lon": ...,
    "reportEpoch": ...,
    "secsSinceReport": ...,
    "predictable": ...,
    "heading": ...,
    "speedKmHr": ...,
}
```

Stream updates for all routes for an agency:
```
$ nextbus sf-muni | jq '.'
[...]
```
