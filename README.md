# rss-scihub-proxy

this is a very simple server that proxies a configurable list of rss feeds, and prepends `https://sci-hub.tw` to the `link` for each entry. it is designed to be used with the eTOC feeds for various academic journals, to allow for a simple way to subscribe to a rss feed of a journal that will go directly to a paper when you click on an entry.

it takes one argument - a link to a toml configuration file, such as the one below:

```toml
show_index = true

[server]
port = 8080
address = "0.0.0.0"
workers = 4

[feeds.opre]
name = "Operations Research"
url = "https://pubsonline.informs.org/action/showFeed?type=etoc&feed=rss&jc=opre"

[feeds.jea]
name = "Journal of Experimental Algorithmics"
url = "https://dl.acm.org/action/showFeed?type=etoc&feed=rss&jc=jev"
```

the `show_index` and `server` sections are optional, and default to the values shown here. the `name` item for each feed is optional (defaulting to the key, such as "opre" or "jea" in these examples), and is only used if `show_index` is true.
