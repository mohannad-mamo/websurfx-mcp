# SearXNG

[SearXNG settings docs](https://docs.searxng.org/admin/settings/index.html)

The official docker image from SearXNG doesn't seem to have json output set by default. As far as I could tell there is no way to change this with an env var. So we have to build our own custom image that copies over server settings to enable json output.

I copied over the settings and limiter files from [Perplexica Repo](https://github.com/ItzCrazyKns/Perplexica/tree/master/searxng)

The key section we care about:

```yaml
search:
  autocomplete: "google"
  formats:
    - html
    - json # enable the json format for the API
```

## NOTE:
This configuration assumes you will run your SearXNG instance in a private network not accessible to the public. If you plan on hosting for public access, you will need to do your own research for setting up SearXNG properly.
