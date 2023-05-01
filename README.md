# reverse-proxy

A basic reverse proxy container for mapping domains to internal services and handling SSL.

Environment Variables:
- `USE_SELF_SIGNED` - `true`/`false` - whether to generate a self signed certificate
- `DOMAIN_MAPS` - `string` - list of domains for the reverse proxy to map to the internal services

DOMAIN_MAPS example:

```dockerfile
DOMAIN_MAPS=studyscore.app:192.168.0.8 api.studyscore.app:192.168.0.9
```