#!/usr/bin/env nu

let endpoint = "http://localhost:8080/a3c/auth0/events"
#let endpoint = "http://auth0.dito.lab.atricore.com:31000/a3c/auth0/events"
#let endpoint = "https://1e55-190-57-207-159.ngrok-free.app/a3c/auth0/events"
let payload = "./tests/data/payloads.json"
#let payload = "./r.log"

open $payload | http post --headers ["Authorization" "Bearer changeme"] --raw --content-type=application/json $endpoint $in
