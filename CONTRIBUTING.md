# Contributing

If you are interested in helping this project complete its stated goal of exporting all the contents of an Enjin website, read on here.

# Prerequisites

Things you'll need to already know, or take a crash course on before you can begin in earnest, are below. If you don't know these things, you can take a Udemy course, google for tutorials, etc. but unfortunately I lack the time to help you on a one-on-one basis with these basics.

 - Ability to program in Rust (preferred) or Python (fallback)
 - Knowledge of basic Git commands, or how to use a Git GUI
 - Basic knowledge of HTTP requests and conceptual knowledge of what JSON-RPC is (https://www.jsonrpc.org/)
 - Knowledge of Enjin's website layout, specifically for the areas of the site that you want to extract which are not yet supported

# The Two Ways of Extracting Data From Enjin

The two major ways of extracting data from Enjin that I've found in my research (spanning years at this point) are:

## The Enjin API

https://www.enjin.com/api

This is what I use currently in Encuum. The Enjin API speaks JSON-RPC on top of HTTPS. It requires a valid username and password to authenticate, and then you are given a session cookie that is valid for 30 days. All subsequent requests after login require the `session_id` parameter to be supplied, in addition to other parameters that vary depending on the request.

If you read my Rust code, you'll see that I have [jsonrpsee](https://github.com/paritytech/jsonrpsee) bindings to the JSON-RPC Enjin API for the bits that I've completed so far. jsonrpsee automatically performs serialization to/from [JSON](https://www.json.org/json-en.html) for the HTTP request and response payloads, by figuring out how to map your method call parameters into the `params` map of an outgoing JSON-RPC request, then when the answer comes back, parsing the JSON `result` map into a `struct` that we must define.

Learning JSON-RPC is _also_ beyond the scope of this document, but there is a pretty good set of documentation at https://docs.rs for jsonrpsee (and all Rust crates).

### Helping with the Enjin API Response Data

You'll notice on the Enjin API website, they document the outbound call requirements (only barely; they give the parameter names and types, but not the semantics), but they don't document the response contents at all.

If you want to help me understand the Enjin protocol without hacking on Rust, and you know some Python, you can create some new files in the `pyapitest` directory that will use the infrastructure in `common.py` plus the `.env` file to generate API responses and save them to disk using `save_auth_req`. 

You can also analyze the responses that come back from Enjin and help document your understanding of what each API call does, and what parameters and response values we need. There are many API calls (e.g. in the undocumented Enjin Forums API) that return data that we already have, or don't need, so we can ignore that when mapping it through jsonrpsee.

You can also make notes such as, any specific permissions required to make an API call, any gotchas, paging needs (e.g. if the response hands you 10 results at a time and you have to loop through the "pages" of the calls), etc.

## Web Scraping

For data we can't seem to find through the Enjin API, web scraping is a less-preferred, buggier fallback, but it does work.

My initial approach for forum extraction was via the Enjin API, but I've since abandoned that in favor of getting as much as possible from the Enjin API, because it's many times faster than driving the web frontend. 

The web frontend also has a tendency to throw up random CAPTCHAs if you're making too many requests, which obviously an automated tool can't answer, so it fails out before finishing the extraction.

If you are looking to extract something specific that can't be extracted using the Enjin API, we may have to reintroduce web scraping into Encuum. My earlier tool using web scraping was at https://github.com/allquixotic/encuum-kt written in Kotlin, but there is a working web scraping crate for Rust called [rust-headless-chrome](https://github.com/rust-headless-chrome/rust-headless-chrome). I prefer this over the playwright-rust crate, which hasn't been updated for months, and is designed with an async API (async is useless because we have to drive the web UI in a specific ordering of operations to navigate pages and extract the data we need.)

To delve into this, we will have to introduce rust-headless-chrome as a dependency of encuum, and build out the infrastructure to set it up, then use it as part of the extraction routines called out of main.rs. It's doable, but it'll require an intermediate knowledge of Rust (or I can do it myself if I have time).

Web scraping then boils down to knowing what reliable CSS or XPath selectors to use to get the data you need, and throwing in the appropriate waits/sleeps between requests. Then of course everything has to be serialized to sqlite.

# Edge-cases

Even if we get encuum working 100% with a given website, it's entirely possible that the _data_ contained in a different Enjin website will be displayed in a different way, either through the Enjin API, or via web scraping. Testing encuum on different websites is important to shake out bugs, where, for example, we might expect a given field to always contain a value, but it might be `null` or omitted on certain websites for some reason.

And of course, your permission level on a given website has a large impact on what is returned to you via the Enjin API and web scraping. If you can't see something, you can't extract it. On the other hand, maybe I wrote encuum with a permission level that couldn't see something that you *can* see, resulting in encuum having an incomplete picture of the API.

