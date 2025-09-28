# Laval Kubernetes Deployment Notes

## TLS certificate installation with acme.sh

The Laval Istio `Gateway` expects a Kubernetes TLS secret named
`laval-gateway-cert` in the same namespace as the gateway resource (default).
The gateway host is configured through the `LAVAL_DOMAIN` environment variable
that the deployment pipeline injects (set this secret in GitHub Actions as
needed). After issuing your certificate with `acme.sh`, install it and load it
into the cluster with the following commands (replace the example domain with
your actual value):

```bash
# Install the certificate material to local files managed by acme.sh
DOMAIN=laval.example.com
acme.sh --install-cert -d "$DOMAIN" \
  --key-file       /tmp/laval-gateway.key \
  --fullchain-file /tmp/laval-gateway.crt

# Create or update the Kubernetes TLS secret that the Istio Gateway uses
kubectl create secret tls laval-gateway-cert \
  --namespace default \
  --key  /tmp/laval-gateway.key \
  --cert /tmp/laval-gateway.crt \
  --dry-run=client -o yaml | kubectl apply -f -
```

This secret is referenced by the HTTPS listener in `k8s/laval.yaml`.
