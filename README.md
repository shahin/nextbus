nextbus
=======

Retrieve real-time location and prediction data from the UmoIQ (nextbus) API and output it as JSON.

install
=======

If you are a Rust programmer, you can install nextbus with cargo:

```
git clone https://github.com/shahin/nextbus.git &&
  cargo install nextbus
```

usage
=====

Get locations of all vehicles for a single route:
```
nextbus locations sf-muni 22 | jq '.' | head
```
```
[
  {
    "id": "5730",
    "route_tag": "22",
    "dir_tag": "22___O_F00",
    "lat": 37.76869,
    "lon": -122.38908,
    "epoch": 1661190376596,
    "predictable": true,
    "heading": 345,
    ...
```

Poll for updates ever 60 seconds, for all routes for an agency:
```
$ nextbus locations sf-muni --pause 60 | jq '.'
```
```
[
  {
    "id": "5816",
    "route_tag": "1",
    "dir_tag": "",
    "lat": 37.779816,
    "lon": -122.4931,
    "epoch": 1661190398737,
    "predictable": true,
    "heading": 269,
    ...
```

Get predicted arrival times for given stop IDs:
```
nextbus predictions sf-muni 22 -- 4618 | jq '.'
```
```
nextbus predictions sf-muni 22 -- 4618 | jq '.'
{
  "predictions": [
    {
      "direction": [
        {
          "title": "Outbound to UCSF Mission Bay",
          "prediction": [
            {
              "epochTime": 1661191317835,
              "seconds": 772,
              "minutes": 12,
              "isDeparture": false,
              "dirTag": "22___O_F00",
              "affectedByLayover": false,
              "delayed": false,
              "slowness": 0,
              "vehicle": "5717",
              "vehiclesInConsist": 0,
              "block": "2206",
              "tripTag": "11031088"
            },
            ...
```

references
==========

- UmoIQ API Specification: https://retro.umoiq.com/xmlFeedDocs/NextBusXMLFeed.pdf
